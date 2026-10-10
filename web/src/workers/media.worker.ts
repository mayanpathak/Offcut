// The media worker: copies a dropped file to OPFS, probes and validates it,
// and decodes its audio to PCM at 48 kHz and 16 kHz (TS §15). The rules are
// in Rust; this file is the browser glue around them.

import { type Bytes, type ClipId, type ClipInfo, type Hz, LIMITS, type RejectReason } from "../gen/domain";
import { type CoreApi, loadCore } from "../wasm/load-core";
import { decodeAudio } from "./media/audio-decode";
import { byteSize, importToOpfs, openSource } from "./media/import";
import type { MediaWorkerApi } from "./protocol";
import { CANCELLED, type Handlers, type JobContext, serveWorker, WorkerFailure } from "./rpc";

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

const handlers: Handlers<MediaWorkerApi> = { importAndProbe, extractAudio };

serveWorker(handlers, { stage: "import", oneWay: [], duringPreview: [] });
