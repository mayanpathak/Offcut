//! [ONLY] What makes a caption easy to read (D-167, D-168): the English words
//! a line should not end on, the words a chunk may start on, the cost of each
//! kind of break and of chunk, and how long a chunk stays up. The search for
//! the layout of least cost is `captions::chunk`'s. All values are (assumption,
//! E-2). The lists are sorted and lower case, and are searched by bisection;
//! a caption word is matched by its lower-cased text, punctuation stripped.

use crate::captions::CaptionWord;

/// Words that open a new clause: a break before one is cheap.
pub(crate) const CLAUSE_STARTERS: &[&str] = &[
    "after", "although", "and", "as", "because", "before", "but", "except", "how", "if", "nor",
    "once", "or", "plus", "since", "so", "than", "that", "then", "though", "till", "unless",
    "until", "what", "whatever", "when", "whenever", "where", "whereas", "wherever", "whether",
    "which", "while", "who", "whoever", "whom", "whose", "why", "yet",
];

/// Words that bind to the word after them: articles, possessives,
/// prepositions, "to", auxiliaries and modals, "not", "very", "really", and
/// cardinals that lead into the word they count. A line or a chunk that ends
/// on one is hard to read.
pub(crate) const BINDERS: &[&str] = &[
    "a", "about", "above", "across", "after", "against", "along", "am", "among", "an", "are",
    "around", "at", "be", "been", "before", "behind", "being", "below", "beside", "between",
    "beyond", "by", "can", "could", "did", "do", "does", "during", "eight", "five", "for", "four",
    "from", "had", "has", "have", "her", "his", "in", "inside", "into", "is", "its", "may",
    "might", "must", "my", "near", "nine", "not", "of", "on", "one", "onto", "our", "over",
    "really", "seven", "shall", "should", "six", "ten", "the", "their", "three", "through", "to",
    "toward", "towards", "two", "under", "upon", "very", "was", "were", "will", "with", "within",
    "without", "would", "your",
];

/// The binders that continue a clause rather than start a phrase:
/// auxiliaries, modals and "not". A break before one ("it / was") separates
/// a subject from its verb, so it is not where a phrase starts.
pub(crate) const VERB_BINDERS: &[&str] = &[
    "am", "are", "be", "been", "being", "can", "could", "did", "do", "does", "had", "has", "have",
    "is", "may", "might", "must", "not", "shall", "should", "was", "were", "will", "would",
];

/// A break after `,` `;` `:` or `—`.
pub(crate) const COST_AFTER_PUNCT: u32 = 0;
/// A break before a clause starter, or before a binder that starts a phrase
/// (an article, a possessive, a preposition): where a phrase starts.
pub(crate) const COST_CLAUSE: u32 = 1;
/// Any other break.
pub(crate) const COST_PLAIN: u32 = 3;
/// A break after a binder.
pub(crate) const COST_AFTER_BINDER: u32 = 12;
/// A clause starter left at the end of a screen: the clause waits for the
/// next one ("it worked because / we cached it"). Below a binder's, so a
/// screen ends on "then" before it ends on "from".
pub(crate) const COST_OPEN_CLAUSE: u32 = 8;
/// A binder left at the end of a screen: the word it binds to is on the next
/// one ("it is a / date parser"). Above a binder's line break, so a screen
/// ends on a binder only where no layout of the run avoids it.
pub(crate) const COST_BINDER_AT_SCREEN_END: u32 = 16;
/// A chunk of one word that no pause sets apart.
pub(crate) const COST_ONE_WORD: u32 = 8;
/// A break that starts a new chunk counts this many times a line break: the
/// reader loses the context when the screen changes, not when a line wraps.
pub(crate) const CHUNK_BREAK_WEIGHT: u32 = 2;
/// And a new chunk costs this besides: of two layouts that read alike, the
/// one with fewer screens wins.
pub(crate) const COST_NEW_CHUNK: u32 = 1;

/// The most two unbalanced lines cost: below a binder's break less a plain
/// one, so balance never buys a line that ends on a binder.
pub(crate) const MAX_IMBALANCE: u32 = 8;

