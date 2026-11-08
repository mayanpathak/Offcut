// The long page's layout (v6.1implementation D-423): the editor's bar (its
// mark and the account link) and one column on the stage. It reads no clip
// store: the two pages share components and tokens, not a store. The dark
// theme and the one polite live region, as the editor has them.

import type { ReactNode } from "react";

import { messages } from "../../copy/messages";
import styles from "../styles/editor.module.css";
import long from "../styles/long.module.css";
import primitives from "../styles/primitives.module.css";

export function LongLayout({
  accountPath,
  announcement,
  children,
}: {
  accountPath: string;
  announcement: string;
  children: ReactNode;
}) {
  return (
    <div data-theme="dark" className={long.page} data-testid="long-page">
      <header className={styles.topBar}>
        <span className={styles.mark}>{messages.editor.mark}</span>
        <span className={styles.spacer} />
        <a
          className={[primitives.button, primitives.ghost, primitives.md, styles.barLink].join(" ")}
          href={accountPath}
          target="_blank"
          rel="noopener"
          data-testid="account-link"
        >
          {messages.editor.account}
          <span className={styles.srOnly}> {messages.editor.newTab}</span>
        </a>
      </header>
      <main className={long.main}>
        <div className={long.column}>{children}</div>
      </main>
      <div className={styles.live} aria-live="polite">
        <span className={styles.srOnly}>{announcement}</span>
      </div>
    </div>
  );
}
