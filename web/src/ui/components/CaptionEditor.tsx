// The transcript editor, the Captions tab (v4implementation D-182, §17.3):
// the transcript as flowing text in blocks that are the scene's caption
// chunks (`SceneReport.chunks`, never a second chunking here), each with the
// time it is shown from. While the preview plays, the spoken word is
// highlighted and, while "Follow playback" is on, kept in view; scrolling by
// hand turns it off. A click on a word seeks to it, pauses, and opens it as
// a field with its text selected: Enter or Tab commits (Tab opens the next
// word), Escape cancels, an empty commit hides the word, and leaving the
// field commits nothing. After a fix, "Change all" may be offered (D-175);
// after a word is hidden, a toast offers to undo it.
// No timeline (PS). It is the canvas's text alternative (D-192).

import { type KeyboardEvent, memo, useEffect, useMemo, useRef, useState } from "react";

import { messages } from "../../copy/messages";
import type { OutMs, Word, WordIdx } from "../../gen/domain";
import { useClipStore } from "../../state/clip-store";
import { usePreviewStore } from "../../state/preview-store";
import { useUiStore } from "../../state/ui-store";
import { pause, scrub, seek } from "../../usecases/control-preview";
import { editWord as fixWord } from "../../usecases/edit-word";
import { replaceAll, replaceAllCandidates, wordAt } from "../../usecases/replace-all";
import { MoreHorizontal } from "../icons";
import { stepHistory } from "./ToastHost";
import { Button } from "../primitives/Button";
import { IconButton } from "../primitives/IconButton";
import { PanelSection } from "../primitives/PanelSection";
import { Switch } from "../primitives/Switch";
import styles from "../styles/panel.module.css";
import { type Actions, WordField, WordMenu } from "./CaptionWordControls";

/** A word stays the spoken one this long after it ends, so the highlight does not flicker between words. */
const SPOKEN_HOLD_MS = 400;
/** Keys that scroll the panel: pressed outside a field, they are the user scrolling. */
const SCROLL_KEYS = new Set(["PageUp", "PageDown", "ArrowUp", "ArrowDown", "Home", "End"]);

/** The transcript editor's id: the preview canvas names it as its text alternative (D-192). */
export const TRANSCRIPT_ID = "transcript-editor";

type Block = { at: OutMs | null; words: WordIdx[] };

/**
 * The blocks: each chunk runs from its first word to the next chunk's first,
 * so a number's other words and a hidden word stay with the words they were
 * said with. Words before the first chunk belong to it.
 */
function blocksOf(count: number, chunks: readonly { start: WordIdx; at: OutMs }[]): Block[] {
  const starts = chunks.length === 0 ? [{ start: 0, at: null }] : chunks;
  return starts.map((chunk, i) => {
    const from = i === 0 ? 0 : chunk.start;
    const to = starts[i + 1]?.start ?? count;
    const words: WordIdx[] = [];
    for (let w = from; w < to; w += 1) {
      words.push(wordAt(w));
    }
    return { at: chunk.at, words };
  });
}

