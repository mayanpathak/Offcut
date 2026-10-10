// `/app`: the editor. One clip at a time, and what the page shows is where
// that clip is on its way: the drop zone, the processing feed, the preview
// with the export, or what went wrong. From the preview and from a clip that
// ended badly there is a way to start over (J2 to J10).

import type { ReactNode } from "react";

import { messages } from "../../copy/messages";
import type { ErrorCode } from "../../gen/domain";
import { forImport } from "../../state/blockers";
import { type ClipState, useClipStore } from "../../state/clip-store";
import { useExportStore } from "../../state/export-store";
import { useModelStore } from "../../state/model-store";
import { dismissClip } from "../../usecases/import-clip";
import { DropZone } from "../components/DropZone";
import { ExportButton } from "../components/ExportButton";
import { ExportProgress } from "../components/ExportProgress";
import { ModelDownloadPanel } from "../components/ModelDownloadPanel";
import { PreviewPlayer } from "../components/PreviewPlayer";
import { ProcessingFeed } from "../components/ProcessingFeed";
import components from "../styles/components.module.css";
import styles from "../styles/pages.module.css";

function assertNever(value: never): never {
  void value;
  throw new Error("a state of the clip this build does not know");
}

/** The three failures of the model download have words of their own (D-57). */
function failureCopy(code: ErrorCode | undefined) {
  if (code === "E_MODEL_DOWNLOAD" || code === "E_MODEL_HASH" || code === "E_MODEL_STORAGE") {
    return messages.errors[code];
  }
  return undefined;
}

/**
 * "Start over": lets go of the clip and brings the drop zone back. While an
 * export runs the clip cannot be let go, and the button waits with it.
 */
function StartOver() {
  // Read so that the button is drawn again when an export starts and when it ends.
  useExportStore((state) => state.status);
  const waiting = forImport() !== null;
  return (
    <button
      className={[components.secondaryButton, waiting ? components.pending : undefined].filter(Boolean).join(" ")}
      type="button"
      disabled={waiting}
      onClick={() => {
        void dismissClip();
      }}
    >
      {messages.editor.startOver}
    </button>
  );
}

/** What is said of a clip that ended badly, and the way on. No code and nothing of the file is shown. */
function Stub({ children, testId }: { children: ReactNode; testId: string }) {
  return (
    <section className={styles.stub} data-testid={testId}>
      {children}
      <StartOver />
    </section>
  );
}

function Processing({ waitingModel }: { waitingModel: boolean }) {
  const model = useModelStore((state) => state.status);
  // The download is shown while the clip waits for it, and what stopped it.
  const showModel = waitingModel && (model === "downloading" || model === "verifying" || model === "failed");
  return (
    <section className={styles.editor}>
      {showModel && <ModelDownloadPanel />}
      <ProcessingFeed />
    </section>
  );
}

function Failed({ code }: { code: ErrorCode | undefined }) {
  const copy = failureCopy(code);
  return (
    <Stub testId="clip-failed">
      {copy === undefined ? (
        <p className={styles.problem} role="alert">
          {messages.editor.failedStub}
        </p>
      ) : (
        <div className={styles.section} role="alert">
          <p className={styles.problem}>{copy.title}</p>
          <p className={styles.text}>{copy.body}</p>
          <p className={styles.text}>{copy.action}</p>
        </div>
      )}
    </Stub>
  );
}

function body(clip: Pick<ClipState, "status" | "waitingModel"> & { code: ErrorCode | undefined }): ReactNode {
  switch (clip.status) {
    case "idle":
      return <DropZone />;
    case "importing":
    case "accepted":
    case "processing":
      return <Processing waitingModel={clip.waitingModel} />;
    case "ready":
    case "updating":
      return (
        <section className={styles.editor}>
          <PreviewPlayer />
          <ExportButton />
          <ExportProgress />
          <div className={styles.startOver}>
            <StartOver />
          </div>
        </section>
      );
    case "rejected":
      return (
        <Stub testId="clip-rejected">
          <p className={styles.problem} role="alert">
            {messages.editor.rejectedStub}
          </p>
        </Stub>
      );
    case "failed":
      return <Failed code={clip.code} />;
    default:
      return assertNever(clip.status);
  }
}

export function EditorPage() {
  const status = useClipStore((state) => state.status);
  const waitingModel = useClipStore((state) => state.waitingModel);
  const code = useClipStore((state) => state.failure?.code);

  return (
    <main className={styles.page} data-testid="editor">
      {body({ status, waitingModel, code })}
    </main>
  );
}
