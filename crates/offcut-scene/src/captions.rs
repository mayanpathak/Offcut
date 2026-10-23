//! Captions: which words are shown together, when, and where (TS §19.4).
//! Every time here is a program time: a word's recording times go through
//! the time map once, in `words`, and are used as they come out (D-126).

use std::collections::BTreeMap;

use offcut_text::{format_quantity, parse_quantity, tokenize};
use offcut_types::{
    Confidence, OutMs, OutSpan, Sentence, Span, TimeMap, TimeMs, Transcript, Word, WordIdx,
};

use crate::SceneError;
pub use crate::captions_draw::draw;
// The inline test of `draw`, below, reads the commands it adds.
#[cfg_attr(not(test), allow(unused_imports))]
use crate::display_list::DrawCmd;
use crate::display_list::{FontKey, Rect};
use crate::fonts::FontSet;
use crate::layout::{ShapedRun, shape};
use crate::readability::{Limits, Unit, chunk_break_cost, chunk_cost, text_of};
use crate::styles::{Case, StyleSpec};
use crate::theme::Theme;

/// A pause this long between two words starts a new chunk.
pub const CHUNK_GAP_MS: u32 = 350;
/// A chunk is shown this long before its first word starts,
pub const CHUNK_LEAD_MS: u32 = 80;
/// and this long after its last word ends.
pub const CHUNK_TAIL_MS: u32 = 120;

/// One word of a caption. A number that was spoken in several words is one
/// caption word; `word` is the first transcript word it stands for.
/// `number`: it shows a number (D-169), made from a number span of the
/// transcript or reading as one the way the captions show it.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionWord {
    pub text: String,
    pub start: OutMs,
    pub end: OutMs,
    pub word: WordIdx,
    pub number: bool,
}

/// A word of a line: its runs, and where it starts, measured from the start
/// of the line. `word` is its place in the chunk's `words`.
#[derive(Clone, Debug, PartialEq)]
pub struct LineWord {
    pub word: usize,
    pub x: f32,
    pub runs: Vec<ShapedRun>,
}

/// A line on the canvas, in logical pixels: the left end of its baseline,
/// and the scale that makes a line too wide for the zone fit it.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub words: Vec<LineWord>,
    pub x: f32,
    pub baseline: f32,
    pub width: f32,
    pub scale: f32,
}

/// Words that are shown together. `show` includes its start and not its end.
#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub words: Vec<CaptionWord>,
    pub show: OutSpan,
    pub lines: Vec<Line>,
}

impl Chunk {
    pub fn visible_at(&self, t: OutMs) -> bool {
        self.show.start <= t && t < self.show.end
    }
}

/// The words captions show (TS §17.2): an edit in place of what was
/// transcribed, no word that was edited to nothing, and one word carrying
/// the display of a number in place of the words it was spoken in. A number
/// with an edited word in it is shown as its words. A word with no program
/// time is not shown.
pub fn words(
    transcript: &Transcript,
    edits: &BTreeMap<WordIdx, String>,
    map: &TimeMap,
) -> Vec<CaptionWord> {
    let count = u32::try_from(transcript.words.len()).unwrap_or(u32::MAX);
    let mut out = Vec::new();
    let mut index = 0u32;
    while index < count {
        let number = transcript.numbers.iter().find(|span| {
            let (start, end) = (span.words.start.get(), span.words.end.get());
            let edited = edits
                .range(span.words.start..span.words.end)
                .next()
                .is_some();
            start == index && end > start && end <= count && !edited
        });
        let first = transcript.words.get(index as usize);
        if let (Some(number), Some(first)) = (number, first) {
            let end = number.words.end.get();
            let last = transcript
                .words
                .get(end.saturating_sub(1) as usize)
                .unwrap_or(first);
            let display = &number.quantity.display;
            let times = (first.start_ms, last.end_ms);
            push(&mut out, (display, true), times, index, map);
            index = end;
            continue;
        }
        if let Some(word) = first {
            let text = edits.get(&WordIdx::new(index)).unwrap_or(&word.text);
            let shown = (text.as_str(), shows_a_number(text));
            push(&mut out, shown, (word.start_ms, word.end_ms), index, map);
        }
        index = index.saturating_add(1);
    }
    out
}

