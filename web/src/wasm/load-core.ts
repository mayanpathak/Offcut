// Loading offcut_core.wasm. This is the only file that fetches, compiles or
// instantiates it.
//
// scripts/build-wasm.sh writes the module and its JavaScript glue to
// ./pkg/core/ (not in git). Vite serves the .wasm file from the app's own
// origin under a content-hashed name, as `application/wasm`.

import {
  type Bytes,
  type ClipInfo,
  type Dbfs,
  type Hz,
  type Lufs,
  type OutSpan,
  type ProbeInfo,
  type Prosody,
  REJECT_REASONS,
  type RejectReason,
  type Sentence,
  type TimeMap,
  type Transcript,
  type VoiceMode,
  type Word,
} from "../gen/domain";
import type { Entry, RouteFlags, RouteReject, SourceInfo, VideoDetail } from "../gen/domain";
import initCore, {
  assemble,
  assemble_voice,
  core_version,
  type DemuxerHandle,
  level_only,
  open_demuxer_reader,
  route_probe,
  type SourceDemuxerHandle,
  StreamResampler,
  measure_prosody,
  normalize_transcript,
  open_demuxer,
  probe_and_validate,
  resample,
  run_chain,
  Sha256Stream,
} from "./pkg/core/offcut_core";
import coreWasmUrl from "./pkg/core/offcut_core_bg.wasm?url";

/** One compressed audio frame, with its place on the clip's timeline in microseconds. */
export type AudioSample = { data: Uint8Array; ptsUs: number; durationUs: number };

/** One word as the recognizer returned it (TS §16.2). The times are whole milliseconds. */
export type RawWord = { text: string; startMs: number; endMs: number; confidence: number };

/** What the voice chain returns (TS §18): the cleaned voice in its own buffer, or that it was cancelled. */
export type ChainResult = { out48: Float32Array<ArrayBuffer>; loudness: Lufs; peak: Dbfs } | { cancelled: true };

/** A file the demuxer has opened. `free()` gives its memory in the module back. */
export type CoreDemuxer = {
  probe(fileSize: Bytes): ProbeInfo;
  /** The raw `avcC` payload. */
  videoDescription(): Uint8Array | undefined;
  /** The `AudioSpecificConfig`. */
  audioDescription(): Uint8Array | undefined;
  audioSampleCount(): number;
  readAudioSample(index: number): AudioSample;
  free(): void;
  /** V6.1 (D-398): HDR and the HEVC string, for the clip page with `high_res`. */
  videoDetail(): VideoDetail;
  /** V6.1: the raw `hvcC` payload of an HEVC track. */
  hevcDescription(): Uint8Array | undefined;
};

/** V6.1 (D-399, D-400): what the demuxer reads a dropped `File` through, in place. */
export type ByteReader = { size(): number; readAt(offset: number, length: number): Uint8Array };

/** V6.1: a long source the demuxer opened in place. `free()` gives its memory back. */
export type CoreSourceDemuxer = {
  probe(fileSize: Bytes): ProbeInfo;
  videoDetail(): VideoDetail;
  hevcDescription(): Uint8Array | undefined;
  videoDescription(): Uint8Array | undefined;
  audioDescription(): Uint8Array | undefined;
  audioSampleCount(): number;
  /** When sample `index` is presented, in microseconds; `undefined` past the end. */
  audioSampleTimeUs(index: number): number | undefined;
  readAudioSample(index: number): AudioSample;
  free(): void;
};

/** V6.1 (D-401): `resample_mono` in pieces. After `finish` the object is spent. */
export type CoreStreamResampler = {
  push(input: Float32Array): Float32Array;
  finish(totalInput: number): Float32Array;
  free(): void;
};

/** V6.1: which path a file takes, or why it is refused. */
export type Routed = { clip: ClipInfo } | { long: SourceInfo } | { rejected: RouteReject };

/**
 * What the module offers. A rejection is a value; a failure is thrown as the
 * plain object `{ code, detail }` the module made (TS §11.1, §11.3).
 */
