// The clip being worked on: where it is on its way, what was found in it,
// and the last thing that went wrong (TS §12.1). Written by the use-cases
// through the actions below; a component reads it and never writes it.

import { create } from "zustand";

import type { ClipSource } from "../gen/api";
import type {
  ClipId,
  ClipInfo,
  DetectedEvent,
  EditState,
  PipelineStage,
  Prosody,
  RejectReason,
  Transcript,
} from "../gen/domain";
import type { AppFailure, FeedLine } from "../workers/protocol";
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
  /** The user's edits. V2 has none: this is the default and never changes. */
  edit: EditState;
  /** The lines of the processing feed, in the order they came. */
  feed: readonly FeedLine[];
  reject?: RejectReason;
  failure?: AppFailure;
  /**
   * The clip's audio at 48 kHz, for the preview, while the clip is `ready`
   * (v2implementation D-51). The export reads its own copy from the device.
   */
  out48: Float32Array | null;
};

/** What a state holds beside its status. */
type Fields = Omit<ClipState, "status">;

/** No clip. `reset()` returns to exactly this, so nothing of the last clip is kept. */
function empty(): Fields {
  return {
    waitingModel: false,
    events: [],
    edit: { word_edits: {}, event_overrides: {}, style: "clean", crop_offset: 0 },
    feed: [],
    out48: null,
  };
}

export const useClipStore = create<ClipState>(() => ({ status: "idle", ...empty() }));

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
  out48: Float32Array;
}): void {
  apply("done", (current) => ({ ...settled(current), ...result }));
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
}
