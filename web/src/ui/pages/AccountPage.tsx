// `/account` (TS C-11, C-18; v6implementation §13). Signed out, the sign-in form.
// Signed in: the plan and its counter; the two offers, or the portal for a
// subscription that is live (D-300); the account's data and its deletion; and a
// way out. A return from the merchant's checkout is read once, taken out of the
// address, and confirmed here whatever the session says, as it may still be
// restoring.

import { useCallback, useEffect, useId, useRef, useState } from "react";
import { Link } from "react-router";

import { PRICES_USD } from "../../config/pricing";
import { messages } from "../../copy/messages";
import type { MeResponse, Offer, SubscriptionSummary } from "../../gen/api";
import { type ErrorCode, LIMITS } from "../../gen/domain";
import { useAuthStore } from "../../state/auth-store";
import { effectiveNow } from "../../state/blockers";
import { useEntitlementStore } from "../../state/entitlement-store";
import { type CheckoutReturn, type ConfirmOutcome, confirmCheckout, readAccount, waitForCreator } from "../../usecases/confirm-checkout";
import { deleteAccount } from "../../usecases/delete-account";
import { exportAccountData } from "../../usecases/export-account-data";
import { openBillingPortal } from "../../usecases/open-billing-portal";
import { signOut } from "../../usecases/sign-out";
import { startCheckout } from "../../usecases/start-checkout";
import { SignInForm } from "../components/SignInDialog";
import components from "../styles/components.module.css";
import styles from "../styles/pages.module.css";

/** An error's title and what to do, on one line. */
function errorText(code: ErrorCode): string {
  const copy: Partial<Record<ErrorCode, { title: string; action: string }>> = messages.errors;
  const words = copy[code];
  return words === undefined ? messages.editor.failedStub : `${words.title}. ${words.action}`;
}

/** A line that says how an action went: `role="status"`, so a reader hears it. */
function StatusLine({ text, failed = false, testId }: { text: string; failed?: boolean; testId: string }) {
  return (
    <p className={[components.status, failed ? components.statusError : undefined].filter(Boolean).join(" ")} role="status" data-testid={testId}>
      {text}
    </p>
  );
}

/** What a line under a button says: nothing, a wait, or what went wrong. */
type Line = { kind: "idle" } | { kind: "busy"; text: string } | { kind: "done"; text: string } | { kind: "failed"; code: ErrorCode };

function lineText(line: Line): string {
  switch (line.kind) {
    case "idle":
      return "";
    case "busy":
    case "done":
      return line.text;
    case "failed":
      return errorText(line.code);
  }
}

/** Runs one action at a time behind a line. `run` answers with the line it ends on. */
function useAction(): { line: Line; busy: boolean; run: (busyText: string, act: () => Promise<Line>) => void } {
  const [line, setLine] = useState<Line>({ kind: "idle" });
  const busy = line.kind === "busy";
  return {
    line,
    busy,
    run: (busyText, act) => {
      if (busy) {
        return;
      }
      setLine({ kind: "busy", text: busyText });
      void act().then(setLine);
    },
  };
}

// --- The return from checkout ---------------------------------------------

type Notice = { kind: "none" } | { kind: "confirming" } | ConfirmOutcome;

/** `?checkout=` as the merchant sent the person back, or `null`. */
function readReturn(): CheckoutReturn | null {
  const value = new URLSearchParams(window.location.search).get("checkout");
  return value === "success" || value === "cancel" ? value : null;
}

/** Read once and taken out of the address, so that a reload neither asks again nor counts the return twice. */
function useCheckoutReturn(onActive: (me: MeResponse) => void): { notice: Notice; again: () => void } {
  const [notice, setNotice] = useState<Notice>({ kind: "none" });
  const settle = useCallback(
    (outcome: ConfirmOutcome) => {
      setNotice(outcome);
      if (outcome.kind === "active") {
        onActive(outcome.me);
      }
    },
    [onActive],
  );
  useEffect(() => {
    // Once: after this the address holds no return, so a second run finds none.
    const result = readReturn();
    if (result === null) {
      return;
    }
    window.history.replaceState(window.history.state, "", window.location.pathname);
    setNotice(result === "success" ? { kind: "confirming" } : { kind: "cancelled" });
    void confirmCheckout(result).then(settle);
  }, [settle]);
  return {
    notice,
    again: () => {
      setNotice({ kind: "confirming" });
      void waitForCreator().then(settle);
    },
  };
}

