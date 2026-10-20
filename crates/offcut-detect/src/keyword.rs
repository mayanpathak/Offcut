//! KeywordPop candidates: a content word the speaker stressed, by its voice
//! or by the words around it (TS §17.4, section 9.6).

use offcut_text::{Token, TokenKind};
use offcut_types::{
    EventKind, EventParams, Prosody, SentenceIdx, Span, Transcript, WordIdx, WordRange,
};

use crate::config::{DetectorConfig, EMPHASIS_WORDS, INTENSIFIERS, STOP_WORDS};
use crate::list::{markers, sentence_tokens};
use crate::number::{Candidate, quantities};

/// One candidate per content word that is not part of a number or a list
/// marker.
pub fn find(toks: &[Token], t: &Transcript, p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate> {
    let taken: Vec<WordRange> = quantities(toks, t)
        .iter()
        .map(|found| found.words)
        .chain(markers(toks, t).iter().map(|marker| marker.words))
        .collect();
    let mut out = Vec::new();
    for (index, range) in (0u32..).zip(sentence_tokens(toks, t)) {
        let sentence = SentenceIdx::new(index);
        let mut previous: Option<&Token> = None;
        for token in toks.get(range).unwrap_or(&[]) {
            let free = !taken
                .iter()
                .any(|words| words.start <= token.word && token.word < words.end);
            if free && let Some(c) = candidate(t, p, cfg, token, previous, sentence) {
                out.push(c);
            }
            previous = Some(token);
        }
    }
    out
}

/// `token` as a keyword, when it is a word of at least `keyword_min_letters`
/// letters and no stop word. `previous` is the token before it in its
/// sentence, `sentence`.
fn candidate(
    t: &Transcript,
    p: &Prosody,
    cfg: &DetectorConfig,
    token: &Token,
    previous: Option<&Token>,
    sentence: SentenceIdx,
) -> Option<Candidate> {
    let text = token.text.as_str();
    let letters = text.chars().filter(|c| c.is_alphabetic()).count();
    if token.kind != TokenKind::Word
        || letters < cfg.keyword_min_letters
        || listed(STOP_WORDS, text)
    {
        return None;
    }
    let index = token.word.get() as usize;
    let word = t.words.get(index)?;
    let (energy_z, pitch_z) = p
        .per_word
        .get(index)
        .map_or((0.0, 0.0), |said| (said.energy_z, said.pitch_z));
    // "literally brutal", or a word that is emphasis by itself.
    let stressed = previous
        .is_some_and(|before| before.kind == TokenKind::Word && listed(INTENSIFIERS, &before.text))
        || listed(EMPHASIS_WORDS, text);

    let mut score = cfg.keyword_base
        + cfg.keyword_energy_weight * (energy_z - 1.0).clamp(0.0, 1.0)
        + cfg.keyword_pitch_weight * (pitch_z - 1.0).clamp(0.0, 1.0);
    if stressed {
        score += cfg.keyword_lexicon_bonus;
    }
    Some(Candidate {
        kind: EventKind::KeywordPop,
        anchors: WordRange {
            start: token.word,
            end: WordIdx::new(token.word.get().saturating_add(1)),
        },
        span: Span::new(word.start_ms, word.end_ms)?,
        score: score.min(1.0) * word.confidence.get(),
        params: EventParams::KeywordPop { word: token.word },
        parts: Vec::new(),
        sentence,
    })
}

fn listed(words: &[&str], word: &str) -> bool {
    words.binary_search(&word).is_ok()
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

    /// Flat, but for the words given as `(index, energy_z, pitch_z)`.
    fn prosody(t: &Transcript, said: &[(usize, f32, f32)]) -> Prosody {
        let mut per_word = vec![
            WordProsody {
                energy_z: 0.0,
                pitch_z: 0.0,
            };
            t.words.len()
        ];
        for (index, energy_z, pitch_z) in said {
            per_word[*index] = WordProsody {
                energy_z: *energy_z,
                pitch_z: *pitch_z,
            };
        }
        Prosody { per_word }
    }

    fn keywords_of(t: &Transcript, p: &Prosody) -> Vec<Candidate> {
        let toks = tokenize(&t.words, &BTreeMap::new());
        find(&toks, t, p, &DetectorConfig::default())
    }

    /// The score of the candidate on word `index`, if there is one.
    fn score_of(text: &str, said: &[(usize, f32, f32)], index: u32) -> Option<f32> {
        let t = transcript(text);
        keywords_of(&t, &prosody(&t, said))
            .iter()
            .find(|c| c.anchors.start == WordIdx::new(index))
            .map(|c| c.score)
    }

    fn close(score: Option<f32>, expected: f32) -> bool {
        score.is_some_and(|score| (score - expected).abs() < 1e-6)
    }

    #[test]
    fn the_five_rows_of_the_plan() {
        // An intensifier before an emphasis word, said loud and high: 1.00.
        assert_eq!(
            score_of("That is literally brutal.", &[(3, 2.0, 1.5)], 3),
            Some(1.0)
        );
        // An emphasis word, a little loud: 0.50 + 0.04 + 0.20.
        assert!(close(
            score_of("That was brutal.", &[(2, 1.2, 0.0)], 2),
            0.74
        ));
        // A written emphasis (section 23, item 27): 0.50 + 0.20 + 0.20.
        assert!(close(
            score_of("Workflow matters.", &[(0, 2.0, 2.0)], 0),
            0.90
        ));
        assert_eq!(score_of("Workflow matters.", &[], 0), Some(0.5));
        // A stop word is no candidate, however loud.
        assert_eq!(score_of("that is it.", &[(0, 3.0, 3.0)], 0), None);
    }

    #[test]
    fn short_words_numbers_and_markers_are_no_candidates() {
        let t = transcript("First, ship this app. It made ten thousand dollars.");
        let words: Vec<u32> = keywords_of(&t, &prosody(&t, &[]))
            .iter()
            .map(|c| c.anchors.start.get())
            .collect();
        // Only "ship": "this" and "made" are stop words; "thousand" and
        // "dollars" are the number's; "First" is a marker; "app", "it" and
        // "ten" are short.
        assert_eq!(words, [1]);
    }

    #[test]
    fn the_intensifier_must_be_right_before_in_the_same_sentence() {
        assert!(close(score_of("It is really workflow.", &[], 3), 0.70));
        assert_eq!(
            score_of("It is really. Workflow matters.", &[], 3),
            Some(0.5)
        );
        assert_eq!(score_of("really nice workflow", &[], 2), Some(0.5));
    }

    #[test]
    fn a_keyword_is_its_one_word_and_its_confidence_counts() {
        let mut t = transcript("That is literally brutal.");
        let p = prosody(&t, &[(3, 2.0, 2.0)]);
        let found = keywords_of(&t, &p);
        let brutal = found.iter().find(|c| c.anchors.start.get() == 3).unwrap();
        assert_eq!(brutal.kind, EventKind::KeywordPop);
        assert_eq!(brutal.anchors.end.get(), 4);
        assert_eq!(
            (brutal.span.start.get(), brutal.span.end.get()),
            (1_200, 1_500)
        );
        assert_eq!(brutal.parts, []);
        assert_eq!(
            brutal.params,
            EventParams::KeywordPop {
                word: WordIdx::new(3)
            }
        );
        t.words[3].confidence = Confidence::new(0.5).unwrap();
        let found = keywords_of(&t, &p);
        assert_eq!(
            found
                .iter()
                .find(|c| c.anchors.start.get() == 3)
                .unwrap()
                .score,
            0.5
        );
    }
}
