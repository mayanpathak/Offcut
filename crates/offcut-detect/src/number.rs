//! NumberReveal candidates: every number of the text as edited, with a score
//! for how much it deserves to be shown large (TS §17.4), and the words that
//! say what it counts (D-107).

use std::ops::Range;

use offcut_text::units_lex::{COUNT_UNIT_MIN, MAGNITUDE_WORDS};
use offcut_text::{Token, TokenKind, parse_quantity};
use offcut_types::{
    EventKind, EventParams, Prosody, Quantity, SentenceIdx, Span, Transcript, Unit, WordIdx,
    WordRange,
};

use crate::config::{DetectorConfig, LABEL_STOP_VERBS, NUMBER_EMPHASIS_Z, NUMBER_MAGNITUDE_MIN};

/// The suffixes of a number in digits that scale it: `10k`, `$1.5m`, `$3b`.
const SCALING_SUFFIXES: &[char] = &['k', 'm', 'b'];
/// "five grand": a thousand, as `offcut-text` reads it.
const GRAND: &str = "grand";

/// Something that may become an event. `score` is not yet compared with a
/// threshold, and a candidate is not an event until it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub kind: EventKind,
    pub anchors: WordRange,
    pub span: Span,
    pub score: f32,
    pub params: EventParams,
    /// The words of each quantity or marker it is built from: what the
    /// exclusions inspect (D-106).
    pub parts: Vec<WordRange>,
}

/// A quantity of the text as edited: its words, its tokens in the whole
/// token list, and its sentence (D-104, D-109).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Found {
    pub words: WordRange,
    pub tokens: Range<usize>,
    pub quantity: Quantity,
    pub sentence: SentenceIdx,
    /// The value came through a suffix, a magnitude word or "grand".
    pub suffixed: bool,
}

/// Every quantity of `toks`, the effective tokens, read sentence by sentence
/// as `normalize_transcript` reads the transcribed words: without edits, the
/// same words and quantities as `t.numbers`. An edited number is read again
/// from its new text. A number with a hidden word gives none: neither what is
/// left of a transcribed number that lost a word, nor a quantity whose own
/// words hold one (TS §17.6).
pub(crate) fn quantities(toks: &[Token], t: &Transcript) -> Vec<Found> {
    let mut found = Vec::new();
    let mut from = 0;
    for (index, sentence) in (0u32..).zip(&t.sentences) {
        let inside = toks
            .get(from..)
            .unwrap_or(&[])
            .iter()
            .take_while(|token| token.word < sentence.words.end)
            .count();
        let to = from + inside;
        read_sentence(toks, from..to, SentenceIdx::new(index), &mut found);
        from = to;
    }
    found.retain(|f| {
        let lost_a_word = t.numbers.iter().any(|n| {
            n.words.start < f.words.end && f.words.start < n.words.end && silent(toks, n.words)
        });
        !silent(toks, f.words) && !lost_a_word
    });
    found
}

/// The quantities of the tokens `range` of one sentence. A token that is part
/// of one quantity is not read again as the start of another.
fn read_sentence(toks: &[Token], range: Range<usize>, sentence: SentenceIdx, out: &mut Vec<Found>) {
    let mut at = range.start;
    while let Some(rest) = toks.get(at..range.end).filter(|rest| !rest.is_empty()) {
        let Some((used, quantity)) = parse_quantity(rest) else {
            // A comma and three digits of one word are the next group of a number
            // the recognizer split (D-42); alone, they are no quantity.
            let headless = matches!(rest, [comma, group, ..] if comma.text == "," && comma.word == group.word
                && group.text.len() == 3 && group.text.bytes().all(|b| b.is_ascii_digit()));
            at += if headless { 2 } else { 1 };
            continue;
        };
        let tokens = at..at + used.max(1);
        let own = toks.get(tokens.clone()).unwrap_or(&[]);
        if let (Some(first), Some(last)) = (own.first(), own.last()) {
            out.push(Found {
                words: WordRange {
                    start: first.word,
                    end: WordIdx::new(last.word.get().saturating_add(1)),
                },
                suffixed: suffixed(own),
                tokens: tokens.clone(),
                quantity,
                sentence,
            });
        }
        at = tokens.end;
    }
}

