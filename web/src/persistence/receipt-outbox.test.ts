// The outbox (TS §23.3, v6implementation §11.5): rows survive a reload, only an
// answer removes one, only the signed-in person's go, and the timers are 5 s,
// 30 s and 5 min. The database is a map that outlives the module, as IndexedDB
// outlives a page; the real thing is read by `export-free.spec.ts`.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ExportId, Plan, UserId } from "../gen/domain";

const database = vi.hoisted(() => ({ rows: new Map<string, unknown>(), fail: false }));

vi.mock("./db", () => ({
  openDb: () => {
    if (database.fail) {
      return Promise.reject(new Error("no storage"));
    }
    return Promise.resolve({
      getAll: () => Promise.resolve([...database.rows.values()]),
      put: (_store: string, value: unknown, key: string) => {
        database.rows.set(key, value);
        return Promise.resolve();
      },
      delete: (_store: string, key: string) => {
        database.rows.delete(key);
        return Promise.resolve();
      },
    });
  },
}));

type Outbox = typeof import("./receipt-outbox");
type Outcome = import("./receipt-outbox").SendOutcome;

const ME: UserId = "me" as never;
const OTHER: UserId = "other" as never;
const id = (name: string): ExportId => name as never;

/** A page load: new module state over the same database. */
async function loadPage(options: { sub?: UserId | null; answers?: Outcome[] } = {}) {
  vi.resetModules();
  const outbox = await import("./receipt-outbox");
  const sent: string[] = [];
  const tokens: string[] = [];
  const reports: string[] = [];
  const mirrors: Record<string, number>[] = [];
  const answers = options.answers ?? [];
  const state = { sub: options.sub === undefined ? ME : options.sub };
  outbox.connectOutbox({
    signedInSub: () => state.sub,
    send: (exportId) => {
      sent.push(exportId);
      return Promise.resolve(answers.shift() ?? { kind: "sent", token: `token-${exportId}` });
    },
    tokenReceived: (token) => tokens.push(token),
    rowsChanged: (free) => mirrors.push(free),
    report: (code) => reports.push(code),
  });
  return { outbox, sent, tokens, reports, mirrors, state, answers };
}

const keys = () => [...database.rows.keys()];

async function put(outbox: Outbox, name: string, plan: Plan = "free", sub: UserId = ME): Promise<void> {
  await outbox.enqueue(id(name), plan, sub);
}

