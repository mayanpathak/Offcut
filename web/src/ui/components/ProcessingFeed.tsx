// J6: what Offcut has done with the clip so far, one line for each thing
// that happened, in the order it happened. A line comes from the pipeline
// through the clip store; none is made up here (PS §10 J6). Shown as chips
// in the processing card (v4implementation D-188); a found moment carries
// the icon of its kind. The steps are announced by the editor's one live
// region (D-192), so the list is not one.

import type { ReactNode } from "react";

import { messages } from "../../copy/messages";
import type { EventKind } from "../../gen/domain";
import { type ClipState, useClipStore } from "../../state/clip-store";
import { ArrowRight, Hash, List, Zap } from "../icons";
import styles from "../styles/components.module.css";

const KIND_ICON: Record<EventKind, ReactNode> = {
  number_reveal: <Hash size={16} />,
  list_reveal: <List size={16} />,
  from_to: <ArrowRight size={16} />,
  keyword_pop: <Zap size={16} />,
};

type FeedLine = ClipState["feed"][number];

function assertNever(value: never): never {
  void value;
  throw new Error("a line of the feed this build does not know");
}

function eventText(eventKind: EventKind, display: string, displayTo: string): string {
  const found = messages.feed.eventFound;
  switch (eventKind) {
    case "number_reveal":
      return found.number_reveal(display);
    case "list_reveal":
      return found.list_reveal(display);
    case "from_to":
      return found.from_to(display, displayTo);
    case "keyword_pop":
      return found.keyword_pop(display);
    default:
      return assertNever(eventKind);
  }
}

function lineText(line: FeedLine): string {
  switch (line.kind) {
    case "transcribing":
      return messages.feed.transcribing;
    case "cleaning_voice":
      return messages.feed.cleaningVoice;
    case "event_found":
      return eventText(line.eventKind, line.display, line.displayTo ?? "");
    default:
      return assertNever(line);
  }
}

export function ProcessingFeed() {
  const feed = useClipStore((state) => state.feed);

  return (
    <ol className={styles.chips} data-testid="feed">
      {feed.map((line, index) => (
        // A line is appended and never moved or dropped: its place is its identity.
        <li key={index} className={line.kind === "event_found" ? styles.chipFound : styles.chip}>
          {line.kind === "event_found" ? KIND_ICON[line.eventKind] : null}
          {lineText(line)}
        </li>
      ))}
    </ol>
  );
}