function CheckoutNotice({ notice, again }: { notice: Notice; again: () => void }) {
  switch (notice.kind) {
    case "none":
      return null;
    case "confirming":
      return <StatusLine text={messages.account.confirming} testId="checkout-notice" />;
    case "active":
      return <StatusLine text={messages.account.active} testId="checkout-notice" />;
    case "cancelled":
      return <StatusLine text={messages.account.cancelled} testId="checkout-notice" />;
    case "failed":
      return <StatusLine text={errorText(notice.code)} failed testId="checkout-notice" />;
    case "pending": {
      const { title, body, action } = messages.errors.E_BILLING_PENDING;
      return (
        <div className={styles.card} role="status" data-testid="checkout-notice" data-code="E_BILLING_PENDING">
          <p className={styles.cardTitle}>{title}</p>
          <p className={styles.text}>{body}</p>
          <p className={styles.text}>{action}</p>
          <div className={components.formRow}>
            <button className={components.secondaryButton} type="button" onClick={again}>
              {messages.account.checkAgain}
            </button>
          </div>
        </div>
      );
    }
  }
}

// --- Signed in --------------------------------------------------------------

/** A subscription the merchant still bills: a second one is refused (D-300). */
const isLive = (subscription: SubscriptionSummary | null) => subscription !== null && subscription.status !== "canceled";

function SubscriptionLine({ subscription }: { subscription: SubscriptionSummary }) {
  const date = messages.account.date(subscription.current_period_end);
  const { interval, term, renews, ends, pastDue } = messages.account;
  let when: string;
  if (subscription.status === "past_due") {
    when = pastDue;
  } else if (subscription.status === "canceled" || subscription.cancel_at_period_end) {
    when = ends(date);
  } else {
    when = renews(date);
  }
  return (
    <p className={styles.text} data-testid="account-subscription">
      {term(interval[subscription.interval], when)}
    </p>
  );
}

/** The plan as this page's token says, with what can be bought or managed. */
function Plan({ me }: { me: MeResponse | null }) {
  // Read so that the plan is drawn again when the token or the outbox changes.
  useEntitlementStore((state) => state.claims);
  useEntitlementStore((state) => state.outboxFree);
  const now = effectiveNow();
  const billing = useAction();
  const subscription = me?.subscription ?? null;
  const headingId = useId();
  const buy = (offer: Offer) => {
    billing.run(messages.upgrade.opening, async () => {
      const outcome = await startCheckout(offer);
      // On success the page is leaving for the merchant's: the line says so until it has.
      return outcome.ok ? { kind: "busy", text: messages.upgrade.opening } : { kind: "failed", code: outcome.code };
    });
  };
  return (
    <section className={styles.section} aria-labelledby={headingId} data-testid="account-plan">
      <h2 className={styles.heading} id={headingId}>
        {messages.account.planHeading}
      </h2>
      {now === null ? null : (
        <p className={styles.text} data-testid="account-plan-line">
          {now.plan === "free" ? messages.account.free(now.freeExportsRemaining, LIMITS.FREE_EXPORTS_PER_MONTH) : messages.account.creator}
        </p>
      )}
      {subscription === null ? null : <SubscriptionLine subscription={subscription} />}
      {now?.plan === "free" && !isLive(subscription) ? (
        <>
          <p className={styles.text}>{messages.upgrade.afterExport(PRICES_USD.creatorMonthly)}</p>
          <div className={components.formRow}>
            <button
              className={components.button}
              type="button"
              disabled={billing.busy}
              onClick={() => {
                buy("creator_monthly");
              }}
            >
              {messages.upgrade.monthly(PRICES_USD.creatorMonthly)}
            </button>
            <button
              className={components.secondaryButton}
              type="button"
              disabled={billing.busy}
              onClick={() => {
                buy("creator_annual");
              }}
            >
              {messages.upgrade.annual(PRICES_USD.creatorAnnual)}
            </button>
          </div>
        </>
      ) : null}
      {subscription === null ? null : (
        <div className={components.formRow}>
          <button
            className={components.secondaryButton}
            type="button"
            disabled={billing.busy}
            onClick={() => {
              billing.run(messages.account.opening, async () => {
                const outcome = await openBillingPortal();
                return outcome.ok ? { kind: "busy", text: messages.account.opening } : { kind: "failed", code: outcome.code };
              });
            }}
          >
            {messages.account.manage}
          </button>
        </div>
      )}
      <StatusLine text={lineText(billing.line)} failed={billing.line.kind === "failed"} testId="billing-status" />
    </section>
  );
}

