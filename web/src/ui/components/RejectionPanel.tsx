// A clip Offcut cannot use: what the limit is and what to do (TS §11.3). A
// rejection is not an error, so there is no retry; the way on is another
// clip. The code is read from the clip store and only marks the panel for
// tests (`data-reject`); no code, size or name of the file is shown.

import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { dismissClip } from "../../usecases/import-clip";
import components from "../styles/components.module.css";

export function RejectionPanel() {
  const code = useClipStore((state) => state.reject);
  const copy = code === undefined ? undefined : messages.rejections[code];
  return (
    <section className={components.rejection} role="alert" data-testid="clip-rejected" data-reject={code}>
      {copy !== undefined && (
        <>
          <h2 className={components.rejectionTitle}>{copy.title}</h2>
          <p className={components.rejectionBody}>{copy.body}</p>
          <p className={components.rejectionAction}>{copy.action}</p>
        </>
      )}
      <button
        className={components.secondaryButton}
        type="button"
        onClick={() => {
          void dismissClip();
        }}
      >
        {messages.editor.startOver}
      </button>
    </section>
  );
}
