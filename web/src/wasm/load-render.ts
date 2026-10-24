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
  Dropped,
  EditPlan,
  EditState,
  ExportProfile,
  OutMs,
  Prosody,
  Px,
  SentenceIdx,
  TimeMap,
  TimeMs,
  Transcript,
  UnixSecs,
  WordIdx,
} from "../gen/domain";
import initRender, {
  detect,
  export_profile_from_token,
  Mp4MuxerHandle,
  open_session,
  preview_profile,
  redetect_sentence,
  type RenderSession as ModuleSession,
  rules_plan,
} from "./pkg/render/offcut_render";
import renderWasmUrl from "./pkg/render/offcut_render_bg.wasm?url";

/** One compressed video frame, with its place on the clip's timeline in microseconds. */
export type VideoSample = { data: Uint8Array; ptsUs: number; durationUs: number; isKeyframe: boolean };

/**
 * What a scene is built from, beside the clip the session was opened with.
 * The plan is validated against the transcript as edited first (TS §40.4).
 */
export type SceneInput = {
  transcript: Transcript;
  plan: EditPlan;
  edit: EditState;
  profile: ExportProfile;
  timeMap: TimeMap;
};

/** Where the main video's source is at a program time (D-126). */
export type SourcePos = { at: TimeMs } | { held: TimeMs };

/** A box of the canvas, as fractions of its width and height. */
type CanvasBox = { x: number; y: number; w: number; h: number };

/** What a point of the canvas is on (D-174): a word with its glyph box, the caption block with its box, or the video. */
export type SceneHit =
  | { kind: "word"; word: WordIdx; rect: CanvasBox }
  | { kind: "captions"; rect: CanvasBox }
  | { kind: "video" };

/**
 * One clip being drawn. This is the module's own object, with the types of
 * what it takes and gives written out; the method names are the module's.
 * `workers/render/video-source.ts` feeds a decoder through four of them
 * (v2implementation D-30). `free()` gives its memory in the module back. It
 * closes no `VideoFrame`.
 */
export type RenderSession = Omit<
  ModuleSession,
  "set_scene" | "summary" | "read_video_sample" | "source_at" | "render_frame" | "resize" | "hit" | "caption_mirror"
> & {
  /** Builds the scene and keeps it in place of the one before; what the plan's validation left out. */
  set_scene(input: SceneInput): { dropped: Dropped[] };
  /** `null` before a scene is set. */
  summary(): ChangeSummary | null;
  read_video_sample(index: number): VideoSample;
  /** Where the source is at the program time `tOut`; `null` before a scene is set. */
  source_at(tOut: OutMs): SourcePos | null;
  /** Draws what the scene shows at the program time `tOut`, with `frame` as its main video. */
  render_frame(frame: VideoFrame, tOut: OutMs): void;
  /** Draws at this size from now on, on the same canvas; the scene is set again at it (v4implementation D-149). */
  resize(width: Px, height: Px): void;
  /** What the point `(x, y)`, fractions of the canvas, is on at `tOut`; `null` outside it or before a scene is set. */
  hit(tOut: OutMs, x: number, y: number): SceneHit | null;
  /** The caption shown at `tOut` as drawn, for development builds' tests (D-155); `null` when none is shown. */
  caption_mirror(tOut: OutMs): { text: string; colors: string[] } | null;
};

/** The module's MP4 writer. `finalize()` returns the size of the file in bytes and frees the writer. */
export type { Mp4MuxerHandle };

/**
 * What the module offers. A failure is thrown as the plain object
 * `{ code, detail }` the module made (TS §11.1).
 */
export type RenderApi = {
  /** The plan of the detector's events (D-124). */
  rulesPlan(transcript: Transcript, events: readonly DetectedEvent[]): EditPlan;
  /** The visual events of a transcript. `edits` is the user's word edits, by word number. */
  detect(transcript: Transcript, prosody: Prosody, edits: Record<number, string>): DetectedEvent[];
  /**
   * The events after an edit in sentence `sentence`: those of `prev` outside
   * its scope are kept, the scope is detected again. A sentence past the last
   * returns `prev`. Nothing calls it in V3 (V4's review flow).
   */
  redetectSentence(
    transcript: Transcript,
    prosody: Prosody,
    edits: Record<number, string>,
    sentence: SentenceIdx,
    prev: readonly DetectedEvent[],
  ): DetectedEvent[];
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
  rulesPlan: (transcript, events) => rules_plan(transcript, [...events]) as EditPlan,
  detect: (transcript, prosody, edits) => detect(transcript, prosody, edits) as DetectedEvent[],
  redetectSentence: (transcript, prosody, edits, sentence, prev) =>
    redetect_sentence(transcript, prosody, edits, sentence, [...prev]) as DetectedEvent[],
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
