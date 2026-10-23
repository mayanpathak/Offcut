// The editor's layout (v4implementation D-178): the top bar across, the stage
// and, once there is a clip on its way, the side panel. The whole layout is
// the window's height; only the panel scrolls. It sets the dark theme on its
// root (D-179) and holds the one `aria-live="polite"` region of the editor
// (D-192), which says the processing steps as they change; the toasts join
// it in Prompt 117.

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { STEPS, stepStates } from "../components/ProcessingCard";
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
      </div>
    </div>
  );
}