describe("the outbox", () => {
  beforeEach(() => {
    database.rows.clear();
    database.fail = false;
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "Date"] });
    vi.setSystemTime(1_000_000);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("a row holds what D-267 names", async () => {
    const { outbox } = await loadPage();
    await put(outbox, "a", "creator");
    expect(database.rows.get("a")).toEqual({
      schemaVersion: 1,
      value: { exportId: "a", enqueuedAt: 1_000_000, attempts: 0, plan: "creator", sub: "me" },
    });
  });

  it("a row survives a reload, and is sent by the next page", async () => {
    const first = await loadPage({ sub: null });
    await put(first.outbox, "a");
    await first.outbox.flushOutbox();
    expect(first.sent).toEqual([]);
    expect(keys()).toEqual(["a"]);

    const second = await loadPage();
    await second.outbox.flushOutbox();
    expect(second.sent).toEqual(["a"]);
    expect(keys()).toEqual([]);
  });

  it("D-268: with no session the rows wait, and nothing is scheduled", async () => {
    const { outbox, sent } = await loadPage({ sub: null });
    await put(outbox, "a");
    await outbox.flushOutbox();
    await vi.advanceTimersByTimeAsync(600_000);
    expect(sent).toEqual([]);
    expect(keys()).toEqual(["a"]);
    expect(vi.getTimerCount()).toBe(0);
  });

  it("D-267: only the signed-in person's rows are sent; another's wait for them", async () => {
    const mine = await loadPage();
    await put(mine.outbox, "mine");
    await put(mine.outbox, "theirs", "free", OTHER);
    await mine.outbox.flushOutbox();
    expect(mine.sent).toEqual(["mine"]);
    expect(keys()).toEqual(["theirs"]);

    const theirs = await loadPage({ sub: OTHER });
    await theirs.outbox.flushOutbox();
    expect(theirs.sent).toEqual(["theirs"]);
    expect(keys()).toEqual([]);
  });

  it("a 2xx removes the row and hands over the token, one row at a time", async () => {
    const { outbox, sent, tokens } = await loadPage();
    await put(outbox, "a");
    await put(outbox, "b");
    await outbox.flushOutbox();
    expect(sent).toEqual(["a", "b"]);
    expect(tokens).toEqual(["token-a", "token-b"]);
    expect(keys()).toEqual([]);
    expect(vi.getTimerCount()).toBe(0);
  });

  it("a refusal removes the row and reports it", async () => {
    const { outbox, reports } = await loadPage({ answers: [{ kind: "refused" }] });
    await put(outbox, "a");
    await outbox.flushOutbox();
    expect(keys()).toEqual([]);
    expect(reports).toEqual(["E_INTERNAL"]);
  });

  it("nothing else removes a row", async () => {
    for (const answer of [{ kind: "later" }, { kind: "no_session" }] as const) {
      database.rows.clear();
      const { outbox, tokens, reports } = await loadPage({ answers: [answer] });
      await put(outbox, "a");
      await outbox.flushOutbox();
      expect(keys(), answer.kind).toEqual(["a"]);
      expect(tokens).toEqual([]);
      expect(reports).toEqual([]);
      vi.clearAllTimers();
    }
  });

  it("a send that cannot go stops the pass: the next row is not tried", async () => {
    const { outbox, sent } = await loadPage({ answers: [{ kind: "later" }] });
    await put(outbox, "a");
    await put(outbox, "b");
    await outbox.flushOutbox();
    expect(sent).toEqual(["a"]);
  });

  it("a failed send counts on the row", async () => {
    const { outbox } = await loadPage({ answers: [{ kind: "later" }] });
    await put(outbox, "a");
    await outbox.flushOutbox();
    expect(database.rows.get("a")).toMatchObject({ value: { attempts: 1 } });
  });

  it("the timers are 5 s, 30 s and 5 min, and then nothing until the next start", async () => {
    const later = { kind: "later" } as const;
    const { outbox, sent, answers } = await loadPage({ answers: [later, later, later, later] });
    await put(outbox, "a");
    await outbox.flushOutbox();
    expect(sent).toHaveLength(1);

    await vi.advanceTimersByTimeAsync(4_999);
    expect(sent).toHaveLength(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(sent).toHaveLength(2);

    await vi.advanceTimersByTimeAsync(29_999);
    expect(sent).toHaveLength(2);
    await vi.advanceTimersByTimeAsync(1);
    expect(sent).toHaveLength(3);

    await vi.advanceTimersByTimeAsync(299_999);
    expect(sent).toHaveLength(3);
    await vi.advanceTimersByTimeAsync(1);
    expect(sent).toHaveLength(4);

    expect(vi.getTimerCount()).toBe(0);
    await vi.advanceTimersByTimeAsync(3_600_000);
    expect(sent).toHaveLength(4);
    expect(keys()).toEqual(["a"]);
    expect(answers).toEqual([]);

    // The next start tries again, and earns the timers back.
    const next = await loadPage({ answers: [later] });
    await next.outbox.flushOutbox();
    expect(next.sent).toEqual(["a"]);
    expect(vi.getTimerCount()).toBe(1);
  });

  it("a timer that finds the row sendable clears it, and no timer is left", async () => {
    const { outbox, sent } = await loadPage({ answers: [{ kind: "later" }] });
    await put(outbox, "a");
    await outbox.flushOutbox();
    await vi.advanceTimersByTimeAsync(5_000);
    expect(sent).toEqual(["a", "a"]);
    expect(keys()).toEqual([]);
    expect(vi.getTimerCount()).toBe(0);
  });

  it("no_session schedules nothing", async () => {
    const { outbox } = await loadPage({ answers: [{ kind: "no_session" }] });
    await put(outbox, "a");
    await outbox.flushOutbox();
    expect(vi.getTimerCount()).toBe(0);
  });

  it("two flushes at once send a row once", async () => {
    const { outbox, sent } = await loadPage();
    await put(outbox, "a");
    await Promise.all([outbox.flushOutbox(), outbox.flushOutbox()]);
    expect(sent).toEqual(["a"]);
  });

  it("the mirror counts Free rows per account, and no Creator row", async () => {
    const { outbox, mirrors } = await loadPage({ sub: null });
    await put(outbox, "a");
    await put(outbox, "b", "creator");
    await put(outbox, "c", "free", OTHER);
    await put(outbox, "d");
    expect(mirrors.at(-1)).toEqual({ me: 2, other: 1 });
    await outbox.remove(id("a"));
    expect(mirrors.at(-1)).toEqual({ me: 1, other: 1 });
  });

  it("a row of a newer build is absent to this one", async () => {
    database.rows.set("new", { schemaVersion: 99, value: { exportId: "new", plan: "free", sub: "me", attempts: 0, enqueuedAt: 0 } });
    const { outbox, sent, mirrors } = await loadPage();
    await outbox.mirrorOutbox();
    expect(mirrors.at(-1)).toEqual({});
    await outbox.flushOutbox();
    expect(sent).toEqual([]);
  });

  it("storage that cannot be read: a flush reports it and does not reject", async () => {
    const { outbox, reports } = await loadPage();
    database.fail = true;
    await expect(outbox.flushOutbox()).resolves.toBeUndefined();
    expect(reports).toEqual(["E_STORAGE_IO"]);
  });

  it("before it is connected a flush does nothing", async () => {
    vi.resetModules();
    const outbox = await import("./receipt-outbox");
    await expect(outbox.flushOutbox()).resolves.toBeUndefined();
  });
});
