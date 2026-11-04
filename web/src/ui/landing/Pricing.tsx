// Section 8: pricing (designimplementation §6, D-329). The plans, from
// `PRICES_USD` and `LIMITS`, with the same test id, heading id and sources
// as before. Free has a way in, "Start free"; Creator is not on sale yet
// (v6implementation D-303), so the section offers no way to buy.

import { PRICES_USD } from "../../config/pricing";
import { messages } from "../../copy/messages";
import { LIMITS } from "../../gen/domain";
import { Check } from "../icons";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-sections.module.css";
import { Chip, CtaLink, Eyebrow } from "./parts";

const CHECK_SIZE = 16;

export function Pricing({ appPath }: { appPath: string }) {
  const pricing = messages.pricing;
  return (
    <section className={landing.section} id="pricing" aria-labelledby="pricing-heading" data-testid="pricing">
      <div className={landing.container}>
        <div className={[landing.sectionHead, landing.sectionHeadCenter].join(" ")}>
          <Eyebrow {...messages.landing.eyebrows.pricing} />
          <h2 className={landing.h2} id="pricing-heading">
            {pricing.heading}
          </h2>
          <p className={landing.sectionLead}>{pricing.notOnSale}</p>
        </div>
        <div className={styles.plans}>
          <div className={styles.plan}>
            <h3 className={styles.planName}>{pricing.free.name}</h3>
            <p className={styles.planPrice}>{pricing.free.price(PRICES_USD.free)}</p>
            <ul className={styles.planFeatures}>
              {pricing.free.features(LIMITS).map((feature) => (
                <li key={feature}>
                  <Check size={CHECK_SIZE} />
                  {feature}
                </li>
              ))}
              <li>
                <Check size={CHECK_SIZE} />
                {pricing.free.everything}
              </li>
            </ul>
            <CtaLink className={styles.planCta} variant="primary" size="small" to={appPath}>
              {messages.landing.cta.startFree}
            </CtaLink>
          </div>
          <div className={[styles.plan, styles.planCreator].join(" ")}>
            <div className={styles.planTop}>
              <h3 className={styles.planName}>{pricing.creator.name}</h3>
              <Chip>{pricing.creator.tag}</Chip>
            </div>
            <p className={styles.planPrice}>{pricing.creator.price(PRICES_USD.creatorMonthly, PRICES_USD.creatorAnnual)}</p>
            <ul className={styles.planFeatures}>
              {pricing.creator.features.map((feature) => (
                <li key={feature}>
                  <Check size={CHECK_SIZE} />
                  {feature}
                </li>
              ))}
            </ul>
            <Chip className={styles.planCta} dot>
              {pricing.creator.soon}
            </Chip>
          </div>
        </div>
        <p className={styles.merchant}>{pricing.merchant}</p>
      </div>
    </section>
  );
}
