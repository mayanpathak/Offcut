//! The edit list of one track (`elst`): where in the media the track starts
//! and how long it is shown (v3implementation §6.1, D-92). Five shapes are
//! accepted; anything else is `Unsupported("elst")`, which `validate_probe`
//! never sees.
//!
//! | # | Shape | Since |
//! |---|---|---|
//! | E0 | No `elst` | V2 |
//! | E1 | One media edit | V2 |
//! | E2 | One empty edit, then one media edit | V2 |
//! | E3 | Two or more media edits, each starting where the previous one ends (within one media tick), optionally after one empty edit | V3 |
//! | E4 | Any of E1-E3, then exactly one trailing empty edit, which is ignored | V3 |

use crate::ContainerError;
use crate::sample_table::Edit;

/// At most this many entries are examined (D-60): a longer list is
/// `Unsupported("elst")`, so a crafted `elst` cannot make this loop long.
const MAX_EDITS: usize = 16;

/// Contiguous media edits may disagree by this many media ticks: the
/// rounding of a segment duration from the movie timescale.
const CONTIGUITY_TICKS: u64 = 1;

/// `edit_offset` is what to subtract from a `pts` to get the time the edit
/// list presents it at; `presented` is how long the track is shown. Both are
/// in media timescale ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EditPlan {
    pub edit_offset: i64,
    pub presented: i64,
}

/// The plan of `edits`, a media edit having `media_time >= 0` and an empty
/// one `media_time == -1`. `media_duration` is the sum of the `stts` runs.
pub(crate) fn resolve_edits(
    edits: &[Edit],
    media_timescale: u32,
    movie_timescale: u32,
    media_duration: i64,
) -> Result<EditPlan, ContainerError> {
    // E0.
    if edits.is_empty() {
        return Ok(EditPlan {
            edit_offset: 0,
            presented: media_duration,
        });
    }
    if edits.len() > MAX_EDITS {
        return Err(ContainerError::Unsupported("elst"));
    }
    let ticks = Ticks {
        media_timescale,
        movie_timescale,
        media_duration,
    };

    // E4: one trailing empty edit shows nothing after the end; it is ignored.
    let edits = match edits {
        [rest @ .., last] if !rest.is_empty() && last.media_time == -1 => rest,
        all => all,
    };
    // E2 and E3: one leading empty edit delays the start.
    let (lead, media) = match edits {
        [empty, rest @ ..] if empty.media_time == -1 => {
            (ticks.to_media(empty.segment_duration)?, rest)
        }
        all => (0, all),
    };
    // What is left must be media edits only: no second empty edit, no
    // negative `media_time` other than -1.
    let (Some(first), Some(last)) = (media.first(), media.last()) else {
        return Err(ContainerError::Unsupported("elst"));
    };
    if media.iter().any(|e| e.media_time < 0) {
        return Err(ContainerError::Unsupported("elst"));
    }
    // As V2: the first media edit's start, less the lead (0 without one).
    let edit_offset = first.media_time.saturating_sub(lead);

    // E1 and E2: one media edit, shown as V2 shows it.
    if media.len() == 1 {
        return Ok(EditPlan {
            edit_offset,
            presented: ticks.shown(first)?,
        });
    }

    // E3: contiguous media edits; the last may run to the end of the media.
    let mut presented = 0i64;
    let mut expected_start = first.media_time;
    for (k, edit) in media.iter().enumerate() {
        if edit.media_time.abs_diff(expected_start) > CONTIGUITY_TICKS {
            return Err(ContainerError::Unsupported("elst"));
        }
        let length = if k + 1 == media.len() {
            ticks.shown(last)?
        } else {
            ticks.to_media(edit.segment_duration)?
        };
        presented = presented
            .checked_add(length)
            .ok_or(ContainerError::Malformed("elst"))?;
        expected_start = edit
            .media_time
            .checked_add(length)
            .ok_or(ContainerError::Malformed("elst"))?;
    }
    Ok(EditPlan {
        edit_offset,
        presented,
    })
}

/// The two time scales of a track, and the length of its media.
struct Ticks {
    media_timescale: u32,
    movie_timescale: u32,
    media_duration: i64,
}

