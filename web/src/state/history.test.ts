// The undo history (v4implementation D-184, §20.3): push, undo, redo, the
// limit of HISTORY_MAX steps, a new change dropping what was undone, and a
// new clip clearing it.

import { beforeEach, describe, expect, it } from "vitest";

import type { EditState } from "../gen/domain";
import { begin, failed, reset } from "./clip-store";
import { clear, HISTORY_MAX, push, redo, undo, useHistoryStore } from "./history";

/** An edit told apart from the others by its crop offset. */
function edit(n: number): EditState {
  return {
    word_edits: {},
    event_overrides: {},
    style: "clean",
    crop_offset: n,
    caption_position: 1,
    highlight_numbers: true,
  };
}

const state = () => useHistoryStore.getState();

describe("history", () => {
  beforeEach(() => {
    clear();
  });

  it("starts empty", () => {
    expect(state()).toMatchObject({ past: [], future: [], canUndo: false, canRedo: false });
    expect(undo(edit(0))).toBeUndefined();
    expect(redo(edit(0))).toBeUndefined();
    expect(state()).toMatchObject({ past: [], future: [] });
  });

  it("undo gives the edit before the last change and keeps the current one for redo", () => {
    push(edit(0));
    push(edit(1));
    expect(state()).toMatchObject({ canUndo: true, canRedo: false });

    expect(undo(edit(2))).toEqual(edit(1));
    expect(state().past).toEqual([edit(0)]);
    expect(state().future).toEqual([edit(2)]);
    expect(state()).toMatchObject({ canUndo: true, canRedo: true });

    expect(undo(edit(1))).toEqual(edit(0));
    expect(state()).toMatchObject({ canUndo: false, canRedo: true });
    expect(state().future).toEqual([edit(2), edit(1)]);
  });

  it("redo brings back what was undone, latest first, and keeps the current edit for undo", () => {
    push(edit(0));
    push(edit(1));
    undo(edit(2));
    undo(edit(1));

    expect(redo(edit(0))).toEqual(edit(1));
    expect(state().past).toEqual([edit(0)]);
    expect(redo(edit(1))).toEqual(edit(2));
    expect(state().past).toEqual([edit(0), edit(1)]);
    expect(state()).toMatchObject({ future: [], canUndo: true, canRedo: false });
    expect(redo(edit(2))).toBeUndefined();
  });

  it(`keeps the last ${String(HISTORY_MAX)} steps`, () => {
    for (let n = 0; n < HISTORY_MAX + 5; n += 1) {
      push(edit(n));
    }
    expect(state().past).toHaveLength(HISTORY_MAX);
    expect(state().past[0]).toEqual(edit(5));
    expect(state().past.at(-1)).toEqual(edit(HISTORY_MAX + 4));

    // Undone to the oldest kept step: one more undo has nothing.
    let current = edit(HISTORY_MAX + 5);
    for (let n = 0; n < HISTORY_MAX; n += 1) {
      const before = undo(current);
      expect(before).toBeDefined();
      current = before ?? current;
    }
    expect(current).toEqual(edit(5));
    expect(undo(current)).toBeUndefined();
  });

  it("a redo past the limit drops the oldest step", () => {
    for (let n = 0; n < HISTORY_MAX; n += 1) {
      push(edit(n));
    }
    undo(edit(HISTORY_MAX));
    push(edit(100));
    expect(state().past).toHaveLength(HISTORY_MAX);
    undo(edit(101));
    redo(edit(100));
    expect(state().past).toHaveLength(HISTORY_MAX);
  });

  it("a new change after an undo drops the redo branch", () => {
    push(edit(0));
    push(edit(1));
    undo(edit(2));
    expect(state().canRedo).toBe(true);

    push(edit(1));
    expect(state()).toMatchObject({ future: [], canRedo: false, canUndo: true });
    expect(state().past).toEqual([edit(0), edit(1)]);
    expect(redo(edit(3))).toBeUndefined();
  });

  it("clear leaves nothing to undo or redo", () => {
    push(edit(0));
    push(edit(1));
    undo(edit(2));
    clear();
    expect(state()).toMatchObject({ past: [], future: [], canUndo: false, canRedo: false });
  });

  it("a new clip clears the history, and so does letting the clip go", () => {
    push(edit(0));
    begin("clip-a" as never, "user");
    expect(state()).toMatchObject({ past: [], canUndo: false });

    push(edit(1));
    failed({ code: "E_INTERNAL", stage: "import", retryable: false });
    reset();
    expect(state()).toMatchObject({ past: [], canUndo: false });
  });
});