/// Whether `text` reads as the captions show a number: `parse_quantity`
/// reads all of it but a closing mark, and `format_quantity` writes it back
/// the same (TS §17.2). "$500" and "3" do; "2024" (shown "2,024") and "five"
/// (shown "5") do not.
fn shows_a_number(text: &str) -> bool {
    let bare = text.trim_end_matches(['.', ',', '!', '?', ';', ':']);
    let word = Confidence::new(1.0).map(|confidence| Word {
        text: bare.to_owned(),
        start_ms: TimeMs::new(0),
        end_ms: TimeMs::new(0),
        confidence,
    });
    let tokens = tokenize(word.as_slice(), &BTreeMap::new());
    parse_quantity(&tokens).is_some_and(|(used, quantity)| {
        used == tokens.len() && format_quantity(quantity.value, &quantity.unit) == bare
    })
}

fn push(
    out: &mut Vec<CaptionWord>,
    (text, number): (&str, bool),
    (start, end): (TimeMs, TimeMs),
    word: u32,
    map: &TimeMap,
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    let spoken = Span {
        start,
        end: end.max(start),
    };
    let Some(shown) = map.out_span(spoken) else {
        return;
    };
    out.push(CaptionWord {
        text: text.to_owned(),
        start: shown.start,
        end: shown.end,
        word: WordIdx::new(word),
        number,
    });
}

/// A word of a chunk that is being put together: its text in the style's
/// case, and the line of the chunk it is on.
struct Placed<'a> {
    word: &'a CaptionWord,
    text: String,
    line: u32,
}

/// The most caption words between two hard breaks that one search lays out
/// (TS §17.2's split): a longer run is cut there.
pub const MAX_SEGMENT_WORDS: usize = 40;

