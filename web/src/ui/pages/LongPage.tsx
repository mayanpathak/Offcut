// `/app/long` (v6.1implementation §10.1, D-422): "Make shorts from a long
// video". What the page shows is where the source is: the drop zone (with
// "Continue your video" when a reload left one), the gate, the work, a
// refusal, a failure, or the transcript. A file of at most 90 s goes to `/app`.

import { useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router";

import { messages } from "../../copy/messages";
import { longErrors, longPageDuration } from "../../copy/long";
import { useAuthStore } from "../../state/auth-store";
import { useSourceStore } from "../../state/source-store";
import { continueSource, resumeSource } from "../../usecases/continue-source";
import { dropSource, recheckGate, takeHandOff } from "../../usecases/import-source";
import { cancelSource } from "../../usecases/run-source-pipeline";
import { DropZone } from "../components/DropZone";
import { SignInDialog } from "../components/SignInDialog";
import { SourceGate } from "../components/SourceGate";
import { SourceProgress } from "../components/SourceProgress";
import { SourceTranscript } from "../components/SourceTranscript";
import { AlertTriangle } from "../icons";
import { LongLayout } from "../layout/LongLayout";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";
import long from "../styles/long.module.css";

type Card = { title: string; body: string; action: string };

function errorCard(code: string): Card | undefined {
  if (code === "E_SOURCE_READ" || code === "E_SOURCE_AUDIO") {
    return longErrors[code];
  }
  const errors: Partial<Record<string, Card>> = messages.errors;
  const rejections: Partial<Record<string, Card>> = messages.rejections;
  return errors[code] ?? rejections[code];
}

/** A refusal or a failure: its words and what to do; Continue where a failure allows it. */
function Problem({ card, code, retry }: { card: Card | undefined; code: string; retry: boolean }) {
  return (
    <section className={styles.card} role="alert" data-testid="source-problem" data-code={code}>
      <span className={styles.cardIconWarn}>
        <AlertTriangle />
      </span>
      {card === undefined ? null : (
        <>
          <h2 className={styles.cardTitle}>{card.title}</h2>
          {/* A body that only repeats the title, with its full stop, is not said twice. */}
          {card.body.replace(/\.$/, "") === card.title ? null : <p className={styles.cardText}>{card.body}</p>}
          {card.action === "" ? null : <p className={styles.cardHint}>{card.action}</p>}
        </>
      )}
      <div className={styles.cardActions}>
        {retry ? (
          <Button
            variant="primary"
            onClick={() => {
              void continueSource();
            }}
          >
            {messages.long.continue}
          </Button>
        ) : null}
        <Button
          variant="secondary"
          onClick={() => {
            void cancelSource().finally(() => {
              useSourceStore.getState().clear();
            });
          }}
        >
          {messages.editor.startOver}
        </Button>
      </div>
    </section>
  );
}

function announcement(status: string, paused: string | undefined): string {
  if (status === "extracting_audio") {
    return messages.long.stage.audio;
  }
  if (status === "paused" && paused !== undefined) {
    return paused;
  }
  if (status === "transcribed") {
    return messages.long.done.title;
  }
  return "";
}

/** Whether the clip page's "Make shorts from it" led here (its navigation state). */
function handedOver(state: unknown): boolean {
  return typeof state === "object" && state !== null && "handOver" in state && state.handOver === true;
}

export function LongPage({ appPath, accountPath }: { appPath: string; accountPath: string }) {
  const navigate = useNavigate();
  const location = useLocation();
  const status = useSourceStore((state) => state.status);
  const rejected = useSourceStore((state) => state.rejected);
  const failure = useSourceStore((state) => state.failure);
  const resumable = useSourceStore((state) => state.resumable);
  const paused = useSourceStore((state) => state.paused);
  const signedIn = useAuthStore((state) => state.status === "signed_in");
  const [signingIn, setSigningIn] = useState(false);

  const take = (file: File): void => {
    void dropSource(file).then((outcome) => {
      if (outcome === "handed_over") {
        void navigate(appPath);
      }
    });
  };

  // A file handed over by the clip page's "Make shorts from it" (D-421), taken
  // only when that button brought the person here: a file left by a refusal
  // the person dismissed is let go, never started on.
  useEffect(() => {
    const file = takeHandOff("long");
    if (file !== undefined && handedOver(location.state)) {
      take(file);
    }
    // Once, on arrival.
  }, []);

  // A cancelled source leaves the drop zone.
  useEffect(() => {
    if (status === "cancelled") {
      useSourceStore.getState().clear();
    }
  }, [status]);

  // A sign-in that completes while the gate waits for it lets the gate ask
  // again: only from `signed_out`, whose answer is never `signed_out` again
  // once signed in, so asking cannot repeat itself.
  const waitsForSignIn = useSourceStore((state) => state.status === "gated" && state.gate?.reason === "signed_out");
  useEffect(() => {
    if (signedIn && waitsForSignIn) {
      void recheckGate();
    }
  }, [signedIn, waitsForSignIn]);

  let content;
  if (status === "gated" || status === "probing") {
    content = <SourceGate onSignIn={() => setSigningIn(true)} />;
  } else if (status === "rejected" && rejected !== null) {
    content = <Problem card={rejected === "REJECT_DURATION" ? longPageDuration : errorCard(rejected)} code={rejected} retry={false} />;
  } else if (status === "failed") {
    const code = failure?.code ?? "E_INTERNAL";
    content = <Problem card={errorCard(code)} code={code} retry={code !== "REJECT_NO_SPEECH"} />;
  } else if (status === "extracting_audio" || status === "transcribing" || status === "paused" || status === "awaiting_file") {
    content = <SourceProgress onSignIn={() => setSigningIn(true)} />;
  } else if (status === "transcribed") {
    content = <SourceTranscript />;
  } else {
    content = (
      <>
        {resumable === null ? null : (
          <div className={long.resume}>
            <Button
              variant="primary"
              onClick={() => {
                void resumeSource();
              }}
              data-testid="source-resume"
            >
              {messages.long.continueVideo}
            </Button>
          </div>
        )}
        <div className={long.stageSlot}>
          <DropZone variant="stage" kind="long" onFile={take} />
        </div>
      </>
    );
  }

  return (
    <LongLayout accountPath={accountPath} announcement={announcement(status, paused === null ? undefined : messages.long.paused[paused.reason])}>
      {content}
      {signingIn ? (
        <SignInDialog
          onClose={() => {
            setSigningIn(false);
          }}
        />
      ) : null}
    </LongLayout>
  );
}
