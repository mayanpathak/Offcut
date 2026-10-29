// The clip machine (TS §12.2, §12.3; v4implementation §20.3): every legal
// transition, written out here as the spec's table, both through
// `transition` and through the clip store's own action from a fresh store;
// every other pair refused, among them the four TS §12.3 names, and
// `updating` refusing a second edit (D-147).

import { beforeEach, describe, expect, it } from "vitest";

import type { ClipInfo, EditState } from "../../gen/domain";
import * as clip from "../clip-store";
import { CLIP_MACHINE, type ClipEvent, type ClipStatus } from "./clip-machine";
import { IllegalTransitionError, transition } from "./transition";

const STATES: readonly ClipStatus[] = ["idle", "importing", "accepted", "processing", "ready", "updating", "rejected", "failed"];
const EVENTS: readonly ClipEvent[] = [
  "import",
  "accept",
  "reject",
  "fail",
  "run",
  "done",
  "no_speech",
  "cancel",
  "edit_start",
  "edit_done",
  "retry",
  "reset",
  "restore",
];

/** TS §12.2's clip table, row by row. */
const LEGAL: readonly [ClipStatus, ClipEvent, ClipStatus][] = [
  ["idle", "import", "importing"],
  ["idle", "restore", "ready"],
  ["importing", "accept", "accepted"],
  ["importing", "reject", "rejected"],
  ["importing", "fail", "failed"],
  ["accepted", "run", "processing"],
  ["accepted", "reset", "idle"],
  ["processing", "done", "ready"],
  ["processing", "no_speech", "rejected"],
  ["processing", "fail", "failed"],
  ["processing", "cancel", "accepted"],
  ["ready", "edit_start", "updating"],
  ["ready", "reset", "idle"],
  ["updating", "edit_done", "ready"],
  ["rejected", "reset", "idle"],
  ["failed", "retry", "processing"],
  ["failed", "reset", "idle"],
];

const EDIT: EditState = {
  word_edits: {},
  event_overrides: {},
  style: "bold",
  crop_offset: 0,
  caption_position: 1,
  highlight_numbers: true,
};
const INFO = { duration: 20_000 } as unknown as ClipInfo;
const FAILURE = { code: "E_INTERNAL", stage: "asr", retryable: false } as const;

/** Each event fired through the store, the way the use-cases fire it. `restore` has no action before V7. */
const FIRE: Record<Exclude<ClipEvent, "restore">, () => void> = {
  import: () => {
    clip.begin("clip-1" as never, "user");
  },
  accept: () => {
    clip.accepted(INFO);
  },
  reject: () => {
    clip.rejected("REJECT_HEVC");
  },
  fail: () => {
    clip.failed(FAILURE);
  },
  run: () => {
    clip.processing("probe_audio", false);
  },
  done: () => {
    clip.ready({
      transcript: { words: [], sentences: [] } as never,
      prosody: {} as never,
      events: [],
      timeMap: { segments: [] },
      prog48: new Float32Array(0),
    });
  },
  no_speech: () => {
    clip.noSpeech();
  },
  cancel: () => {
    clip.cancelledRun();
  },
  edit_start: () => {
    clip.editStarted(EDIT);
  },
  edit_done: () => {
    clip.editDone();
  },
  retry: () => {
    clip.retried();
  },
  reset: () => {
    clip.reset();
  },
};

/** A fresh store in `status`, as if it had got there. */
function freshIn(status: ClipStatus): void {
  clip.useClipStore.setState({ ...clip.useClipStore.getInitialState(), status, clipInfo: INFO }, true);
}

describe("clip machine", () => {
  beforeEach(() => {
    clip.useClipStore.setState(clip.useClipStore.getInitialState(), true);
  });

  it("has exactly the table of TS §12.2", () => {
    for (const from of STATES) {
      for (const event of EVENTS) {
        const legal = LEGAL.find(([f, e]) => f === from && e === event);
        if (legal === undefined) {
          expect(() => transition("clip", CLIP_MACHINE, from, event), `${from} + ${event}`).toThrow(IllegalTransitionError);
        } else {
          expect(transition("clip", CLIP_MACHINE, from, event)).toBe(legal[2]);
        }
      }
    }
  });

  it.each(LEGAL.filter(([, event]) => event !== "restore"))("%s --%s--> %s through the store", (from, event, to) => {
    freshIn(from);
    FIRE[event as Exclude<ClipEvent, "restore">]();
    expect(clip.useClipStore.getState().status).toBe(to);
  });

  it.each<[ClipStatus, ClipEvent]>([
    ["rejected", "run"],
    ["importing", "run"],
    ["processing", "import"],
    ["ready", "import"],
  ])("refuses %s --%s-- (TS §12.3)", (from, event) => {
    expect(() => transition("clip", CLIP_MACHINE, from, event)).toThrow(IllegalTransitionError);
    // Through the store too, and the state is left as it was.
    freshIn(from);
    if (event !== "restore") {
      expect(() => {
        FIRE[event]();
      }).toThrow(IllegalTransitionError);
    }
    expect(clip.useClipStore.getState().status).toBe(from);
  });

  it("refuses a second edit while one is in flight (D-147)", () => {
    freshIn("ready");
    clip.editStarted(EDIT);
    expect(clip.useClipStore.getState().status).toBe("updating");
    expect(() => {
      clip.editStarted(EDIT);
    }).toThrow(IllegalTransitionError);
    expect(clip.useClipStore.getState().status).toBe("updating");
  });

  it("a cancelled run goes back to accepted with its file kept and the cancelled card's flag", () => {
    freshIn("processing");
    clip.pushFeed({ kind: "transcribing" });
    clip.cancelledRun();
    const state = clip.useClipStore.getState();
    expect(state.status).toBe("accepted");
    expect(state.cancelled).toBe(true);
    expect(state.clipInfo).toBe(INFO);
    expect(state.feed).toEqual([]);
    // Processing again clears the flag.
    clip.processing("probe_audio", false);
    expect(clip.useClipStore.getState().cancelled).toBe(false);
  });

  it("a cancelled import ends idle, through failed, in one call", () => {
    clip.begin("clip-2" as never, "user");
    clip.cancelledImport();
    expect(clip.useClipStore.getState().status).toBe("idle");
    expect(clip.useClipStore.getState().failure).toBeUndefined();
  });
});
