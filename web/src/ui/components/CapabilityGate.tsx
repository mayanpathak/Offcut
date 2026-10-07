// Shows its children only on a browser that can run Offcut. While the check
// runs it says so; on an unsupported browser it shows the unsupported page
// in place (TS §13.1).

import type { ReactNode } from "react";

import { messages } from "../../copy/messages";
import { useCapabilityStore } from "../../state/capability-store";
import { UnsupportedPage } from "../pages/UnsupportedPage";
import styles from "../styles/pages.module.css";

export function CapabilityGate({ children }: { children: ReactNode }) {
  const status = useCapabilityStore((state) => state.status);
  const reason = useCapabilityStore((state) => state.reason);

  if (status === "supported") {
    return children;
  }
  if (status === "unsupported" && reason !== undefined) {
    return <UnsupportedPage reason={reason} />;
  }
  return (
    <main className={styles.page}>
      <p className={styles.text} role="status">
        {messages.capability.checking}
      </p>
    </main>
  );
}
