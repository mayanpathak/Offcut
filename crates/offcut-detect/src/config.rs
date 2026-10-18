//! The numbers and the words the detector decides by. This is the only file
//! that holds a threshold or a word list (TS §17.4, D-84); all of them are
//! (assumption), tuned at M1.3 and re-checked by E-7.

use offcut_types::EventKind;

/// All four thresholds are here from the start, so that V3 adds fields and
/// renames none. From V3 on the field names are frozen; values are tuned by
/// the sweep of `precision_recall.rs` only.
#[derive(Clone, Debug, PartialEq)]
pub struct DetectorConfig {
    pub threshold_from_to: f32,
    pub threshold_list: f32,
    pub threshold_number: f32,
    pub threshold_keyword: f32,
    /// What every number starts with.
    pub number_base: f32,
    /// Added for a currency, a percent sign or a unit.
    pub number_unit_bonus: f32,
    /// Added for a magnitude of at least 1,000.
    pub number_magnitude_bonus: f32,
    /// Added when one of the words was said with emphasis.
    pub number_energy_bonus: f32,
    /// An overlay is on the screen this long before its first word (TS §17.5),
    pub overlay_lead_ms: u32,
    /// and a number this long after its last: the hold and the fade of TS §19.5.
    pub number_hold_ms: u32,

    // FromTo, new in V3 (TS §17.4).
    pub from_to_base: f32,
    /// "from a to b".
    pub from_to_frame_bonus: f32,
    /// A change verb before `a` or between, and "to" before `b`.
    pub from_to_change_verb_bonus: f32,
    /// Both ends carry the same unit in their own words,
    pub from_to_same_unit_bonus: f32,
    /// or one takes the other's.
    pub from_to_inherit_bonus: f32,
    pub from_to_close_bonus: f32,
    /// Word tokens between the two ends, at most (`FROMTO_MAX_GAP`),
    pub from_to_max_gap_words: usize,
    /// and at most this many for the close bonus.
    pub from_to_close_gap_words: usize,
    /// Hold 1,500 and fade 200 (TS §19.5).
    pub from_to_hold_ms: u32,

    // ListReveal, new in V3 (TS §17.4, D-105).
    pub list_base: f32,
    pub list_all_markers_bonus: f32,
    /// Every marker is an ordinal word.
    pub list_ordinal_bonus: f32,
    pub list_marker_only_base: f32,
    /// No gap between two markers is over `list_marker_gap_ms`.
    pub list_marker_only_bonus: f32,
    /// Every marker starts this soon after the list's first word (`LIST_WINDOW`).
    pub list_window_ms: u32,
    pub list_marker_gap_ms: u32,
    pub list_count_min: u8,
    pub list_count_max: u8,
    pub list_item_max_words: usize,
    /// Hold 1,500 after the last item and fade 200 (TS §19.5).
    pub list_hold_ms: u32,

    // KeywordPop, new in V3 (TS §17.4, §17.5).
    pub keyword_base: f32,
    pub keyword_energy_weight: f32,
    pub keyword_pitch_weight: f32,
    /// An intensifier before the word, or an emphasis word.
    pub keyword_lexicon_bonus: f32,
    pub keyword_min_letters: usize,
    pub keyword_min_gap_ms: u32,

    // NumberReveal and resolution, new in V3 (TS §17.4, §17.5).
    pub number_label_max_words: usize,
    /// At most this many overlays start in any `overlay_window_ms` (`OVERLAY_MAX_PER_10S`).
    pub overlay_max_per_window: usize,
    pub overlay_window_ms: u32,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            threshold_from_to: 0.85,
            threshold_list: 0.85,
            threshold_number: 0.80,
            threshold_keyword: 0.80,
            number_base: 0.50,
            number_unit_bonus: 0.30,
            number_magnitude_bonus: 0.20,
            number_energy_bonus: 0.10,
            overlay_lead_ms: 150,
            number_hold_ms: 1_400,

            from_to_base: 0.55,
            from_to_frame_bonus: 0.25,
            from_to_change_verb_bonus: 0.15,
            from_to_same_unit_bonus: 0.15,
            from_to_inherit_bonus: 0.05,
            from_to_close_bonus: 0.05,
            from_to_max_gap_words: 8,
            from_to_close_gap_words: 6,
            from_to_hold_ms: 1_700,

            list_base: 0.60,
            list_all_markers_bonus: 0.30,
            list_ordinal_bonus: 0.10,
            list_marker_only_base: 0.65,
            list_marker_only_bonus: 0.20,
            list_window_ms: 45_000,
            list_marker_gap_ms: 15_000,
            list_count_min: 2,
            list_count_max: 7,
            list_item_max_words: 5,
            list_hold_ms: 1_700,

