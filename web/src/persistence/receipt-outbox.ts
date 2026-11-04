// Usage receipts waiting to reach the server (TS §23.1, v6implementation §11.5).
// A row is an export's id and whose it was; it survives a reload. The server,
// the stores and analytics are handed in through `connectOutbox`, since
// `persistence` imports none of them.

import type { ErrorCode, ExportId, Plan, UserId } from "../gen/domain";
import { openDb } from "./db";
import { DB_VERSION, type ReceiptOutboxRecord, STORES, type Versioned } from "./schema";

/** The waits after a failed flush. After the last, the next try is the next app start or `online`. */
export const FLUSH_WAITS_MS = [5_000, 30_000, 300_000];

/**
 * What became of one send. `sent` removes the row and takes the fresh token.
 * `refused` is an answer that can never change, and removes it. `later` keeps
 * it, for a timer. `no_session` keeps it, with no timer: it needs a sign-in (D-268).
 */
export type SendOutcome = { kind: "sent"; token: string } | { kind: "refused" } | { kind: "later" } | { kind: "no_session" };

export type OutboxLinks = {
  /** Whose rows may be sent now: `null` when nobody is signed in. */
  signedInSub(): UserId | null;
  send(exportId: ExportId): Promise<SendOutcome>;
  tokenReceived(token: string): void;
  /** The unsent Free rows per account, each time they change. */
  rowsChanged(freePerAccount: Record<string, number>): void;
  report(code: ErrorCode): void;
};

let links: OutboxLinks | undefined;
let nextWait = 0;
let timer: ReturnType<typeof setTimeout> | undefined;
let flushing: Promise<void> | undefined;
let flushAgain = false;

/** Given once, at app start. */
export function connectOutbox(given: OutboxLinks): void {
  links = given;
}

async function rows(): Promise<ReceiptOutboxRecord[]> {
  const db = await openDb();
  const stored: Versioned<ReceiptOutboxRecord>[] = await db.getAll(STORES.receiptOutbox);
  // Written by a newer build: absent to this one (TS §23.3). Its keys are not
  // known to this build's readers, so they go when that build removes them.
  return stored.filter((row) => row.schemaVersion <= DB_VERSION).map((row) => row.value);
}

/** Tells the store what waits. Called after every change, and once at app start. */
export async function mirrorOutbox(): Promise<void> {
  const freePerAccount: Record<string, number> = {};
  for (const row of await rows()) {
    if (row.plan === "free") {
      freePerAccount[row.sub] = (freePerAccount[row.sub] ?? 0) + 1;
    }
  }
  links?.rowsChanged(freePerAccount);
}

/** The new row is unsent until a flush says otherwise. A new row earns the timers again. */
export async function enqueue(exportId: ExportId, plan: Plan, sub: UserId): Promise<void> {
  const db = await openDb();
  const value: ReceiptOutboxRecord = { exportId, enqueuedAt: Date.now(), attempts: 0, plan, sub };
  const stored: Versioned<ReceiptOutboxRecord> = { schemaVersion: DB_VERSION, value };
  await db.put(STORES.receiptOutbox, stored, exportId);
  nextWait = 0;
  await mirrorOutbox();
}

export async function remove(exportId: ExportId): Promise<void> {
  const db = await openDb();
  await db.delete(STORES.receiptOutbox, exportId);
  await mirrorOutbox();
}

async function countAttempt(row: ReceiptOutboxRecord): Promise<void> {
  const db = await openDb();
  const stored: Versioned<ReceiptOutboxRecord> = {
    schemaVersion: DB_VERSION,
    value: { ...row, attempts: row.attempts + 1 },
  };
  await db.put(STORES.receiptOutbox, stored, row.exportId);
}

function scheduleRetry(): void {
  const wait = FLUSH_WAITS_MS[nextWait];
  if (wait === undefined || timer !== undefined) {
    return;
  }
  nextWait += 1;
  timer = setTimeout(() => {
    timer = undefined;
    void flushOutbox();
  }, wait);
}

/** One pass over the signed-in person's rows. It stops at the first that cannot go now: the others would meet the same. */
async function flushOnce(connected: OutboxLinks): Promise<void> {
  const sub = connected.signedInSub();
  if (sub === null) {
    return;
  }
  const mine = (await rows()).filter((row) => row.sub === sub);
  for (const row of mine) {
    const outcome = await connected.send(row.exportId);
    switch (outcome.kind) {
      case "sent":
        // The token first: until the row goes, the count is low by one and not high.
        connected.tokenReceived(outcome.token);
        await remove(row.exportId);
        break;
      case "refused":
        connected.report("E_INTERNAL");
        await remove(row.exportId);
        break;
      case "later":
        await countAttempt(row);
        scheduleRetry();
        return;
      case "no_session":
        return;
    }
  }
  nextWait = 0;
}

/** Sends what may be sent. It never rejects, and a call during a flush makes one more pass after it. */
export function flushOutbox(): Promise<void> {
  if (flushing !== undefined) {
    flushAgain = true;
    return flushing;
  }
  const connected = links;
  if (connected === undefined) {
    return Promise.resolve();
  }
  flushing = (async () => {
    try {
      do {
        flushAgain = false;
        await flushOnce(connected);
      } while (flushAgain);
    } catch {
      // Storage cannot be read: the rows stay, for the next start.
      connected.report("E_STORAGE_IO");
    } finally {
      flushing = undefined;
    }
  })();
  return flushing;
}
