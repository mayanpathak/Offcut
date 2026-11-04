// Section 12: the footer (designimplementation §6). The legal pages and the
// contact link, as before, under the same test id; and a link to the
// settings, named "Settings" so that "What leaves your device" stays the
// one link of that name on the page (D-334).

import { Link } from "react-router";

import { LEGAL_DOCS, type LegalDoc, messages } from "../../copy/messages";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";

export function SiteFooter({ legalPath, settingsPath }: { legalPath: (doc: LegalDoc) => string; settingsPath: string }) {
  const copy = messages.landing;
  return (
    <footer className={styles.footer} data-testid="site-footer">
      <div className={[landing.container, styles.footerInner].join(" ")}>
        <div className={styles.footerBrand}>
          <span className={landing.wordmark}>
            <span className={landing.wordmarkO}>{copy.wordmark.slice(0, 1)}</span>
            {copy.wordmark.slice(1)}
          </span>
          <p className={styles.footerLine}>{copy.footer.line}</p>
        </div>
        <nav className={styles.footerLinks} aria-label={copy.footer.legal}>
          {LEGAL_DOCS.map((doc) => (
            <Link key={doc} to={legalPath(doc)}>
              {messages.legal.docs[doc].linkLabel}
            </Link>
          ))}
          <Link to={settingsPath}>{copy.footer.settingsLink}</Link>
          <a href={`mailto:${messages.legal.contactEmail}`}>{messages.legal.footerContact}</a>
        </nav>
      </div>
    </footer>
  );
}
