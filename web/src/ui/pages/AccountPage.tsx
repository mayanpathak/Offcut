// `/account`. For now, the sign-in form, and once signed in a way out. The plan,
// the counter, the subscription and the data controls arrive with the checkout.

import { useState } from "react";
import { Link } from "react-router";

import { messages } from "../../copy/messages";
import type { ErrorCode } from "../../gen/domain";
import { useAuthStore } from "../../state/auth-store";
import { signOut } from "../../usecases/sign-out";
import { SignInForm } from "../components/SignInDialog";
import styles from "../styles/pages.module.css";

function SignedIn() {
  const [failed, setFailed] = useState<ErrorCode | null>(null);
  const copy: Partial<Record<ErrorCode, { title: string; action: string }>> = messages.errors;
  const words = failed === null ? undefined : copy[failed];
  return (
    <div className={styles.section} data-testid="account-signed-in">
      <p>{messages.account.signedIn}</p>
      <div>
        <button
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
      <p role="status" data-testid="sign-out-status">
        {failed === null ? "" : words === undefined ? messages.editor.failedStub : `${words.title}. ${words.action}`}
      </p>
    </div>
  );
}

export function AccountPage({ homePath }: { homePath: string }) {
  const status = useAuthStore((state) => state.status);
  return (
    <main className={styles.page}>
      <section className={styles.section}>
        <Link to={homePath}>{messages.settings.backLink}</Link>
        <h1 className={styles.title}>{messages.account.title}</h1>
        {status === "signed_in" ? <SignedIn /> : <SignInForm />}
      </section>
    </main>
  );
}
