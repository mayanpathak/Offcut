// The media worker: copies a dropped file to OPFS, probes and validates it,
// and decodes its audio to PCM at 48 kHz and 16 kHz (TS §15). The rules are
// in Rust; this file is the browser glue around them.

import { type Bytes, type ClipId, type ClipInfo, type Hz, LIMITS, type RejectReason } from "../gen/domain";
import { type CoreApi, loadCore } from "../wasm/load-core";
import { decodeAudio } from "./media/audio-decode";
import { byteSize, importToOpfs, openSource } from "./media/import";
import type { MediaWorkerApi } from "./protocol";
import type { Entry, ProbeInfo, RouteFlags, RouteReject, SourceInfo, VideoDetail } from "../gen/domain";
import { SOURCE_LIMITS } from "../gen/domain";
import { fileHandle, OpfsError, paths } from "../persistence/opfs";
import type { CoreSourceDemuxer, Routed } from "../wasm/load-core";
import { type PcmFile, SourceAudioError, writeWholePcm16 } from "./media/audio-region";
import { fileRandomAccess, SourceReadError } from "./media/file-random-access";
import type { SourceFailed, SourceId } from "./protocol";
import { CANCELLED, type Handlers, type JobContext, serveWorker, WorkerFailure } from "./rpc-worker";

type Probed = { ok: ClipInfo } | { rejected: RejectReason };

// The clip this worker accepted last. `extractAudio` is given a clip id only
// (TS §14.2) and needs the clip's `ClipInfo`.
let accepted: { clipId: ClipId; info: ClipInfo } | undefined;

/** Steps 3 to 6 of `importAndProbe`, on a source file that is open. */
async function probeSource(core: CoreApi, handle: FileSystemSyncAccessHandle): Promise<Probed> {
  const size = byteSize(handle);
  const demuxer = core.openDemuxer(handle);
  if ("rejected" in demuxer) {
    return demuxer;
  }
  try {
    const video = demuxer.probe(size).video;
    const description = demuxer.videoDescription();
    let supported = false;
    if (video?.codec === "h264" && description !== undefined) {
      try {
        const answer = await VideoDecoder.isConfigSupported({
          codec: video.codec_string,
          description,
          codedWidth: video.coded_width,
          codedHeight: video.coded_height,
        });
        supported = answer.supported === true;
      } catch {
        // A config the browser cannot even read is one it cannot decode.
        supported = false;
      }
    }
    return core.probeAndValidate(demuxer, size, supported);
  } finally {
    demuxer.free();
  }
}

/** The PCM as exactly `length` samples: cut, or padded with silence (D-34). */
function fit(pcm: Float32Array, length: number): Float32Array {
  if (pcm.length === length) {
    return pcm;
  }
  const fitted = new Float32Array(length);
  fitted.set(pcm.subarray(0, length));
  return fitted;
}

async function importAndProbe(p: { clipId: ClipId; file: File }, ctx: JobContext): Promise<Probed | typeof CANCELLED> {
  // The size rule comes before any copy (TS §15.2).
  if ((p.file.size as Bytes) > LIMITS.MAX_FILE_SIZE) {
    return { rejected: "REJECT_FILE_SIZE" };
  }
  const handle = await importToOpfs(
    p.clipId,
    p.file,
    () => ctx.isCancelled(),
    (progress) => {
      ctx.progress(progress);
    },
  );
  if (handle === CANCELLED) {
    return CANCELLED;
  }
  try {
    const result = await probeSource(await loadCore(), handle);
    accepted = "ok" in result ? { clipId: p.clipId, info: result.ok } : undefined;
    return result;
  } finally {
    // The render worker opens the source later; it must be closed here (TS §31).
    handle.close();
  }
}

async function extractAudio(
  p: { clipId: ClipId },
  ctx: JobContext,
): Promise<{ pcm48: Float32Array; pcm16: Float32Array } | typeof CANCELLED> {
  const core = await loadCore();
  const handle = await openSource(p.clipId, { create: false });
  try {
    // A clip this worker did not just accept is probed again.
    if (accepted?.clipId !== p.clipId) {
      const result = await probeSource(core, handle);
      if (!("ok" in result)) {
        throw new WorkerFailure("E_INTERNAL", "NotAccepted");
      }
      accepted = { clipId: p.clipId, info: result.ok };
    }
    const info = accepted.info;
    const mono = await decodeAudio(handle, info, () => ctx.isCancelled());
    if (mono === CANCELLED) {
      return CANCELLED;
    }
    const rate = info.audio_sample_rate;
    return {
      pcm48: fit(core.resample(mono, rate, 48_000 as Hz), Math.round(info.duration * 48)),
      pcm16: fit(core.resample(mono, rate, 16_000 as Hz), Math.round(info.duration * 16)),
    };
  } finally {
    handle.close();
  }
}

