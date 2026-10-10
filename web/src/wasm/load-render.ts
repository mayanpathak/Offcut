// Loading offcut_render.wasm. This is the only file that fetches, compiles or
// instantiates it.
//
// scripts/build-wasm.sh writes the module and its JavaScript glue to
// ./pkg/render/ (not in git). Vite serves the .wasm file from the app's own
// origin under a content-hashed name, as `application/wasm`.
//
// The main thread calls `preloadRender` only. `loadRender` is for the render
// worker: everything this module does is media work (INV-17).

import type {
  ChangeSummary,
  ClipInfo,
  DetectedEvent,
  EditState,
  ExportProfile,
  Prosody,
  Px,
  Transcript,
  UnixSecs,
} from "../gen/domain";
import initRender, {
  detect,
  export_profile_from_token,
  Mp4MuxerHandle,
  open_session,
  preview_profile,
  type RenderSession as ModuleSession,
} from "./pkg/render/offcut_render";
import renderWasmUrl from "./pkg/render/offcut_render_bg.wasm?url";

/** One compressed video frame, with its place on the clip's timeline in microseconds. */
export type VideoSample = { data: Uint8Array; ptsUs: number; durationUs: number; isKeyframe: boolean };

/** What a scene is built from, beside the clip the session was opened with. */
export type SceneInput = {
  transcript: Transcript;
  events: readonly DetectedEvent[];
  edit: EditState;
  profile: ExportProfile;
};

/**
 * One clip being drawn. This is the module's own object, with the types of
 * what it takes and gives written out; the method names are the module's.
 * `workers/render/video-source.ts` feeds a decoder through four of them
 * (v2implementation D-30). `free()` gives its memory in the module back. It
 * closes no `VideoFrame`.
 */
export type RenderSession = Omit<ModuleSession, "set_scene" | "summary" | "read_video_sample"> & {
  /** Builds the scene and keeps it in place of the one before. */
  set_scene(input: SceneInput): void;
  /** `null` before a scene is set. */
  summary(): ChangeSummary | null;
  read_video_sample(index: number): VideoSample;
};

/** The module's MP4 writer. `finalize()` returns the size of the file in bytes and frees the writer. */
export type { Mp4MuxerHandle };

/**
 * What the module offers. A failure is thrown as the plain object
 * `{ code, detail }` the module made (TS §11.1).
 */
export type RenderApi = {
  /** The visual events of a transcript. `edits` is the user's word edits, by word number. */
  detect(transcript: Transcript, prosody: Prosody, edits: Record<number, string>): DetectedEvent[];
  /** The profile of a token that one of `keys` verifies, at the time `now`. Throws `E_ENTITLEMENT_INVALID` for any other token. */
  exportProfileFromToken(token: string, keys: readonly Uint8Array[], now: UnixSecs): ExportProfile;
  previewProfile(): ExportProfile;
  openSession(clip: ClipInfo, source: FileSystemSyncAccessHandle): Promise<RenderSession>;
  newMuxer(
    sink: { writeAt(offset: number, data: Uint8Array): void },
    v: { width: Px; height: Px; avcc: Uint8Array; frameCountHint: number },
    asc: Uint8Array,
  ): Mp4MuxerHandle;
};

let compiled: Promise<WebAssembly.Module> | undefined;
let loaded: Promise<RenderApi> | undefined;

/** Fetches and compiles the module, once. A failed attempt is forgotten, so a later call tries again. */
function compile(): Promise<WebAssembly.Module> {
  compiled ??= WebAssembly.compileStreaming(fetch(renderWasmUrl)).catch((error: unknown) => {
    compiled = undefined;
    throw error;
  });
  return compiled;
}

/**
 * Fetches and compiles the module without running it, so the render worker
 * does not wait for the download when the first clip arrives. Safe to call
 * any number of times.
 */
export async function preloadRender(): Promise<void> {
  await compile();
}

const api: RenderApi = {
  detect: (transcript, prosody, edits) => detect(transcript, prosody, edits) as DetectedEvent[],
  exportProfileFromToken: (token, keys, now) => export_profile_from_token(token, [...keys], now) as ExportProfile,
  previewProfile: () => preview_profile() as ExportProfile,
  openSession: (clip, source) => open_session(clip, source),
  newMuxer: (sink, v, asc) => new Mp4MuxerHandle(sink, v.width, v.height, v.avcc, v.frameCountHint, asc),
};

/** Instantiates the compiled module, compiling it first if needed. For the render worker. */
export function loadRender(): Promise<RenderApi> {
  loaded ??= compile()
    .then((module) => initRender({ module_or_path: module }))
    .then((): RenderApi => api)
    .catch((error: unknown) => {
      loaded = undefined;
      throw error;
    });
  return loaded;
}
