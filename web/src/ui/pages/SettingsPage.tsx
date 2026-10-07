// `/settings`. V1 has the "What leaves your device" table, which the landing
// page links to from its first day. The rest of PS §12.7 arrives in V8.

import { Link } from "react-router";

import { messages } from "../../copy/messages";
import { WhatLeavesTable } from "../components/WhatLeavesTable";
import styles from "../styles/pages.module.css";

export function SettingsPage({ homePath }: { homePath: string }) {
  return (
    <main className={styles.page}>
      <section className={styles.section}>
        <Link to={homePath}>{messages.settings.backLink}</Link>
        <h1 className={styles.title}>{messages.settings.title}</h1>
        <WhatLeavesTable />
      </section>
    </main>
  );
}
