// The stage (v4implementation D-178, §17.2): a 9:16 frame at the largest size
// that fits, on the dark backdrop, holding the card of the clip's state or
// the preview, and a row under it for the player's transport. The row is
// there in every state, so the frame never moves between processing and
// ready (G 10.9). A failure of a ready clip shows its card over the preview,
// the side panel still usable (§17.2).

import { useClipStore } from "../../state/clip-store";
import { useModelStore } from "../../state/model-store";
import { usePreviewStore } from "../../state/preview-store";
import { DropZone } from "../components/DropZone";
import { ErrorPanel, readyContext } from "../components/ErrorPanel";
import { ModelDownloadPanel } from "../components/ModelDownloadPanel";
import { PreviewPlayer, PreviewTransport } from "../components/PreviewPlayer";
import { ProcessingCard } from "../components/ProcessingCard";
import { RejectionPanel } from "../components/RejectionPanel";
import styles from "../styles/editor.module.css";

/** What the stage shows. `cancelled`: the run was cancelled, and the clip waits for "Process again" (D-152). */
export type StageView = "empty" | "download" | "processing" | "cancelled" | "ready" | "rejected" | "failed";

function assertNever(value: never): never {
  void value;
  throw new Error("a state of the clip this build does not know");
}

/** The stage's view of the clip and of the model download it may be waiting for. */
export function useStageView(): StageView {
  const status = useClipStore((state) => state.status);
  const cancelled = useClipStore((state) => state.cancelled);
  const waitingModel = useClipStore((state) => state.waitingModel);
  const model = useModelStore((state) => state.status);
  switch (status) {
    case "idle":
      return "empty";
    case "accepted":
      if (cancelled) {
        return "cancelled";
      }
      return "processing";
    case "importing":
    case "processing":
      // The download is shown while the clip waits for it, and what stopped it.
      return waitingModel && (model === "downloading" || model === "verifying" || model === "failed")
        ? "download"
        : "processing";
    case "ready":
    case "updating":
      return "ready";
    case "rejected":
      return "rejected";
    case "failed":
      return "failed";
    default:
      return assertNever(status);
  }
}

/** The preview, and the card of its failure over it while there is one. The canvas is new after each reload (D-153). */
function ReadyStage() {
  const failure = useClipStore((state) => state.failure);
  const during = useClipStore((state) => state.failureDuring);
  const canvas = usePreviewStore((state) => state.canvas);
  return (
    <>
      <PreviewPlayer key={canvas} />
      {failure === undefined ? null : (
        <div className={styles.frameOver}>
          <ErrorPanel failure={failure} context={readyContext(failure, during)} testId="preview-failed" compact />
        </div>
      )}
    </>
  );
}

function FrameContent({ view }: { view: StageView }) {
  const failure = useClipStore((state) => state.failure);
  switch (view) {
    case "empty":
      return <DropZone variant="stage" />;
    case "download":
      return <ModelDownloadPanel />;
    case "processing":
      return <ProcessingCard cancelled={false} />;
    case "cancelled":
      return <ProcessingCard cancelled />;
    case "ready":
      return <ReadyStage />;
    case "rejected":
      return <RejectionPanel />;
    case "failed":
      return <ErrorPanel failure={failure} context="clip" testId="clip-failed" />;
    default:
      return assertNever(view);
  }
}

export function Stage({ view }: { view: StageView }) {
  return (
    <main className={styles.stage}>
      <div className={styles.column}>
        <div className={view === "empty" ? styles.frameOpen : styles.frame}>
          <FrameContent view={view} />
        </div>
        <div className={styles.below}>{view === "ready" ? <PreviewTransport /> : null}</div>
      </div>
    </main>
  );
}
