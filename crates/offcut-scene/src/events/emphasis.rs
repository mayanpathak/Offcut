//! Emphasis: one word that carries a sentence, marked in the captions
//! (v6.2implementation D-481, TS §47.3). Like a KeywordPop (D-169) it has no
//! visual of its own in any zone: its track hands the word, its treatment
//! and its program start to the captions (`captions_draw.rs`), which colour
//! it, underline it or highlight it from then until its chunk ends.
//!
//! `size: Punch` (a punch word, D-482) is the chunking's: `build_scene`
//! hands the word to `captions::chunk_punched`, which shows it alone, in
//! capitals, at `PUNCH_SCALE`; the captions mark it with its treatment there.

use offcut_types::{EmphasisSize, OutMs, OutSpan, Treatment, WordIdx};

use crate::display_list::DrawCmd;

/// The word the captions mark, how, and from when.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    pub word: WordIdx,
    pub treatment: Treatment,
    pub size: EmphasisSize,
    pub start: OutMs,
}

/// The item's word, from the start of its program span (its anchor is the
/// word itself, `AnchorRule::Point`).
pub fn layout(word: WordIdx, treatment: Treatment, size: EmphasisSize, span: OutSpan) -> Track {
    Track {
        word,
        treatment,
        size,
        start: span.start,
    }
}

/// Nothing: the captions draw the mark.
pub fn draw(track: &Track, t: OutMs, out: &mut Vec<DrawCmd>) {
    let _ = (track, t, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_track_is_the_word_its_treatment_and_its_start_and_draws_nothing() {
        let span = OutSpan::new(OutMs::new(1_250), OutMs::new(1_500)).unwrap();
        for treatment in [
            Treatment::Colour,
            Treatment::Underline,
            Treatment::Highlight,
        ] {
            for size in [EmphasisSize::Normal, EmphasisSize::Punch] {
                let track = layout(WordIdx::new(1), treatment, size, span);
                assert_eq!(
                    track,
                    Track {
                        word: WordIdx::new(1),
                        treatment,
                        size,
                        start: OutMs::new(1_250),
                    }
                );
                let mut out = Vec::new();
                for t in [0, 1_250, 1_400, 1_500, 9_000] {
                    draw(&track, OutMs::new(t), &mut out);
                }
                assert!(out.is_empty());
            }
        }
    }
}
