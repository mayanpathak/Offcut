// "Fix it everywhere" (v4implementation D-175): after a word is fixed, every
// other word that was heard as the same word, and that the user has not
// edited, takes the fix too, in one change and so one undo step (D-184).
// Words are compared by their letters and digits, case-folded, whole: the
// punctuation around a word is its own and stays.

import type { EditState, WordIdx } from "../gen/domain";
import { useClipStore } from "../state/clip-store";
import { useHistoryStore } from "../state/history";
import { update } from "./review-update";

/** The punctuation before a word, its letters and digits, and the punctuation after. */
const PARTS = /^([^\p{L}\p{N}]*)(.*?)([^\p{L}\p{N}]*)$/su;

function partsOf(text: string): { before: string; core: string; after: string } {
  const [, before = "", core = "", after = ""] = PARTS.exec(text) ?? [];
  return { before, core, after };
}

function fold(text: string): string {
  return partsOf(text).core.toLocaleLowerCase();
}

/**
 * The one place of the page that makes a place in `transcript.words` a
 * `WordIdx` (v2implementation D-59): this file's, and the transcript
 * editor's, which lists every word.
 */
export function wordAt(i: number): WordIdx {
  return i as WordIdx;
}

/**
 * The words that would change with `fromWord`: heard as the same word, not
 * edited yet, `fromWord` itself left out. The toast's count (Prompt 115).
 */
export function replaceAllCandidates(fromWord: WordIdx): WordIdx[] {
  const { transcript, edit } = useClipStore.getState();
  const heard = transcript?.words[fromWord]?.text;
  if (transcript === undefined || heard === undefined || fold(heard) === "") {
    return [];
  }
  const same = fold(heard);
  const found: WordIdx[] = [];
  transcript.words.forEach((word, i) => {
    const at = wordAt(i);
    if (at !== fromWord && !(at in edit.word_edits) && fold(word.text) === same) {
      found.push(at);
    }
  });
  return found;
}

/** Whether two edits hold the same record, word edits and overrides compared by their entries. */
function sameRecord<V>(a: Readonly<Record<string, V>>, b: Readonly<Record<string, V>>, leaveOut?: string): boolean {
  const keys = new Set([...Object.keys(a), ...Object.keys(b)].filter((key) => key !== leaveOut));
  return [...keys].every((key) => a[key] === b[key]);
}

/**
 * Whether the history's last step is the edit of `fromWord` to `to` that the
 * toast was offered for: the step holds the edit before `previous`, and the
 * two differ in `fromWord`'s text alone. "Change all" then joins that step
 * (D-175: "Fix it everywhere" is one change), so one undo gives all of the
 * words back (v4implementation §20.4); after any other change it is a step
 * of its own.
 */
function joinsTheFix(fromWord: WordIdx, to: string): (previous: EditState) => boolean {
  return (previous) => {
    const top = useHistoryStore.getState().past.at(-1);
    if (top === undefined || previous.word_edits[fromWord] !== to || top.word_edits[fromWord] === to) {
      return false;
    }
    const others =
      top.style === previous.style &&
      top.crop_offset === previous.crop_offset &&
      top.caption_position === previous.caption_position &&
      top.highlight_numbers === previous.highlight_numbers &&
      sameRecord(top.event_overrides, previous.event_overrides);
    return others && sameRecord(top.word_edits, previous.word_edits, String(fromWord));
  };
}

/**
 * Gives every candidate of `fromWord` the text `to`, each keeping its own
 * punctuation; an empty `to` hides them. The events are found again over the
 * whole transcript (D-175). Right after the edit of `fromWord` it offers to
 * repeat, it is one undo step with that edit. Resolves with how many words
 * changed, `fromWord` not counted: 0 when there was nothing to change or the
 * change failed.
 */
export async function replaceAll(fromWord: WordIdx, to: string): Promise<number> {
  const words = replaceAllCandidates(fromWord);
  const { transcript } = useClipStore.getState();
  if (words.length === 0 || transcript === undefined) {
    return 0;
  }
  const core = partsOf(to).core;
  let made: unknown;
  await update(
    (current) => {
      const word_edits = { ...current.word_edits };
      for (const at of words) {
        const own = partsOf(transcript.words[at]?.text ?? "");
        word_edits[at] = core === "" ? "" : `${own.before}${core}${own.after}`;
      }
      const next = { ...current, word_edits };
      made = next;
      return next;
    },
    "plan",
    "replace_all",
    undefined,
    joinsTheFix(fromWord, to),
  );
  return useClipStore.getState().edit === made ? words.length : 0;
}
