// The waitlist form, also used on the unsupported page (C-16).
//
// It submits through script only. The CSP has `form-action 'none'`, so a
// native form post would be blocked; the submit handler always prevents it.

import { useId, useState } from "react";

import { messages } from "../../copy/messages";
import type { Wanted } from "../../gen/api";
import { type NotifyMeOutcome, submitNotifyMe } from "../../usecases/submit-notify-me";
import styles from "../styles/components.module.css";

type Failure = Extract<NotifyMeOutcome, { ok: false }>;

type Status =
  | { kind: "idle" }
  | { kind: "sending" }
  // The request is taking long: the API is probably waking up.
  | { kind: "waking" }
  | { kind: "done" }
  | { kind: "error"; failure: Failure };

function failureText(failure: Failure): string {
  const copy = messages.notifyMe;
  const byReason: Record<Failure["reason"], string> = {
    invalid_email: copy.invalidEmail,
    rate_limited: copy.rateLimited(failure.retryAfterSecs),
    offline: copy.offline,
    unavailable: copy.unavailable,
  };
  return byReason[failure.reason];
}

function statusText(status: Status): string {
  if (status.kind === "error") {
    return failureText(status.failure);
  }
  const byKind: Record<Exclude<Status["kind"], "error">, string> = {
    idle: "",
    sending: messages.notifyMe.sending,
    waking: messages.api.waking,
    done: messages.notifyMe.success,
  };
  return byKind[status.kind];
}

const STATUS_CLASS: Partial<Record<Status["kind"], string | undefined>> = {
  done: styles.statusDone,
  error: styles.statusError,
};

/**
 * `wanted` lists what the visitor can ask to be told about. With more than
 * one entry the form shows a choice, starting at `preselect`.
 */
export function NotifyMeForm({ wanted, preselect }: { wanted: readonly Wanted[]; preselect?: Wanted }) {
  const [email, setEmail] = useState("");
  const [choice, setChoice] = useState(preselect ?? wanted[0]);
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const emailId = useId();
  const choiceId = useId();

  const busy = status.kind === "sending" || status.kind === "waking";
  const hasChoice = wanted.length > 1;

  async function submit(): Promise<void> {
    if (busy || choice === undefined) {
      return;
    }
    setStatus({ kind: "sending" });
    const outcome = await submitNotifyMe(email, choice, () => {
      // Only while still waiting: a late call must not undo the result.
      setStatus((current) => (current.kind === "sending" ? { kind: "waking" } : current));
    });
    if (outcome.ok) {
      setEmail("");
      setStatus({ kind: "done" });
    } else {
      setStatus({ kind: "error", failure: outcome });
    }
  }

  return (
    <form
      className={styles.form}
      noValidate
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <label className={styles.label} htmlFor={hasChoice ? choiceId : emailId}>
        {hasChoice ? messages.notifyMe.labelUnsupported : messages.notifyMe.label}
      </label>
      <div className={styles.formRow}>
        {hasChoice && (
          <select
            id={choiceId}
            className={styles.field}
            value={choice}
            onChange={(event) => {
              setChoice(wanted.find((option) => option === event.target.value));
            }}
          >
            {wanted.map((option) => (
              <option key={option} value={option}>
                {messages.notifyMe.wanted[option]}
              </option>
            ))}
          </select>
        )}
        <input
          id={emailId}
          className={styles.field}
          type="email"
          name="email"
          autoComplete="email"
          aria-label={messages.notifyMe.emailLabel}
          placeholder={messages.notifyMe.emailLabel}
          value={email}
          onChange={(event) => {
            setEmail(event.target.value);
          }}
        />
        <button className={styles.button} type="submit" disabled={busy}>
          {messages.notifyMe.submit}
        </button>
      </div>
      <p className={[styles.status, STATUS_CLASS[status.kind]].filter(Boolean).join(" ")} role="status">
        {statusText(status)}
      </p>
    </form>
  );
}
