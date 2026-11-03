// The editor's layout (v4implementation D-178): the top bar across, the stage
// and, once there is a clip on its way, the side panel. The whole layout is
// the window's height; only the panel scrolls. It sets the dark theme on its
// root (D-179) and holds the one `aria-live="polite"` region of the editor
// (D-192), which says the processing steps as they change and a finished
// export's outcome, and holds the toasts' stack. It mounts the keyboard shortcuts and their sheet (D-185).

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { useExportStore } from "../../state/export-store";
import { outcomeAnnouncement } from "../components/ChangeSummary";
import { STEPS, stepStates } from "../components/ProcessingCard";
import { ShortcutsSheet } from "../components/ShortcutsSheet";
import { ToastHost } from "../components/ToastHost";
import { useShortcuts } from "../hooks/use-shortcuts";
import styles from "../styles/editor.module.css";
import { SidePanel } from "./SidePanel";
import { Stage, type StageView } from "./Stage";
import { TopBar } from "./TopBar";

/** What the live region says: the setup while the model downloads, else the steps under way, else a finished export's outcome. */
function Announcement({ view }: { view: StageView }) {
  const stage = useClipStore((state) => state.stage);
  const feed = useClipStore((state) => state.feed);
  const exported = useExportStore((state) => state.status);
  const summary = useExportStore((state) => state.summary);
  const answered = useExportStore((state) => state.answered);
  let text = "";
  if (view === "download") {
    text = messages.modelDownload.title;
  } else if (view === "processing") {
    const states = stepStates(stage, feed);
    text = STEPS.filter((step) => states[step] === "running")
      .map((step) => messages.processing.steps[step])
      .join(", ");
  } else if (exported === "done" && summary !== undefined) {
    text = outcomeAnnouncement(summary, answered);
  }
  return <span className={styles.srOnly}>{text}</span>;
}

export function EditorLayout({ view, accountPath }: { view: StageView; accountPath: string }) {
  useShortcuts();
  const panel = view === "download" || view === "processing" || view === "cancelled" || view === "ready";
  return (
    <div data-theme="dark" className={panel ? styles.editorWithPanel : styles.editor} data-testid="editor">
      <TopBar view={view} accountPath={accountPath} />
      <Stage view={view} />
      {panel ? (
        <aside className={styles.panel}>
          <SidePanel ready={view === "ready"} />
        </aside>
      ) : null}
      <div className={styles.live} aria-live="polite">
        <Announcement view={view} />
        <ToastHost />
      </div>
      <ShortcutsSheet />
    </div>
  );
}
