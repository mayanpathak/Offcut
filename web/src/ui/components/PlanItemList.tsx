// The Visuals tab (v4implementation D-187, D-154): the plan's suggested
// visuals in time order, each a row with its kind's icon, its friendly
// label, its figure, its time (a seek) and a switch; "N of M visuals on" over
// them. With none, the sentence of PS §10 J7 and what Offcut looks for. The
// figure is the detector's own display text (INV-22): nothing is parsed here.
// V6.2 (P194): the five plan kinds' own icons, and "Regenerate plan" under the
// list with `ai_plan` and the settings switch on (§8.3).

import { type ReactNode, useState } from "react";

import { FEATURES } from "../../config/features";
import { messages } from "../../copy/messages";
import type { EventKind, PlanItem, Word } from "../../gen/domain";
import { useClipStore } from "../../state/clip-store";
import { usePlanStore } from "../../state/plan-store";
import { usePreviewStore } from "../../state/preview-store";
import { useUiStore } from "../../state/ui-store";
import { pause, scrub } from "../../usecases/control-preview";
import { regeneratePlan } from "../../usecases/regenerate-plan";
import { toggleEvent } from "../../usecases/toggle-event";
import { ArrowRight, Hash, Heading, Highlighter, List, RefreshCw, Shapes, Smile, Zap, ZoomIn } from "../icons";
import { Button } from "../primitives/Button";
import { PanelSection } from "../primitives/PanelSection";
import { Switch } from "../primitives/Switch";
import styles from "../styles/panel.module.css";
import { CutsSection } from "./CutsSection";

const KIND_ICON: Record<EventKind, ReactNode> = {
  number_reveal: <Hash />,
  list_reveal: <List />,
  from_to: <ArrowRight />,
  keyword_pop: <Zap />,
  // V6.2 (D-446, P194): each kind its own Lucide icon.
  emphasis: <Highlighter />,
  icon_pop: <Shapes />,
  punch_in: <ZoomIn />,
  emoji: <Smile />,
  hook_title: <Heading />,
};

/**
 * "Regenerate plan" (§8.3): one request; a plan that keeps an item replaces
 * the clip's, and anything else leaves it and says why in a toast.
 */
function RegeneratePlan() {
  const switchOn = usePlanStore((state) => state.switchOn);
  const ready = useClipStore((state) => state.status === "ready");
  const locked = usePreviewStore((state) => state.status === "locked");
  const [busy, setBusy] = useState(false);
  if (!FEATURES.ai_plan || !switchOn) {
    return null;
  }
  const copy = messages.plan;
  return (
    <div className={styles.regenerate}>
      <Button
        variant="secondary"
        size="sm"
        icon={<RefreshCw size={16} />}
        busy={busy}
        disabled={!ready || locked}
        {...(locked ? { disabledReason: messages.editor.availableAfterExport } : {})}
        data-testid="regenerate-plan"
        onClick={() => {
          setBusy(true);
          void regeneratePlan().then((result) => {
            setBusy(false);
            if (!result.ok) {
              useUiStore.getState().pushToast({ id: "plan-regenerate", message: copy.regenerateLine(result.why) });
            }
          });
        }}
      >
        {copy.regenerate}
      </Button>
    </div>
  );
}

/** The words of a range, as the transcript editor shows them (an edit in place of what was heard). */
function wordsOf(words: readonly Word[], edits: Readonly<Record<number, string>>, from: number, to: number): string {
  const shown: string[] = [];
  for (let i = from; i < to; i += 1) {
    const text = edits[i] ?? words[i]?.text ?? "";
    if (text !== "") {
      shown.push(text);
    }
  }
  return shown.join(" ");
}

/** What a row says the visual shows: the detector's display text, or the item's words. */
function figureOf(item: PlanItem, words: readonly Word[], edits: Readonly<Record<number, string>>): string {
  const own = (): string => wordsOf(words, edits, item.anchors.start, item.anchors.end);
  if (item.params.kind !== "detected") {
    // A plan of a model (V6.2) points at words.
    return own();
  }
  const params = item.params.params;
  switch (params.kind) {
    case "number_reveal":
      return params.value.display;
    case "from_to":
      return messages.visuals.fromTo(params.from.display, params.to.display);
    case "list_reveal":
      return params.header === "" ? messages.visuals.points(params.count) : params.header;
    case "keyword_pop":
      return wordsOf(words, edits, params.word, params.word + 1);
    // V6.2's kinds (D-446) come from a model as word pointers, never as the detector's reading.
    case "emphasis":
    case "icon_pop":
    case "punch_in":
    case "emoji":
    case "hook_title":
      return own();
    default:
      return own();
  }
}

export function PlanItemList() {
  const plan = useClipStore((state) => state.plan);
  const words = useClipStore((state) => state.transcript?.words) ?? [];
  const edits = useClipStore((state) => state.edit.word_edits);
  const overrides = useClipStore((state) => state.edit.event_overrides);
  const locked = usePreviewStore((state) => state.status === "locked");
  const reason = locked ? { disabledReason: messages.editor.availableAfterExport } : {};
  // In time order; every V4 item is shown until the user says otherwise (D-145).
  const items = [...(plan?.items ?? [])].sort((a, b) => a.anchors.start - b.anchors.start);
  const on = items.filter((item) => overrides[item.id] ?? true).length;

  if (items.length === 0) {
    return (
      <PanelSection title={messages.panel.visuals}>
        <p className={styles.empty} data-testid="no-events-found">
          {messages.visuals.noEventsFound}
        </p>
        <p className={styles.note}>{messages.visuals.whatWeLookFor}</p>
        <RegeneratePlan />
        <CutsSection />
      </PanelSection>
    );
  }
  return (
    <PanelSection title={messages.panel.visuals} hint={messages.visuals.count(on, items.length)}>
      <ul className={styles.rows} data-testid="plan-items">
        {items.map((item) => {
          const startMs = words[item.anchors.start]?.start_ms ?? 0;
          const time = messages.preview.clock(startMs);
          return (
            <li key={item.id} className={styles.row} data-kind={item.kind}>
              <span className={styles.rowIcon} aria-hidden="true">
                {KIND_ICON[item.kind]}
              </span>
              <button
                type="button"
                className={styles.rowTime}
                aria-label={messages.visuals.goTo(time)}
                onClick={() => {
                  // Program time is the recording's in V4 (D-164); the preview stays paused there.
                  void pause().then(() => scrub(startMs));
                }}
              >
                {time}
              </button>
              <span className={styles.rowSwitch}>
                <Switch
                  checked={overrides[item.id] ?? true}
                  label={messages.visuals.kind[item.kind]}
                  hint={figureOf(item, words, edits)}
                  disabled={locked}
                  {...reason}
                  onChange={(checked) => {
                    void toggleEvent(item.id, checked);
                  }}
                />
              </span>
            </li>
          );
        })}
      </ul>
      <RegeneratePlan />
      <CutsSection />
    </PanelSection>
  );
}