/** A modal that asks before the account goes. The safe answer has the focus; Escape is that answer too. */
function DeleteDialog({ onClose, onDeleted }: { onClose: () => void; onDeleted: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const keep = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const action = useAction();
  const { title, body, confirm, keep: keepText } = messages.account.deleteDialog;

  useEffect(() => {
    const element = dialog.current;
    if (element !== null && !element.open) {
      element.showModal();
      keep.current?.focus();
    }
  }, []);

  return (
    <dialog ref={dialog} className={styles.dialog} aria-labelledby={titleId} data-testid="delete-dialog" onClose={onClose}>
      <h2 className={styles.heading} id={titleId}>
        {title}
      </h2>
      <p className={styles.text}>{body}</p>
      <div className={components.formRow}>
        <button ref={keep} className={components.secondaryButton} type="button" onClick={onClose}>
          {keepText}
        </button>
        <button
          className={styles.dangerButton}
          type="button"
          disabled={action.busy}
          onClick={() => {
            action.run(messages.account.deleting, async () => {
              const outcome = await deleteAccount();
              if (outcome.ok) {
                onDeleted();
                return { kind: "idle" };
              }
              return { kind: "failed", code: outcome.code };
            });
          }}
        >
          {confirm}
        </button>
      </div>
      <StatusLine text={lineText(action.line)} failed={action.line.kind === "failed"} testId="delete-status" />
    </dialog>
  );
}

function AccountData({ onDeleted }: { onDeleted: () => void }) {
  const [asking, setAsking] = useState(false);
  const data = useAction();
  const headingId = useId();
  return (
    <section className={styles.section} aria-labelledby={headingId} data-testid="account-data">
      <h2 className={styles.heading} id={headingId}>
        {messages.account.dataHeading}
      </h2>
      <div className={components.formRow}>
        <button
          className={components.secondaryButton}
          type="button"
          disabled={data.busy}
          onClick={() => {
            data.run(messages.account.preparing, async () => {
              const outcome = await exportAccountData();
              return outcome.ok ? { kind: "done", text: messages.account.exported } : { kind: "failed", code: outcome.code };
            });
          }}
        >
          {messages.account.exportData}
        </button>
        <button
          className={styles.dangerButton}
          type="button"
          onClick={() => {
            setAsking(true);
          }}
        >
          {messages.account.delete}
        </button>
      </div>
      <StatusLine text={lineText(data.line)} failed={data.line.kind === "failed"} testId="data-status" />
      {asking ? (
        <DeleteDialog
          onClose={() => {
            setAsking(false);
          }}
          onDeleted={onDeleted}
        />
      ) : null}
    </section>
  );
}

function SignOut() {
  const [failed, setFailed] = useState<ErrorCode | null>(null);
  return (
    <div className={styles.section}>
      <div className={components.formRow}>
        <button
          className={components.secondaryButton}
          type="button"
          onClick={() => {
            void signOut().then((outcome) => {
              setFailed(outcome.ok ? null : outcome.code);
            });
          }}
        >
          {messages.account.signOut}
        </button>
      </div>
      <StatusLine text={failed === null ? "" : errorText(failed)} failed={failed !== null} testId="sign-out-status" />
    </div>
  );
}

function SignedIn({ me, onDeleted }: { me: MeResponse | null; onDeleted: () => void }) {
  const email = useAuthStore((state) => state.email);
  return (
    <div className={styles.section} data-testid="account-signed-in">
      <p className={styles.text}>{email === null ? messages.account.signedIn : messages.account.email(email)}</p>
      <Plan me={me} />
      <AccountData onDeleted={onDeleted} />
      <SignOut />
    </div>
  );
}

export function AccountPage({ homePath }: { homePath: string }) {
  const status = useAuthStore((state) => state.status);
  const [me, setMe] = useState<MeResponse | null>(null);
  const [read, setRead] = useState<ErrorCode | null>(null);
  const [deleted, setDeleted] = useState(false);
  const checkout = useCheckoutReturn(setMe);
  const signedIn = status === "signed_in";

  // What the account holds: its subscription, which the token does not say.
  useEffect(() => {
    if (!signedIn) {
      return;
    }
    void readAccount().then((result) => {
      if (result.ok) {
        setMe(result.me);
        setRead(null);
      } else {
        setRead(result.code);
      }
    });
  }, [signedIn]);

  return (
    <main className={styles.page}>
      <section className={styles.section}>
        <Link to={homePath}>{messages.settings.backLink}</Link>
        <h1 className={styles.title}>{messages.account.title}</h1>
        <CheckoutNotice notice={checkout.notice} again={checkout.again} />
        {deleted && !signedIn ? <StatusLine text={messages.account.deleted} testId="account-deleted" /> : null}
        {signedIn && read !== null ? <StatusLine text={errorText(read)} failed testId="account-status" /> : null}
        {signedIn ? (
          <SignedIn
            me={me}
            onDeleted={() => {
              setDeleted(true);
            }}
          />
        ) : (
          <SignInForm />
        )}
      </section>
    </main>
  );
}
