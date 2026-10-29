// The shortcuts sheet (v4implementation D-185): the keyboard map of
// `use-shortcuts.ts`, each shortcut with its keys. Opened with `?` or the top
// bar's keyboard button. A modal `<dialog>`: it takes the focus itself when
// it opens (so its title is read, and the close button's tooltip does not
// pop up), the focus stays inside while it is open, Escape and the close
// button close it, and the focus goes back to where it was. It fades and
// rises in over --base (§17.4).

import { Fragment, useEffect, useRef } from "react";

import { messages } from "../../copy/messages";
import { useUiStore } from "../../state/ui-store";
import { X } from "../icons";
import { IconButton } from "../primitives/IconButton";
import { isMac, Kbd, type KbdProps } from "../primitives/Kbd";
import styles from "../styles/editor.module.css";

const keys = messages.keys;

/** Each row: what it does, and its keys; more than one set of keys reads "or" between them. */
function rows(): { label: string; keys: KbdProps[] }[] {
  const s = messages.shortcuts;
  return [
    { label: s.playPause, keys: [{ keys: [keys.space] }] },
    { label: s.stepFrame, keys: [{ keys: [keys.left] }, { keys: [keys.right] }] },
    { label: s.stepSecond, keys: [{ keys: [keys.shift, keys.left] }, { keys: [keys.shift, keys.right] }] },
    { label: messages.editor.undo, keys: [{ keys: [keys.z], mod: true }] },
    {
      label: messages.editor.redo,
      // Ctrl+Y is redo on Windows, not on macOS.
      keys: [{ keys: [keys.shift, keys.z], mod: true }, ...(isMac() ? [] : [{ keys: [keys.modOther, keys.y] }])],
    },
    { label: s.close, keys: [{ keys: [keys.escape] }] },
    { label: s.help, keys: [{ keys: [keys.question] }] },
  ];
}

export function ShortcutsSheet() {
  const open = useUiStore((state) => state.shortcuts);
  const show = useUiStore((state) => state.showShortcuts);
  const dialog = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const element = dialog.current;
    if (element === null) {
      return;
    }
    if (open && !element.open) {
      element.showModal();
      element.focus();
    } else if (!open && element.open) {
      element.close();
    }
  }, [open]);

  return (
    <dialog
      ref={dialog}
      className={styles.sheet}
      tabIndex={-1}
      aria-labelledby="shortcuts-title"
      data-testid="shortcuts-sheet"
      // Escape, the close button, or `close()` above: the store follows.
      onClose={() => {
        show(false);
      }}
      onClick={(event) => {
        // A click on the backdrop lands on the dialog itself; its content fills it, so a click there does not.
        if (event.target === event.currentTarget) {
          show(false);
        }
      }}
    >
      <div className={styles.sheetBody}>
        <div className={styles.sheetHead}>
          <h2 id="shortcuts-title" className={styles.sheetTitle}>
            {messages.shortcuts.title}
          </h2>
          <IconButton
            icon={<X />}
            label={messages.shortcuts.dismiss}
            size="sm"
            onClick={() => {
              show(false);
            }}
          />
        </div>
        <dl className={styles.sheetList}>
          {rows().map((row) => (
            <div key={row.label} className={styles.sheetRow}>
              <dt>{row.label}</dt>
              <dd className={styles.sheetKeys}>
                {row.keys.map((k, i) => (
                  <Fragment key={k.keys.join("+") + String(k.mod ?? false)}>
                    {i > 0 ? <span className={styles.sheetOr}>{keys.or}</span> : null}
                    <Kbd {...k} />
                  </Fragment>
                ))}
              </dd>
            </div>
          ))}
        </dl>
      </div>
    </dialog>
  );
}