export type CoreApi = {
  coreVersion(): string;
  openDemuxer(handle: FileSystemSyncAccessHandle): CoreDemuxer | { rejected: RejectReason };
  probeAndValidate(
    d: CoreDemuxer,
    fileSize: Bytes,
    decodeSupported: boolean,
  ): { ok: ClipInfo } | { rejected: RejectReason };
  resample(input: Float32Array, from: Hz, to: Hz): Float32Array;
  /** The transcript of these words: their sentences, and the numbers that were spoken. */
  normalizeTranscript(raw: readonly RawWord[], modelVersion: string): Transcript;
  newSha256(): { update(chunk: Uint8Array): void; finalizeHex(): string };
  /** The voice chain (TS §18). `onBlock` gets the samples done so far once per second of audio; `true` cancels. */
  runChain(pcm48: Float32Array, onBlock: (done: number) => boolean): ChainResult;
  /** Per-word prosody of the cleaned audio. Reads only. */
  measureProsody(out48: Float32Array, words: readonly Word[], sentences: readonly Sentence[]): Prosody;
  /**
   * The program's audio: `out48` played on `timeMap`, in a buffer of its own,
   * and its loudness (TS §40.6). V4 takes the identity map only; any other
   * is thrown as `E_DSP` `Unsupported`.
   */
  assemble(
    out48: Float32Array,
    timeMap: TimeMap,
    speech: readonly OutSpan[],
  ): { prog48: Float32Array<ArrayBuffer>; loudness: Lufs };
  /** V6.2 (D-487): `pcm48` brought to the loudness target under the ceiling, nothing else: `lvl48`. */
  levelOnly(pcm48: Float32Array): { lvl48: Float32Array<ArrayBuffer>; loudness: Lufs };
  /** V6.2 (D-487): `assemble` on the voice `voice` names: `voice48` is `out48` ("clean") or `lvl48` ("level"). */
  assembleVoice(
    voice48: Float32Array,
    voice: VoiceMode,
    timeMap: TimeMap,
    speech: readonly OutSpan[],
  ): { prog48: Float32Array<ArrayBuffer>; loudness: Lufs };
  /** V6.1: opens a dropped file through `reader`, copying nothing (D-399). */
  openDemuxerReader(reader: ByteReader): CoreSourceDemuxer | { rejected: RejectReason };
  /** V6.1: `route_probe` (TS §43.1) over a probe and a detail from either demuxer. */
  routeProbe(probe: ProbeInfo, detail: VideoDetail, decodeSupported: boolean, entry: Entry, flags: RouteFlags): Routed;
  /** V6.1: a resampler fed in pieces. Throws `E_SOURCE_AUDIO` for a rate of 0. */
  newStreamResampler(from: Hz, to: Hz): CoreStreamResampler;
};

let compiled: Promise<WebAssembly.Module> | undefined;
let loaded: Promise<CoreApi> | undefined;

/** Fetches and compiles the module, once. A failed attempt is forgotten, so a later call tries again. */
function compile(): Promise<WebAssembly.Module> {
  compiled ??= WebAssembly.compileStreaming(fetch(coreWasmUrl)).catch((error: unknown) => {
    compiled = undefined;
    throw error;
  });
  return compiled;
}

/**
 * Fetches and compiles the module without running it, so the first worker
 * that needs it does not wait for the download. Compiling is the step the
 * CSP must allow (`'wasm-unsafe-eval'`). Safe to call any number of times.
 */
export async function preloadCore(): Promise<void> {
  await compile();
}

// The module's own object behind each demuxer this file handed out.
const handles = new WeakMap<CoreDemuxer, DemuxerHandle>();

function isRejection(thrown: unknown): thrown is { rejected: RejectReason } {
  if (typeof thrown !== "object" || thrown === null || !("rejected" in thrown)) {
    return false;
  }
  const reasons: readonly unknown[] = REJECT_REASONS;
  return reasons.includes(thrown.rejected);
}

function wrapDemuxer(handle: DemuxerHandle): CoreDemuxer {
  const demuxer: CoreDemuxer = {
    probe: (fileSize) => handle.probe(fileSize) as ProbeInfo,
    videoDescription: () => handle.video_description(),
    audioDescription: () => handle.audio_description(),
    audioSampleCount: () => handle.audio_sample_count(),
    readAudioSample: (index) => handle.read_audio_sample(index) as AudioSample,
    free: () => {
      handles.delete(demuxer);
      handle.free();
    },
    videoDetail: () => handle.video_detail() as VideoDetail,
    hevcDescription: () => handle.hevc_description(),
  };
  handles.set(demuxer, handle);
  return demuxer;
}