            keyword_base: 0.50,
            keyword_energy_weight: 0.20,
            keyword_pitch_weight: 0.20,
            keyword_lexicon_bonus: 0.20,
            keyword_min_letters: 4,
            keyword_min_gap_ms: 2_500,

            number_label_max_words: 3,
            overlay_max_per_window: 3,
            overlay_window_ms: 10_000,
        }
    }
}

impl DetectorConfig {
    /// A candidate of this kind becomes an event at this score or above.
    pub fn threshold(&self, kind: EventKind) -> f32 {
        match kind {
            EventKind::NumberReveal => self.threshold_number,
            EventKind::ListReveal => self.threshold_list,
            EventKind::FromTo => self.threshold_from_to,
            EventKind::KeywordPop => self.threshold_keyword,
        }
    }

    /// How long an event of this kind stays after its last word. A KeywordPop
    /// is no overlay and holds nothing.
    pub fn hold_ms(&self, kind: EventKind) -> u32 {
        match kind {
            EventKind::NumberReveal => self.number_hold_ms,
            EventKind::ListReveal => self.list_hold_ms,
            EventKind::FromTo => self.from_to_hold_ms,
            EventKind::KeywordPop => 0,
        }
    }
}

/// A number from this magnitude on earns `number_magnitude_bonus` (V2's rule).
pub const NUMBER_MAGNITUDE_MIN: f64 = 1_000.0;
/// A word said this much louder than its sentence earns `number_energy_bonus`.
pub const NUMBER_EMPHASIS_Z: f32 = 1.0;
/// A change verb is looked for among this many tokens before a FromTo's first
/// quantity (section 9.4), as well as between the two.
pub const FROM_TO_VERB_REACH: usize = 3;
/// A list without an announcer needs at least this many markers (D-105).
pub const LIST_MARKER_ONLY_MIN: usize = 3;
/// A list item ends at a comma once it holds this many words (D-107).
pub const LIST_ITEM_COMMA_WORDS: usize = 2;
/// X1: a bare whole number in this range, in plain digits, is a year.
pub const YEAR_FIRST: f64 = 1_900.0;
pub const YEAR_LAST: f64 = 2_099.0;
/// X2: a number written with this many decimal points is a version: "3.12.1".
pub const VERSION_DOTS: usize = 2;
/// X4: a bare run of at least this many digits is phone-like.
pub const PHONE_DIGITS_MIN: usize = 7;
/// X5: a bare number under this, in absolute value, is too small to show.
pub const BARE_NUMBER_MIN: f64 = 10.0;
/// Scores are sums of the two-decimal values above, and an `f32` sum can land
/// a few units in the last place under the value it stands for: 0.65 + 0.20 is
/// 0.84999996, under a threshold of 0.85 (V2 CL known issue 22). Before the
/// threshold, a score this close to a whole hundredth is taken as that
/// hundredth (`lib.rs` step 5, the one place). A score that is really between
/// two hundredths, such as 0.795, is not moved (section 23, item 27).
pub const SCORE_NOISE: f32 = 1e-6;

// The grammar words the finders look for. They say what a frame is; they are
// not tuned.

/// "from 90 minutes to 10 minutes" (section 9.4).
pub const FROM: &str = "from";
pub const TO: &str = "to";
/// "number two, …": a cardinal after it is a list marker (D-105).
pub const NUMBER: &str = "number";

// The lexicons (section 9.1). Each is sorted, without duplicates, lower-case,
// and matched against whole effective tokens.

