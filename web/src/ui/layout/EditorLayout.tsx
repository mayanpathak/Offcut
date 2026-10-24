// The editor's layout (v4implementation D-178): the top bar across, the stage
// and, once there is a clip on its way, the side panel. The whole layout is
// the window's height; only the panel scrolls. It sets the dark theme on its
// root (D-179) and holds the one `aria-live="polite"` region of the editor
// (D-192), which says the processing steps as they change, and holds the
// toast shown now (Prompt 117 gives the toasts their host).

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { useUiStore } from "../../state/ui-store";
import { STEPS, stepStates } from "../components/ProcessingCard";
import { Toast } from "../primitives/Toast";
import styles from "../styles/editor.module.css";
import { SidePanel } from "./SidePanel";
import { Stage, type StageView } from "./Stage";
import { TopBar } from "./TopBar";

/** What the live region says: the setup while the model downloads, else the steps under way. */
function Announcement({ view }: { view: StageView }) {
  const stage = useClipStore((state) => state.stage);
  const feed = useClipStore((state) => state.feed);
  let text = "";
  if (view === "download") {
    text = messages.modelDownload.title;
  } else if (view === "processing") {
    const states = stepStates(stage, feed);
    text = STEPS.filter((step) => states[step] === "running")
      .map((step) => messages.processing.steps[step])
      .join(", ");
  }
  return <span className={styles.srOnly}>{text}</span>;
}

/** The oldest toast of the queue (D-181), one at a time; the next shows when it goes. */
function CurrentToast() {
  const toast = useUiStore((state) => state.toasts[0]);
  const dismiss = useUiStore((state) => state.dismissToast);
  if (toast === undefined) {
    return null;
  }
  return (
    <Toast
      key={toast.id}
      message={toast.message}
      {...(toast.action === undefined ? {} : { action: toast.action.label, onAction: toast.action.run })}
      onDismiss={() => {
        dismiss(toast.id);
      }}
    />
  );
}

export function EditorLayout({ view }: { view: StageView }) {
  const panel = view === "download" || view === "processing" || view === "cancelled" || view === "ready";
  return (
    <div data-theme="dark" className={panel ? styles.editorWithPanel : styles.editor} data-testid="editor">
      <TopBar view={view} />
      <Stage view={view} />
      {panel ? (
        <aside className={styles.panel}>
          <SidePanel ready={view === "ready"} />
        </aside>
      ) : null}
      <div className={styles.live} aria-live="polite">
        <Announcement view={view} />
        <CurrentToast />
      </div>
    </div>
  );
}
