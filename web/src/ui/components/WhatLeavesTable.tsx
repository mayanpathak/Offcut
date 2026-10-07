// "What leaves your device" (PS §12.7): four fixed rows, then every analytics
// event the app can send. The event rows are read from the generated
// allowlist, so this table cannot fall behind it (TS §25.1 P-4).

import { messages } from "../../copy/messages";
import { ANALYTICS_EVENT_DOCS } from "../../gen/api";
import styles from "../styles/components.module.css";

export function WhatLeavesTable() {
  const copy = messages.settings;
  return (
    <div className={styles.tables}>
      <h2 className={styles.tableHeading}>{copy.whatLeavesHeading}</h2>
      <table className={styles.table} data-testid="what-leaves">
        <thead>
          <tr>
            <th scope="col">{copy.columns.data}</th>
            <th scope="col">{copy.columns.leaves}</th>
          </tr>
        </thead>
        <tbody>
          {copy.rows.map((row) => (
            <tr key={row.data}>
              <th scope="row">{row.data}</th>
              <td>{row.leaves}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <h2 className={styles.tableHeading}>{copy.eventListHeading}</h2>
      <table className={styles.table} data-testid="analytics-events">
        <thead>
          <tr>
            <th scope="col">{copy.eventColumns.event}</th>
            <th scope="col">{copy.eventColumns.description}</th>
          </tr>
        </thead>
        <tbody>
          {ANALYTICS_EVENT_DOCS.map((doc) => (
            <tr key={doc.name}>
              <th scope="row">
                {copy.events[doc.name]}
                <code className={styles.eventName}>{doc.name}</code>
              </th>
              <td>{doc.description}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