// --- V6.1: the long path (v6.1implementation §6.4, §7.2) ---------------------

/** The demuxer calls routing needs; both demuxers have them. */
type Routable = Pick<CoreSourceDemuxer, "probe" | "videoDetail" | "hevcDescription" | "videoDescription">;

/** The browser's answer for the video: H.264 with its `avcC`, HEVC with its `hvcC` (D-398). */
async function decodeSupported(d: Routable, probe: ProbeInfo, detail: VideoDetail): Promise<boolean> {
  const video = probe.video;
  if (video === null) {
    return false;
  }
  const config =
    video.codec === "h264"
      ? { codec: video.codec_string, description: d.videoDescription() }
      : video.codec === "hevc"
        ? { codec: detail.hevc_codec_string ?? undefined, description: d.hevcDescription() }
        : undefined;
  if (config?.codec === undefined || config.description === undefined) {
    return false;
  }
  try {
    const answer = await VideoDecoder.isConfigSupported({
      codec: config.codec,
      description: config.description,
      codedWidth: video.coded_width,
      codedHeight: video.coded_height,
    });
    return answer.supported === true;
  } catch {
    // A config the browser cannot even read is one it cannot decode.
    return false;
  }
}

async function routeOpened(core: CoreApi, d: Routable, size: Bytes, entry: Entry, flags: RouteFlags): Promise<Routed> {
  const probe = d.probe(size);
  const detail = d.videoDetail();
  return core.routeProbe(probe, detail, await decodeSupported(d, probe, detail), entry, flags);
}

/** A thrown source failure as the value the long path returns (`protocol.ts`, `SourceFailed`). */
function asSourceFailed(thrown: unknown): SourceFailed | undefined {
  if (thrown instanceof SourceReadError) {
    return { failed: "E_SOURCE_READ", detail: "Read" };
  }
  if (thrown instanceof SourceAudioError) {
    return { failed: "E_SOURCE_AUDIO", detail: thrown.detail };
  }
  if (typeof thrown === "object" && thrown !== null && "code" in thrown) {
    const detail = "detail" in thrown && typeof thrown.detail === "string" ? thrown.detail : "Error";
    if (thrown.code === "E_SOURCE_READ" || thrown.code === "E_SOURCE_AUDIO") {
      return { failed: thrown.code, detail };
    }
  }
  return undefined;
}

/** Opens a dropped file in place; a refusal or a failure to read is a value. */
function openInPlace(core: CoreApi, file: File): CoreSourceDemuxer | { rejected: RejectReason } | SourceFailed {
  try {
    return core.openDemuxerReader(fileRandomAccess(file));
  } catch (thrown) {
    const failed = asSourceFailed(thrown);
    if (failed !== undefined) {
      return failed;
    }
    throw thrown;
  }
}

async function routeFile(p: { file: File; entry: Entry; flags: RouteFlags }): Promise<Routed | SourceFailed> {
  const core = await loadCore();
  const d = openInPlace(core, p.file);
  if ("rejected" in d || "failed" in d) {
    return d;
  }
  try {
    return await routeOpened(core, d, p.file.size as Bytes, p.entry, p.flags);
  } catch (thrown) {
    const failed = asSourceFailed(thrown);
    if (failed !== undefined) {
      return failed;
    }
    throw thrown;
  } finally {
    d.free();
  }
}

