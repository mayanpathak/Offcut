// The rule every review change goes through (v4implementation D-147,
// Appendix C.2): the clip holds the new edit before the worker answers, the
// scene is built from it, the edit is written to the device, and the change
// is counted. A second change waits for the one in flight. This is the part
// the two drags of the preview need (D-183); V4 Prompt 114 adds the pause and
// restore, the plan mode of a word edit, the save state and the history.

import { track } from "../analytics/client";
import type { ReviewActionKind } from "../gen/api";
import type { EditState } from "../gen/domain";
import * as editsRepo from "../persistence/edits-repo";
import { OpfsError } from "../persistence/opfs";
import * as clipStore from "../state/clip-store";
import { pool, WorkerCallError } from "../workers/pool";
import type { AppFailure } from "../workers/protocol";

/** What `review_action` names a change. `caption_position` joins the server's list in V4 Prompt 120 (§13). */
export type ReviewAction = ReviewActionKind | "caption_position";

/**
 * The actions the server's list holds now: the others are not sent, as the
 * allowlist would drop them. Typed against the generated list, so a member
 * it gains fails the build until it is named here.
 */
const SENT = { word_edit: true, event_toggle: true, style: true, crop_offset: true } satisfies Record<
  ReviewActionKind,
  true
>;

function sent(action: ReviewAction): action is ReviewActionKind {
  return action in SENT;
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

/** The change in flight, settled however it ended: the next one starts after it (D-147). */
let inFlight: Promise<void> = Promise.resolve();

async function apply(change: (edit: EditState) => EditState, action: ReviewAction): Promise<void> {
  // 1. Only a clip that is ready takes a change: the controls are disabled otherwise.
  const clip = clipStore.useClipStore.getState();
  const { clipId, transcript, plan, timeMap } = clip;
  const built = clipId !== undefined && transcript !== undefined && plan !== undefined && timeMap !== undefined;
  if (clip.status !== "ready" || !built) {
    return;
  }
  // 2. ready → updating, with the new edit.
  const previous = clip.edit;
  const next = change(previous);
  clipStore.editStarted(next);
  try {
    // 5. The scene of the new edit; the paused preview is drawn again with it (D-173).
    const report = await pool.render.setScene({ transcript, plan, edit: next, timeMap, profile: "preview" });
    if ("cancelled" in report) {
      throw new WorkerCallError({ code: "E_INTERNAL", stage: "preview", retryable: false, detail: "Cancelled" });
    }
    // 6.
    await editsRepo.put(clipId, { editState: next });
  } catch (error) {
    // The clip goes back to the edit before; the preview keeps its last good frame.
    const failure = failureOf(error);
    clipStore.editFailed(previous, failure ?? { code: "E_INTERNAL", stage: "preview", retryable: false });
    if (failure === null) {
      throw error;
    }
    return;
  }
  // 7, 8.
  clipStore.editDone();
  if (sent(action)) {
    track({ name: "review_action", props: { action } });
  }
}

/**
 * Applies `change` to the clip's edit and builds the scene from the result
 * (mode `scene`: nothing but the scene changes). Resolves when the change is
 * shown or has failed; a failure is stored on the clip, not thrown.
 */
export function update(change: (edit: EditState) => EditState, mode: "scene", action: ReviewAction): Promise<void> {
  const run = inFlight.then(() => apply(change, action));
  inFlight = run.catch(() => undefined);
  return run;
}