/// Whether one of these words has no token: it was hidden (an edit of `""`).
fn silent(toks: &[Token], words: WordRange) -> bool {
    (words.start.get()..words.end.get()).any(|word| {
        let word = WordIdx::new(word);
        let at = toks.partition_point(|token| token.word < word);
        toks.get(at).is_none_or(|token| token.word != word)
    })
}

fn suffixed(own: &[Token]) -> bool {
    own.iter().any(|token| match token.kind {
        TokenKind::Digits => token.text.ends_with(SCALING_SUFFIXES),
        TokenKind::Word => {
            token.text == GRAND || MAGNITUDE_WORDS.iter().any(|(word, _)| *word == token.text)
        }
        TokenKind::Punct => false,
    })
}

/// One candidate for each quantity of the text as edited.
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate> {
    quantities(toks, t)
        .into_iter()
        .filter_map(|found| candidate(toks, t, p, cfg, found))
        .collect()
}

fn candidate(
    toks: &[Token],
    t: &Transcript,
    p: &Prosody,
    cfg: &DetectorConfig,
    found: Found,
) -> Option<Candidate> {
    let range = found.words.start.get() as usize..found.words.end.get() as usize;
    let words = t.words.get(range.clone())?;
    let (first, last) = (words.first()?, words.last()?);

    let mut score = cfg.number_base;
    if explicit_unit(&found.quantity) {
        score += cfg.number_unit_bonus;
    }
    if found.quantity.value.abs() >= NUMBER_MAGNITUDE_MIN || found.suffixed {
        score += cfg.number_magnitude_bonus;
    }
    let emphasized = p
        .per_word
        .get(range)
        .is_some_and(|said| said.iter().any(|word| word.energy_z >= NUMBER_EMPHASIS_Z));
    if emphasized {
        score += cfg.number_energy_bonus;
    }
    // A number is never more certain than the least certain of its words.
    let heard = words
        .iter()
        .map(|word| word.confidence.get())
        .fold(1.0f32, f32::min);
    let score = score.min(1.0) * heard;

    let label = label(toks, t, cfg, &found);
    Some(Candidate {
        kind: EventKind::NumberReveal,
        anchors: found.words,
        span: Span::new(first.start_ms, last.end_ms)?,
        score,
        params: EventParams::NumberReveal {
            value: found.quantity,
            label,
        },
        parts: vec![found.words],
    })
}

/// A unit that says what the number is: a currency, a share, a factor, a
/// time, a size, or a count of at least `COUNT_UNIT_MIN` (D-101).
fn explicit_unit(quantity: &Quantity) -> bool {
    match &quantity.unit {
        Unit::None => false,
        Unit::Count { .. } => quantity.value >= COUNT_UNIT_MIN,
        Unit::Usd
        | Unit::Eur
        | Unit::Gbp
        | Unit::Inr
        | Unit::Percent
        | Unit::Times
        | Unit::Milliseconds
        | Unit::Seconds
        | Unit::Minutes
        | Unit::Hours
        | Unit::Days
        | Unit::Weeks
        | Unit::Months
        | Unit::Years
        | Unit::Bytes
        | Unit::Kilobytes
        | Unit::Megabytes
        | Unit::Gigabytes => true,
    }
}

