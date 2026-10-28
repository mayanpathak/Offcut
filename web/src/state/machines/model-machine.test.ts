// The speech model's machine (TS §12.2, §12.3; v4implementation §20.3): the
// table, written out here, through `transition` and through the model
// store's actions from a fresh store; `verifying → downloading` refused
// without going through `failed`.

import { beforeEach, describe, expect, it } from "vitest";

import * as model from "../model-store";
import { MODEL_MACHINE, type ModelEvent, type ModelStatus } from "./model-machine";
import { IllegalTransitionError, transition } from "./transition";

const STATES: readonly ModelStatus[] = ["unknown", "absent", "partial", "ready", "downloading", "verifying", "failed"];
const EVENTS: readonly ModelEvent[] = [
  "inspected_absent",
  "inspected_partial",
  "inspected_ready",
  "start",
  "last_byte",
  "net_fail",
  "fail_download",
  "fail_storage",
  "hash_ok",
  "hash_bad",
  "cleared",
];

/** TS §12.2's model table, row by row. */
const LEGAL: readonly [ModelStatus, ModelEvent, ModelStatus][] = [
  ["unknown", "inspected_absent", "absent"],
  ["unknown", "inspected_partial", "partial"],
  ["unknown", "inspected_ready", "ready"],
  ["absent", "start", "downloading"],
  ["partial", "start", "downloading"],
  ["partial", "cleared", "absent"],
  ["ready", "cleared", "absent"],
  ["downloading", "last_byte", "verifying"],
  ["downloading", "net_fail", "partial"],
  ["downloading", "fail_download", "failed"],
  ["downloading", "fail_storage", "failed"],
  ["verifying", "hash_ok", "ready"],
  ["verifying", "hash_bad", "failed"],
  ["failed", "start", "downloading"],
];

/** Each event fired through the store, the way the model manager fires it. */
const FIRE: Record<ModelEvent, () => void> = {
  inspected_absent: () => {
    model.inspected("absent");
  },
  inspected_partial: () => {
    model.inspected("partial");
  },
  inspected_ready: () => {
    model.inspected("ready");
  },
  start: model.start,
  last_byte: model.lastByte,
  net_fail: model.netFail,
  fail_download: () => {
    model.fail("E_MODEL_DOWNLOAD");
  },
  fail_storage: () => {
    model.fail("E_MODEL_STORAGE");
  },
  hash_ok: model.hashOk,
  hash_bad: () => {
    model.fail("E_MODEL_HASH");
  },
  cleared: model.cleared,
};

function freshIn(status: ModelStatus): void {
  model.useModelStore.setState({ ...model.useModelStore.getInitialState(), status }, true);
}

describe("model machine", () => {
  beforeEach(() => {
    model.useModelStore.setState(model.useModelStore.getInitialState(), true);
  });

  it("has exactly the table of TS §12.2", () => {
    for (const from of STATES) {
      for (const event of EVENTS) {
        const legal = LEGAL.find(([f, e]) => f === from && e === event);
        if (legal === undefined) {
          expect(() => transition("model", MODEL_MACHINE, from, event), `${from} + ${event}`).toThrow(IllegalTransitionError);
        } else {
          expect(transition("model", MODEL_MACHINE, from, event)).toBe(legal[2]);
        }
      }
    }
  });

  it.each(LEGAL)("%s --%s--> %s through the store", (from, event, to) => {
    freshIn(from);
    FIRE[event]();
    expect(model.useModelStore.getState().status).toBe(to);
  });

  it("refuses verifying → downloading without failed", () => {
    expect(() => transition("model", MODEL_MACHINE, "verifying", "start")).toThrow(IllegalTransitionError);
    freshIn("verifying");
    expect(model.start).toThrow(IllegalTransitionError);
    expect(model.useModelStore.getState().status).toBe("verifying");
    // Through failed it may.
    model.fail("E_MODEL_HASH");
    expect(model.useModelStore.getState().status).toBe("failed");
    model.start();
    expect(model.useModelStore.getState().status).toBe("downloading");
  });

  it("a wrong hash before the last byte goes through verifying, and keeps its code", () => {
    freshIn("downloading");
    model.fail("E_MODEL_HASH");
    expect(model.useModelStore.getState()).toMatchObject({ status: "failed", error: "E_MODEL_HASH" });
  });

  it("an interrupted download keeps its bytes for the next run", () => {
    freshIn("downloading");
    model.progress({ done: 30 as never, total: 100 as never, etaSecs: 4 });
    model.netFail();
    expect(model.useModelStore.getState()).toMatchObject({ status: "partial", done: 30, total: 100, etaSecs: null });
  });
});
