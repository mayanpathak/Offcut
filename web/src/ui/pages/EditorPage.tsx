// `/app`. A placeholder in V1 (D-16): the editor arrives in V2, and until
// then this page says so and offers the waitlist.

import { messages } from "../../copy/messages";
import { NotifyMeForm } from "../components/NotifyMeForm";
import styles from "../styles/pages.module.css";

export function EditorPage() {
  return (
    <main className={styles.page}>
      <section className={styles.section}>
        <p className={styles.lead} data-testid="not-ready">
          {messages.dropZone.notReady}
        </p>
        <NotifyMeForm wanted={["launch"]} />
      </section>
    </main>
  );
}
