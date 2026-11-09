// The gate before a long video's work (v6.1implementation §10.2): one reason
// at a time, in order: signed out (Sign in), the cloud being checked or off,
// too little room on the disk, then the line on what is sent and Start. Only
// one primary button is shown at a time.

import { messages } from "../../copy/messages";
import { useSourceStore } from "../../state/source-store";
import { startSource } from "../../usecases/import-source";
import { AlertTriangle } from "../icons";
import { Button } from "../primitives/Button";
import { Spinner } from "../primitives/Spinner";
import styles from "../styles/components.module.css";

export function SourceGate({ onSignIn }: { onSignIn: () => void }) {
  const gate = useSourceStore((state) => state.gate);
  const copy = messages.long.gate;
  const reason = gate?.reason ?? "checking";
  return (
    <section className={styles.card} data-testid="source-gate" data-reason={reason}>
      {reason === "checking" ? (
        <p className={styles.cardText}>
          <Spinner /> {copy.checking}
        </p>
      ) : null}
      {reason === "signed_out" ? (
        <>
          <p className={styles.cardText}>{copy.signedOut}</p>
          <div className={styles.cardActions}>
            <Button variant="primary" onClick={onSignIn}>
              {copy.signIn}
            </Button>
          </div>
        </>
      ) : null}
      {reason === "unavailable" ? (
        <p className={styles.cardText} role="alert">
          <span className={styles.cardIconWarn}>
            <AlertTriangle />
          </span>{" "}
          {copy.unavailable}
        </p>
      ) : null}
      {reason === "storage_low" ? (
        <p className={styles.cardText} role="alert">
          {messages.blockers.B_STORAGE_LOW}
        </p>
      ) : null}
      {reason === "ready" ? (
        <>
          <p className={styles.cardText}>{copy.sends}</p>
          <div className={styles.cardActions}>
            <Button
              variant="primary"
              onClick={() => {
                void startSource();
              }}
              data-testid="source-start"
            >
              {copy.start}
            </Button>
          </div>
        </>
      ) : null}
    </section>
  );
}