/// The nouns of a list announcer: "three reasons". Never count nouns (D-101).
#[rustfmt::skip]
pub const LIST_NOUNS: &[&str] = &[
    "benefits", "examples", "factors", "facts", "features", "habits", "ideas", "kinds", "lessons",
    "levels", "mistakes", "myths", "options", "parts", "phases", "points", "principles", "problems",
    "questions", "reasons", "rules", "secrets", "signs", "stages", "steps", "things", "tips",
    "tools", "tricks", "types", "ways",
];
/// A list marker and the place it names.
#[rustfmt::skip]
pub const ORDINAL_MARKERS: &[(&str, u8)] = &[
    ("fifth", 5), ("first", 1), ("firstly", 1), ("fourth", 4), ("second", 2), ("secondly", 2),
    ("seventh", 7), ("sixth", 6), ("third", 3), ("thirdly", 3),
];
/// The verbs of a change: "grew from 40 to 90%".
#[rustfmt::skip]
pub const CHANGE_VERBS: &[&str] = &[
    "boosted", "climbed", "cut", "declined", "decreased", "down", "dropped", "fell", "grew",
    "increased", "jumped", "lowered", "moved", "raised", "reduced", "rose", "shrank", "slashed",
    "soared", "up", "went",
];
/// A word before a keyword that says it is stressed: "literally brutal".
#[rustfmt::skip]
pub const INTENSIFIERS: &[&str] = &[
    "absolutely", "completely", "extremely", "incredibly", "insanely", "literally", "massively",
    "really", "ridiculously", "seriously", "so", "super", "totally", "truly", "very",
];
/// Words that are a keyword's emphasis by themselves. "always" and "never"
/// are not: either would put a KeywordPop on nearly every clip.
#[rustfmt::skip]
pub const EMPHASIS_WORDS: &[&str] = &[
    "brutal", "crazy", "critical", "essential", "exactly", "fastest", "huge", "impossible",
    "incredible", "insane", "massive", "perfect", "secret", "terrible", "worst", "wrong",
];
/// Words that are never a keyword.
#[rustfmt::skip]
pub const STOP_WORDS: &[&str] = &[
    "about", "above", "actually", "after", "again", "against", "also", "another", "back",
    "basically", "because", "been", "before", "being", "below", "between", "both", "could", "does",
    "doing", "done", "down", "during", "each", "either", "even", "ever", "every", "feel", "from",
    "going", "gonna", "have", "here", "into", "just", "kind", "know", "like", "look", "made",
    "make", "many", "mean", "might", "mine", "more", "most", "much", "must", "need", "neither",
    "only", "onto", "other", "ours", "over", "really", "right", "said", "same", "says", "should",
    "some", "sort", "still", "stuff", "such", "take", "than", "that", "their", "them", "then",
    "there", "these", "they", "thing", "things", "think", "this", "those", "through", "too",
    "under", "until", "upon", "very", "wanna", "want", "well", "were", "what", "when", "where",
    "which", "while", "will", "with", "without", "would", "yeah", "your", "yours",
];
/// A NumberReveal label stops before one of these verbs (D-107).
#[rustfmt::skip]
pub const LABEL_STOP_VERBS: &[&str] = &[
    "am", "are", "be", "became", "become", "been", "can", "could", "did", "do", "does", "feels",
    "felt", "gave", "get", "gets", "goes", "got", "had", "has", "have", "is", "made", "make",
    "means", "said", "says", "should", "takes", "took", "was", "went", "were", "will", "would",
];
/// Words a list marker may follow and an item drops at its start.
#[rustfmt::skip]
pub const FILLERS: &[&str] = &[
    "actually", "ah", "also", "and", "basically", "er", "like", "now", "ok", "okay", "right", "so",
    "then", "uh", "um", "well",
];
/// A number right after one of these is a version or a product (X2).
#[rustfmt::skip]
pub const VERSION_CONTEXT: &[&str] = &[
    "android", "angular", "build", "chapter", "chrome", "claude", "edition", "episode", "firefox",
    "galaxy", "gpt", "ios", "ipad", "iphone", "java", "level", "llama", "macos", "model", "node",
    "page", "part", "php", "pixel", "python", "react", "release", "room", "rust", "season",
    "series", "ubuntu", "update", "v", "version", "vue", "windows",
];
/// And right before one of these (X2): "iPhone 15 Pro".
pub const VERSION_AFTER: &[&str] = &["max", "mini", "plus", "pro", "ultra"];
/// A number right before one of these is a time of day (X3). "a.m." is
/// tokenized as the word `a.m` and a full stop, so the list holds `a.m`.
pub const CLOCK_WORDS: &[&str] = &["a.m", "am", "o'clock", "p.m", "pm"];

#[cfg(test)]
mod tests {
    use offcut_text::units_lex::COUNT_NOUNS;

    use super::*;

    /// Strictly ascending (so sorted and without duplicates), and lower-case.
    fn ordered(name: &str, words: &[&str]) {
        assert!(
            words.windows(2).all(|w| w[0] < w[1]),
            "{name} is not sorted or has a duplicate: {words:?}"
        );
        assert!(
            words.iter().all(|w| *w == w.to_lowercase()),
            "{name} is not lower-case: {words:?}"
        );
    }

