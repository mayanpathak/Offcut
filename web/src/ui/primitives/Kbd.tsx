// A keyboard shortcut, as its keys (v4implementation D-181). With `mod` the
// shortcut starts with the platform's own modifier: ⌘ on macOS, Ctrl
// elsewhere. The other keys are named by the caller, from messages.ts.

import { messages } from "../../copy/messages";
import styles from "../styles/primitives.module.css";

// The navigator field that TypeScript's DOM types do not declare.
type NavigatorHints = Navigator & { userAgentData?: { platform?: string } };

const hints = (): NavigatorHints => navigator;
const isMac = (): boolean => hints().userAgentData?.platform === "macOS";

export type KbdProps = {
  keys: readonly string[];
  mod?: boolean;
};

export function Kbd({ keys, mod = false }: KbdProps) {
  const mac = isMac();
  const all = mod ? [mac ? messages.keys.modMac : messages.keys.modOther, ...keys] : keys;
  // macOS writes a shortcut as one run of keys (⌘Z); elsewhere they are joined by "+".
  const joiner = mac ? "" : "+";
  return (
    <kbd className={styles.kbd}>
      {all.map((key, index) => (
        <span key={`${String(index)}-${key}`} className={styles.kbdGroup}>
          {index > 0 && joiner !== "" ? <span className={styles.kbdJoin}>{joiner}</span> : null}
          <kbd className={styles.kbdKey}>{key}</kbd>
        </span>
      ))}
    </kbd>
  );
}
