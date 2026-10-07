// The drop zone and the sample-clip button. In V1 both are present and
// inactive: a drop or a click shows the "not ready" message. V2 wires them
// to `importClip`.
//
// A dropped file is never read, stored or inspected here. The handlers do
// not touch `event.dataTransfer` at all.

import { useState } from "react";

import { messages } from "../../copy/messages";
import { LIMITS } from "../../gen/domain";
import styles from "../styles/components.module.css";

export function DropZone() {
  const [notReadyShown, setNotReadyShown] = useState(false);
  const showNotReady = () => {
    setNotReadyShown(true);
  };

  return (
    <div className={styles.dropZone}>
      <div
        className={styles.dropTarget}
        role="button"
        tabIndex={0}
        aria-disabled="true"
        data-testid="drop-zone"
        onClick={showNotReady}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            showNotReady();
          }
        }}
        // Without these two the browser would open the dropped file in the tab.
        onDragOver={(event) => {
          event.preventDefault();
        }}
        onDrop={(event) => {
          event.preventDefault();
          showNotReady();
        }}
      >
        {messages.dropZone.prompt(LIMITS)}
      </div>
      <button className={styles.secondaryButton} type="button" aria-disabled="true" onClick={showNotReady}>
        {messages.dropZone.sampleButton}
      </button>
      <p className={styles.notice} role="status">
        {notReadyShown ? messages.dropZone.notReady : null}
      </p>
    </div>
  );
}