    #[test]
    fn the_defaults_are_v2s_ten_and_the_v3_table() {
        let cfg = DetectorConfig::default();
        let v2 = (
            [cfg.threshold_from_to, cfg.threshold_list],
            [cfg.threshold_number, cfg.threshold_keyword],
            [cfg.number_base, cfg.number_unit_bonus],
            [cfg.number_magnitude_bonus, cfg.number_energy_bonus],
            (cfg.overlay_lead_ms, cfg.number_hold_ms),
        );
        assert_eq!(
            v2,
            (
                [0.85, 0.85],
                [0.80, 0.80],
                [0.50, 0.30],
                [0.20, 0.10],
                (150, 1_400)
            )
        );
        let from_to = [
            cfg.from_to_base,
            cfg.from_to_frame_bonus,
            cfg.from_to_change_verb_bonus,
            cfg.from_to_same_unit_bonus,
            cfg.from_to_inherit_bonus,
            cfg.from_to_close_bonus,
        ];
        assert_eq!(from_to, [0.55, 0.25, 0.15, 0.15, 0.05, 0.05]);
        assert_eq!(
            (cfg.from_to_max_gap_words, cfg.from_to_close_gap_words),
            (8, 6)
        );
        let list = [
            cfg.list_base,
            cfg.list_all_markers_bonus,
            cfg.list_ordinal_bonus,
            cfg.list_marker_only_base,
            cfg.list_marker_only_bonus,
        ];
        assert_eq!(list, [0.60, 0.30, 0.10, 0.65, 0.20]);
        assert_eq!(
            (cfg.list_window_ms, cfg.list_marker_gap_ms),
            (45_000, 15_000)
        );
        assert_eq!(
            (
                cfg.list_count_min,
                cfg.list_count_max,
                cfg.list_item_max_words
            ),
            (2, 7, 5)
        );
        let keyword = [
            cfg.keyword_base,
            cfg.keyword_energy_weight,
            cfg.keyword_pitch_weight,
            cfg.keyword_lexicon_bonus,
        ];
        assert_eq!(keyword, [0.50, 0.20, 0.20, 0.20]);
        assert_eq!(
            (cfg.keyword_min_letters, cfg.keyword_min_gap_ms),
            (4, 2_500)
        );
        assert_eq!(
            (
                cfg.number_label_max_words,
                cfg.overlay_max_per_window,
                cfg.overlay_window_ms
            ),
            (3, 3, 10_000)
        );
    }

    #[test]
    fn each_kind_has_its_threshold_and_its_hold() {
        let cfg = DetectorConfig::default();
        let kinds = [
            EventKind::NumberReveal,
            EventKind::ListReveal,
            EventKind::FromTo,
            EventKind::KeywordPop,
        ];
        let thresholds = kinds.map(|kind| cfg.threshold(kind));
        assert_eq!(thresholds, [0.80, 0.85, 0.85, 0.80]);
        let holds = kinds.map(|kind| cfg.hold_ms(kind));
        assert_eq!(holds, [1_400, 1_700, 1_700, 0]);
        // A threshold read by kind is the field, so tuning one moves the other.
        let tuned = DetectorConfig {
            threshold_list: 0.95,
            ..DetectorConfig::default()
        };
        assert_eq!(tuned.threshold(EventKind::ListReveal), 0.95);
    }

    #[test]
    fn every_lexicon_is_sorted_unique_and_lower_case() {
        let markers: Vec<&str> = ORDINAL_MARKERS.iter().map(|(word, _)| *word).collect();
        let lexicons: [(&str, &[&str]); 11] = [
            ("LIST_NOUNS", LIST_NOUNS),
            ("ORDINAL_MARKERS", &markers),
            ("CHANGE_VERBS", CHANGE_VERBS),
            ("INTENSIFIERS", INTENSIFIERS),
            ("EMPHASIS_WORDS", EMPHASIS_WORDS),
            ("STOP_WORDS", STOP_WORDS),
            ("LABEL_STOP_VERBS", LABEL_STOP_VERBS),
            ("FILLERS", FILLERS),
            ("VERSION_CONTEXT", VERSION_CONTEXT),
            ("VERSION_AFTER", VERSION_AFTER),
            ("CLOCK_WORDS", CLOCK_WORDS),
        ];
        for (name, words) in lexicons {
            ordered(name, words);
        }
        let sizes = lexicons.map(|(_, words)| words.len());
        assert_eq!(sizes, [31, 10, 21, 15, 16, 104, 35, 16, 37, 5, 5]);
        // Every ordinal names a place a list can have.
        let cfg = DetectorConfig::default();
        assert!(
            ORDINAL_MARKERS
                .iter()
                .all(|(_, n)| (1..=cfg.list_count_max).contains(n))
        );
    }

    #[test]
    fn no_list_noun_is_a_count_noun() {
        // Else "three reasons" would carry a unit and no announcer would be found (D-101).
        let both: Vec<&str> = LIST_NOUNS
            .iter()
            .copied()
            .filter(|noun| COUNT_NOUNS.binary_search(noun).is_ok())
            .collect();
        assert_eq!(both, Vec::<&str>::new());
    }
}
