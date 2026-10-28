// The faults a development build can cause on purpose (v4implementation
// D-155, G 14.5, Prompt 124): a worker that crashes, one that hangs, and a
// scene that fails to build once. Only `ui/dev-hooks.ts` calls these, and a
// production build has neither it nor the workers' halves (rpc-worker.ts).

import { pool } from "../workers/pool";
import type { WorkerKind } from "../workers/protocol";

/** Worker `kind` throws where nothing catches it: the page sees its crash (D-153). */
export function crashWorker(kind: WorkerKind): void {
  pool[kind].notify("devCrash", {});
}

/** Worker `kind` stops answering: a cancel then restarts it after 2 s (D-152). */
export function hangWorker(kind: WorkerKind): void {
  pool[kind].notify("devHang", {});
}

/** The render worker's next `setScene` fails: a review change then fails as one can (D-190). */
export function failNextSetScene(): void {
  pool.render.notify("devFailNext", { method: "setScene" });
}
