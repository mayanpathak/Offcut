// The words of the recognizer's windows as one list of words on the clip's
// own timeline (TS §16.4). A time here is the time the word was spoken:
// nothing below drops, joins or moves a word because of a pause (INV-5).

import type { Confidence, DurMs, TimeMs } from "../../gen/domain";
import type { RawWord } from "./whisper-runtime";

/** The shortest a word is given, in milliseconds, where the next word leaves room. */
export const ASR_MIN_WORD = 40;

/** What one window gave: its place in the clip, and its words with times counted from its start. */
export type AsrWindow = { offsetMs: number; lengthMs: number; words: RawWord[] };

/** The line between two windows that overlap: the middle of what they share. */
function divide(earlier: AsrWindow, later: AsrWindow): number {
  return (later.offsetMs + earlier.offsetMs + earlier.lengthMs) / 2;
}

/**
 * One list from the windows, with every time on the clip's timeline. Two
 * windows that overlap both heard the words in the overlap; a word there is
 * taken from the window whose edge is farther from it, which is the earlier
 * window up to the middle of the overlap and the later one from there on.
 */
export function mergeWindows(windows: readonly AsrWindow[]): RawWord[] {
  const merged: RawWord[] = [];
  for (const [index, window] of windows.entries()) {
    const before = windows[index - 1];
    const after = windows[index + 1];
    const from = before === undefined ? Number.NEGATIVE_INFINITY : divide(before, window);
    const to = after === undefined ? Number.POSITIVE_INFINITY : divide(window, after);
    for (const word of window.words) {
      const startMs = word.startMs + window.offsetMs;
      const endMs = word.endMs + window.offsetMs;
      const middle = (startMs + endMs) / 2;
      if (middle < from || middle >= to) {
        continue;
      }
      // The two windows may place one word on either side of the line: it is one word.
      const last = merged.at(-1);
      if (last !== undefined && last.text.trim() === word.text.trim() && startMs < last.endMs) {
        continue;
      }
      merged.push({ ...word, startMs: startMs as TimeMs, endMs: endMs as TimeMs });
    }
  }
  return merged;
}

const within = (ms: number, durationMs: number): number => Math.min(Math.max(ms, 0), durationMs);

/**
 * Words in order, inside the clip, none starting before the one before it
 * ended, and none shorter than `ASR_MIN_WORD` where the next word allows.
 * The confidence is 1 until the recognizer's own figure is read (TE-2).
 */
export function postProcess(words: RawWord[], durationMs: DurMs): RawWord[] {
  const placed: RawWord[] = [];
  let previousEnd = 0;
  for (const word of words) {
    const startMs = Math.max(within(word.startMs, durationMs), previousEnd);
    const endMs = Math.max(within(word.endMs, durationMs), startMs);
    placed.push({ ...word, startMs: startMs as TimeMs, endMs: endMs as TimeMs, confidence: 1 as Confidence });
    previousEnd = endMs;
  }
  return placed.map((word, index) => {
    const room = Math.min(placed[index + 1]?.startMs ?? durationMs, durationMs);
    const endMs = Math.max(word.endMs, Math.min(word.startMs + ASR_MIN_WORD, room));
    return endMs === word.endMs ? word : { ...word, endMs: endMs as TimeMs };
  });
}