/** The word spoken at `t`: the last one started, while it lasts and a moment after. */
function spokenAt(words: readonly Word[], t: number): number | null {
  let low = 0;
  let high = words.length - 1;
  let found = -1;
  while (low <= high) {
    const mid = (low + high) >> 1;
    if ((words[mid]?.start_ms ?? Infinity) <= t) {
      found = mid;
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }
  const word = words[found];
  return word !== undefined && t < word.end_ms + SPOKEN_HOLD_MS ? found : null;
}

/** The word spoken now while the preview plays, followed every frame from the last clock sync; `null` otherwise. */
function useSpokenWord(words: readonly Word[]): number | null {
  const status = usePreviewStore((state) => state.status);
  const position = usePreviewStore((state) => state.position);
  const [spoken, setSpoken] = useState<number | null>(null);
  const synced = useRef({ ms: 0, at: 0 });
  useEffect(() => {
    synced.current = { ms: position ?? 0, at: performance.now() };
  }, [position]);
  useEffect(() => {
    if (status !== "playing") {
      setSpoken(null);
      return undefined;
    }
    let frame = 0;
    const tick = (): void => {
      setSpoken(spokenAt(words, synced.current.ms + performance.now() - synced.current.at));
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [status, words]);
  return spoken;
}

/** The button of word `i` in the editor, if it is drawn. */
function wordButton(root: HTMLElement | null, i: number): HTMLElement | null {
  return root?.querySelector<HTMLElement>(`button[data-word="${String(i)}"]`) ?? null;
}

type BlockProps = {
  block: Block;
  words: readonly Word[];
  edits: Readonly<Record<number, string>>;
  /** Each of these is `null` unless its word is in this block, so that only the blocks concerned draw again. */
  spoken: number | null;
  editing: number | null;
  hot: number | null;
  menu: WordIdx | null;
  /** The word that takes the Tab stop of the transcript (a roving tab index). */
  stop: number | null;
  actions: Actions;
};

const TranscriptBlock = memo(function TranscriptBlock(props: BlockProps) {
  const { block, words, edits, actions } = props;
  const { at } = block;
  const hot = props.hot === null ? null : wordAt(props.hot);
  return (
    <div className={styles.block}>
      {at === null ? null : (
        <button
          type="button"
          className={styles.blockTime}
          onClick={() => {
            // A block's time seeks without playing (§17.4).
            void pause().then(() => seek(at));
          }}
        >
          {messages.preview.clock(at)}
        </button>
      )}
      {block.words.map((w) => {
        const heard = words[w]?.text ?? "";
        const edit = edits[w];
        const hidden = edit === "";
        const kind = hidden ? styles.wordHidden : edit === undefined ? styles.word : styles.wordEdited;
        return (
          <span key={w} className={styles.wordSlot} onPointerEnter={() => actions.hot(w)}>
            {props.editing === w ? (
              <WordField word={w} text={hidden ? heard : (edit ?? heard)} actions={actions} />
            ) : (
              <button
                type="button"
                data-word={w}
                className={props.spoken === w ? `${kind ?? ""} ${styles.spoken ?? ""}` : kind}
                tabIndex={props.stop === w ? 0 : -1}
                onFocus={() => actions.focused(w)}
                onClick={() => actions.open(w)}
                onKeyDown={(e) => {
                  if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
                    e.preventDefault();
                    actions.menu(w);
                  }
                }}
              >
                {hidden ? heard : (edit ?? heard)}
              </button>
            )}
            {hidden ? (
              <Button variant="ghost" size="sm" onClick={() => actions.set(w, heard)}>
                {messages.captions.show}
              </Button>
            ) : null}
          </span>
        );
      })}
      {/* The hovered word's ⋯ and its menu, at the row's end: the text never moves under the pointer. */}
      {props.hot !== null && props.editing !== props.hot ? (
        <span className={styles.wordMore}>
          <IconButton icon={<MoreHorizontal />} label={messages.captions.wordMenu} size="sm" onClick={() => actions.menu(hot)} />
        </span>
      ) : null}
      {props.menu === null ? null : (
        <WordMenu word={props.menu} heard={words[props.menu]?.text ?? ""} edit={edits[props.menu]} actions={actions} />
      )}
    </div>
  );
});

export function CaptionEditor() {
  const transcript = useClipStore((state) => state.transcript);
  const chunks = useClipStore((state) => state.chunks);
  const edits = useClipStore((state) => state.edit.word_edits);
  const follow = useUiStore((state) => state.follow);
  const setFollow = useUiStore((state) => state.setFollow);
  const editing = useUiStore((state) => state.editingWord);
  const words = useMemo(() => transcript?.words ?? [], [transcript]);
  const blocks = useMemo(() => blocksOf(words.length, chunks), [words, chunks]);
  const spoken = useSpokenWord(words);
  const root = useRef<HTMLDivElement>(null);
  const [hot, setHot] = useState<WordIdx | null>(null);
  const [menu, setMenu] = useState<WordIdx | null>(null);
  const [stop, setStop] = useState<number>(0);

  // While following, the spoken word stays in view.
  useEffect(() => {
    if (follow && spoken !== null) {
      wordButton(root.current, spoken)?.scrollIntoView({ block: "nearest" });
    }
  }, [follow, spoken]);

  const actions = useRef<Actions>({
    open: (word) => {
      // While an export runs, the words wait for it, as every review control does (TS §12.4).
      if (usePreviewStore.getState().status === "locked") {
        return;
      }
      const ui = useUiStore.getState();
      ui.editWord(word);
      setMenu(null);
      // The preview goes to the word and stays there (D-182). Program time is the recording's (D-164).
      const at = useClipStore.getState().transcript?.words[word]?.start_ms ?? 0;
      void pause().then(() => scrub(at));
    },
    commit: (word, text, then) => {
      const ui = useUiStore.getState();
      const all = useClipStore.getState().transcript?.words ?? [];
      const step = then === "next" ? 1 : -1;
      // The next word along that is not hidden.
      let next: number = word + step;
      const shown = useClipStore.getState().edit.word_edits;
      while (then !== "close" && next >= 0 && next < all.length && shown[wordAt(next)] === "") {
        next += step;
      }
      const target = then !== "close" && next >= 0 && next < all.length ? wordAt(next) : null;
      ui.editWord(target);
      if (target === null) {
        // After the field has gone: back to the word's button.
        requestAnimationFrame(() => wordButton(root.current, word)?.focus());
      }
      const heard = all[word]?.text ?? "";
      const before = shown[word] ?? heard;
      void fixWord(word, text).then(() => {
        offerChangeAll(word, heard, text);
        offerUndoHide(word, heard, before);
      });
    },
    cancel: (word, refocus) => {
      const ui = useUiStore.getState();
      // A field left for another word's has nothing to close.
      if (ui.editingWord !== word) {
        return;
      }
      ui.editWord(null);
      if (refocus) {
        // After the field has gone: back to the word's button.
        requestAnimationFrame(() => wordButton(root.current, word)?.focus());
      }
    },
    set: (word, text) => {
      const heard = useClipStore.getState().transcript?.words[word]?.text ?? "";
      const before = useClipStore.getState().edit.word_edits[word] ?? heard;
      void fixWord(word, text).then(() => {
        offerUndoHide(word, heard, before);
      });
    },
    menu: (word) => {
      setMenu(word);
    },
    hot: (word) => {
      setHot(word);
    },
    focused: (word) => {
      setStop(word);
      setHot(word);
    },
  }).current;

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>): void => {
    if (e.target instanceof HTMLInputElement) {
      return;
    }
    // The words are one Tab stop; the arrow keys move along them.
    const at = e.target instanceof HTMLElement ? e.target.dataset.word : undefined;
    if (at !== undefined && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      e.preventDefault();
      wordButton(root.current, Number(at) + (e.key === "ArrowRight" ? 1 : -1))?.focus();
    } else if (SCROLL_KEYS.has(e.key)) {
      setFollow(false);
    }
  };

  return (
    <PanelSection
      title={messages.panel.captions}
      hint={status === "locked" ? messages.editor.availableAfterExport : messages.captions.hint}
    >
      <Switch
        checked={follow}
        label={messages.captions.follow}
        hint={messages.captions.followHint}
        onChange={(on) => {
          setFollow(on);
          const now = spoken ?? spokenAt(words, usePreviewStore.getState().position ?? 0);
          if (on && now !== null) {
            wordButton(root.current, now)?.scrollIntoView({ block: "nearest" });
          }
        }}
      />
      <div
        ref={root}
        id={TRANSCRIPT_ID}
        className={styles.transcript}
        data-testid="caption-editor"
        onWheel={() => setFollow(false)}
        onTouchMove={() => setFollow(false)}
        onKeyDown={onKeyDown}
        onPointerLeave={() => setHot(null)}
      >
        {blocks.map((block, i) => {
          const first = block.words[0] ?? 0;
          const last = block.words.at(-1) ?? -1;
          const inside = <T extends number>(w: T | null): T | null => (w !== null && w >= first && w <= last ? w : null);
          return (
            // By place: a new chunking updates the blocks where they are, and moves no element with the focus.
            <TranscriptBlock
              key={i}
              block={block}
              words={words}
              edits={edits}
              spoken={inside(spoken)}
              editing={inside(editing)}
              hot={inside(hot)}
              menu={inside(menu)}
              stop={inside(stop)}
              actions={actions}
            />
          );
        })}
      </div>
    </PanelSection>
  );
}

/** After a fix from `heard` to `text` that applied: "Change all N" when other words were heard the same (D-175). */
function offerChangeAll(word: WordIdx, heard: string, text: string): void {
  if (text === "" || text === heard || useClipStore.getState().edit.word_edits[word] !== text) {
    return;
  }
  const n = replaceAllCandidates(word).length;
  if (n === 0) {
    return;
  }
  useUiStore.getState().pushToast({
    id: "replace-all",
    message: messages.captions.replaceAll(n, heard, text),
    action: {
      label: messages.captions.changeAll,
      run: () => {
        void replaceAll(word, text);
      },
    },
  });
}

/**
 * After word `word` was hidden (an empty commit, or "Hide word"): "“…” hidden."
 * with "Undo" (§17.3). Undo is the history's step while nothing has changed
 * since; after another change, it gives the word back the text it had.
 */
function offerUndoHide(word: WordIdx, heard: string, before: string): void {
  const after = useClipStore.getState().edit;
  if (before === "" || after.word_edits[word] !== "") {
    return;
  }
  useUiStore.getState().pushToast({
    id: "hide",
    message: messages.captions.hidden(heard),
    action: {
      label: messages.editor.undo,
      run: () => {
        if (useClipStore.getState().edit === after) {
          void stepHistory("undo");
        } else {
          void fixWord(word, before);
        }
      },
    },
  });
}
