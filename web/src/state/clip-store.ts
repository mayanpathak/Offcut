// The clip being worked on: where it is on its way, what was found in it,
// and the last thing that went wrong (TS §12.1). Written by the use-cases
// through the actions below; a component reads it and never writes it.

import { create } from "zustand";

import type { ClipSource } from "../gen/api";
import type {
  ClipId,
  ClipInfo,
  DetectedEvent,
  EditPlan,
  EditState,
  PipelineStage,
  Prosody,
  RejectReason,
  TimeMap,
  Transcript,
} from "../gen/domain";
import type { AppFailure, FeedLine } from "../workers/protocol";
import { clear as clearHistory } from "./history";
import { CLIP_MACHINE, type ClipEvent, type ClipStatus } from "./machines/clip-machine";
import { transition } from "./machines/transition";

export type ClipState = {
  status: ClipStatus;
  clipId?: ClipId;
  source?: ClipSource;
  clipInfo?: ClipInfo;
  /** The stage of the pipeline that is running. Present in `processing` only. */
  stage?: PipelineStage;
  /** The pipeline is held up by the download of the speech model. */
  waitingModel: boolean;
  transcript?: Transcript;
  prosody?: Prosody;
  events: readonly DetectedEvent[];
  /**
   * The plan the scene is built from: the rules plan of `events` in V4
   * (D-124). In memory only; a restored clip makes it again (D-82).
   */
  plan?: EditPlan;
  /** How the program plays the recording: the identity in V4 (D-164). */
  timeMap?: TimeMap;
  /** The user's edits: the default until a review change (D-147). */
  edit: EditState;
  /** The lines of the processing feed, in the order they came. */
  feed: readonly FeedLine[];
  reject?: RejectReason;
  failure?: AppFailure;
  /**
   * The program's audio at 48 kHz, for the preview, while the clip is
   * `ready` (D-127). The export reads the cleaned voice from the device,
   * `out48.f32` (v2implementation D-51, D-129).
   */
  prog48: Float32Array | null;
};

/** What a state holds beside its status. */
type Fields = Omit<ClipState, "status">;

/** No clip. `reset()` returns to exactly this, so nothing of the last clip is kept. */
function empty(): Fields {
  return {
    waitingModel: false,
    events: [],
    edit: { word_edits: {}, event_overrides: {}, style: "clean", crop_offset: 0, caption_position: 1, highlight_numbers: true },
    feed: [],
    prog48: null,
  };
}

export const useClipStore = create<ClipState>(() => ({ status: "idle", ...empty() }));

/**
 * Whether the crop offset moves the picture: a frame wider than 9:16 loses
 * its sides, and the offset says which (D-131). A narrower one ignores it.
 * The rule is `crop_rect`'s, in `offcut-scene/src/framing.rs`.
 */
export function cropMoves(info: ClipInfo): boolean {
  return info.display_width * 16 > info.display_height * 9;
}

/**
 * Applies `event`. `next` gives what the state holds with its new status,
 * from what it holds now. It is stored only if the transition was legal, and
 * it is stored whole: a field `next` leaves out is dropped.
 */
function apply(event: ClipEvent, next: (current: Fields) => Fields): void {
  const { status: from, ...current } = useClipStore.getState();
  const status = transition("clip", CLIP_MACHINE, from, event);
  // No legal transition leads back to the same state: an unchanged state
  // means the attempt was illegal and was reported.
  if (status !== from) {
    useClipStore.setState({ ...next(current), status }, true);
  }
}

/** What the state holds once no stage of the pipeline is running. */
function settled(current: Fields): Fields {
  const rest: Fields = { ...current, waitingModel: false };
  delete rest.stage;
  return rest;
}

/** idle → importing */
export function begin(clipId: ClipId, source: ClipSource): void {
  apply("import", () => ({ ...empty(), clipId, source }));
  // The history is of one clip (D-184).
  clearHistory();
}

/** importing → accepted */
export function accepted(info: ClipInfo): void {
  apply("accept", (current) => ({ ...settled(current), clipInfo: info }));
}

/** importing → rejected: the clip cannot be used, and why. */
export function rejected(reason: RejectReason): void {
  apply("reject", (current) => ({ ...settled(current), reject: reason }));
}

/** importing | processing → failed */
export function failed(failure: AppFailure): void {
  apply("fail", (current) => ({ ...settled(current), failure }));
}

/**
 * accepted → processing, at `stage`. While the clip is processing already,
 * the stage and the wait for the model are what change, not the state.
 */
export function processing(stage: PipelineStage, waitingModel: boolean): void {
  if (useClipStore.getState().status === "processing") {
    useClipStore.setState({ stage, waitingModel });
  } else {
    apply("run", (current) => ({ ...current, stage, waitingModel }));
  }
}

/** One more line of the feed. A line is appended and never dropped (TS §14.3). */
export function pushFeed(line: FeedLine): void {
  useClipStore.setState((current) => ({ feed: [...current.feed, line] }));
}

/** processing → ready */
export function ready(result: {
  transcript: Transcript;
  prosody: Prosody;
  events: readonly DetectedEvent[];
  timeMap: TimeMap;
  prog48: Float32Array;
}): void {
  apply("done", (current) => ({ ...settled(current), ...result }));
}

/** The plan the scene is built from now. The clip's state does not change. */
export function setPlan(plan: EditPlan): void {
  useClipStore.setState({ plan });
}

/** The events found now, after an edit (D-146). The clip's state does not change. */
export function setEvents(events: readonly DetectedEvent[]): void {
  useClipStore.setState({ events });
}

/** ready → updating, holding the edit asked for before the worker answers (D-147: edits are optimistic). */
export function editStarted(edit: EditState): void {
  apply("edit_start", (current) => ({ ...current, edit }));
}

/** updating → ready: the scene shows the edit. */
export function editDone(): void {
  apply("edit_done", (current) => current);
}

/** updating → ready with the edit before, and why the change did not apply (D-147). */
export function editFailed(previous: EditState, failure: AppFailure): void {
  apply("edit_done", (current) => ({ ...current, edit: previous, failure }));
}

/** processing → rejected: fewer words were heard than a clip needs. */
export function noSpeech(): void {
  apply("no_speech", (current) => ({ ...settled(current), reject: "REJECT_NO_SPEECH" }));
}

/**
 * A failure that does not change the clip's state: the preview's, with the
 * clip still `ready`. The clip machine has no way from `ready` to `failed`.
 */
export function noteFailure(failure: AppFailure): void {
  useClipStore.setState({ failure });
}

/** rejected | failed | ready | accepted → idle. Nothing of the clip is kept, its audio neither. */
export function reset(): void {
  apply("reset", empty);
  clearHistory();
}
