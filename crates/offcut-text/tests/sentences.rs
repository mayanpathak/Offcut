#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Where sentences end (TS §17.2; v3implementation §8.6, §19.10): V2's full
//! stops and pauses, then no sentence of more than 40 words.

use offcut_text::segment_sentences;
use offcut_types::{Confidence, Sentence, TimeMs, Word};

fn word(text: &str, start: u32, end: u32) -> Word {
    Word {
        text: text.to_owned(),
        start_ms: TimeMs::new(start),
        end_ms: TimeMs::new(end),
        confidence: Confidence::new(1.0).unwrap(),
    }
}

/// `n` words of 300 ms with gaps of 100 ms, except `gap_ms` after word `after` (1-based; 0 for none).
fn spaced(n: u32, after: u32, gap_ms: u32) -> Vec<Word> {
    let mut at = 0;
    (0..n)
        .map(|i| {
            let w = word("w", at, at + 300);
            at += 300 + if i + 1 == after { gap_ms } else { 100 };
            w
        })
        .collect()
}

fn ranges(sentences: &[Sentence]) -> Vec<(u32, u32)> {
    sentences
        .iter()
        .map(|s| (s.words.start.get(), s.words.end.get()))
        .collect()
}

#[test]
fn a_full_stop_and_a_pause_each_end_a_sentence() {
    let words = [
        word("Hi.", 0, 400),
        word("There", 500, 900),
        word("now", 1_700, 2_000),
    ];
    assert_eq!(ranges(&segment_sentences(&words)), [(0, 1), (1, 2), (2, 3)]);
}

#[test]
fn a_question_mark_and_an_exclamation_mark_each_end_one() {
    let words = [
        word("why?", 0, 300),
        word("because", 400, 700),
        word("wow!", 800, 1_100),
        word("yes", 1_200, 1_500),
    ];
    assert_eq!(ranges(&segment_sentences(&words)), [(0, 1), (1, 3), (3, 4)]);
}

#[test]
fn a_gap_of_exactly_700_ms_ends_a_sentence_and_699_ms_does_not() {
    let at_700 = [word("so", 0, 300), word("then", 1_000, 1_300)];
    assert_eq!(ranges(&segment_sentences(&at_700)), [(0, 1), (1, 2)]);
    let at_699 = [word("so", 0, 300), word("then", 999, 1_300)];
    assert_eq!(ranges(&segment_sentences(&at_699)), [(0, 2)]);
}

#[test]
fn forty_one_words_with_equal_gaps_split_after_the_first() {
    assert_eq!(
        ranges(&segment_sentences(&spaced(41, 0, 100))),
        [(0, 1), (1, 41)]
    );
}

#[test]
fn forty_one_words_split_at_their_one_longer_gap() {
    assert_eq!(
        ranges(&segment_sentences(&spaced(41, 25, 600))),
        [(0, 25), (25, 41)]
    );
}

#[test]
fn a_hundred_and_twenty_words_make_sentences_of_at_most_40_in_order() {
    let words = spaced(120, 0, 100);
    let sentences = segment_sentences(&words);
    assert!(
        sentences
            .iter()
            .all(|s| s.words.end.get() - s.words.start.get() <= 40)
    );
    let covered: Vec<u32> = sentences
        .iter()
        .flat_map(|s| s.words.start.get()..s.words.end.get())
        .collect();
    assert_eq!(covered, (0..120).collect::<Vec<_>>());
}

#[test]
fn no_words_no_sentence() {
    assert!(segment_sentences(&[]).is_empty());
}

#[test]
fn a_sentence_runs_from_its_first_word_to_its_last() {
    let words = spaced(120, 60, 900);
    for s in segment_sentences(&words) {
        let first = &words[s.words.start.get() as usize];
        let last = &words[s.words.end.get() as usize - 1];
        assert_eq!((s.start_ms, s.end_ms), (first.start_ms, last.end_ms));
    }
}
