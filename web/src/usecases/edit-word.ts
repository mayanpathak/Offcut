// A word edit, end to end (v4implementation D-146): the word's text as the
// user fixed it, through the update rule in mode `plan`, which finds the
// events of its sentence and the plan again before the scene is built. The
// word's timing never changes; an empty text hides the word, and the text it
// was heard as restores it.

import type { WordIdx } from "../gen/domain";
import { useClipStore } from "../state/clip-store";
import { update } from "./review-update";

/** Sets word `word` to `text`. A text the word already shows changes nothing. */
export function editWord(word: WordIdx, text: string): Promise<void> {
  const { transcript, edit } = useClipStore.getState();
  const heard = transcript?.words[word]?.text;
  if (heard === undefined || (edit.word_edits[word] ?? heard) === text) {
    return Promise.resolve();
  }
  return update(
    (current) => {
      // The word as heard is no edit: it is restored by leaving it out.
      const others = Object.entries(current.word_edits).filter(([at]) => Number(at) !== word);
      const word_edits = text === heard ? Object.fromEntries(others) : { ...Object.fromEntries(others), [word]: text };
      return { ...current, word_edits };
    },
    "plan",
    "word_edit",
  );
}
