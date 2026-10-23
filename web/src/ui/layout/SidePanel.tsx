// The side panel (v4implementation D-178, §17.3): the export's progress while
// one runs, then four tabs, Captions · Style · Visuals · Framing, the choice
// kept in the interface store. While the clip is on its way the tabs are
// disabled and hold a placeholder. On a ready clip each tab shows what it
// will edit, read from the clip: the controls are wired by Prompts 113-117
// (`STATIC` below marks each one), so here they show the clip's values and
// change nothing.

import type { ReactNode } from "react";

import { messages } from "../../copy/messages";
import type { EventKind, StyleId } from "../../gen/domain";
import { useClipStore } from "../../state/clip-store";
import { type Tab, useUiStore } from "../../state/ui-store";
import { ExportProgress } from "../components/ExportProgress";
import { ArrowRight, Crop, Hash, List, Palette, Sparkles, Type, Zap } from "../icons";
import { PanelSection } from "../primitives/PanelSection";
import { SegmentedControl } from "../primitives/SegmentedControl";
import { Skeleton } from "../primitives/Skeleton";
import { Slider } from "../primitives/Slider";
import { Switch } from "../primitives/Switch";
import { Tabs } from "../primitives/Tabs";
import styles from "../styles/panel.module.css";

const TABS = [
  { id: "captions", label: messages.panel.captions, icon: <Type /> },
  { id: "style", label: messages.panel.style, icon: <Palette /> },
  { id: "visuals", label: messages.panel.visuals, icon: <Sparkles /> },
  { id: "framing", label: messages.panel.framing, icon: <Crop /> },
] as const;

/** A control of this shell that its flow's prompt wires (113-117). */
const STATIC = (): void => undefined;

const STYLES: readonly { id: StyleId; name: string; line: string }[] = [
  { id: "clean", name: messages.style.clean, line: messages.style.cleanLine },
  { id: "bold", name: messages.style.bold, line: messages.style.boldLine },
  { id: "tech", name: messages.style.mono, line: messages.style.monoLine },
];

type Position = "top" | "middle" | "bottom";
/** D-171's snap points (`placement.rs`); Prompt 116 gives them the page's home. */
const SNAPS: Record<Position, number> = { top: 0.15, middle: 0.55, bottom: 1 };
const POSITIONS: readonly Position[] = ["top", "middle", "bottom"];

const KIND_ICON: Record<EventKind, ReactNode> = {
  number_reveal: <Hash />,
  list_reveal: <List />,
  from_to: <ArrowRight />,
  keyword_pop: <Zap />,
};

function assertNever(value: never): never {
  void value;
  throw new Error("a tab this build does not know");
}

function CaptionsTab() {
  const follow = useUiStore((state) => state.follow);
  const setFollow = useUiStore((state) => state.setFollow);
  const transcript = useClipStore((state) => state.transcript);
  return (
    <PanelSection title={messages.panel.captions} hint={messages.captions.hint}>
      <Switch checked={follow} label={messages.captions.follow} hint={messages.captions.followHint} onChange={setFollow} />
      {transcript?.sentences.map((sentence) => (
        <p key={sentence.start_ms} className={styles.transcript}>
          {transcript.words
            .slice(sentence.words.start, sentence.words.end)
            .map((word) => word.text)
            .join(" ")}
        </p>
      ))}
    </PanelSection>
  );
}

function StyleTab() {
  const edit = useClipStore((state) => state.edit);
  return (
    <PanelSection title={messages.panel.style}>
      <ul className={styles.cards}>
        {STYLES.map((style) => (
          <li key={style.id} className={style.id === edit.style ? styles.styleCardOn : styles.styleCard}>
            <span className={styles.thumb} aria-hidden="true" />
            <span className={styles.styleName}>{style.name}</span>
            <span className={styles.styleLine}>{style.line}</span>
          </li>
        ))}
      </ul>
      <Switch
        checked={edit.highlight_numbers}
        label={messages.style.highlightNumbers}
        hint={messages.style.highlightNumbersHint}
        onChange={STATIC}
      />
    </PanelSection>
  );
}

function VisualsTab() {
  const events = useClipStore((state) => state.events);
  const words = useClipStore((state) => state.transcript?.words) ?? [];
  const overrides = useClipStore((state) => state.edit.event_overrides);
  const on = events.filter((event) => overrides[event.id] ?? event.enabled).length;
  if (events.length === 0) {
    return (
      <PanelSection title={messages.panel.visuals}>
        <p className={styles.empty}>{messages.visuals.noEventsFound}</p>
        <p className={styles.note}>{messages.visuals.whatWeLookFor}</p>
      </PanelSection>
    );
  }
  return (
    <PanelSection title={messages.panel.visuals} hint={messages.visuals.count(on, events.length)}>
      <ul className={styles.rows}>
        {events.map((event) => (
          <li key={event.id} className={styles.row}>
            <span className={styles.rowIcon} aria-hidden="true">
              {KIND_ICON[event.kind]}
            </span>
            <span className={styles.rowText}>
              <span className={styles.rowKind}>{messages.visuals.kind[event.kind]}</span>
              <span className={styles.rowWords}>
                {words
                  .slice(event.anchors.start, event.anchors.end)
                  .map((word) => word.text)
                  .join(" ")}
              </span>
            </span>
            <span className={styles.rowTime}>{messages.preview.clock(event.span.start)}</span>
          </li>
        ))}
      </ul>
    </PanelSection>
  );
}

function FramingTab() {
  const placement = useClipStore((state) => state.edit.caption_position);
  const crop = useClipStore((state) => state.edit.crop_offset);
  const landscape = useClipStore((state) => state.clipInfo?.orientation === "landscape");
  const nearest = POSITIONS.reduce((best, position) =>
    Math.abs(SNAPS[position] - placement) < Math.abs(SNAPS[best] - placement) ? position : best,
  );
  return (
    <>
      <PanelSection title={messages.framing.captionPosition} hint={messages.framing.dragCaptions}>
        <SegmentedControl
          label={messages.framing.captionPosition}
          value={nearest}
          onChange={STATIC}
          options={POSITIONS.map((position) => ({ value: position, label: messages.framing[position] }))}
        />
      </PanelSection>
      {landscape ? (
        <PanelSection title={messages.framing.crop} hint={messages.framing.dragVideo}>
          <Slider
            label={messages.framing.crop}
            min={-1}
            max={1}
            step={0.01}
            value={crop}
            format={messages.framing.cropValue}
            onCommit={STATIC}
          />
        </PanelSection>
      ) : null}
    </>
  );
}

function TabContent({ tab }: { tab: Tab }) {
  switch (tab) {
    case "captions":
      return <CaptionsTab />;
    case "style":
      return <StyleTab />;
    case "visuals":
      return <VisualsTab />;
    case "framing":
      return <FramingTab />;
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
