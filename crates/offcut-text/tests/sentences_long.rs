#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Sentences of a long source (v6.1implementation §11.1, TS §43.6): over
//! 45 minutes of words every word is in exactly one sentence, in order; no
//! sentence has more than `SENTENCE_MAX_WORDS`; none crosses a pause between
//! two loops of the reference clip; and a stretch the provider wrote without a
//! stop is split at its longest pauses. The words are built as the merged
//! windows of `long_12min.mp4`-like sound give them: Script A's sentences
//! (fixtures/speech/README.md) looped, with 5 s of room tone between loops.

use offcut_text::{SENTENCE_GAP, SENTENCE_MAX_WORDS, segment_sentences};
use offcut_types::{Confidence, Sentence, TimeMs, Word};

const SCRIPT_A: &str = "Last year I almost quit making videos. Editing ate every evening, and almost nobody watched. \
So I added it up. I had spent three thousand hours editing by hand. \
That is a really BRUTAL way to make anything. The slow part was always the rough cut. \
Then I changed my workflow, and the rough cut went from ninety minutes to ten minutes. \
That change gave me my evenings back. And honestly, it was not luck.";

fn word(text: &str, start: u32, end: u32) -> Word {
    Word {
        text: text.to_owned(),
        start_ms: TimeMs::new(start),
        end_ms: TimeMs::new(end),
        confidence: Confidence::new(1.0).unwrap(),
    }
}

/// `loops` readings of Script A, 330 ms a word with 90 ms between words, 5 s
/// of room tone after each reading. With `stops: false` the provider wrote no
/// full stop, and only the 400 ms between sentences marks where they end.
fn long_source(loops: u32, stops: bool) -> Vec<Word> {
    let mut out = Vec::new();
    let mut at = 0u32;
    for _ in 0..loops {
        for sentence in SCRIPT_A.split_inclusive(['.', '!', '?']) {
            let texts: Vec<&str> = sentence.split_whitespace().collect();
            for (i, text) in texts.iter().enumerate() {
                let text = if stops {
                    (*text).to_owned()
                } else {
                    text.trim_end_matches(['.', '?', '!']).to_owned()
                };
                out.push(word(&text, at, at + 330));
                at += 330 + if i + 1 == texts.len() { 400 } else { 90 };
            }
        }
        at += 5_000;
    }
    out
}

/// Every word in exactly one sentence, in order, and the rule of 40 words held.
fn check(words: &[Word], sentences: &[Sentence]) {
    let mut next = 0u32;
    for s in sentences {
        assert_eq!(s.words.start.get(), next, "a word is missing or repeated");
        assert!(s.words.end.get() > s.words.start.get());
        let n = (s.words.end.get() - s.words.start.get()) as usize;
        assert!(n <= SENTENCE_MAX_WORDS, "a sentence of {n} words");
        // TS §43.6's MAX_SENTENCE_MS (25 s, assumption) holds over this corpus; the code's rule is the 40 words.
        let span = s.end_ms.get() - s.start_ms.get();
        assert!(span <= 25_000, "a sentence of {span} ms");
        next = s.words.end.get();
    }
    assert_eq!(next as usize, words.len());
}

/// No sentence holds two words with a pause of `SENTENCE_GAP` or more between them.
fn no_sentence_crosses_a_long_pause(words: &[Word], sentences: &[Sentence]) {
    for s in sentences {
        let piece = &words[s.words.start.get() as usize..s.words.end.get() as usize];
        for pair in piece.windows(2) {
            let gap = pair[1].start_ms.get() - pair[0].end_ms.get();
            assert!(
                gap < SENTENCE_GAP.get(),
                "a sentence crosses a pause of {gap} ms"
            );
        }
    }
}

#[test]
fn forty_five_minutes_with_stops_are_the_script_s_sentences_and_none_crosses_a_loop() {
    // 69 readings of about 39 s (the README's Script A: 75 words, 9 sentences): 45 minutes.
    let words = long_source(69, true);
    assert!(words.last().unwrap().end_ms.get() >= 2_700_000);
    let sentences = segment_sentences(&words);
    check(&words, &sentences);
    no_sentence_crosses_a_long_pause(&words, &sentences);
    // Every reading gives Script A's nine sentences.
    assert_eq!(sentences.len(), 69 * 9);
}

#[test]
fn a_long_stretch_without_stops_is_split_at_its_longest_pauses_and_kept_under_forty_words() {
    // The provider wrote no stop: only the 400 ms between sentences remains,
    // under the 700 ms gap, so each reading is one run of 80 words.
    let words = long_source(9, false);
    let sentences = segment_sentences(&words);
    check(&words, &sentences);
    no_sentence_crosses_a_long_pause(&words, &sentences);
    // Every split falls after a word that ended a sentence of the script: at a 400 ms pause.
    for s in &sentences {
        let last = s.words.end.get() as usize - 1;
        if last + 1 < words.len() {
            let gap = words[last + 1].start_ms.get() - words[last].end_ms.get();
            assert!(gap >= 400, "a split at a pause of {gap} ms");
        }
    }
}