impl Ticks {
    /// Ticks of the movie timescale as ticks of the media timescale, to the nearest.
    fn to_media(&self, movie_ticks: u64) -> Result<i64, ContainerError> {
        if self.movie_timescale == 0 {
            return Err(ContainerError::Malformed("mvhd"));
        }
        let per = u128::from(self.movie_timescale);
        let ticks = (u128::from(movie_ticks) * u128::from(self.media_timescale) + per / 2) / per;
        i64::try_from(ticks).map_err(|_| ContainerError::Malformed("elst"))
    }

    /// How long a media edit shows its media. An edit of length 0 shows the
    /// media from its start time to its end.
    fn shown(&self, media: &Edit) -> Result<i64, ContainerError> {
        if media.segment_duration == 0 {
            Ok(self.media_duration.saturating_sub(media.media_time).max(0))
        } else {
            self.to_media(media.segment_duration)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn media(segment_duration: u64, media_time: i64) -> Edit {
        Edit {
            segment_duration,
            media_time,
        }
    }

    fn empty(segment_duration: u64) -> Edit {
        Edit {
            segment_duration,
            media_time: -1,
        }
    }

    fn plan(edit_offset: i64, presented: i64) -> Result<EditPlan, ContainerError> {
        Ok(EditPlan {
            edit_offset,
            presented,
        })
    }

    #[test]
    fn e0_no_edits() {
        assert_eq!(resolve_edits(&[], 30_000, 1_000, 90_000), plan(0, 90_000));
    }

    #[test]
    fn e1_one_media_edit() {
        // 3,000 movie ticks at 1,000/s is 3 s: 90,000 ticks of a 30,000 media timescale.
        assert_eq!(
            resolve_edits(&[media(3_000, 2_000)], 30_000, 1_000, 120_000),
            plan(2_000, 90_000)
        );
    }

    #[test]
    fn e2_an_empty_edit_then_a_media_edit() {
        let edits = [empty(500), media(4_000, 0)];
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 4_000),
            plan(-500, 4_000)
        );
    }

    #[test]
    fn e3_three_contiguous_media_edits() {
        let edits = [
            media(1_000, 1_000),
            media(1_000, 2_000),
            media(1_000, 3_000),
        ];
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 9_000),
            plan(1_000, 3_000)
        );
    }

    #[test]
    fn e3_one_tick_of_mismatch_is_accepted() {
        let edits = [
            media(1_000, 1_000),
            media(1_000, 2_001),
            media(1_000, 3_001),
        ];
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 9_000),
            plan(1_000, 3_000)
        );
    }

    #[test]
    fn e3_two_ticks_of_mismatch_are_refused() {
        let edits = [media(1_000, 1_000), media(1_000, 2_002)];
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 9_000),
            Err(ContainerError::Unsupported("elst"))
        );
    }

    #[test]
    fn e4_a_trailing_empty_edit_is_ignored() {
        let e1 = resolve_edits(&[media(3_000, 2_000)], 30_000, 1_000, 120_000);
        assert_eq!(
            resolve_edits(&[media(3_000, 2_000), empty(700)], 30_000, 1_000, 120_000),
            e1
        );
    }

    #[test]
    fn two_empty_edits_in_a_row_are_refused() {
        let edits = [empty(500), empty(500), media(1_000, 0)];
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 9_000),
            Err(ContainerError::Unsupported("elst"))
        );
    }

    #[test]
    fn seventeen_contiguous_media_edits_are_refused() {
        let edits: Vec<Edit> = (0..17).map(|k| media(100, k * 100)).collect();
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 9_000),
            Err(ContainerError::Unsupported("elst"))
        );
    }

    #[test]
    fn a_last_edit_of_length_zero_runs_to_the_end() {
        let edits = [media(0, 1_500)];
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 9_000).map(|p| p.presented),
            Ok(7_500)
        );
    }

    #[test]
    fn a_sum_that_overflows_is_malformed() {
        let edits = [
            media(u64::from(u32::MAX), i64::MAX - 10),
            media(1_000, i64::MAX - 5),
        ];
        assert_eq!(
            resolve_edits(&edits, 1_000, 1_000, 9_000),
            Err(ContainerError::Malformed("elst"))
        );
    }

    #[test]
    fn a_movie_timescale_of_zero_is_malformed() {
        assert_eq!(
            resolve_edits(&[media(3_000, 0)], 30_000, 0, 90_000),
            Err(ContainerError::Malformed("mvhd"))
        );
    }
}
