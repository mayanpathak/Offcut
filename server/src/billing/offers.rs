//! Offer to the merchant's product id, and back (TS §22.5, the only place). The
//! server holds no amounts: the merchant does, and the page shows `pricing.ts`.

use offcut_api_types::{BillingInterval, Offer};

use crate::config::Config;

/// The offers this deployment sells, read from its configuration.
pub struct Offers<'a> {
    pub monthly: &'a str,
    pub annual: &'a str,
    pub founding: &'a str,
    pub founding_enabled: bool,
}

impl<'a> Offers<'a> {
    pub fn of(config: &'a Config) -> Offers<'a> {
        Offers {
            monthly: &config.billing_price_creator_monthly,
            annual: &config.billing_price_creator_annual,
            founding: &config.billing_price_creator_annual_founding,
            founding_enabled: config.founding_offer_enabled,
        }
    }

    /// The product a checkout sells. The founding offer only while it is on
    /// (TS §22.5, founding release of `buildplan.md` §8A.6).
    pub fn product(&self, offer: Offer) -> Option<&'a str> {
        match offer {
            Offer::CreatorMonthly => Some(self.monthly),
            Offer::CreatorAnnual => Some(self.annual),
            Offer::CreatorAnnualFounding => self.founding_enabled.then_some(self.founding),
        }
    }

    /// The interval of a product an event names (v6implementation D-277). A
    /// founding product counts while the offer is off too: someone may hold one.
    pub fn interval(&self, product: &str) -> Option<BillingInterval> {
        if product == self.monthly {
            Some(BillingInterval::Monthly)
        } else if product == self.annual || product == self.founding {
            Some(BillingInterval::Annual)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offers(founding_enabled: bool) -> Offers<'static> {
        Offers {
            monthly: "prod_m",
            annual: "prod_a",
            founding: "prod_f",
            founding_enabled,
        }
    }

    #[test]
    fn each_offer_sells_its_product() {
        let on = offers(true);
        assert_eq!(on.product(Offer::CreatorMonthly), Some("prod_m"));
        assert_eq!(on.product(Offer::CreatorAnnual), Some("prod_a"));
        assert_eq!(on.product(Offer::CreatorAnnualFounding), Some("prod_f"));
    }

    #[test]
    fn the_founding_offer_is_not_sold_while_it_is_off() {
        let off = offers(false);
        assert_eq!(off.product(Offer::CreatorAnnualFounding), None);
        assert_eq!(off.product(Offer::CreatorAnnual), Some("prod_a"));
    }

    #[test]
    fn a_product_reads_back_as_its_interval_and_an_unknown_one_as_none() {
        let off = offers(false);
        assert_eq!(off.interval("prod_m"), Some(BillingInterval::Monthly));
        assert_eq!(off.interval("prod_a"), Some(BillingInterval::Annual));
        assert_eq!(off.interval("prod_f"), Some(BillingInterval::Annual));
        assert_eq!(off.interval("prod_other"), None);
    }
}
