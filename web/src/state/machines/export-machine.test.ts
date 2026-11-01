// The export machine (TS §12.2, §12.3; v5implementation §17.3): every legal
// transition, written out here as the spec's table, both through `transition`
// and through the export store's own action from a fresh store; every other
// pair refused, among them the two TS §12.3 names.

import { beforeEach, describe, expect, it } from "vitest";

import * as store from "../export-store";
import { EXPORT_MACHINE, type ExportEvent, type ExportStatus } from "./export-machine";
import { IllegalTransitionError, transition } from "./transition";

const STATES: readonly ExportStatus[] = ["idle", "gating", "rendering", "muxing", "saving", "done", "cancelled", "failed"];
const EVENTS: readonly ExportEvent[] = ["start", "blocked", "clear", "cache_hit", "encoded", "finalized", "saved", "cancel", "cancel_done", "fail", "retry", "reset"];

/** TS §12.2's export table, row by row. */
const LEGAL: readonly [ExportStatus, ExportEvent, ExportStatus][] = [
  ["idle", "start", "gating"],
  ["gating", "blocked", "idle"],
  ["gating", "clear", "rendering"],
  ["gating", "cache_hit", "saving"],
  ["rendering", "encoded", "muxing"],
  ["rendering", "cancel", "cancelled"],
  ["rendering", "fail", "failed"],
  ["muxing", "finalized", "saving"],
  ["muxing", "cancel", "cancelled"],
  ["muxing", "fail", "failed"],
  ["saving", "saved", "done"],
  ["saving", "fail", "failed"],
  ["done", "reset", "idle"],
  ["cancelled", "cancel_done", "idle"],
  ["failed", "retry", "gating"],
  ["failed", "reset", "idle"],
];

type Id = NonNullable<store.ExportState["exportId"]>;
const ID = "export-1" as unknown as Id;
const SUMMARY = { voice_cleaned: true, captions_emphasized: 0, visual_moments: 0 };
const FAILURE = { code: "E_INTERNAL", stage: "render_encode", retryable: false } as const;

/**
 * Each event fired through the store, the way the use-case fires it. `cancel`
 * and `cancel_done` are taken by `start-export.ts` through the same table, and
 * `retry` is never fired (D-153).
 */
const FIRE: Partial<Record<ExportEvent, () => void>> = {
  start: () => {
    store.start(ID);
  },
  blocked: () => {
    store.blocked();
  },
  clear: () => {
    store.clear();
  },
  cache_hit: () => {
    store.cacheHit(ID);
  },
  encoded: () => {
    store.encoded();
  },
  finalized: () => {
    store.finalized();
  },
  saved: () => {
    store.saved(SUMMARY, false);
  },
  fail: () => {
    store.fail(FAILURE);
  },
  reset: () => {
    store.reset();
  },
};

/** A fresh store in `status`, as if it had got there. */
function freshIn(status: ExportStatus): void {
  store.useExportStore.setState({ ...store.useExportStore.getInitialState(), status, exportId: ID }, true);
}

const state = () => store.useExportStore.getState();

describe("export machine", () => {
  beforeEach(() => {
    store.useExportStore.setState(store.useExportStore.getInitialState(), true);
  });

  it("has exactly the table of TS §12.2", () => {
    for (const from of STATES) {
      for (const event of EVENTS) {
        const legal = LEGAL.find(([f, e]) => f === from && e === event);
        if (legal === undefined) {
          expect(() => transition("export", EXPORT_MACHINE, from, event), `${from} + ${event}`).toThrow(IllegalTransitionError);
        } else {
          expect(transition("export", EXPORT_MACHINE, from, event)).toBe(legal[2]);
        }
      }
    }
  });

  it.each(LEGAL.filter(([, event]) => FIRE[event] !== undefined))("%s --%s--> %s through the store", (from, event, to) => {
    freshIn(from);
    FIRE[event]?.();
    expect(state().status).toBe(to);
  });

  it.each<[ExportStatus, ExportEvent]>([
    ["rendering", "clear"],
    ["done", "clear"],
    ["gating", "fail"],
    ["gating", "cancel"],
  ])("refuses %s --%s--, through the store too", (from, event) => {
    expect(() => transition("export", EXPORT_MACHINE, from, event)).toThrow(IllegalTransitionError);
    const fire = FIRE[event];
    if (fire !== undefined) {
      freshIn(from);
      expect(fire).toThrow(IllegalTransitionError);
      expect(state().status).toBe(from);
    }
  });

  it("names no way from rendering to rendering, or from done to rendering (TS §12.3)", () => {
    for (const event of EVENTS) {
      if (event !== "reset") {
        expect(EXPORT_MACHINE.done[event]).toBeUndefined();
      }
      expect(EXPORT_MACHINE.rendering[event]).not.toBe("rendering");
    }
  });
});

describe("what the export store keeps", () => {
  beforeEach(() => {
    store.useExportStore.setState(store.useExportStore.getInitialState(), true);
  });

  it("a blocked start says why, and the next start forgets it", () => {
    store.start(ID);
    store.blockedBy("B_STORAGE_LOW");
    expect(state().status).toBe("idle");
    expect(state().blocker).toBe("B_STORAGE_LOW");
    store.start(ID);
    expect(state().blocker).toBeUndefined();
  });

  it("a hit keeps its export's id, and saved keeps the summary", () => {
    store.start("another" as unknown as Id);
    store.cacheHit(ID);
    expect(state().exportId).toBe(ID);
    store.saved(SUMMARY, true);
    expect(state()).toMatchObject({ status: "done", exportId: ID, summary: SUMMARY, answered: true });
  });

  it("an answer is kept past a reset, for the id it was given to, and changes no state", () => {
    const id = "answered-export" as unknown as Id;
    freshIn("done");
    store.useExportStore.setState({ exportId: id });
    expect(store.hasAnswered(id)).toBe(false);
    store.answered();
    expect(state().answered).toBe(true);
    expect(state().status).toBe("done");
    store.reset();
    expect(state().answered).toBe(false);
    expect(state().summary).toBeUndefined();
    expect(store.hasAnswered(id)).toBe(true);
    expect(store.hasAnswered(ID)).toBe(false);
  });

  it("an answer with no export is nothing", () => {
    store.answered();
    expect(state().answered).toBe(false);
  });
});
