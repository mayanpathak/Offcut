// The source machine (v6.1implementation §9.4, §11.3): every legal pair,
// through `transition` and through the store's own action, and every other
// pair refused, one illegal pair per state at least.

import { beforeEach, describe, expect, it } from "vitest";

import { useSourceStore } from "../source-store";
import { SOURCE_MACHINE, type SourceEvent, type SourceStatus } from "./source-machine";
import { IllegalTransitionError, transition } from "./transition";

const STATES = Object.keys(SOURCE_MACHINE) as SourceStatus[];
const EVENTS: readonly SourceEvent[] = [
  "drop",
  "gate",
  "ungate",
  "reject",
  "hand_over",
  "accept",
  "audio_done",
  "transcribed",
  "pause",
  "continue_audio",
  "continue_transcribe",
  "restore_audio",
  "restore_transcribe",
  "restore_transcribed",
  "file_again",
  "cancel",
  "fail",
  "retry_audio",
  "retry_transcribe",
  "reset",
];

/** §9.4, row by row. */
const LEGAL: readonly [SourceStatus, SourceEvent, SourceStatus][] = [
  ["idle", "drop", "probing"],
  ["idle", "restore_audio", "awaiting_file"],
  ["idle", "restore_transcribe", "transcribing"],
  ["idle", "restore_transcribed", "transcribed"],
  ["probing", "gate", "gated"],
  ["probing", "reject", "rejected"],
  ["probing", "hand_over", "idle"],
  ["probing", "accept", "extracting_audio"],
  ["probing", "fail", "failed"],
  ["gated", "ungate", "probing"],
  ["gated", "reset", "idle"],
  ["rejected", "reset", "idle"],
  ["extracting_audio", "audio_done", "transcribing"],
  ["extracting_audio", "pause", "paused"],
  ["extracting_audio", "cancel", "cancelled"],
  ["extracting_audio", "fail", "failed"],
  ["transcribing", "transcribed", "transcribed"],
  ["transcribing", "pause", "paused"],
  ["transcribing", "cancel", "cancelled"],
  ["transcribing", "fail", "failed"],
  ["paused", "continue_audio", "extracting_audio"],
  ["paused", "continue_transcribe", "transcribing"],
  ["paused", "cancel", "cancelled"],
  ["paused", "fail", "failed"],
  ["awaiting_file", "file_again", "extracting_audio"],
  ["awaiting_file", "cancel", "cancelled"],
  ["awaiting_file", "reset", "idle"],
  ["transcribed", "reset", "idle"],
  ["cancelled", "reset", "idle"],
  ["failed", "retry_audio", "awaiting_file"],
  ["failed", "retry_transcribe", "transcribing"],
  ["failed", "cancel", "cancelled"],
  ["failed", "reset", "idle"],
];

describe("source machine", () => {
  beforeEach(() => {
    useSourceStore.setState({ status: "idle" });
  });

  it("has every state of §9.4 and nothing more", () => {
    expect(STATES.sort()).toEqual(
      ["awaiting_file", "cancelled", "extracting_audio", "failed", "gated", "idle", "paused", "probing", "rejected", "transcribed", "transcribing"].sort(),
    );
  });

  it.each(LEGAL)("%s --%s--> %s, through transition and the store", (from, event, to) => {
    expect(transition("source", SOURCE_MACHINE, from, event)).toBe(to);
    useSourceStore.setState({ status: from });
    useSourceStore.getState().send(event);
    expect(useSourceStore.getState().status).toBe(to);
  });

  it("refuses every other pair", () => {
    const legal = new Set(LEGAL.map(([from, event]) => `${from}:${event}`));
    let refused = 0;
    for (const from of STATES) {
      let refusedHere = 0;
      for (const event of EVENTS) {
        if (legal.has(`${from}:${event}`)) {
          continue;
        }
        expect(() => transition("source", SOURCE_MACHINE, from, event)).toThrow(IllegalTransitionError);
        refused += 1;
        refusedHere += 1;
      }
      expect(refusedHere, from).toBeGreaterThan(0);
    }
    expect(refused).toBe(STATES.length * EVENTS.length - LEGAL.length);
  });

  it("names the pairs that must never happen", () => {
    // Nothing is transcribed before its sound is written; a rejected file starts nothing.
    for (const [from, event] of [
      ["probing", "audio_done"],
      ["extracting_audio", "transcribed"],
      ["rejected", "accept"],
      ["transcribed", "pause"],
      ["idle", "accept"],
    ] as const) {
      expect(() => transition("source", SOURCE_MACHINE, from, event)).toThrow(IllegalTransitionError);
    }
  });

  it("clears to idle with nothing held", () => {
    useSourceStore.setState({ status: "transcribed", sourceId: "x" as never, differentFile: true });
    useSourceStore.getState().clear();
    const state = useSourceStore.getState();
    expect(state.status).toBe("idle");
    expect(state.sourceId).toBeNull();
    expect(state.differentFile).toBe(false);
  });
});
