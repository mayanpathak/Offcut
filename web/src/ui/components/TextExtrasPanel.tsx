// V6.2: "Text for your post" (v6.2implementation §12, D-454, P194): the hook,
// the post caption and the hashtags of the model's plan the clip uses, as its
// validation kept them, each with Copy. Under the visuals list, with
// `ai_plan` on; nothing on a rules plan, which has none. The text is the
// person's to paste and edit: nothing here sends it anywhere.

import { useEffect, useId, useState } from "react";

import { FEATURES } from "../../config/features";
import { messages } from "../../copy/messages";
import { useClipStore } from "../../state/clip-store";
import { Check, Copy } from "../icons";
import { Button } from "../primitives/Button";
import { PanelSection } from "../primitives/PanelSection";
import styles from "../styles/panel.module.css";

/** How long "Copied" shows after a copy. */
const COPIED_MS = 2_000;

function Extra({ label, text, testId }: { label: string; text: string; testId: string }) {
  const copy = messages.plan.extras;
  const labelId = useId();
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) {
      return undefined;
    }
    const timer = window.setTimeout(() => {
      setCopied(false);
    }, COPIED_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [copied]);
  return (
    <div className={styles.extra} role="group" aria-labelledby={labelId} data-testid={testId}>
      <div className={styles.extraHead}>
        <span id={labelId} className={styles.extraLabel}>
          {label}
        </span>
        <Button
          variant="ghost"
          size="sm"
          icon={copied ? <Check size={16} /> : <Copy size={16} />}
          onClick={() => {
            // A browser that refuses the clipboard leaves the text to select by hand.
            void navigator.clipboard.writeText(text).then(
              () => {
                setCopied(true);
              },
              () => undefined,
            );
          }}
        >
          {copied ? copy.copied : copy.copy}
        </Button>
      </div>
      <p className={styles.extraText}>{text}</p>
      <span className={styles.srOnly} aria-live="polite">
        {copied ? copy.copied : ""}
      </span>
    </div>
  );
}

export function TextExtrasPanel() {
  const extras = useClipStore((state) => state.planExtras);
  if (!FEATURES.ai_plan || extras === undefined) {
    return null;
  }
  const copy = messages.plan.extras;
  const hashtags = extras.hashtags.join(" ");
  if (extras.hook === null && extras.post_caption === null && hashtags === "") {
    return null;
  }
  return (
    <PanelSection title={copy.title}>
      <div className={styles.extras} data-testid="text-extras">
        {extras.hook === null ? null : <Extra label={copy.hook} text={extras.hook} testId="extra-hook" />}
        {extras.post_caption === null ? null : (
          <Extra label={copy.caption} text={extras.post_caption} testId="extra-caption" />
        )}
        {hashtags === "" ? null : <Extra label={copy.hashtags} text={hashtags} testId="extra-hashtags" />}
      </div>
    </PanelSection>
  );
}
