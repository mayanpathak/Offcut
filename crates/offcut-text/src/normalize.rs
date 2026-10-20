//! From the words a recognizer returned to a [`Transcript`]. This is the
//! only place a `Transcript` is built (TS §10.8); it is never changed after.

use std::collections::BTreeMap;

use offcut_types::{
    Confidence, NormalizedSpan, Sentence, TimeMs, Transcript, Word, WordIdx, WordRange,
};
use serde::Deserialize;

use crate::numbers::parse_quantity;
use crate::sentences::segment_sentences;
use crate::tokenize::{Token, tokenize};

/// One word as the recognizer gives it (TS §16.2). It crosses only the call
/// from the worker into this crate, so it is not one of the shared types.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawWord {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
    pub confidence: f32,
}

/// The transcript of `raw`: its words, its sentences, and the numbers that
/// were spoken. The same words give the same transcript.
pub fn normalize_transcript(raw: Vec<RawWord>, model_version: &str) -> Transcript {
    let words: Vec<Word> = raw.into_iter().filter_map(word).collect();
    let sentences = segment_sentences(&words);
    let numbers = numbers_by_sentence(&tokenize(&words, &BTreeMap::new()), &sentences);
    Transcript {
        words,
        sentences,
        numbers,
        model_version: model_version.to_owned(),
    }
}

/// A word with its text trimmed and its values in range. `None` for a word
/// with no text.
fn word(raw: RawWord) -> Option<Word> {
    let text = raw.text.trim();
    if text.is_empty() {
        return None;
    }
    // Not a number at all counts as certain: the recognizer gave no figure.
    let confidence = if raw.confidence.is_nan() {
        1.0
    } else {
        raw.confidence.clamp(0.0, 1.0)
    };
    Some(Word {
        text: text.to_owned(),
        start_ms: TimeMs::new(raw.start_ms),
        end_ms: TimeMs::new(raw.end_ms.max(raw.start_ms)),
        // In range by now, so this is always `Some`.
        confidence: Confidence::new(confidence)?,
    })
}

/// The quantities of each sentence on its own, in word order: a quantity
/// never crosses a sentence (D-104). Tokens are in word order, so each
/// sentence's tokens are one run of them.
fn numbers_by_sentence(tokens: &[Token], sentences: &[Sentence]) -> Vec<NormalizedSpan> {
    let mut rest = tokens;
    let mut spans = Vec::new();
    for sentence in sentences {
        let inside = rest
            .iter()
            .take_while(|t| t.word < sentence.words.end)
            .count();
        let (own, after) = rest.split_at(inside.min(rest.len()));
        spans.extend(numbers(own));
        rest = after;
    }
    spans
}

/// Every quantity in the tokens, with the words it was spoken in. A token
/// that is part of one quantity is not read again as the start of another.
fn numbers(tokens: &[Token]) -> Vec<NormalizedSpan> {
    let mut spans = Vec::new();
    let mut rest = tokens;
    while let Some(first) = rest.first() {
        let Some((used, quantity)) = parse_quantity(rest) else {
            // A word that begins with a comma and three digits is the next group of a
            // number the recognizer split (D-42). Without that number before it in
            // its sentence, it is no quantity of its own.
            let headless = matches!(rest, [comma, group, ..] if comma.text == "," && comma.word == group.word
                && group.text.len() == 3 && group.text.bytes().all(|b| b.is_ascii_digit()));
            rest = rest.get(if headless { 2 } else { 1 }..).unwrap_or(&[]);
            continue;
        };
        let last = rest.get(used.saturating_sub(1)).unwrap_or(first);
        spans.push(NormalizedSpan {
            words: WordRange {
                start: first.word,
                end: WordIdx::new(last.word.get().saturating_add(1)),
            },
            quantity,
        });
        rest = rest.get(used.max(1)..).unwrap_or(&[]);
    }
    spans
}

#[cfg(test)]
mod tests {
    use offcut_types::Unit;

    use super::*;

    fn raw(text: &str, start_ms: u32, end_ms: u32, confidence: f32) -> RawWord {
        RawWord {
            text: text.to_owned(),
            start_ms,
            end_ms,
            confidence,
        }
    }

