// The rule every review change goes through (v4implementation D-147,
// Appendix C.2). A preview that plays is paused for the change and plays
// again after it; the clip holds the new edit before the worker answers
// (edits are optimistic and local, TS §13.4); a word edit finds the events
// and the plan again (mode `plan`), every change builds the scene again; the
// edit is written to the device, the change can be undone (D-184), and it is
// counted. A second change waits for the one in flight, and each of its
// calls waits for the render worker to be free. Nothing here awaits the
// network.

import { track } from "../analytics/client";
import type { ReviewActionKind } from "../gen/api";
import type { DetectedEvent, EditPlan, EditState, EventKind, SentenceIdx, Transcript } from "../gen/domain";
import * as editsRepo from "../persistence/edits-repo";
import { OpfsError } from "../persistence/opfs";
import * as clipStore from "../state/clip-store";
import * as history from "../state/history";
import { usePreviewStore } from "../state/preview-store";
import { useUiStore } from "../state/ui-store";
import { type Cancelled, pool, WorkerCallError, whenFree } from "../workers/pool";
import type { AppFailure } from "../workers/protocol";
import { pause, play } from "./control-preview";

/** What `review_action` names a change: one of the server's list, which holds every review control since Prompt 120 (§13). */
export type ReviewAction = ReviewActionKind;

/** What a toggle's `review_action` says beside its name: the kind of the visual and whether it is on (enums only, INV-18). */
export type ReviewDetail = { event_kind: EventKind; enabled: boolean };

/** The one place of this file that makes a place in `transcript.sentences` a `SentenceIdx` (v2implementation D-59). */
function sentenceAt(i: number): SentenceIdx {
  return i as SentenceIdx;
}

/** What was thrown, as the failure the clip store keeps; `null` for what is a fault of this file. */
function failureOf(error: unknown): AppFailure | null {
  if (error instanceof WorkerCallError) {
    return error.failure;
  }
  if (error instanceof OpfsError) {
    return { code: error.kind === "quota" ? "E_STORAGE_QUOTA" : "E_STORAGE_IO", stage: "storage", retryable: true };
  }
  return null;
}

/** A worker's answer, or the failure of a call that was cancelled. */
function answered<T>(result: T | Cancelled): T {
  if (typeof result === "object" && result !== null && "cancelled" in result) {
    throw new WorkerCallError({ code: "E_INTERNAL", stage: "preview", retryable: false, detail: "Cancelled" });
  }
  return result;
}

/** Whether two edits have the same word edits: if not, the events and the plan are found again (D-184). */
export function sameWords(a: EditState, b: EditState): boolean {
  const left = Object.entries(a.word_edits);
  const right = new Map(Object.entries(b.word_edits));
  return left.length === right.size && left.every(([word, text]) => right.get(word) === text);
}

/**
 * The events of the transcript with `next`'s word edits (D-146, D-175): the
 * one sentence whose words changed detected again, or, when several did
 * (a replace-all, an undo of one), the whole transcript.
 */
async function eventsFor(transcript: Transcript, previous: EditState, next: EditState): Promise<DetectedEvent[]> {
  const before = new Map(Object.entries(previous.word_edits));
  const after = new Map(Object.entries(next.word_edits));
  const changed = [...new Set([...before.keys(), ...after.keys()])].filter((w) => before.get(w) !== after.get(w));
  const sentences = new Set(
    changed.map((w) => transcript.sentences.findIndex((s) => s.words.start <= Number(w) && Number(w) < s.words.end)),
  );
  const [only] = sentences;
  if (sentences.size === 1 && only !== undefined && only >= 0) {
    const sentence = sentenceAt(only);
    return answered(await whenFree(() => pool.render.redetectSentence({ sentence, wordEdits: next.word_edits })));
  }
  const { prosody } = clipStore.useClipStore.getState();
  if (prosody === undefined) {
    throw new Error("review-update: a ready clip has no prosody");
  }
  return answered(await whenFree(() => pool.render.detect({ transcript, prosody, wordEdits: next.word_edits })));
}

/** The change in flight, settled however it ended: the next one starts after it (D-147). */
let inFlight: Promise<void> = Promise.resolve();

