// J9: signing in (TS C-8, v6implementation §13). The limits come above the email
// field, so nobody meets them as a surprise later. `SignInForm` is the form, on
// `/account` and on a callback whose link no longer works; `SignInDialog` is the
// same form as a modal `<dialog>`: the focus stays inside and Escape closes it.
//
// It submits through script only: the CSP has `form-action 'none'`.

import { type ReactNode, useEffect, useId, useRef, useState } from "react";

import { PRICES_USD } from "../../config/pricing";
import { messages } from "../../copy/messages";
import { type ErrorCode, LIMITS } from "../../gen/domain";
import { useAuthStore } from "../../state/auth-store";
import { type LinkOutcome, requestMagicLink, stopWaitingForLink } from "../../usecases/request-magic-link";
import { X } from "../icons";
import { IconButton } from "../primitives/IconButton";
import styles from "../styles/components.module.css";
import editor from "../styles/editor.module.css";

type Failure = Extract<LinkOutcome, { ok: false }>;

type Status = { kind: "idle" } | { kind: "sending" } | { kind: "waking" } | { kind: "error"; failure: Failure };

function errorText(code: ErrorCode): string {
  const copy: Partial<Record<ErrorCode, { title: string; action: string }>> = messages.errors;
  const words = copy[code];
  return words === undefined ? messages.editor.failedStub : `${words.title}. ${words.action}`;
}

function failureText(failure: Failure): string {
  if (failure.reason === "invalid_email") {
    return messages.notifyMe.invalidEmail;
  }
  return failure.code === "E_API_RATE_LIMITED" ? messages.notifyMe.rateLimited(failure.retryAfterSecs) : errorText(failure.code);
}

function statusText(status: Status): string {
  switch (status.kind) {
    case "idle":
      return "";
    case "sending":
      return messages.signIn.sending;
    case "waking":
      return messages.api.waking;
    case "error":
      return failureText(status.failure);
  }
}

/** Sends a link, keeping what the line under the form says. */
function useSend(): { status: Status; busy: boolean; send: (email: string) => Promise<void> } {
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const busy = status.kind === "sending" || status.kind === "waking";
  return {
    status,
    busy,
    send: async (email) => {
      if (busy) {
        return;
      }
      setStatus({ kind: "sending" });
      const outcome = await requestMagicLink(email, () => {
        // Only while still waiting: a late call must not undo the result.
        setStatus((current) => (current.kind === "sending" ? { kind: "waking" } : current));
      });
      setStatus(outcome.ok ? { kind: "idle" } : { kind: "error", failure: outcome });
    },
  };
}

function StatusLine({ status }: { status: Status }) {
  return (
    <p className={[styles.status, status.kind === "error" ? styles.statusError : undefined].filter(Boolean).join(" ")} role="status" data-testid="sign-in-status">
      {statusText(status)}
    </p>
  );
}

/** The wait for the link, with a way to ask for it again. */
function LinkSent({ email }: { email: string }) {
  const { status, busy, send } = useSend();
  return (
    <div className={styles.form} data-testid="sign-in-sent">
      <p className={styles.label}>{messages.signIn.sentTitle}</p>
      <p className={styles.notice}>{messages.signIn.sentBody(email)}</p>
      <p className={styles.notice}>{messages.signIn.spamHint}</p>
      <div className={styles.formRow}>
        <button
          className={styles.secondaryButton}
          type="button"
          disabled={busy}
          onClick={() => {
            void send(email);
          }}
        >
          {messages.signIn.resend}
        </button>
      </div>
      <StatusLine status={status} />
    </div>
  );
}

function EmailEntry() {
  const [email, setEmail] = useState("");
  const { status, busy, send } = useSend();
  const emailId = useId();
  return (
    <form
      className={styles.form}
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        void send(email);
      }}
    >
      <p className={styles.notice} data-testid="sign-in-limits">
        {messages.signIn.limits(LIMITS.FREE_EXPORTS_PER_MONTH, PRICES_USD.creatorMonthly)}
      </p>
      <label className={styles.label} htmlFor={emailId}>
        {messages.signIn.emailLabel}
      </label>
      <div className={styles.formRow}>
        <input
          id={emailId}
          className={styles.field}
          type="email"
          name="email"
          autoComplete="email"
          value={email}
          onChange={(event) => {
            setEmail(event.target.value);
          }}
          data-testid="sign-in-email"
        />
        <button className={styles.button} type="submit" disabled={busy}>
          {messages.signIn.submit}
        </button>
      </div>
      <StatusLine status={status} />
    </form>
  );
}

export function SignInForm() {
  const status = useAuthStore((state) => state.status);
  const email = useAuthStore((state) => state.email);
  return <div className={styles.signIn}>{status === "link_sent" && email !== null ? <LinkSent email={email} /> : <EmailEntry />}</div>;
}

/** A modal the gate opens, for as long as it is drawn. Escape, the close button and the backdrop all call `onClose`. */
export function GateDialog({ title, testId, onClose, children }: { title: string; testId: string; onClose: () => void; children: ReactNode }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const element = dialog.current;
    if (element !== null && !element.open) {
      element.showModal();
      // Past the close button, which would pop its tooltip: to what the person came to do.
      (element.querySelector("input") ?? element).focus();
    }
  }, []);

  return (
    <dialog
      ref={dialog}
      className={editor.sheet}
      tabIndex={-1}
      aria-labelledby={titleId}
      data-testid={testId}
      onClose={onClose}
      onClick={(event) => {
        // A click on the backdrop lands on the dialog itself; its content fills it, so a click there does not.
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div className={editor.sheetBody}>
        <div className={editor.sheetHead}>
          <h2 id={titleId} className={editor.sheetTitle}>
            {title}
          </h2>
          <IconButton icon={<X />} label={messages.signIn.close} size="sm" onClick={onClose} />
        </div>
        {children}
      </div>
    </dialog>
  );
}

/** `title` is the export gate's unless a caller names its own (V6.2 P206a: the AI plan's toast). */
export function SignInDialog({ onClose, title = messages.signIn.title }: { onClose: () => void; title?: string }) {
  return (
    <GateDialog
      title={title}
      testId="sign-in-dialog"
      onClose={() => {
        stopWaitingForLink();
        onClose();
      }}
    >
      <SignInForm />
    </GateDialog>
  );
}
