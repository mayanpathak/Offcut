//! Where one sentence ends and the next begins (TS §17.2).

use offcut_types::{DurMs, Sentence, Word, WordIdx, WordRange};

/// A silence this long between two words starts a new sentence.
pub const SENTENCE_GAP: DurMs = DurMs::new(700);
/// A sentence with more words than this is split at its longest gap (TS §17.2).
pub const SENTENCE_MAX_WORDS: usize = 40;

/// The sentences of `words`: every word is in exactly one, in order. A
/// sentence ends after a word that ends in `.`, `?` or `!`, and before a word
/// that starts at least [`SENTENCE_GAP`] after the one before it ended. A
/// sentence of more than [`SENTENCE_MAX_WORDS`] words is then split at its
/// longest internal gap, the earliest of equal gaps, until none is longer.
pub fn segment_sentences(words: &[Word]) -> Vec<Sentence> {
    by_stops_and_pauses(words)
        .into_iter()
        .flat_map(|s| split_long(words, s))
        .collect()
}

/// V2's rule: a full stop, a question mark, an exclamation mark, or a pause.
fn by_stops_and_pauses(words: &[Word]) -> Vec<Sentence> {
    let mut sentences = Vec::new();
    let mut first: Option<(u32, &Word)> = None;
    let mut following = words.iter().skip(1);
    for (index, word) in (0u32..).zip(words) {
        let (start, opened) = *first.get_or_insert((index, word));
        let next = following.next();
        let stops = word.text.ends_with(['.', '?', '!']);
        let pause = next.is_some_and(|next| {
            next.start_ms
                .duration_since(word.end_ms)
                .is_some_and(|gap| gap >= SENTENCE_GAP)
        });
        if stops || pause || next.is_none() {
            sentences.push(Sentence {
                words: WordRange {
                    start: WordIdx::new(start),
                    end: WordIdx::new(index.saturating_add(1)),
                },
                start_ms: opened.start_ms,
                end_ms: word.end_ms,
            });
            first = None;
        }
    }
    sentences
}

/// `sentence` as pieces of at most [`SENTENCE_MAX_WORDS`] words, in order: a
/// longer piece is split after the word followed by its longest gap (the
/// earliest of equal gaps), and each half is checked again.
fn split_long(words: &[Word], sentence: Sentence) -> Vec<Sentence> {
    let mut done = Vec::new();
    // Pieces still to check, the next one last.
    let mut pending = vec![(sentence.words.start.get(), sentence.words.end.get())];
    while let Some((start, end)) = pending.pop() {
        let Some(piece) = words.get(start as usize..end as usize) else {
            continue;
        };
        if piece.len() <= SENTENCE_MAX_WORDS {
            if let (Some(first), Some(last)) = (piece.first(), piece.last()) {
                done.push(Sentence {
                    words: WordRange {
                        start: WordIdx::new(start),
                        end: WordIdx::new(end),
                    },
                    start_ms: first.start_ms,
                    end_ms: last.end_ms,
                });
            }
            continue;
        }
        let gap = |w: &[Word]| match w {
            [a, b] => b.start_ms.get().saturating_sub(a.end_ms.get()),
            _ => 0,
        };
        let longest = piece.windows(2).map(gap).max().unwrap_or(0);
        let after = piece
            .windows(2)
            .position(|w| gap(w) == longest)
            .unwrap_or(0) as u32;
        let cut = start + after + 1;
        pending.push((cut, end));
        pending.push((start, cut));
    }
    done
}

#[cfg(test)]
mod tests {
    use offcut_types::{Confidence, TimeMs};

    use super::*;

    fn word(text: &str, start: u32, end: u32) -> Word {
        Word {
            text: text.to_owned(),
            start_ms: TimeMs::new(start),
            end_ms: TimeMs::new(end),
            confidence: Confidence::new(1.0).unwrap(),
        }
    }

    #[test]
    fn a_full_stop_and_a_pause_of_700_ms_each_end_a_sentence() {
        let words = [
            word("Hi.", 0, 400),
            word("There", 500, 900),
            word("now", 1_700, 2_000),
        ];
        let sentences = segment_sentences(&words);
        let ranges: Vec<(u32, u32, u32, u32)> = sentences
            .iter()
            .map(|s| {
                (
                    s.words.start.get(),
                    s.words.end.get(),
                    s.start_ms.get(),
                    s.end_ms.get(),
                )
            })
            .collect();
        assert_eq!(
            ranges,
            [(0, 1, 0, 400), (1, 2, 500, 900), (2, 3, 1_700, 2_000)]
        );

        // 699 ms is not a pause; 700 ms is. A question mark ends a sentence too.
        let words = [
            word("so", 0, 300),
            word("then", 999, 1_200),
            word("why?", 1_900, 2_200),
            word("because", 2_300, 2_600),
        ];
        let sentences = segment_sentences(&words);
        let ends: Vec<u32> = sentences.iter().map(|s| s.words.end.get()).collect();
        assert_eq!(ends, [2, 3, 4]);
        assert_eq!(sentences[0].end_ms, TimeMs::new(1_200));
        assert!(segment_sentences(&[]).is_empty());
    }

    /// `n` words of 300 ms with gaps of 100 ms, except `gap_ms` after word `after`.
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

    fn lengths(sentences: &[Sentence]) -> Vec<u32> {
        sentences
            .iter()
            .map(|s| s.words.end.get() - s.words.start.get())
            .collect()
    }

    /// v3implementation §8.7: the 40-word split and V2's gap rule.
    #[test]
    fn a_sentence_over_40_words_is_split_at_its_longest_gap() {
        // A gap of 900 ms: V2's rule splits there before the 40-word rule is reached.
        assert_eq!(lengths(&segment_sentences(&spaced(45, 30, 900))), [30, 15]);
        // 650 ms is no pause, but it is the longest gap of a 45-word sentence.
        let split = segment_sentences(&spaced(45, 30, 650));
        assert_eq!(lengths(&split), [30, 15]);
        assert_eq!(
            (split[1].words.start.get(), split[1].start_ms.get()),
            (30, 30 * 400 + (650 - 100))
        );
        // Equal gaps: the earliest is after the first word, so one word goes at a time.
        let equal = segment_sentences(&spaced(90, 0, 100));
        let mut expected = vec![1; 50];
        expected.push(40);
        assert_eq!(lengths(&equal), expected);
        let covered: Vec<u32> = equal
            .iter()
            .flat_map(|s| s.words.start.get()..s.words.end.get())
            .collect();
        assert_eq!(covered, (0..90).collect::<Vec<_>>());
    }
}
