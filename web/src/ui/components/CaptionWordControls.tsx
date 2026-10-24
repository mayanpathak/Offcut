// The controls of one word of the transcript editor (v4implementation
// D-182), split from `CaptionEditor.tsx`: the open field, which commits only
// on Enter or Tab (Appendix A: never on leaving it), and the word's menu,
// "Hide" and "Restore original".

import { type KeyboardEvent, useEffect, useRef } from "react";

import { messages } from "../../copy/messages";
import type { WordIdx } from "../../gen/domain";
import styles from "../styles/panel.module.css";

/** What the editor's words do, the same object for every render: the blocks do not draw again for it. */
export type Actions = {
  open(word: WordIdx): void;
  commit(word: WordIdx, text: string, then: "close" | "next" | "previous"): void;
  /** Closes the field without a change; `refocus`: back to the word's button (Escape). */
  cancel(word: WordIdx, refocus: boolean): void;
  set(word: WordIdx, text: string): void;
  menu(word: WordIdx | null): void;
  hot(word: WordIdx | null): void;
  focused(word: WordIdx): void;
};

/** The open field of a word: its text selected; only Enter or Tab commits, Escape and leaving it do not. */
export function WordField({ word, text, actions }: { word: WordIdx; text: string; actions: Actions }) {
  const field = useRef<HTMLInputElement>(null);
  const closed = useRef(false);
  useEffect(() => {
    field.current?.focus();
    field.current?.select();
    field.current?.scrollIntoView({ block: "nearest" });
  }, []);
  const close = (how: () => void): void => {
    closed.current = true;
    how();
  };
  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>): void => {
    const value = e.currentTarget.value.trim();
    if (e.key === "Enter") {
      e.preventDefault();
      close(() => actions.commit(word, value, "close"));
    } else if (e.key === "Tab") {
      e.preventDefault();
      close(() => actions.commit(word, value, e.shiftKey ? "previous" : "next"));
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      close(() => actions.cancel(word, true));
    }
  };
  return (
    <input
      ref={field}
      className={styles.wordField}
      defaultValue={text}
      aria-label={messages.captions.wordField}
      onKeyDown={onKeyDown}
      onBlur={() => {
        // Judged a frame later: a field drawn again by a scene that came in
        // meanwhile (the blocks follow the new chunks) is still the open one.
        requestAnimationFrame(() => {
          const active = document.activeElement;
          const refocused = active instanceof HTMLInputElement && active.className === field.current?.className;
          if (!closed.current && !refocused) {
            closed.current = true;
            actions.cancel(word, false);
          }
        });
      }}
    />
  );
}

/** A word's menu (⋯, or the context-menu key): hide it, or restore what was heard. */
export function WordMenu({ word, heard, edit, actions }: { word: WordIdx; heard: string; edit: string | undefined; actions: Actions }) {
  const menu = useRef<HTMLDivElement>(null);
  useEffect(() => {
    menu.current?.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
  }, []);
  const items = [
    { label: messages.captions.hide, disabled: edit === "", run: () => actions.set(word, "") },
    { label: messages.captions.restore, disabled: edit === undefined, run: () => actions.set(word, heard) },
  ];
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>): void => {
    const buttons = [...(menu.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
    const at = buttons.findIndex((b) => b === document.activeElement);
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      actions.menu(null);
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const step = e.key === "ArrowDown" ? 1 : buttons.length - 1;
      buttons[(at + step) % buttons.length]?.focus();
    }
  };
  return (
    <div
      ref={menu}
      role="menu"
      aria-label={messages.captions.wordMenu}
      className={styles.wordMenu}
      onKeyDown={onKeyDown}
      onBlur={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget)) {
          actions.menu(null);
        }
      }}
    >
      {items.map((item) => (
        <button
          key={item.label}
          type="button"
          role="menuitem"
          className={styles.menuItem}
          disabled={item.disabled}
          onClick={() => {
            actions.menu(null);
            item.run();
          }}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
