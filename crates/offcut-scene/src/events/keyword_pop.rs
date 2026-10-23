//! KeywordPop: a stressed word, popped in the captions (TS §19.5, §8.7). It
//! has no visual of its own in the event zone: its track hands the word and
//! its program start to the captions, which colour and scale it
//! (`captions_draw.rs`, D-169).

use offcut_types::{OutMs, OutSpan, WordIdx};

use crate::display_list::DrawCmd;

/// The word the captions pop, and when the pop starts.
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub word: WordIdx,
    pub start: OutMs,
}

/// The item's word, from the start of its program span.
pub fn layout(word: WordIdx, span: OutSpan) -> Track {
    Track {
        word,
        start: span.start,
    }
}

/// Nothing: the captions draw the pop.
pub fn draw(track: &Track, t: OutMs, out: &mut Vec<DrawCmd>) {
    let _ = (track, t, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_track_is_the_word_from_its_start_and_draws_nothing() {
        let span = OutSpan::new(OutMs::new(1_250), OutMs::new(1_500)).unwrap();
        let track = layout(WordIdx::new(1), span);
        assert_eq!(
            (track.word, track.start),
            (WordIdx::new(1), OutMs::new(1_250))
        );
        let mut out = Vec::new();
        for t in [0, 1_250, 1_400, 9_000] {
            draw(&track, OutMs::new(t), &mut out);
        }
        assert!(out.is_empty());
    }
}
