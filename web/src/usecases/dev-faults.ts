// The faults a development build can cause on purpose (v4implementation
// D-155, G 14.5, Prompt 124): a worker that crashes, one that hangs, and a
// scene that fails to build once. Only `ui/dev-hooks.ts` calls these, and a
// production build has neither it nor the workers' halves (rpc-worker.ts).

import { setAiPlanSwitch, setEnhanceDefault } from "../persistence/prefs-repo";
import { setPlanSwitch } from "../state/plan-store";
import { pool, whenFree } from "../workers/pool";
import type { ImageCheckResult, VideoMirror, WorkerKind } from "../workers/protocol";

/** Worker `kind` throws where nothing catches it: the page sees its crash (D-153). */
export function crashWorker(kind: WorkerKind): void {
  pool[kind].notify("devCrash", {});
}

/** Worker `kind` stops answering: a cancel then restarts it after 2 s (D-152). */
export function hangWorker(kind: WorkerKind): void {
  pool[kind].notify("devHang", {});
}

/** An export's video encoder fails at `frame`, on the next `times` attempts (v5implementation D-218). */
export function failEncoderAt(frame: number, times: number): void {
  pool.render.notify("devFailEncoder", { at: frame, times });
}

/** The render worker's next `setScene` fails: a review change then fails as one can (D-190). */
export function failNextSetScene(): void {
  pool.render.notify("devFailNext", { method: "setScene" });
}

/**
 * V6.2 (TE-30, v6.2implementation D-474): the render worker draws the frame of
 * the `image_basic` golden and reads it back, for `image-parity.spec.ts`.
 */
/** V6.2 (P209, DC-16): the main video at `at`, for `enhance-panel.spec.ts`; `null` before a scene. */
export async function videoMirror(at: number): Promise<VideoMirror | null> {
  const result = await whenFree(() => pool.render.videoMirror({ atMs: at }));
  if (result !== null && "cancelled" in result) {
    throw new Error("the video mirror was cancelled");
  }
  return result;
}

/**
 * V6.2 (P209, DC-16): the next clip is made as V5 made it: `Enhance::v5()`
 * as its start and the AI plan switch off, for `flags-identity.spec.ts`.
 */
export async function startAsV5(): Promise<void> {
  setPlanSwitch(false);
  await setAiPlanSwitch(false);
  await setEnhanceDefault({ voice: "clean", motion: false });
}

export async function imageCheck(): Promise<ImageCheckResult> {
  const result = await pool.render.devImageCheck();
  if ("cancelled" in result) {
    throw new Error("the image check was cancelled");
  }
  return result;
}
