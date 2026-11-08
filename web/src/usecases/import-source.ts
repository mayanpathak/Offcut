// A long video dropped on `/app/long` (v6.1implementation §9.3, D-415, D-416,
// D-420, D-421). It is routed where it is (nothing is copied), gated (signed
// in, cloud available, room on the disk, then the line on what is sent and
// Start), and, once started, becomes the one source this device keeps. A file
// of at most 90 s is handed to the clip page. The `File` lives in memory only:
// never in a store, a URL or a log, and its name is never read (P-11).

import { FEATURES } from "../config/features";
import type { RouteFlags, SourceInfo } from "../gen/domain";
import * as clipsRepo from "../persistence/clips-repo";
import * as opfs from "../persistence/opfs";
import type { SourceId, SourceRecord } from "../persistence/schema";
import * as sources from "../persistence/sources-repo";
import { useAuthStore } from "../state/auth-store";
import { forSource } from "../state/blockers";
import { settledAvailability } from "../state/cloud-store";
import { useSourceStore } from "../state/source-store";
import { pool, WorkerCallError } from "../workers/pool";
import { runSourcePipeline } from "./run-source-pipeline";

/** How long the gate waits for an API that has not said whether it transcribes (CT D-374). */
export const SOURCE_WAKE_WAIT_MS = 45_000;
/** Room kept free beyond `pcm16.f32` (D-420, assumption until TE-17's second read). */
export const LONG_SCRATCH_BYTES = 2 * 1024 ** 3;

// --- The hand-over between the two pages (D-421) -----------------------------

let handed: { file: File; to: "clip" | "long" } | undefined;

/** Keeps `file` for the other page's mount. Memory only; read once. */
export function handOff(file: File, to: "clip" | "long"): void {
  handed = { file, to };
}

/** The file handed to this page, once; `undefined` when there is none. */
export function takeHandOff(to: "clip" | "long"): File | undefined {
  if (handed?.to !== to) {
    return undefined;
  }
  const { file } = handed;
  handed = undefined;
  return file;
}

// --- The source being started ----------------------------------------------

/** The dropped file, held while the gate waits for Start. Memory only. */
let pending: { file: File; info: SourceInfo; hash: string } | undefined;

/** The one place of this file that makes a source's id. */
export function newSourceId(): SourceId {
  return crypto.randomUUID() as SourceId;
}

function flags(): RouteFlags {
  return { long_video: FEATURES.long_video, high_res: FEATURES.high_res, hdr_ok: false };
}

/** Bytes `pcm16.f32` takes for `durationMs` of sound, with the scratch room (D-420). */
export function neededBytes(durationMs: number): number {
  return Math.round(durationMs * 16) * 4 + LONG_SCRATCH_BYTES;
}

/** The first gate reason that stands, or `ready` (§10.2). */
async function gateReason(info: SourceInfo): Promise<{ reason: "signed_out" | "unavailable" | "storage_low" | "ready"; neededBytes?: number }> {
  if (useAuthStore.getState().status !== "signed_in") {
    return { reason: "signed_out" };
  }
  useSourceStore.getState().patch({ gate: { reason: "checking" } });
  if ((await settledAvailability(SOURCE_WAKE_WAIT_MS)) !== "available") {
    return { reason: "unavailable" };
  }
  const free = await opfs.freeBytes();
  const needed = neededBytes(info.duration);
  if (free !== null && free < needed) {
    return { reason: "storage_low", neededBytes: needed };
  }
  return { reason: "ready" };
}

/** The outcome of a drop, for the page: where it went. */
export type DropOutcome = "gated" | "rejected" | "handed_over" | "failed" | "blocked";

/**
 * A file dropped on the long page: routed in place, then gated. Nothing is
 * written before Start. A short file goes to the clip page (D-421).
 */
export async function dropSource(file: File): Promise<DropOutcome> {
  if (!FEATURES.long_video || forSource() !== null) {
    return "blocked";
  }
  const store = useSourceStore.getState();
  store.clear();
  useSourceStore.getState().send("drop");
  let hash: string;
  let routed: Awaited<ReturnType<typeof pool.media.routeFile>>;
  try {
    hash = await clipsRepo.hashPrefix(file);
    routed = await pool.media.routeFile({ file, entry: "long", flags: flags() });
  } catch (error) {
    const failure = error instanceof WorkerCallError ? error.failure : undefined;
    useSourceStore.getState().patch({ failure: { code: failure?.code ?? "E_SOURCE_READ" } });
    useSourceStore.getState().send("fail");
    return "failed";
  }
  if (typeof routed !== "object" || routed === null || "cancelled" in routed) {
    useSourceStore.getState().patch({ failure: { code: "E_INTERNAL" } });
    useSourceStore.getState().send("fail");
    return "failed";
  }
  if ("failed" in routed) {
    useSourceStore.getState().patch({ failure: { code: routed.failed, detail: routed.detail } });
    useSourceStore.getState().send("fail");
    return "failed";
  }
  if ("rejected" in routed) {
    useSourceStore.getState().patch({ rejected: routed.rejected });
    useSourceStore.getState().send("reject");
    return "rejected";
  }
  if ("clip" in routed) {
    // "This one is short, so it opens as a clip."
    handOff(file, "clip");
    useSourceStore.getState().send("hand_over");
    return "handed_over";
  }
  pending = { file, info: routed.long, hash };
  useSourceStore.getState().patch({ info: routed.long });
  const gate = await gateReason(routed.long);
  useSourceStore.getState().patch({ gate });
  useSourceStore.getState().send("gate");
  return "gated";
}

/** After a sign-in, or when the cloud may be back: the gate asks again. */
export async function recheckGate(): Promise<void> {
  const state = useSourceStore.getState();
  if (state.status !== "gated" || pending === undefined) {
    return;
  }
  state.send("ungate");
  const gate = await gateReason(pending.info);
  useSourceStore.getState().patch({ gate });
  useSourceStore.getState().send("gate");
}

/** Start, from the gate's last line: the previous source goes (D-415), this one is made and runs. */
export async function startSource(): Promise<void> {
  const state = useSourceStore.getState();
  if (state.status !== "gated" || state.gate?.reason !== "ready" || pending === undefined || forSource() !== null) {
    return;
  }
  const { file, info, hash } = pending;
  pending = undefined;
  const sourceId = newSourceId();
  const now = Date.now();
  const record: SourceRecord = {
    sourceId,
    createdAt: now,
    lastOpenedAt: now,
    sourceInfo: info,
    sourceBytes: file.size as SourceRecord["sourceBytes"],
    sourceHashPrefix: hash,
    stage: "extracting_audio",
    pcm16Seconds: 0,
    levelP95: null,
    windowsDone: 0,
    windowsTotal: 0,
  };
  try {
    await sources.removeAll();
    await sources.put(record);
  } catch {
    useSourceStore.getState().send("ungate");
    useSourceStore.getState().patch({ failure: { code: "E_STORAGE_IO" } });
    useSourceStore.getState().send("fail");
    return;
  }
  useSourceStore.getState().send("ungate");
  useSourceStore.getState().patch({ sourceId, gate: null });
  useSourceStore.getState().send("accept");
  await runSourcePipeline(record, file);
}
