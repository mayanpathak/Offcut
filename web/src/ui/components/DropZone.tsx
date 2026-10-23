// J2: the drop zone and the sample-clip button. A dropped file, the first one
// when several are dropped, goes to `importClip`, and so does a file chosen
// after a click on the zone; the button fetches the sample clip and imports
// that.
//
// The file is handed on as it came. Its name is never read here or anywhere
// else (TS §25.1 P-11).

import { type DragEvent, useRef, useState } from "react";

import { messages } from "../../copy/messages";
import { LIMITS } from "../../gen/domain";
import { type BlockerCode, forImport } from "../../state/blockers";
import { useCapabilityStore } from "../../state/capability-store";
import { useClipStore } from "../../state/clip-store";
import { useExportStore } from "../../state/export-store";
import { importClip, importSampleClip } from "../../usecases/import-clip";
import { ArrowUp } from "../icons";
import { Button } from "../primitives/Button";
import styles from "../styles/components.module.css";

// What the file chooser offers first. It is a hint to the browser: whether a
// clip can be used is decided where it is probed, not here.
const PICKER_ACCEPT = "video/mp4,video/quicktime,.mp4,.mov";

/** The words of a blocker. The ones no person can meet yet have none (D-57). */
function blockerText(code: BlockerCode): string {
  const copy: Partial<Record<BlockerCode, string>> = messages.blockers;
  return copy[code] ?? "";
}


export type DropZoneProps = {
  /**
   * `landing`: the landing page's zone, as it was. `stage`: the editor's
   * empty stage (v4implementation §17.2, G 10.6), the whole frame a place to
   * drop, outlined in the brand colour while a file is over it.
   */
  variant?: "landing" | "stage";
};

export function DropZone({ variant = "landing" }: DropZoneProps) {
  // A blocker is said once a clip was offered, not before: while the browser
  // is still being checked, nothing has been refused.
  const [offered, setOffered] = useState(false);
  const [fetchingSample, setFetchingSample] = useState(false);
  const [over, setOver] = useState(false);
  const picker = useRef<HTMLInputElement>(null);
  // Read so that the zone is drawn again when what `forImport()` asks about changes.
  useCapabilityStore((state) => state.status);
  useClipStore((state) => state.status);
  useExportStore((state) => state.status);
  const blocker = forImport();
  const stage = variant === "stage";

  const offer = (file: File | undefined): void => {
    setOffered(true);
    if (file !== undefined && forImport() === null) {
      void importClip(file, "user");
    }
  };

  const sample = (): void => {
    setOffered(true);
    if (fetchingSample || forImport() !== null) {
      return;
    }
    setFetchingSample(true);
    void importSampleClip().finally(() => {
      setFetchingSample(false);
    });
  };

  let line = "";
  if (offered && blocker !== null) {
    line = blockerText(blocker);
  } else if (fetchingSample) {
    line = messages.dropZone.fetchingSample;
  }

  // On the zone itself on the landing page, on the whole frame on the stage.
  // Without the first and the last, the browser would open the dropped file in the tab.
  const dropHandlers = {
    onDragOver: (event: DragEvent<HTMLElement>) => {
      event.preventDefault();
      setOver(true);
    },
    onDragLeave: (event: DragEvent<HTMLElement>) => {
      if (!(event.relatedTarget instanceof Node && event.currentTarget.contains(event.relatedTarget))) {
        setOver(false);
      }
    },
    onDrop: (event: DragEvent<HTMLElement>) => {
      event.preventDefault();
      setOver(false);
      offer(event.dataTransfer.files[0]);
    },
  };

  const target = (
    <div
      className={stage ? styles.dropTargetStage : styles.dropTarget}
      role="button"
      tabIndex={0}
      data-testid="drop-zone"
      // A click, or Enter or the space bar, opens the browser's file chooser.
      onClick={() => {
        picker.current?.click();
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          picker.current?.click();
        }
      }}
      {...(stage ? {} : dropHandlers)}
    >
      {stage ? (
        <span className={styles.dropIcon} aria-hidden="true">
          <ArrowUp />
        </span>
      ) : null}
      {messages.dropZone.prompt(LIMITS)}
    </div>
  );
  const input = (
    <input
      ref={picker}
      type="file"
      accept={PICKER_ACCEPT}
      hidden
      tabIndex={-1}
      onChange={(event) => {
        offer(event.target.files?.[0]);
        // The same file can be chosen again: the field keeps nothing.
        event.target.value = "";
      }}
    />
  );

  if (stage) {
    return (
      <div className={over ? styles.dropStageOver : styles.dropStage} {...dropHandlers}>
        {target}
        {input}
        <span className={styles.dropOr}>{messages.dropZone.or}</span>
        <Button variant="secondary" busy={fetchingSample} onClick={sample}>
          {messages.dropZone.sampleButton}
        </Button>
        <p className={styles.dropBrowsers}>{messages.landing.supportedBrowsers}</p>
        <p className={styles.dropLine} role="status">
          {line}
        </p>
      </div>
    );
  }

  return (
    <div className={styles.dropZone}>
      {target}
      {input}
      <button
        className={[styles.secondaryButton, fetchingSample ? styles.pending : undefined].filter(Boolean).join(" ")}
        type="button"
        aria-busy={fetchingSample}
        onClick={sample}
      >
        {messages.dropZone.sampleButton}
      </button>
      <p className={styles.notice} role="status">
        {line}
      </p>
    </div>
  );
}
