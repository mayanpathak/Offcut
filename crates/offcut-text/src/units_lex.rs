//! Every word list of the number grammar (v3implementation §8.4; D-84, D-101,
//! D-102): currencies, percent, magnitudes, units of time and of data, count
//! nouns, and the comparatives that make "times" a multiplier. Each list is a
//! sorted `const` slice of lower-case words, searched with `binary_search`; a
//! test checks that every slice is sorted, without duplicates and lower-case.
//!
//! Public because `offcut-detect` reads `COMPARATIVES` (D-102). "pound" and
//! "pounds" are absent on purpose (a weight as often as a currency, D-102), and
//! so are "tb" and "terabytes": `Unit` has no variant for them.

use offcut_types::Unit;

/// A currency the grammar reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrencyUnit {
    Usd,
    Eur,
    Gbp,
    Inr,
}

/// A unit of time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

/// A unit of data size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataUnit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
}

/// Sorted by code point: U+0024, U+00A3, U+20AC, U+20B9.
pub const CURRENCY_SYMBOLS: &[(char, CurrencyUnit)] = &[
    ('$', CurrencyUnit::Usd),
    ('£', CurrencyUnit::Gbp),
    ('€', CurrencyUnit::Eur),
    ('₹', CurrencyUnit::Inr),
];
pub const CURRENCY_WORDS: &[(&str, CurrencyUnit)] = &[
    ("buck", CurrencyUnit::Usd),
    ("bucks", CurrencyUnit::Usd),
    ("dollar", CurrencyUnit::Usd),
    ("dollars", CurrencyUnit::Usd),
    ("eur", CurrencyUnit::Eur),
    ("euro", CurrencyUnit::Eur),
    ("euros", CurrencyUnit::Eur),
    ("gbp", CurrencyUnit::Gbp),
    ("inr", CurrencyUnit::Inr),
    ("rs", CurrencyUnit::Inr),
    ("rupee", CurrencyUnit::Inr),
    ("rupees", CurrencyUnit::Inr),
    ("usd", CurrencyUnit::Usd),
];
/// And the two-token "per cent".
pub const PERCENT_WORDS: &[&str] = &["pct", "percent"];
pub const MAGNITUDE_WORDS: &[(&str, f64)] = &[
    ("billion", 1e9),
    ("million", 1e6),
    ("thousand", 1e3),
    ("trillion", 1e12),
];
pub const TIME_WORDS: &[(&str, TimeUnit)] = &[
    ("day", TimeUnit::Days),
    ("days", TimeUnit::Days),
    ("hour", TimeUnit::Hours),
    ("hours", TimeUnit::Hours),
    ("hr", TimeUnit::Hours),
    ("hrs", TimeUnit::Hours),
    ("millisecond", TimeUnit::Milliseconds),
    ("milliseconds", TimeUnit::Milliseconds),
    ("min", TimeUnit::Minutes),
    ("mins", TimeUnit::Minutes),
    ("minute", TimeUnit::Minutes),
    ("minutes", TimeUnit::Minutes),
    ("month", TimeUnit::Months),
    ("months", TimeUnit::Months),
    ("ms", TimeUnit::Milliseconds),
    ("sec", TimeUnit::Seconds),
    ("second", TimeUnit::Seconds),
    ("seconds", TimeUnit::Seconds),
    ("secs", TimeUnit::Seconds),
    ("week", TimeUnit::Weeks),
    ("weeks", TimeUnit::Weeks),
    ("year", TimeUnit::Years),
    ("years", TimeUnit::Years),
    ("yr", TimeUnit::Years),
    ("yrs", TimeUnit::Years),
];
pub const DATA_WORDS: &[(&str, DataUnit)] = &[
    ("byte", DataUnit::Bytes),
    ("bytes", DataUnit::Bytes),
    ("gb", DataUnit::Gigabytes),
    ("gigabyte", DataUnit::Gigabytes),
    ("gigabytes", DataUnit::Gigabytes),
    ("kb", DataUnit::Kilobytes),
    ("kilobyte", DataUnit::Kilobytes),
    ("kilobytes", DataUnit::Kilobytes),
    ("mb", DataUnit::Megabytes),
    ("megabyte", DataUnit::Megabytes),
    ("megabytes", DataUnit::Megabytes),
];
pub const COUNT_NOUNS: &[&str] = &[
    "apps",
    "calls",
    "clicks",
    "clients",
    "clips",
    "comments",
    "commits",
    "companies",
    "customers",
    "downloads",
    "emails",
    "employees",
    "followers",
    "impressions",
    "installs",
    "likes",
    "lines",
    "listeners",
    "members",
    "messages",
    "orders",
    "pages",
    "people",
    "posts",
    "projects",
    "readers",
    "requests",
    "sales",
    "shares",
    "signups",
    "stars",
    "students",
    "subscribers",
    "users",
    "videos",
    "viewers",
    "views",
    "visitors",
    "websites",
    "words",
];
pub const COMPARATIVES: &[&str] = &[
    "better", "bigger", "cheaper", "easier", "faster", "harder", "higher", "larger", "less",
    "longer", "lower", "more", "quicker", "shorter", "slower", "smaller", "stronger",
];
/// D-101: a count earns the explicit-unit bonus from here (assumption, M1.3).
pub const COUNT_UNIT_MIN: f64 = 100.0;