/// The least time a chunk is shown, when the silence after it allows (D-168).
pub(crate) const MIN_CHUNK_MS: u32 = 700;
/// The silence on both sides that makes one word a beat of its own.
pub(crate) const ISOLATED_WORD_PAUSE_MS: u32 = 250;

/// A caption word as the lists know it: lower case, without the punctuation
/// around it.
fn bare(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

fn listed(list: &[&str], word: &str) -> bool {
    list.binary_search(&bare(word).as_str()).is_ok()
}

/// A word that ends on `,` `;` `:` or `—`: the mark of a beat in speech. A
/// break after it costs nothing, and it may stand alone as a chunk.
pub(crate) fn ends_a_phrase(word: &str) -> bool {
    word.ends_with([',', ';', ':', '—'])
}

/// Whether a line that ends on `word` is cut inside a phrase: a binder that
/// no `,` `;` `:` `—` closes. The one question about the lists outside this
/// crate: the property checks of `tests/line_breaks.rs` ask it.
pub fn binds_to_next(word: &str) -> bool {
    !ends_a_phrase(word) && listed(BINDERS, word)
}

/// The cost of a break between the caption words `prev` and `next`, at the
/// end of a line inside a chunk (D-167).
pub(crate) fn break_cost(prev: &str, next: &str) -> u32 {
    if ends_a_phrase(prev) {
        COST_AFTER_PUNCT
    } else if listed(BINDERS, prev) {
        COST_AFTER_BINDER
    } else if listed(CLAUSE_STARTERS, next)
        || (listed(BINDERS, next) && !listed(VERB_BINDERS, next))
    {
        COST_CLAUSE
    } else {
        COST_PLAIN
    }
}

/// The cost of starting a new chunk between `prev` and `next`: a line
/// break's, except that a binder or a clause starter left at the end of a
/// screen costs `COST_BINDER_AT_SCREEN_END` or `COST_OPEN_CLAUSE`; weighed
/// `CHUNK_BREAK_WEIGHT` times, plus `COST_NEW_CHUNK`.
pub(crate) fn chunk_break_cost(prev: &str, next: &str) -> u32 {
    let cost = if ends_a_phrase(prev) {
        COST_AFTER_PUNCT
    } else if listed(BINDERS, prev) {
        COST_BINDER_AT_SCREEN_END
    } else if listed(CLAUSE_STARTERS, prev) {
        COST_OPEN_CLAUSE
    } else {
        break_cost(prev, next)
    };
    cost * CHUNK_BREAK_WEIGHT + COST_NEW_CHUNK
}

/// The cost of two lines of `a` and `b` characters: none when they differ by
/// at most a third of the longer, else their difference, at most
/// `MAX_IMBALANCE`.
pub(crate) fn imbalance(a: u32, b: u32) -> u32 {
    let (long, short) = (a.max(b), a.min(b));
    let difference = long - short;
    if difference.saturating_mul(3) <= long {
        0
    } else {
        difference.min(MAX_IMBALANCE)
    }
}

/// A caption word as the break search weighs it: its text in the style's
/// case and its length in characters.
pub(crate) struct Unit<'a> {
    pub(crate) word: &'a CaptionWord,
    pub(crate) text: String,
    pub(crate) chars: u32,
}

/// What a chunk may hold (TS §19.4).
pub(crate) struct Limits {
    pub(crate) per_line: u32,
    pub(crate) max_lines: u32,
    pub(crate) max_words: usize,
}

