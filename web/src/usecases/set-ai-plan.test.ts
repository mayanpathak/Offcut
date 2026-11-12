// V6.2: the settings switch "AI edit plan" (v6.2implementation §8, P193; TS
// §40.8): on unless this device turned it off, kept in `meta` under `aiPlan`,
// held for the page at once. The database is a map in memory.

import { beforeEach, describe, expect, it, vi } from "vitest";

import { getAiPlanSwitch } from "../persistence/prefs-repo";
import { usePlanStore } from "../state/plan-store";
import { chooseAiPlan } from "./set-ai-plan";

const meta = vi.hoisted(() => ({ values: new Map<string, unknown>(), failRead: false, failWrite: false }));

vi.mock("../persistence/db", () => ({
  metaGet: (key: string) => (meta.failRead ? Promise.reject(new Error("no storage")) : Promise.resolve(meta.values.get(key))),
  metaSet: (key: string, value: unknown) => {
    if (meta.failWrite) {
      return Promise.reject(new Error("no storage"));
    }
    meta.values.set(key, value);
    return Promise.resolve();
  },
}));

beforeEach(() => {
  meta.values.clear();
  meta.failRead = false;
  meta.failWrite = false;
  usePlanStore.setState({ switchOn: true });
});

describe("the AI plan switch", () => {
  it("is on for a first visit, and for anything stored but false", async () => {
    expect(usePlanStore.getState().switchOn).toBe(true);
    expect(await getAiPlanSwitch()).toBe(true);
    meta.values.set("aiPlan", "no");
    expect(await getAiPlanSwitch()).toBe(true);
    meta.values.set("aiPlan", false);
    expect(await getAiPlanSwitch()).toBe(false);
  });

  it("is on when the storage cannot be read", async () => {
    meta.failRead = true;
    expect(await getAiPlanSwitch()).toBe(true);
  });

  it("holds the choice for the page at once and keeps it under aiPlan", async () => {
    chooseAiPlan(false);
    expect(usePlanStore.getState().switchOn).toBe(false);
    await vi.waitFor(() => expect(meta.values.get("aiPlan")).toBe(false));
    expect(await getAiPlanSwitch()).toBe(false);
    chooseAiPlan(true);
    expect(usePlanStore.getState().switchOn).toBe(true);
    await vi.waitFor(() => expect(meta.values.get("aiPlan")).toBe(true));
  });

  it("a write that fails keeps the choice for this page only, with no throw", async () => {
    meta.failWrite = true;
    chooseAiPlan(false);
    expect(usePlanStore.getState().switchOn).toBe(false);
    await Promise.resolve();
    expect(meta.values.has("aiPlan")).toBe(false);
  });
});