/// The least-cost layout of `units[from..to]`, a run between two hard breaks
/// (D-167): its chunks, each `(start, end, second line)`. `best[i]` is the
/// least cost of the first `i` words, and the start of its last chunk; of two
/// layouts of one cost, the one whose last chunk starts later wins: the
/// earlier screens are the fuller, as a reader fills them. A run of
/// one word is one chunk.
fn search(
    units: &[Unit<'_>],
    from: usize,
    to: usize,
    limits: &Limits,
) -> Vec<(usize, usize, Option<usize>)> {
    let n = to - from;
    let mut best: Vec<Option<(u32, usize, Option<usize>)>> = vec![None; n + 1];
    if let Some(first) = best.first_mut() {
        *first = Some((0, 0, None));
    }
    for i in 1..=n {
        for j in i.saturating_sub(limits.max_words)..i {
            let Some(&Some((so_far, _, _))) = best.get(j) else {
                continue;
            };
            let Some((cost, second)) = chunk_cost(units, from + j, from + i, limits) else {
                continue;
            };
            let cut = if j == 0 {
                0
            } else {
                chunk_break_cost(text_of(units, from + j - 1), text_of(units, from + j))
            };
            let total = so_far.saturating_add(cut).saturating_add(cost);
            if let Some(slot) = best.get_mut(i)
                && slot.is_none_or(|(least, ..)| total <= least)
            {
                *slot = Some((total, j, second.map(|k| k - from)));
            }
        }
    }
    let mut chunks = Vec::new();
    let mut end = n;
    while end > 0 {
        let Some(&Some((_, start, second))) = best.get(end) else {
            break;
        };
        chunks.push((from + start, from + end, second.map(|k| from + k)));
        end = start;
    }
    chunks.reverse();
    chunks
}

/// The chunks of `words`, each one shaped and placed. Hard breaks end a run
/// of words: a sentence end and a pause of `CHUNK_GAP_MS`. Inside a run the
/// chunks and their lines are the ones of least cost (D-167), within the
/// style's `max_words`, `max_lines` and characters per line. The lines are
/// centred in `block`, the caption block of the clip's placement (D-171).
pub fn chunk(
    words: &[CaptionWord],
    sentences: &[Sentence],
    style: &StyleSpec,
    theme: &Theme,
    fonts: &FontSet,
    block: Rect,
) -> Result<Vec<Chunk>, SceneError> {
    let limits = Limits {
        per_line: style.max_chars_per_line.max(1),
        max_lines: style.max_lines.max(1),
        max_words: style.max_words.max(1) as usize,
    };
    let sentence_of = |word: &CaptionWord| sentences.partition_point(|s| s.words.end <= word.word);
    let units: Vec<Unit<'_>> = words
        .iter()
        .map(|word| {
            let text = match style.case {
                Case::AsSpoken => word.text.clone(),
                Case::Upper => word.text.to_uppercase(),
            };
            let chars = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
            Unit { word, text, chars }
        })
        .collect();

    let mut groups: Vec<Vec<Placed<'_>>> = Vec::new();
    let mut from = 0;
    for to in 1..=units.len() {
        let hard = match (units.get(to - 1), units.get(to)) {
            (Some(prev), Some(next)) => {
                sentence_of(prev.word) != sentence_of(next.word)
                    || next.word.start.get().saturating_sub(prev.word.end.get()) >= CHUNK_GAP_MS
                    || to - from >= MAX_SEGMENT_WORDS
            }
            _ => true,
        };
        if !hard {
            continue;
        }
        for (start, end, second) in search(&units, from, to, &limits) {
            let placed = (start..end).filter_map(|i| {
                let unit = units.get(i)?;
                let line = u32::from(second.is_some_and(|k| i >= k));
                Some(Placed {
                    word: unit.word,
                    text: unit.text.clone(),
                    line,
                })
            });
            groups.push(placed.collect());
        }
        from = to;
    }

    let font = theme.font(style.font);
    let space: f32 = shape(" ", font, style.size, fonts)?
        .iter()
        .map(|run| run.width)
        .sum();
    let mut chunks: Vec<Chunk> = Vec::with_capacity(groups.len());
    for group in &groups {
        let (Some(first), Some(last)) = (group.first(), group.last()) else {
            continue;
        };
        let lines = lines(group, space, style, block, (font, fonts))?;
        // Shown from a little before its first word to a little after its
        // last, and never while the chunk before it is still shown. The
        // chunk before stays at least until its own last word has ended.
        let mut start = first.word.start.get().saturating_sub(CHUNK_LEAD_MS);
        if let Some(before) = chunks.last_mut() {
            let last_word_end = before.words.last().map_or(0, |word| word.end.get());
            let hand_over = start.max(last_word_end).min(before.show.end.get());
            before.show.end = OutMs::new(hand_over.max(before.show.start.get()));
            start = start.max(before.show.end.get());
        }
        let end = last.word.end.get().saturating_add(CHUNK_TAIL_MS);
        chunks.push(Chunk {
            words: group.iter().map(|placed| placed.word.clone()).collect(),
            show: OutSpan {
                start: OutMs::new(start),
                end: OutMs::new(end.max(start)),
            },
            lines,
        });
    }
    Ok(chunks)
}

/// The lines of one chunk: centred in the caption block, one under the other.
fn lines(
    group: &[Placed<'_>],
    space: f32,
    style: &StyleSpec,
    block: Rect,
    (font, fonts): (FontKey, &FontSet),
) -> Result<Vec<Line>, SceneError> {
    let mut lines: Vec<Line> = Vec::new();
    let (mut ascent, mut descent) = (0.0f32, 0.0f32);
    for (index, placed) in group.iter().enumerate() {
        let runs = shape(&placed.text, font, style.size, fonts)?;
        for run in &runs {
            ascent = ascent.max(run.ascent);
            descent = descent.max(run.descent);
        }
        while lines.len() <= placed.line as usize {
            lines.push(Line {
                words: Vec::new(),
                x: 0.0,
                baseline: 0.0,
                width: 0.0,
                scale: 1.0,
            });
        }
        let Some(line) = lines.get_mut(placed.line as usize) else {
            continue;
        };
        let x = if line.words.is_empty() {
            0.0
        } else {
            line.width + space
        };
        line.width = x + runs.iter().map(|run| run.width).sum::<f32>();
        line.words.push(LineWord {
            word: index,
            x,
            runs,
        });
    }

    let height = ascent + descent;
    let top = block.y + (block.h - height * lines.len() as f32) / 2.0;
    // What is drawn around the outermost glyphs stays in the block as well.
    let room = block.w - style.decoration_width();
    for (index, line) in lines.iter_mut().enumerate() {
        line.scale = if line.width > room {
            room / line.width
        } else {
            1.0
        };
        line.x = block.x + (block.w - line.width * line.scale) / 2.0;
        line.baseline = top + height * index as f32 + ascent;
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use offcut_types::StyleId;

    use offcut_types::{CaptionPlacement, DurMs};

    use super::*;
    use crate::readability::hold_for_reading;
    use crate::safe_area::{ZoneId, zone};
    use crate::styles::spec;
    use crate::testing::{transcript, word};

    fn chunks_of(timed: &[(&str, u32, u32)]) -> Vec<Chunk> {
        let spoken: Vec<_> = timed.iter().map(|w| word(w.0, w.1, w.2)).collect();
        let transcript = transcript(&spoken);
        let map = TimeMap::identity(DurMs::new(10_000));
        let words = words(&transcript, &BTreeMap::new(), &map);
        let (theme, fonts) = (Theme::default_for(StyleId::Clean), FontSet::embedded());
        chunk(
            &words,
            &transcript.sentences,
            &spec(StyleId::Clean),
            &theme,
            &fonts,
            zone(ZoneId::Caption, CaptionPlacement::bottom()),
        )
        .unwrap()
    }

    #[test]
    fn twelve_words_in_one_sentence_make_chunks_of_at_most_5_words_and_2_lines_of_18_characters() {
        let texts = [
            "we",
            "spent",
            "three",
            "long",
            "months",
            "building",
            "something",
            "nobody",
            "wanted",
            "to",
            "use",
            "today.",
        ];
        let timed: Vec<(&str, u32, u32)> = (0u32..)
            .zip(texts)
            .map(|(i, text)| (text, i * 300, i * 300 + 250))
            .collect();
        let chunks = chunks_of(&timed);

        assert!(chunks.len() >= 3);
        for chunk in &chunks {
            assert!(chunk.words.len() <= 5, "{:?}", chunk.words);
            assert!(chunk.lines.len() <= 2);
            for line in &chunk.lines {
                let text: Vec<&str> = line
                    .words
                    .iter()
                    .map(|w| chunk.words[w.word].text.as_str())
                    .collect();
                assert!(text.join(" ").chars().count() <= 18, "{text:?}");
                let caption_zone = zone(ZoneId::Caption, CaptionPlacement::bottom());
                assert!(line.x >= caption_zone.x);
                assert!(line.x + line.width <= caption_zone.x + caption_zone.w);
            }
        }
        // The words keep their order. A number is shown as its display, also
        // one that was spoken as a word (TS §17.2).
        let shown: Vec<&str> = chunks
            .iter()
            .flat_map(|c| c.words.iter().map(|w| w.text.as_str()))
            .collect();
        let mut expected = texts;
        expected[2] = "3";
        assert_eq!(shown, expected);
        // The chunks never overlap in time.
        for pair in chunks.windows(2) {
            assert!(pair[0].show.start < pair[0].show.end);
            assert!(pair[0].show.end <= pair[1].show.start);
        }
    }

    #[test]
    fn words_a_pause_apart_are_two_chunks_and_nothing_shows_in_the_pause() {
        let chunks = chunks_of(&[("hello", 1_000, 1_300), ("again", 5_000, 5_300)]);
        assert_eq!(chunks.len(), 2);
        // 80 ms before the first word, 120 ms after the last; no time is moved.
        let shows: Vec<(u32, u32)> = chunks
            .iter()
            .map(|c| (c.show.start.get(), c.show.end.get()))
            .collect();
        assert_eq!(shows, [(920, 1_420), (4_920, 5_420)]);
        assert!(chunks.iter().all(|c| !c.visible_at(OutMs::new(3_000))));
        assert!(chunks[0].visible_at(OutMs::new(920)));
        assert!(!chunks[0].visible_at(OutMs::new(1_420)));

        // 349 ms is not a pause; 350 ms is.
        assert_eq!(chunks_of(&[("one", 0, 300), ("two", 649, 900)]).len(), 1);
        assert_eq!(chunks_of(&[("one", 0, 300), ("two", 650, 900)]).len(), 2);
        // A full stop ends the chunk. The next one waits for its last word to end.
        let split = chunks_of(&[("Done.", 0, 300), ("Next", 320, 600)]);
        assert_eq!(split.len(), 2);
        assert_eq!(split[0].show.end.get(), 300);
        assert_eq!(split[1].show.start.get(), 300);
    }

    #[test]
    fn inside_a_chunk_exactly_one_glyph_run_is_the_spoken_word_on_the_pill() {
        // D-170: Clean's spoken word is in ink on the pill, the others white
        // with a 5 lp outline over their shadow.
        let style = spec(StyleId::Clean);
        let theme = Theme::default_for(StyleId::Clean);
        let chunks = chunks_of(&[
            ("we", 1_000, 1_200),
            ("made", 1_250, 1_500),
            ("it", 1_600, 1_800),
        ]);
        assert_eq!(chunks.len(), 1);
        let active_at = |t: u32| {
            let mut out = Vec::new();
            draw(&chunks[0], OutMs::new(t), &style, &theme, &mut out);
            let words: Vec<(String, Option<f32>, bool)> = out
                .into_iter()
                .filter_map(|cmd| match cmd {
                    DrawCmd::GlyphRun {
                        fill, text, stroke, ..
                    } if fill != theme.shadow => {
                        Some((text, stroke.map(|s| s.width), fill == theme.on_accent))
                    }
                    DrawCmd::GlyphRun { .. }
                    | DrawCmd::FillRect { .. }
                    | DrawCmd::PushLayer { .. }
                    | DrawCmd::PopLayer => None,
                    DrawCmd::FillPath { .. } | DrawCmd::Video { .. } | DrawCmd::Image { .. } => {
                        panic!("a caption draws no path, video or image")
                    }
                })
                .collect();
            // While the pill slides in (90 ms), the spoken word is also
            // drawn outlined under it.
            let texts: std::collections::BTreeSet<&str> =
                words.iter().map(|w| w.0.as_str()).collect();
            assert_eq!(texts, ["it", "made", "we"].into());
            words
                .into_iter()
                .filter_map(|(text, stroke, on_pill)| {
                    assert_eq!(stroke, (!on_pill).then_some(5.0), "{text}");
                    on_pill.then_some(text)
                })
                .collect::<Vec<String>>()
        };
        assert_eq!(active_at(1_250), ["made"]);
        assert_eq!(active_at(1_499), ["made"]);
        assert_eq!(active_at(1_700), ["it"]);
        // Between two words, and before the first, no word is active.
        assert!(active_at(1_520).is_empty());
        assert!(active_at(950).is_empty());
    }

    #[test]
    fn the_reading_floor_holds_a_short_chunk_into_the_silence_after_it() {
        // "Yes." alone is up 400 ms; the pause after it lets it stay 700.
        let mut held = chunks_of(&[
            ("Yes.", 1_000, 1_200),
            ("Then", 2_500, 2_800),
            ("we", 2_880, 3_000),
            ("left.", 3_080, 3_400),
        ]);
        let before = held.clone();
        assert_eq!(
            (before[0].show.start.get(), before[0].show.end.get()),
            (920, 1_320)
        );
        hold_for_reading(&mut held, OutMs::new(10_000));
        assert_eq!(
            (held[0].show.start.get(), held[0].show.end.get()),
            (920, 1_620)
        );
        // A chunk up 700 ms or more is not changed, and no word moves.
        assert_eq!(held[1].show, before[1].show);
        for (after, was) in held.iter().zip(&before) {
            assert_eq!(after.words, was.words);
            assert_eq!(after.lines, was.lines);
        }
    }

    #[test]
    fn the_reading_floor_stops_at_the_next_chunk_and_at_the_end_of_the_program() {
        let mut held = chunks_of(&[("Hi.", 1_000, 1_100), ("Okay.", 1_400, 1_600)]);
        assert_eq!(held[1].show.start.get(), 1_320);
        hold_for_reading(&mut held, OutMs::new(1_900));
        // Held to the next chunk's start, not to 920 + 700.
        assert_eq!(
            (held[0].show.start.get(), held[0].show.end.get()),
            (920, 1_320)
        );
        // The last chunk is held to the end of the program, not to 1,320 + 700.
        assert_eq!(
            (held[1].show.start.get(), held[1].show.end.get()),
            (1_320, 1_900)
        );
        // Never shortened: a program end before a chunk's own end leaves it.
        let mut late = chunks_of(&[("Hi.", 1_000, 1_100)]);
        hold_for_reading(&mut late, OutMs::new(1_000));
        assert_eq!(late[0].show.end.get(), 1_220);
    }

    #[test]
    fn a_caption_word_shows_a_number_when_made_from_one_or_read_as_one() {
        let spoken = [
            word("it", 0, 200),
            word("costs", 250, 500),
            word("three", 550, 800),
            word("dollars", 850, 1_100),
            word("in", 1_150, 1_300),
            word("2024.", 1_350, 1_700),
        ];
        let t = transcript(&spoken);
        let map = TimeMap::identity(DurMs::new(10_000));
        let shown = |edits: &BTreeMap<WordIdx, String>| -> Vec<(String, bool)> {
            let shown = words(&t, edits, &map);
            shown.into_iter().map(|w| (w.text, w.number)).collect()
        };
        // "three dollars" and "2024." are number spans, shown as their
        // displays.
        let plain = shown(&BTreeMap::new());
        let numbers: Vec<&str> = plain.iter().filter(|w| w.1).map(|w| w.0.as_str()).collect();
        assert_eq!(numbers, ["$3", "2,024"], "{plain:?}");
        // A word edited to "2024" is not a number: the captions would show
        // one as "2,024".
        let edits = BTreeMap::from([(WordIdx::new(4), "2024".to_owned())]);
        assert_eq!(shown(&edits)[3], ("2024".to_owned(), false));
        // A number with an edited word is shown as its words: an edit that
        // reads as a number shows one, one that does not is a word.
        let edits = BTreeMap::from([(WordIdx::new(2), "$500".to_owned())]);
        let edited = shown(&edits);
        assert_eq!(edited[2], ("$500".to_owned(), true));
        assert_eq!(edited[3], ("dollars".to_owned(), false));
        let edits = BTreeMap::from([(WordIdx::new(2), "five,".to_owned())]);
        assert_eq!(shown(&edits)[2], ("five,".to_owned(), false));
        assert!(shows_a_number("40%") && shows_a_number("3,000.") && !shows_a_number("x3"));
    }
}
