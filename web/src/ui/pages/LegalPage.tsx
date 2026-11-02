// `/legal/:doc`: the Terms, the Privacy Policy and the Refund and Cancellation
// Policy (TS §13.1). The words are static, in messages.ts. A document that does
// not exist goes back to the landing page, as any unknown path does.

import { Link, Navigate, useParams } from "react-router";

import { LEGAL_DOCS, type LegalDoc, messages } from "../../copy/messages";
import styles from "../styles/pages.module.css";

function isLegalDoc(doc: string | undefined): doc is LegalDoc {
  return LEGAL_DOCS.some((known) => known === doc);
}

export function LegalPage({ homePath }: { homePath: string }) {
  const { doc } = useParams();
  if (!isLegalDoc(doc)) {
    return <Navigate to={homePath} replace />;
  }
  const page = messages.legal.docs[doc];
  return (
    <main className={styles.page}>
      <article className={styles.section}>
        <Link to={homePath}>{messages.settings.backLink}</Link>
        <h1 className={styles.title}>{page.title}</h1>
        <p className={styles.muted}>{messages.legal.updated}</p>
        {page.sections.map((section) => (
          <section className={styles.section} key={section.heading}>
            <h2 className={styles.heading}>{section.heading}</h2>
            {section.body.map((paragraph) => (
              <p className={styles.text} key={paragraph}>
                {paragraph}
              </p>
            ))}
          </section>
        ))}
        <p className={styles.text}>
          {messages.legal.contactLead} <a href={`mailto:${messages.legal.contactEmail}`}>{messages.legal.contactEmail}</a>
        </p>
      </article>
    </main>
  );
}
