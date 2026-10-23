// The stage (v4implementation D-178, §17.2): a 9:16 frame at the largest size
// that fits, on the dark backdrop, holding the card of the clip's state or
// the preview, and a row under it for the player's transport. The row is
// there in every state, so the frame never moves between processing and
// ready (G 10.9).

import { useClipStore } from "../../state/clip-store";
import { useModelStore } from "../../state/model-store";
import { DropZone } from "../components/DropZone";
import { ErrorPanel } from "../components/ErrorPanel";
import { ModelDownloadPanel } from "../components/ModelDownloadPanel";
import { PreviewPlayer, PreviewTransport } from "../components/PreviewPlayer";
import { ProcessingCard } from "../components/ProcessingCard";
import { RejectionPanel } from "../components/RejectionPanel";
import styles from "../styles/editor.module.css";

/** What the stage shows. `cancelled` is reached from Prompt 119, which adds the cancel. */
export type StageView = "empty" | "download" | "processing" | "cancelled" | "ready" | "rejected" | "failed";

function assertNever(value: never): never {
  void value;
  throw new Error("a state of the clip this build does not know");
}

/** The stage's view of the clip and of the model download it may be waiting for. */
export function useStageView(): StageView {
  const status = useClipStore((state) => state.status);
  const waitingModel = useClipStore((state) => state.waitingModel);
  const model = useModelStore((state) => state.status);
  switch (status) {
    case "idle":
      return "empty";
    case "importing":
    case "accepted":
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

function FrameContent({ view }: { view: StageView }) {
  const code = useClipStore((state) => state.failure?.code);
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
      return <PreviewPlayer />;
    case "rejected":
      return <RejectionPanel />;
    case "failed":
      return <ErrorPanel code={code} testId="clip-failed" />;
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
