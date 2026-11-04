// `/auth/callback?token=…`: the emailed link (TS C-8). The token leaves the address
// bar first, before anything else can read or log the address, and only then is
// it verified. It is kept in memory for a retry after a failed connection.

import { useEffect, useRef, useState } from "react";
import { Link } from "react-router";

import { messages } from "../../copy/messages";
import type { ErrorCode } from "../../gen/domain";
import { completeSignIn } from "../../usecases/complete-sign-in";
import { SignInForm } from "../components/SignInDialog";
import styles from "../styles/pages.module.css";

type View = { kind: "verifying" } | { kind: "waking" } | { kind: "done" } | { kind: "failed"; code: ErrorCode };

/** The link's token, now out of the address. The state goes back as it was: the router keeps its own in it. */
function takeToken(): string | null {
  const token = new URLSearchParams(window.location.search).get("token");
  window.history.replaceState(window.history.state, "", window.location.pathname);
  return token;
}

/** A link that can never work offers a new one; any other failure, another try with this one. */
function linkIsDead(code: ErrorCode): boolean {
  return code === "E_AUTH_LINK_INVALID" || code === "E_AUTH_LINK_EXPIRED";
}

function Failed({ code, onRetry }: { code: ErrorCode; onRetry: () => void }) {
  const copy: Partial<Record<ErrorCode, { title: string; body: string; action: string }>> = messages.errors;
  const words = copy[code];
  return (
    <div className={styles.section} data-testid="callback-failed" data-code={code}>
      <h1 className={styles.title}>{words?.title ?? messages.editor.failedStub}</h1>
      {words === undefined ? null : <p>{words.body}</p>}
      {words === undefined ? null : <p>{words.action}</p>}
      {linkIsDead(code) ? (
        <SignInForm />
      ) : (
        <button type="button" onClick={onRetry}>
          {messages.recovery.tryAgain}
        </button>
      )}
    </div>
  );
}

export function AuthCallbackPage({ appPath }: { appPath: string }) {
  const [view, setView] = useState<View>({ kind: "verifying" });
  const token = useRef<string | null | undefined>(undefined);

  async function verify(): Promise<void> {
    const link = token.current;
    if (link === null || link === undefined) {
      setView({ kind: "failed", code: "E_AUTH_LINK_INVALID" });
      return;
    }
    setView({ kind: "verifying" });
    const outcome = await completeSignIn(link, () => {
      // Only while still waiting: a late call must not undo the result.
      setView((current) => (current.kind === "verifying" ? { kind: "waking" } : current));
    });
    setView(outcome.ok ? { kind: "done" } : { kind: "failed", code: outcome.code });
  }

  useEffect(() => {
    // Once for the page: the first read has already taken the token out of the address.
    if (token.current === undefined) {
      token.current = takeToken();
      void verify();
    }
  }, []);

  return (
    <main className={styles.page}>
      {view.kind === "verifying" || view.kind === "waking" ? (
        <section className={styles.section}>
          <h1 className={styles.title}>{messages.callback.verifying}</h1>
          <p role="status">{view.kind === "waking" ? messages.api.waking : ""}</p>
        </section>
      ) : null}
      {view.kind === "done" ? (
        <section className={styles.section} data-testid="callback-done">
          <h1 className={styles.title}>{messages.callback.signedIn}</h1>
          <p>{messages.callback.returnToTab}</p>
          <Link to={appPath}>{messages.callback.openApp}</Link>
        </section>
      ) : null}
      {view.kind === "failed" ? (
        <Failed
          code={view.code}
          onRetry={() => {
            void verify();
          }}
        />
      ) : null}
    </main>
  );
}