/// The words after the quantity that say what it counts (D-107): up to
/// `number_label_max_words` words of its sentence, stopping at punctuation,
/// at a verb of `LABEL_STOP_VERBS`, or at a number in digits. `None` when no
/// word was taken.
fn label(toks: &[Token], t: &Transcript, cfg: &DetectorConfig, found: &Found) -> Option<String> {
    let end = t.sentences.get(found.sentence.get() as usize)?.words.end;
    let mut taken: Vec<&str> = Vec::new();
    for token in toks
        .get(found.tokens.end..)?
        .iter()
        .take_while(|token| token.word < end)
    {
        if taken.len() >= cfg.number_label_max_words {
            break;
        }
        match token.kind {
            TokenKind::Word
                if LABEL_STOP_VERBS
                    .binary_search(&token.text.as_str())
                    .is_err() =>
            {
                taken.push(&token.text);
            }
            TokenKind::Word | TokenKind::Digits | TokenKind::Punct => break,
        }
    }
    (!taken.is_empty()).then(|| taken.join(" "))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use offcut_text::{RawWord, normalize_transcript, tokenize};
    use offcut_types::{Confidence, WordProsody};

    use super::*;

    /// Words 300 ms long and 100 ms apart, all certain.
    fn transcript(text: &str) -> Transcript {
        let raw = (0u32..)
            .zip(text.split_whitespace())
            .map(|(i, word)| RawWord {
                text: word.to_owned(),
                start_ms: i * 400,
                end_ms: i * 400 + 300,
                confidence: 1.0,
            })
            .collect();
        normalize_transcript(raw, "test")
    }

    fn flat(t: &Transcript) -> Prosody {
        let neutral = WordProsody {
            energy_z: 0.0,
            pitch_z: 0.0,
        };
        Prosody {
            per_word: vec![neutral; t.words.len()],
        }
    }

    fn edited(t: &Transcript, edits: &[(u32, &str)]) -> Vec<Token> {
        let edits = edits
            .iter()
            .map(|(word, text)| (WordIdx::new(*word), (*text).to_owned()))
            .collect();
        tokenize(&t.words, &edits)
    }

    fn candidates(text: &str, edits: &[(u32, &str)]) -> Vec<Candidate> {
        let t = transcript(text);
        find(
            &edited(&t, edits),
            &t,
            &flat(&t),
            &DetectorConfig::default(),
        )
    }

    fn range(start: u32, end: u32) -> WordRange {
        WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        }
    }

    fn value_and_label(c: &Candidate) -> (&Quantity, Option<&str>) {
        match &c.params {
            EventParams::NumberReveal { value, label } => (value, label.as_deref()),
            EventParams::ListReveal { .. }
            | EventParams::FromTo { .. }
            | EventParams::KeywordPop { .. } => panic!("{:?}", c.params),
        }
    }

    #[test]
    fn without_edits_the_quantities_are_the_transcripts_numbers() {
        let t = transcript(
            "I had spent 3000 hours. Last month those clips earned $12 ,000 in sales. \
             About ten thousand people and 40% of them came twice. ,000 alone is nothing.",
        );
        let found = quantities(&tokenize(&t.words, &BTreeMap::new()), &t);
        let pairs: Vec<(WordRange, &Quantity)> =
            found.iter().map(|f| (f.words, &f.quantity)).collect();
        let numbers: Vec<(WordRange, &Quantity)> =
            t.numbers.iter().map(|n| (n.words, &n.quantity)).collect();
        assert_eq!(pairs, numbers);
        let sentences: Vec<u32> = found.iter().map(|f| f.sentence.get()).collect();
        assert_eq!(sentences, [0, 1, 2, 2]);
    }

    #[test]
    fn an_edit_of_3000_to_4000_gives_one_quantity_of_4000_at_the_same_words() {
        let t = transcript("I had spent 3000 hours editing by hand.");
        let found = quantities(&edited(&t, &[(3, "4000")]), &t);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].words, range(3, 5));
        assert_eq!(found[0].tokens, 3..5);
        let quantity = &found[0].quantity;
        assert_eq!((quantity.value, &quantity.unit), (4_000.0, &Unit::Hours));
        // An edit that makes a number where there was none is read too.
        let t = transcript("We made a lot there.");
        let found = quantities(&edited(&t, &[(3, "$5k")]), &t);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].words, range(3, 4));
        assert_eq!(found[0].quantity.display, "$5k");
    }

    #[test]
    fn a_number_with_a_hidden_word_gives_none() {
        // What is left of "$12 ,000" is not "$12": the number lost a word.
        assert_eq!(candidates("they earned $12 ,000 in sales.", &[(3, "")]), []);
        // A quantity whose words hold a hidden one: "ten … dollars".
        assert_eq!(candidates("about ten thousand dollars.", &[(2, "")]), []);
        // A hidden word next to a number takes nothing from it.
        let kept = candidates("they earned $12 ,000 in sales.", &[(1, "")]);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].anchors, range(2, 4));
    }

    #[test]
    fn the_five_labels_of_script_a() {
        let subscribers = Unit::Count {
            noun: "subscribers".to_owned(),
        };
        let rows = [
            (
                "I had spent 3000 hours editing by hand.",
                3_000.0,
                Unit::Hours,
                "3,000 hours",
                Some("editing by hand"),
            ),
            (
                "Last month those clips earned $12 ,000 in sales.",
                12_000.0,
                Unit::Usd,
                "$12k",
                Some("in sales"),
            ),
            (
                "They grew to 10,000 subscribers, which was wild.",
                10_000.0,
                subscribers,
                "10,000 subscribers",
                None,
            ),
            (
                "That is 40% of all views.",
                40.0,
                Unit::Percent,
                "40%",
                Some("of all views"),
            ),
            (
                "It hit $2m and stopped.",
                2_000_000.0,
                Unit::Usd,
                "$2m",
                Some("and stopped"),
            ),
        ];
        for (text, value, unit, display, label) in rows {
            let found = candidates(text, &[]);
            assert_eq!(found.len(), 1, "{text}");
            let (quantity, got) = value_and_label(&found[0]);
            let read = (quantity.value, &quantity.unit, quantity.display.as_str());
            assert_eq!(read, (value, &unit, display), "{text}");
            assert_eq!(got, label, "{text}");
        }
    }

    #[test]
    fn a_label_stops_at_a_stop_verb_a_number_or_three_words() {
        let label = |text: &str| {
            let found = candidates(text, &[]);
            value_and_label(&found[0]).1.map(str::to_owned)
        };
        assert_eq!(
            label("We got 500 in total was the news."),
            Some("in total".to_owned())
        );
        assert_eq!(label("That $5k is what we made."), None);
        assert_eq!(
            label("We spent 90 minutes on 3 drafts."),
            Some("on".to_owned())
        );
        assert_eq!(
            label("It took 3000 hours of very hard work."),
            Some("of very hard".to_owned())
        );
        // A count noun belongs to the number, not to its label.
        assert_eq!(label("We got 500 users."), None);
        // Nothing after the number, no label.
        assert_eq!(label("It cost $500"), None);
    }

    #[test]
    fn a_number_scores_by_its_unit_its_magnitude_and_its_emphasis() {
        let score = |text: &str| candidates(text, &[])[0].score;
        assert_eq!(score("10,000 subscribers"), 1.0);
        assert_eq!(score("$500"), 0.80);
        assert_eq!(score("3000"), 0.70);
        assert_eq!(score("ten thousand"), 0.70);
        assert_eq!(score("three"), 0.50);
        // A count is a unit from 100 on (D-101).
        assert_eq!(score("a hundred people"), 0.80);
        assert_eq!(score("50 people"), 0.50);
        // A suffix earns the magnitude bonus below 1,000 too.
        assert_eq!(score("$0.5k"), 1.0);

        // Emphasis on any word of the number; the least certain word counts.
        let cfg = DetectorConfig::default();
        let mut t = transcript("$12 ,000");
        let toks = edited(&t, &[]);
        let mut p = flat(&t);
        p.per_word[1].energy_z = 1.0;
        assert_eq!(find(&toks, &t, &p, &cfg)[0].score, 1.0);
        t.words[1].confidence = Confidence::new(0.5).unwrap();
        assert_eq!(find(&toks, &t, &p, &cfg)[0].score, 0.5);
        let t = transcript("40%");
        let mut p = flat(&t);
        p.per_word[0].energy_z = 1.0;
        // 0.50 + 0.30 + 0.10 in f32.
        assert!((find(&edited(&t, &[]), &t, &p, &cfg)[0].score - 0.90).abs() < 1e-6);
        p.per_word[0].energy_z = 0.99;
        assert_eq!(find(&edited(&t, &[]), &t, &p, &cfg)[0].score, 0.80);
    }

    #[test]
    fn a_candidate_spans_its_words_and_has_them_as_its_one_part() {
        let found = candidates("Last month those clips earned $12 ,000 in sales.", &[]);
        let c = &found[0];
        assert_eq!(c.kind, EventKind::NumberReveal);
        assert_eq!(c.anchors, range(5, 7));
        assert_eq!(c.parts, [range(5, 7)]);
        assert_eq!((c.span.start.get(), c.span.end.get()), (2_000, 2_700));
    }
}