impl From<CurrencyUnit> for Unit {
    fn from(c: CurrencyUnit) -> Unit {
        match c {
            CurrencyUnit::Usd => Unit::Usd,
            CurrencyUnit::Eur => Unit::Eur,
            CurrencyUnit::Gbp => Unit::Gbp,
            CurrencyUnit::Inr => Unit::Inr,
        }
    }
}

impl From<TimeUnit> for Unit {
    fn from(t: TimeUnit) -> Unit {
        match t {
            TimeUnit::Milliseconds => Unit::Milliseconds,
            TimeUnit::Seconds => Unit::Seconds,
            TimeUnit::Minutes => Unit::Minutes,
            TimeUnit::Hours => Unit::Hours,
            TimeUnit::Days => Unit::Days,
            TimeUnit::Weeks => Unit::Weeks,
            TimeUnit::Months => Unit::Months,
            TimeUnit::Years => Unit::Years,
        }
    }
}

impl From<DataUnit> for Unit {
    fn from(d: DataUnit) -> Unit {
        match d {
            DataUnit::Bytes => Unit::Bytes,
            DataUnit::Kilobytes => Unit::Kilobytes,
            DataUnit::Megabytes => Unit::Megabytes,
            DataUnit::Gigabytes => Unit::Gigabytes,
        }
    }
}

/// The value of `word` in a list of `(word, value)` sorted by word.
pub(crate) fn lookup<T: Copy>(list: &[(&str, T)], word: &str) -> Option<T> {
    list.binary_search_by(|(key, _)| (*key).cmp(word))
        .ok()
        .and_then(|i| list.get(i))
        .map(|(_, v)| *v)
}

/// Whether `word` is in a sorted list of words.
pub(crate) fn contains(list: &[&str], word: &str) -> bool {
    list.binary_search(&word).is_ok()
}

/// The currency of a symbol.
pub(crate) fn currency_symbol(c: char) -> Option<CurrencyUnit> {
    CURRENCY_SYMBOLS
        .binary_search_by(|(key, _)| key.cmp(&c))
        .ok()
        .and_then(|i| CURRENCY_SYMBOLS.get(i))
        .map(|(_, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Strictly ascending (so sorted and without duplicates), and lower-case.
    fn ordered(keys: &[&str]) {
        assert!(
            keys.windows(2).all(|w| w[0] < w[1]),
            "not sorted or duplicated: {keys:?}"
        );
        assert!(
            keys.iter().all(|k| *k == k.to_lowercase()),
            "not lower-case: {keys:?}"
        );
    }

    fn keys_of<'a, T>(list: &[(&'a str, T)]) -> Vec<&'a str> {
        list.iter().map(|(k, _)| *k).collect()
    }

    #[test]
    fn currency_symbols_are_sorted_by_code_point() {
        let chars: Vec<char> = CURRENCY_SYMBOLS.iter().map(|(c, _)| *c).collect();
        assert!(chars.windows(2).all(|w| w[0] < w[1]), "{chars:?}");
        assert_eq!(currency_symbol('€'), Some(CurrencyUnit::Eur));
    }

    #[test]
    fn currency_words_are_ordered() {
        ordered(&keys_of(CURRENCY_WORDS));
        assert_eq!(lookup(CURRENCY_WORDS, "rupees"), Some(CurrencyUnit::Inr));
        assert_eq!(lookup(CURRENCY_WORDS, "pounds"), None);
    }

    #[test]
    fn percent_words_are_ordered() {
        ordered(PERCENT_WORDS);
    }

    #[test]
    fn magnitude_words_are_ordered() {
        ordered(&keys_of(MAGNITUDE_WORDS));
        assert_eq!(lookup(MAGNITUDE_WORDS, "million"), Some(1e6));
    }

    #[test]
    fn time_words_are_ordered() {
        ordered(&keys_of(TIME_WORDS));
        assert_eq!(lookup(TIME_WORDS, "hrs"), Some(TimeUnit::Hours));
    }

    #[test]
    fn data_words_are_ordered() {
        ordered(&keys_of(DATA_WORDS));
        assert_eq!(lookup(DATA_WORDS, "tb"), None);
        assert_eq!(lookup(DATA_WORDS, "terabytes"), None);
    }

    #[test]
    fn count_nouns_are_ordered() {
        ordered(COUNT_NOUNS);
        assert!(contains(COUNT_NOUNS, "subscribers"));
        // The list nouns of offcut-detect are never count nouns (D-101).
        assert!(!contains(COUNT_NOUNS, "reasons"));
    }

    #[test]
    fn comparatives_are_ordered() {
        ordered(COMPARATIVES);
        assert!(contains(COMPARATIVES, "faster"));
    }
}
