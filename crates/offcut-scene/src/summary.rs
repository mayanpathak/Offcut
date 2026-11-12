//! What the scene changed, for "What we changed" (TS §10.6, D-144).

use offcut_types::{ChangeSummary, EventKind};

use crate::events::EventTrack;

/// The voice is always cleaned (the chain runs since V3); the captions are
/// emphasized by each KeywordPop and each Emphasis drawn (V6.2, D-481: both
/// mark a caption word); every other item drawn is a visual moment. A
/// disabled or undrawable item has no track, and is not counted.
pub fn change_summary(tracks: &[EventTrack]) -> ChangeSummary {
    let in_captions =
        |track: &EventTrack| matches!(track.kind(), EventKind::KeywordPop | EventKind::Emphasis);
    let count = |pops: bool| {
        let n = tracks
            .iter()
            .filter(|track| in_captions(track) == pops)
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    };
    ChangeSummary {
        voice_cleaned: true,
        captions_emphasized: count(true),
        visual_moments: count(false),
    }
}

#[cfg(test)]
mod tests {
    use offcut_types::{OutMs, OutSpan, WordIdx};

    use super::*;
    use crate::events::keyword_pop;

    #[test]
    fn pops_emphasize_the_captions_and_every_other_track_is_a_moment() {
        let span = OutSpan::new(OutMs::new(0), OutMs::new(100)).unwrap();
        let pop = || EventTrack::KeywordPop(keyword_pop::layout(WordIdx::new(0), span));
        let none = change_summary(&[]);
        assert_eq!(
            (
                none.voice_cleaned,
                none.captions_emphasized,
                none.visual_moments
            ),
            (true, 0, 0)
        );
        let two = change_summary(&[pop(), pop()]);
        assert_eq!((two.captions_emphasized, two.visual_moments), (2, 0));
    }

    #[test]
    fn an_emphasis_emphasizes_the_captions_as_a_keyword_pop_does() {
        use crate::events::{emphasis, punch_in};
        use offcut_types::{EmphasisSize, Treatment, ZoomMotion, ZoomScale};
        let span = OutSpan::new(OutMs::new(0), OutMs::new(100)).unwrap();
        let mark = |treatment| {
            let word = WordIdx::new(0);
            EventTrack::Emphasis(emphasis::layout(
                word,
                treatment,
                EmphasisSize::Normal,
                span,
            ))
        };
        let pop = EventTrack::KeywordPop(keyword_pop::layout(WordIdx::new(1), span));
        let zoom = EventTrack::PunchIn(punch_in::layout(ZoomScale::Mild, ZoomMotion::Snap, span));
        let all = [
            mark(Treatment::Colour),
            mark(Treatment::Underline),
            mark(Treatment::Highlight),
            pop,
            zoom,
        ];
        let summary = change_summary(&all);
        assert_eq!(
            (summary.captions_emphasized, summary.visual_moments),
            (4, 1)
        );
    }
}