pub(crate) fn text_of<'u>(units: &'u [Unit<'_>], i: usize) -> &'u str {
    units.get(i).map_or("", |unit| unit.text.as_str())
}

/// The characters of `units[a..b]` set on one line, a space between words.
fn width(units: &[Unit<'_>], a: usize, b: usize) -> u32 {
    let words = units.get(a..b).unwrap_or_default();
    let spaces = u32::try_from(words.len().saturating_sub(1)).unwrap_or(u32::MAX);
    words
        .iter()
        .fold(spaces, |sum, unit| sum.saturating_add(unit.chars))
}

/// A line of `units[a..b]` fits: within the characters, or a single word,
/// which V2's line rule scales to fit.
fn fits(units: &[Unit<'_>], a: usize, b: usize, limits: &Limits) -> bool {
    b - a == 1 || width(units, a, b) <= limits.per_line
}

/// A word set apart by a pause of `ISOLATED_WORD_PAUSE_MS` on both sides; the
/// start and the end of the captions count as pauses.
fn isolated(units: &[Unit<'_>], i: usize) -> bool {
    let gap = |a: Option<&Unit<'_>>, b: Option<&Unit<'_>>| match (a, b) {
        (Some(a), Some(b)) => b.word.start.get().saturating_sub(a.word.end.get()),
        _ => u32::MAX,
    };
    let before = gap(i.checked_sub(1).and_then(|j| units.get(j)), units.get(i));
    let after = gap(units.get(i), units.get(i + 1));
    before >= ISOLATED_WORD_PAUSE_MS && after >= ISOLATED_WORD_PAUSE_MS
}

/// The cost of the chunk `units[a..b]` (§8.9's `cost_of_chunk`) and where
/// its second line starts: one line when it fits, else the best of its splits
/// into two lines (the break's cost and the lines' imbalance). `None` for a
/// chunk the style cannot hold. Of two splits of one cost, the earlier wins.
pub(crate) fn chunk_cost(
    units: &[Unit<'_>],
    a: usize,
    b: usize,
    limits: &Limits,
) -> Option<(u32, Option<usize>)> {
    if b - a > limits.max_words {
        return None;
    }
    let alone = if b - a == 1 && !isolated(units, a) && !ends_a_phrase(text_of(units, a)) {
        COST_ONE_WORD
    } else {
        0
    };
    if fits(units, a, b, limits) {
        return Some((alone, None));
    }
    if limits.max_lines < 2 {
        return None;
    }
    let mut best: Option<(u32, Option<usize>)> = None;
    for k in a + 1..b {
        if fits(units, a, k, limits) && fits(units, k, b, limits) {
            let cost = break_cost(text_of(units, k - 1), text_of(units, k))
                + imbalance(width(units, a, k), width(units, k, b));
            if best.is_none_or(|(least, _)| cost < least) {
                best = Some((cost, Some(k)));
            }
        }
    }
    best.map(|(cost, second)| (cost + alone, second))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_lists_are_sorted_lower_case_and_without_repeats() {
        for list in [CLAUSE_STARTERS, BINDERS, VERB_BINDERS] {
            assert!(list.windows(2).all(|pair| pair[0] < pair[1]), "{list:?}");
            assert!(
                list.iter().all(|w| *w == w.to_lowercase() && *w == bare(w)),
                "{list:?}"
            );
        }
        assert!(CLAUSE_STARTERS.len() >= 35, "{}", CLAUSE_STARTERS.len());
        assert!(BINDERS.len() >= 70, "{}", BINDERS.len());
        // Every verb binder is a binder.
        assert!(VERB_BINDERS.iter().all(|w| BINDERS.contains(w)));
    }

    #[test]
    fn each_kind_of_break_has_its_cost() {
        assert_eq!(break_cost("Honestly,", "it"), COST_AFTER_PUNCT);
        assert_eq!(break_cost("rains;", "we"), COST_AFTER_PUNCT);
        assert_eq!(break_cost("worked", "because"), COST_CLAUSE);
        assert_eq!(break_cost("light", "And"), COST_CLAUSE);
        // Before a binder a phrase starts; after one it is cut.
        assert_eq!(break_cost("shipped", "the"), COST_CLAUSE);
        assert_eq!(break_cost("the", "product"), COST_AFTER_BINDER);
        assert_eq!(break_cost("To", "build"), COST_AFTER_BINDER);
        assert_eq!(break_cost("favorite", "coffee"), COST_PLAIN);
        // Before an auxiliary no phrase starts: "it / was" is a plain break.
        assert_eq!(break_cost("it", "was"), COST_PLAIN);
        // Inside a chunk "which / it" is a plain break; at the end of a
        // screen a clause starter costs a binder's, and every screen break
        // counts twice and one more.
        assert_eq!(break_cost("which", "it"), COST_PLAIN);
        let screen = |cost: u32| cost * CHUNK_BREAK_WEIGHT + COST_NEW_CHUNK;
        assert_eq!(chunk_break_cost("which", "it"), screen(COST_OPEN_CLAUSE));
        assert_eq!(chunk_break_cost("because", "we"), screen(COST_OPEN_CLAUSE));
        assert_eq!(
            chunk_break_cost("the", "product"),
            screen(COST_BINDER_AT_SCREEN_END)
        );
        assert_eq!(chunk_break_cost("then,", "from"), screen(COST_AFTER_PUNCT));
        assert_eq!(chunk_break_cost("worked", "because"), screen(COST_CLAUSE));
        assert_eq!(chunk_break_cost("found", "my"), screen(COST_CLAUSE));
        assert_eq!(chunk_break_cost("favorite", "coffee"), screen(COST_PLAIN));
        assert_eq!(
            chunk_break_cost("Honestly,", "it"),
            screen(COST_AFTER_PUNCT)
        );
        // Matched without the punctuation around the word, in any case.
        assert_eq!(break_cost("(THE", "app"), COST_AFTER_BINDER);
        assert_eq!(break_cost("app", "\"Which"), COST_CLAUSE);
        assert!(ends_a_phrase("day,") && !ends_a_phrase("day"));
        assert!(binds_to_next("the") && binds_to_next("To") && !binds_to_next("to,"));
        assert!(!binds_to_next("app") && !binds_to_next("because"));
    }

    #[test]
    fn two_lines_are_balanced_within_a_third_of_the_longer() {
        assert_eq!(imbalance(9, 12), 0);
        // 12 - 8 = 4, a third of 12: still balanced; one character more is not.
        assert_eq!(imbalance(12, 8), 0);
        assert_eq!(imbalance(12, 7), 5);
        assert_eq!(imbalance(16, 3), MAX_IMBALANCE);
        assert_eq!(imbalance(14, 6), MAX_IMBALANCE);
        assert_eq!(imbalance(12, 5), 7);
        assert_eq!(imbalance(0, 0), 0);
    }

    #[test]
    fn a_chunk_costs_its_line_split_and_one_word_alone_costs_unless_set_apart() {
        let timed = [
            ("If", 0, 100),
            ("your", 180, 300),
            ("customers", 380, 700),
            ("wait,", 780, 900),
            ("Now.", 2_000, 2_200),
        ];
        let words: Vec<CaptionWord> = timed
            .iter()
            .enumerate()
            .map(|(i, (text, start, end))| CaptionWord {
                text: (*text).to_owned(),
                start: offcut_types::OutMs::new(*start),
                end: offcut_types::OutMs::new(*end),
                word: offcut_types::WordIdx::new(u32::try_from(i).unwrap()),
                number: false,
            })
            .collect();
        let units: Vec<Unit<'_>> = words
            .iter()
            .map(|word| Unit {
                word,
                text: word.text.clone(),
                chars: u32::try_from(word.text.chars().count()).unwrap(),
            })
            .collect();
        let clean = Limits {
            per_line: 16,
            max_lines: 2,
            max_words: 4,
        };
        // "If your customers" is 17 characters: two lines, and "If / your
        // customers" (a phrase starts, 1; unbalanced, capped) beats "If your /
        // customers" (after a binder, 12).
        assert_eq!(
            chunk_cost(&units, 0, 3, &clean),
            Some((COST_CLAUSE + MAX_IMBALANCE, Some(1)))
        );
        // One word alone, not set apart by pauses: 8; after a comma: 0; set
        // apart by a second of silence: 0.
        assert_eq!(
            chunk_cost(&units, 1, 2, &clean),
            Some((COST_ONE_WORD, None))
        );
        assert_eq!(chunk_cost(&units, 3, 4, &clean), Some((0, None)));
        assert_eq!(chunk_cost(&units, 4, 5, &clean), Some((0, None)));
        // More words than the style holds, or two lines where it has one.
        assert_eq!(chunk_cost(&units, 0, 5, &clean), None);
        let one_line = Limits {
            max_lines: 1,
            ..clean
        };
        assert_eq!(chunk_cost(&units, 0, 3, &one_line), None);
    }
}