function wrapSource(handle: SourceDemuxerHandle): CoreSourceDemuxer {
  return {
    probe: (fileSize) => handle.probe(fileSize) as ProbeInfo,
    videoDetail: () => handle.video_detail() as VideoDetail,
    hevcDescription: () => handle.hevc_description(),
    videoDescription: () => handle.video_description(),
    audioDescription: () => handle.audio_description(),
    audioSampleCount: () => handle.audio_sample_count(),
    audioSampleTimeUs: (index) => handle.audio_sample_time_us(index),
    readAudioSample: (index) => handle.read_audio_sample(index) as AudioSample,
    free: () => {
      handle.free();
    },
  };
}

function wrapResampler(resampler: StreamResampler): CoreStreamResampler {
  let spent = false;
  return {
    push: (input) => resampler.push(input),
    finish: (totalInput) => {
      // The module frees the resampler when it hands the tail over.
      spent = true;
      return resampler.finish(totalInput);
    },
    free: () => {
      if (!spent) {
        spent = true;
        resampler.free();
      }
    },
  };
}

const api: CoreApi = {
  coreVersion: core_version,
  openDemuxer(handle) {
    try {
      return wrapDemuxer(open_demuxer(handle));
    } catch (thrown) {
      // A file that is not an MP4 or MOV, or a broken one: a value. Anything
      // else is a failure and goes on to the caller as it was thrown.
      if (isRejection(thrown)) {
        return thrown;
      }
      throw thrown;
    }
  },
  probeAndValidate(d, fileSize, decodeSupported) {
    const handle = handles.get(d);
    if (handle === undefined) {
      throw new Error("probeAndValidate: this demuxer was freed, or was not opened by openDemuxer");
    }
    return probe_and_validate(handle, fileSize, decodeSupported) as { ok: ClipInfo } | { rejected: RejectReason };
  },
  resample: (input, from, to) => resample(input, from, to),
  normalizeTranscript: (raw, modelVersion) => normalize_transcript(raw, modelVersion) as Transcript,
  newSha256() {
    const stream = new Sha256Stream();
    return {
      update: (chunk) => {
        stream.update(chunk);
      },
      // The module frees the stream when it hands the digest over.
      finalizeHex: () => stream.finalize_hex(),
    };
  },
  runChain: (pcm48, onBlock) => run_chain(pcm48, onBlock) as ChainResult,
  measureProsody: (out48, words, sentences) => measure_prosody(out48, words, sentences) as Prosody,
  assemble: (out48, timeMap, speech) =>
    assemble(out48, timeMap, [...speech]) as { prog48: Float32Array<ArrayBuffer>; loudness: Lufs },
  levelOnly: (pcm48) => level_only(pcm48) as { lvl48: Float32Array<ArrayBuffer>; loudness: Lufs },
  assembleVoice: (voice48, voice, timeMap, speech) =>
    assemble_voice(voice48, voice, timeMap, [...speech]) as { prog48: Float32Array<ArrayBuffer>; loudness: Lufs },
  openDemuxerReader(reader) {
    try {
      return wrapSource(open_demuxer_reader(reader));
    } catch (thrown) {
      if (isRejection(thrown)) {
        return thrown;
      }
      throw thrown;
    }
  },
  routeProbe: (probe, detail, decodeSupported, entry, flags) =>
    route_probe(probe, detail, decodeSupported, entry, flags) as Routed,
  newStreamResampler: (from, to) => wrapResampler(new StreamResampler(from, to)),
};

/**
 * Instantiates the compiled module, compiling it first if needed. For the
 * workers, and for the model manager, which hashes a downloaded file on the
 * main thread in short slices (v2implementation D-36). That is the one use
 * on the main thread: no media work runs there (INV-17).
 */
export function loadCore(): Promise<CoreApi> {
  loaded ??= compile()
    .then((module) => initCore({ module_or_path: module }))
    .then((): CoreApi => api)
    .catch((error: unknown) => {
      loaded = undefined;
      throw error;
    });
  return loaded;
}
