// Loading offcut_core.wasm. This is the only file that fetches, compiles or
// instantiates it.
//
// scripts/build-wasm.sh writes the module and its JavaScript glue to
// ./pkg/core/ (not in git). Vite serves the .wasm file from the app's own
// origin under a content-hashed name, as `application/wasm`.

import {
  type Bytes,
  type ClipInfo,
  type Hz,
  type ProbeInfo,
  REJECT_REASONS,
  type RejectReason,
  type Transcript,
} from "../gen/domain";
import initCore, {
  core_version,
  type DemuxerHandle,
  normalize_transcript,
  open_demuxer,
  probe_and_validate,
  resample,
  Sha256Stream,
} from "./pkg/core/offcut_core";
import coreWasmUrl from "./pkg/core/offcut_core_bg.wasm?url";

/** One compressed audio frame, with its place on the clip's timeline in microseconds. */
export type AudioSample = { data: Uint8Array; ptsUs: number; durationUs: number };

/** One word as the recognizer returned it (TS §16.2). The times are whole milliseconds. */
export type RawWord = { text: string; startMs: number; endMs: number; confidence: number };

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
};

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
  };
  handles.set(demuxer, handle);
  return demuxer;
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