async function apply(
  change: (edit: EditState) => EditState,
  mode: "plan" | "scene",
  action: ReviewAction,
  detail: ReviewDetail | undefined,
  joinsLastStep: ((previous: EditState) => boolean) | undefined,
): Promise<void> {
  // 1. Only a clip that is ready takes a change, and none while an export
  //    has the preview locked (TS §12.4): the controls are disabled then.
  const clip = clipStore.useClipStore.getState();
  const { clipId, transcript, timeMap, plan: planBefore } = clip;
  const built = clipId !== undefined && transcript !== undefined && planBefore !== undefined && timeMap !== undefined;
  if (clip.status !== "ready" || !built || usePreviewStore.getState().status === "locked") {
    return;
  }
  // 2. ready → updating, with the new edit.
  const previous = clip.edit;
  const next = change(previous);
  clipStore.editStarted(next);
  // 3.
  const resume = usePreviewStore.getState().status === "playing";
  if (resume) {
    await pause();
  }
  const ui = useUiStore.getState();
  try {
    // 4. A word edit: the events and their plan, set on the clip once the scene shows them.
    let events: readonly DetectedEvent[] = clip.events;
    let plan: EditPlan = planBefore;
    if (mode === "plan") {
      const found = await eventsFor(transcript, previous, next);
      events = found;
      plan = answered(await whenFree(() => pool.render.rulesPlan({ transcript, events: found })));
    }
    // 5. The paused preview is drawn again with the new scene (D-173).
    const report = answered(
      await whenFree(() => pool.render.setScene({ transcript, plan, edit: next, timeMap, profile: "preview" })),
    );
    clipStore.setChunks(report.chunks);
    if (mode === "plan") {
      clipStore.setEvents(events);
      clipStore.setPlan(plan);
    }
    // 6. The save state shows while the write is pending (D-184).
    ui.setSave("saving");
    try {
      await editsRepo.put(clipId, { editState: next });
    } finally {
      // Written or not, the device holds the edit the clip shows: the new one, or the one before.
      ui.setSave("saved");
    }
  } catch (error) {
    // The clip goes back to the edit before; the preview keeps its last good
    // frame. The card that shows the failure sends `client_error` (D-153).
    const failure = failureOf(error);
    clipStore.editFailed(previous, failure ?? { code: "E_INTERNAL", stage: "preview", retryable: false });
    if (mode === "plan") {
      // The worker kept the events it found for the next edit: give it back those of the edit before.
      const { prosody } = clipStore.useClipStore.getState();
      if (prosody !== undefined) {
        await whenFree(() => pool.render.detect({ transcript, prosody, wordEdits: previous.word_edits })).catch(
          () => undefined,
        );
      }
    }
    if (failure === null) {
      throw error;
    }
    return;
  }
  // 7.
  clipStore.editDone();
  if (resume) {
    await play();
  }
  // An undo or a redo moves along the history itself (undo-redo.ts); a
  // change that joins the step before it is undone with it (D-175).
  if (action !== "undo" && action !== "redo" && joinsLastStep?.(previous) !== true) {
    history.push(previous);
  }
  // 8.
  track({ name: "review_action", props: { action, ...detail } });
}

/**
 * Applies `change` to the clip's edit (D-147). Mode `plan` (word edits) finds
 * the events and the plan again before the scene is built; mode `scene` (a
 * toggle, the style, the framing) builds the scene only. Resolves when the
 * change is shown or has failed: a failure is stored on the clip, not
 * thrown; a fault of this file is thrown. `joinsLastStep`, given the edit
 * before the change, says whether the change belongs to the history's last
 * step: then no step of its own is pushed, and one undo takes both back.
 */
export function update(
  change: (edit: EditState) => EditState,
  mode: "plan" | "scene",
  action: ReviewAction,
  detail?: ReviewDetail,
  joinsLastStep?: (previous: EditState) => boolean,
): Promise<void> {
  const run = inFlight.then(() => apply(change, mode, action, detail, joinsLastStep));
  inFlight = run.catch(() => undefined);
  return run;
}

/**
 * "Undo this change" (D-190), after a change failed: the clip holds the edit
 * before it already (`editFailed`), and the scene is built from that edit
 * again, so the preview shows what the clip holds. It waits for a change in
 * flight, as a change does. Another failure is stored as the first was.
 */
export function undoFailedChange(): Promise<void> {
  const run = inFlight.then(async () => {
    const clip = clipStore.useClipStore.getState();
    const { transcript, plan, timeMap, edit } = clip;
    if (clip.status !== "ready" || transcript === undefined || plan === undefined || timeMap === undefined) {
      return;
    }
    try {
      const report = answered(
        await whenFree(() => pool.render.setScene({ transcript, plan, edit, timeMap, profile: "preview" })),
      );
      clipStore.setChunks(report.chunks);
    } catch (error) {
      const failure = failureOf(error);
      if (failure === null) {
        throw error;
      }
      clipStore.noteFailure(failure, "edit");
      return;
    }
    clipStore.clearFailure();
  });
  inFlight = run.catch(() => undefined);
  return run;
}

/** Resolves when no change is in flight. */
export function settled(): Promise<void> {
  return inFlight;
}