async function importAndRoute(
  p: { clipId: ClipId; file: File; flags: RouteFlags },
  ctx: JobContext,
): Promise<{ ok: ClipInfo } | { rejected: RouteReject } | typeof CANCELLED> {
  // The size rule comes before any copy; 16 GB with `high_res` on (DC-3).
  const limit = p.flags.high_res ? SOURCE_LIMITS.MAX_FILE_SIZE_HIGH_RES : LIMITS.MAX_FILE_SIZE;
  if ((p.file.size as Bytes) > limit) {
    return { rejected: "REJECT_FILE_SIZE" };
  }
  const handle = await importToOpfs(
    p.clipId,
    p.file,
    () => ctx.isCancelled(),
    (progress) => {
      ctx.progress(progress);
    },
  );
  if (handle === CANCELLED) {
    return CANCELLED;
  }
  try {
    const core = await loadCore();
    const d = core.openDemuxer(handle);
    if ("rejected" in d) {
      return d;
    }
    try {
      // As read (P174): the render worker, frozen, configures its decoder with the `avcC`
      // only, so an HEVC clip could not be previewed or exported. HEVC stays refused on the
      // clip page (ON section 6, "H.264 up to 4K only"); the long page reads HEVC sources' sound.
      if (d.probe(byteSize(handle)).video?.codec === "hevc") {
        return { rejected: "REJECT_HEVC" };
      }
      const routed = await routeOpened(core, d, byteSize(handle), "clip", p.flags);
      if ("clip" in routed) {
        accepted = { clipId: p.clipId, info: routed.clip };
        return { ok: routed.clip };
      }
      accepted = undefined;
      // The clip page never routes to the long path (rule 15): a long file is too long here.
      return "rejected" in routed ? routed : { rejected: "REJECT_DURATION" };
    } finally {
      d.free();
    }
  } finally {
    // The render worker opens the source later; it must be closed here (TS §31).
    handle.close();
  }
}

/** `sources/<id>/pcm16.f32` through a sync handle; the directory is made when absent. */
async function openPcm16(sourceId: SourceId): Promise<FileSystemSyncAccessHandle> {
  try {
    return await (await fileHandle(paths.sourcePcm16(sourceId), { create: true })).createSyncAccessHandle();
  } catch (error) {
    throw new WorkerFailure(
      error instanceof OpfsError && error.kind === "quota" ? "E_STORAGE_QUOTA" : "E_STORAGE_IO",
      error instanceof Error ? error.name : "Error",
    );
  }
}

async function extractSourcePcm16(
  p: { sourceId: SourceId; file: File; info: SourceInfo; fromSeconds: number },
  ctx: JobContext,
): Promise<{ done: true; levelP95: number } | SourceFailed | typeof CANCELLED> {
  const core = await loadCore();
  const d = openInPlace(core, p.file);
  if ("failed" in d) {
    return d;
  }
  if ("rejected" in d) {
    // It opened before: it is not the same file any more.
    return { failed: "E_SOURCE_READ", detail: "Changed" };
  }
  let handle: FileSystemSyncAccessHandle;
  try {
    handle = await openPcm16(p.sourceId);
  } catch (error) {
    d.free();
    throw error;
  }
  const file: PcmFile = {
    getSize: () => handle.getSize(),
    truncate: (bytes) => {
      handle.truncate(bytes);
    },
    write: (data, at) => {
      if (handle.write(data, { at }) !== data.byteLength) {
        throw new WorkerFailure("E_STORAGE_IO", "ShortWrite");
      }
    },
    read: (into, at) => handle.read(into, { at }),
    flush: () => {
      handle.flush();
    },
  };
  try {
    const result = await writeWholePcm16(
      file,
      d,
      p.info,
      p.fromSeconds,
      (doneMs) => {
        ctx.progress({ stage: "import", done: doneMs, total: p.info.duration });
      },
      () => undefined,
      () => ctx.isCancelled(),
      { resampler: (from, to) => core.newStreamResampler(from as Hz, to as Hz) },
    );
    return result === CANCELLED ? CANCELLED : { done: true, levelP95: result.levelP95 };
  } catch (thrown) {
    const failed = asSourceFailed(thrown);
    if (failed !== undefined) {
      return failed;
    }
    if (thrown instanceof DOMException && thrown.name === "QuotaExceededError") {
      throw new WorkerFailure("E_STORAGE_QUOTA", thrown.name);
    }
    throw thrown;
  } finally {
    handle.close();
    d.free();
  }
}

const handlers: Handlers<MediaWorkerApi> = {
  importAndProbe,
  extractAudio,
  routeFile,
  importAndRoute,
  extractSourcePcm16,
};

serveWorker(handlers, { stage: "import", oneWay: [], duringPreview: [] });
