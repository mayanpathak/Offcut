// The side panel (v4implementation D-178, §17.3): the export's progress while
// one runs, then four tabs, Captions · Style · Visuals · Framing, the choice
// kept in the interface store. While the clip is on its way the tabs are
// disabled and hold a placeholder. On a ready clip each tab has one job:
// the transcript editor (D-182), the style cards (D-186), the suggested
// visuals and the framing (D-187).

import { messages } from "../../copy/messages";
import { type Tab, useUiStore } from "../../state/ui-store";
import { CaptionEditor } from "../components/CaptionEditor";
import { ExportProgress } from "../components/ExportProgress";
import { FramingPanel } from "../components/FramingPanel";
import { PlanItemList } from "../components/PlanItemList";
import { StylePanel } from "../components/StylePanel";
import { Crop, Palette, Sparkles, Type } from "../icons";
import { Skeleton } from "../primitives/Skeleton";
import { Tabs } from "../primitives/Tabs";
import styles from "../styles/panel.module.css";

const TABS = [
  { id: "captions", label: messages.panel.captions, icon: <Type /> },
  { id: "style", label: messages.panel.style, icon: <Palette /> },
  { id: "visuals", label: messages.panel.visuals, icon: <Sparkles /> },
  { id: "framing", label: messages.panel.framing, icon: <Crop /> },
] as const;

function assertNever(value: never): never {
  void value;
  throw new Error("a tab this build does not know");
}

function TabContent({ tab }: { tab: Tab }) {
  switch (tab) {
    case "captions":
      return <CaptionEditor />;
    case "style":
      return <StylePanel />;
    case "visuals":
      return <PlanItemList />;
    case "framing":
      return <FramingPanel />;
    default:
      return assertNever(tab);
  }
}

/** `ready`: the clip can be edited; otherwise the tabs wait for it. */
export function SidePanel({ ready }: { ready: boolean }) {
  const tab = useUiStore((state) => state.tab);
  const selectTab = useUiStore((state) => state.selectTab);
  return (
    <div className={styles.panel}>
      <div className={styles.export}>
        <ExportProgress />
      </div>
      <div className={styles.tabs}>
        <Tabs tabs={TABS} selected={tab} onSelect={selectTab} label={messages.panel.label} disabled={!ready}>
          {ready ? (
            <TabContent tab={tab} />
          ) : (
            <div className={styles.waiting}>
              <Skeleton size="short" />
              <Skeleton />
              <Skeleton />
              <Skeleton size="block" />
            </div>
          )}
        </Tabs>
      </div>
    </div>
  );
}