    #[test]
    fn a_word_without_text_is_dropped_and_a_confidence_of_1_4_becomes_1() {
        let transcript = normalize_transcript(
            vec![
                raw("  We ", 0, 200, 1.4),
                raw("   ", 200, 300, 0.9),
                raw("made", 300, 600, -0.2),
                raw("$12,000.", 700, 600, f32::NAN),
            ],
            "asr-en-v1",
        );
        let words: Vec<(&str, u32, u32, f32)> = transcript
            .words
            .iter()
            .map(|w| {
                (
                    w.text.as_str(),
                    w.start_ms.get(),
                    w.end_ms.get(),
                    w.confidence.get(),
                )
            })
            .collect();
        assert_eq!(
            words,
            [
                ("We", 0, 200, 1.0),
                ("made", 300, 600, 0.0),
                // An end before its start is raised to the start.
                ("$12,000.", 700, 700, 1.0),
            ]
        );
        assert_eq!(transcript.model_version, "asr-en-v1");
        assert_eq!(transcript.sentences.len(), 1);

        // The number is the third word that was kept, whatever was dropped before it.
        assert_eq!(transcript.numbers.len(), 1);
        let span = &transcript.numbers[0];
        assert_eq!((span.words.start.get(), span.words.end.get()), (2, 3));
        assert_eq!(
            (
                span.quantity.value,
                &span.quantity.unit,
                span.quantity.display.as_str()
            ),
            (12_000.0, &Unit::Usd, "$12k")
        );

        // The same amount as a recognizer writes it, in two words: one span over both.
        let split = normalize_transcript(
            vec![
                raw("earned", 0, 300, 1.0),
                raw("$12", 320, 900, 1.0),
                raw(",000", 900, 1_300, 1.0),
                raw("in", 1_320, 1_400, 1.0),
            ],
            "asr-en-v1",
        );
        let spans: Vec<(u32, u32, &str)> = split
            .numbers
            .iter()
            .map(|n| {
                (
                    n.words.start.get(),
                    n.words.end.get(),
                    n.quantity.display.as_str(),
                )
            })
            .collect();
        assert_eq!(spans, [(1, 3, "$12k")]);
        assert_eq!(split.words.len(), 4);
    }

    #[test]
    fn the_same_raw_words_twice_give_equal_transcripts() {
        let spoken = || {
            vec![
                raw("It", 0, 150, 0.98),
                raw("took", 160, 400, 0.91),
                raw("two", 420, 600, 0.95),
                raw("hundred", 610, 900, 0.93),
                raw("and", 910, 1_000, 0.6),
                raw("fifty", 1_010, 1_300, 0.97),
                raw("days,", 1_310, 1_700, 0.99),
                raw("40%", 2_500, 2_900, 0.88),
                raw("less.", 2_910, 3_300, 0.9),
            ]
        };
        let first = normalize_transcript(spoken(), "asr-en-v1");
        assert_eq!(first, normalize_transcript(spoken(), "asr-en-v1"));

        // What it holds: two sentences, split by the pause, and two numbers
        // with the words they were spoken in.
        let sentences: Vec<(u32, u32)> = first
            .sentences
            .iter()
            .map(|s| (s.words.start.get(), s.words.end.get()))
            .collect();
        assert_eq!(sentences, [(0, 7), (7, 9)]);
        let numbers: Vec<(u32, u32, &str)> = first
            .numbers
            .iter()
            .map(|n| {
                (
                    n.words.start.get(),
                    n.words.end.get(),
                    n.quantity.display.as_str(),
                )
            })
            .collect();
        assert_eq!(numbers, [(2, 7, "250 days"), (7, 8, "40%")]);
    }

    /// v3implementation §8.7 and D-104: a sentence boundary between `$12` and
    /// `,000` (a pause of 900 ms) leaves one number, `$12`.
    #[test]
    fn a_number_never_crosses_a_sentence() {
        let transcript = normalize_transcript(
            vec![
                raw("it", 0, 200, 1.0),
                raw("was", 250, 500, 1.0),
                raw("$12", 550, 900, 1.0),
                raw(",000", 1_800, 2_100, 1.0),
                raw("in", 2_150, 2_300, 1.0),
                raw("sales.", 2_350, 2_800, 1.0),
            ],
            "asr-en-v1",
        );
        assert_eq!(transcript.sentences.len(), 2);
        let numbers: Vec<(u32, u32, &str)> = transcript
            .numbers
            .iter()
            .map(|n| {
                (
                    n.words.start.get(),
                    n.words.end.get(),
                    n.quantity.display.as_str(),
                )
            })
            .collect();
        assert_eq!(numbers, [(2, 3, "$12")]);
    }
}
