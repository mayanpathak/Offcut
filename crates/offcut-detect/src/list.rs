//! ListReveal candidates: a count with a list noun followed by its markers,
//! or three or more ordinal markers on their own (TS §17.4, D-105), with the
//! header and the items a list shows (D-107).

use std::ops::Range;

use offcut_text::{Token, TokenKind, parse_quantity};
use offcut_types::{
    EventKind, EventParams, ListItem, Prosody, SentenceIdx, Span, TimeMs, Transcript, Unit,
    WordIdx, WordRange,
};

use crate::config::{
    DetectorConfig, FILLERS, LIST_ITEM_COMMA_WORDS, LIST_MARKER_ONLY_MIN, LIST_NOUNS, NUMBER,
    ORDINAL_MARKERS,
};
use crate::number::{Candidate, Found, quantities};

/// A cardinal at the start of a sentence followed by one of these is a
/// marker: "1, sleep", "two: eat" (D-105).
const MARKER_MARKS: &[&str] = &[",", ".", ":"];
/// An item ends at one of these (D-107),
const ITEM_STOPS: &[&str] = &["!", ".", ":", ";", "?"];
/// and at a comma once it holds `LIST_ITEM_COMMA_WORDS` words.
const COMMA: &str = ",";

/// The word or two at the start of a sentence that say which place of a list
/// follows: "First", "number two", "2," (D-105). Also read by the exclusions
/// (X6).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Marker {
    pub ordinal: u8,
    pub words: WordRange,
    /// "first" to "seventh", "firstly" to "thirdly"; not a cardinal.
    pub ordinal_word: bool,
    /// The start of its first word.
    pub at: TimeMs,
    pub sentence: SentenceIdx,
}

/// A marker and the first token after it, where its item starts.
struct Placed {
    marker: Marker,
    item_from: usize,
}

/// The tokens of each sentence, as a range of `toks`.
pub(crate) fn sentence_tokens(toks: &[Token], t: &Transcript) -> Vec<Range<usize>> {
    t.sentences
        .iter()
        .map(|s| {
            toks.partition_point(|tk| tk.word < s.words.start)
                ..toks.partition_point(|tk| tk.word < s.words.end)
        })
        .collect()
}

/// Every marker of the text as edited, in order: at most one per sentence,
/// and only as its first word after any fillers. "It was the first time" has
/// none.
pub(crate) fn markers(toks: &[Token], t: &Transcript) -> Vec<Marker> {
    placed(toks, t).into_iter().map(|p| p.marker).collect()
}

fn placed(toks: &[Token], t: &Transcript) -> Vec<Placed> {
    let mut out = Vec::new();
    for (index, range) in (0u32..).zip(sentence_tokens(toks, t)) {
        let sentence = toks.get(range.clone()).unwrap_or(&[]);
        // Punctuation and fillers may come first: "So, first, …".
        let lead = sentence
            .iter()
            .take_while(|tk| match tk.kind {
                TokenKind::Punct => true,
                TokenKind::Word => listed(FILLERS, &tk.text),
                TokenKind::Digits => false,
            })
            .count();
        let rest = sentence.get(lead..).unwrap_or(&[]);
        let Some(found) = marker_at(rest) else {
            continue;
        };
        let own = rest.get(..found.own).unwrap_or(&[]);
        let (Some(first), Some(last)) = (own.first(), own.last()) else {
            continue;
        };
        let Some(word) = t.words.get(first.word.get() as usize) else {
            continue;
        };
        out.push(Placed {
            marker: Marker {
                ordinal: found.ordinal,
                words: WordRange {
                    start: first.word,
                    end: after(last.word),
                },
                ordinal_word: found.ordinal_word,
                at: word.start_ms,
                sentence: SentenceIdx::new(index),
            },
            item_from: range.start + lead + found.used,
        });
    }
    out
}

/// A marker at the start of `rest`: its place, how many tokens are its words,
/// how many tokens it uses with its mark, and whether it is an ordinal word.
struct AtStart {
    ordinal: u8,
    own: usize,
    used: usize,
    ordinal_word: bool,
}

fn marker_at(rest: &[Token]) -> Option<AtStart> {
    let first = rest.first()?;
    if first.kind == TokenKind::Word {
        if let Ok(at) =
            ORDINAL_MARKERS.binary_search_by(|(word, _)| (*word).cmp(first.text.as_str()))
        {
            let (_, ordinal) = ORDINAL_MARKERS.get(at)?;
            return Some(AtStart {
                ordinal: *ordinal,
                own: 1,
                used: 1,
                ordinal_word: true,
            });
        }
        if first.text == NUMBER {
            return Some(AtStart {
                ordinal: place(rest.get(1..2)?)?,
                own: 2,
                used: 2,
                ordinal_word: false,
            });
        }
    }
    let ordinal = place(rest.get(..1)?)?;
    let mark = rest.get(1)?;
    let marked = mark.kind == TokenKind::Punct && MARKER_MARKS.contains(&mark.text.as_str());
    marked.then_some(AtStart {
        ordinal,
        own: 1,
        used: 2,
        ordinal_word: false,
    })
}

/// The place one token names when it is a whole number a list can have: from
/// 1 to the last place an ordinal marker names.
fn place(one: &[Token]) -> Option<u8> {
    let (used, quantity) = parse_quantity(one)?;
    let last = ORDINAL_MARKERS.iter().map(|(_, n)| *n).max()?;
    let value = quantity.value;
    let whole = value.fract() == 0.0 && (1.0..=f64::from(last)).contains(&value);
    (used == 1 && quantity.unit == Unit::None && whole).then_some(value as u8)
}

/// The lists of the text as edited, announced ones first.
pub fn find(toks: &[Token], t: &Transcript, _p: &Prosody, cfg: &DetectorConfig) -> Vec<Candidate> {
    let placed = placed(toks, t);
    // A marker belongs to at most one list.
    let mut claimed = vec![false; placed.len()];
    let mut out = Vec::new();

    for found in quantities(toks, t) {
        let Some((count, noun)) = announcer(toks, t, &found) else {
            continue;
        };
        let Some(start) = t.words.get(found.words.start.get() as usize) else {
            continue;
        };
        let wanted = (f64::from(cfg.list_count_min)..=f64::from(cfg.list_count_max))
            .contains(&count)
            .then_some(count as u8);
        // An announced count outside the range still takes the markers that
        // follow it: "eight reasons" then "First" to "Seventh" is not a list
        // of seven (TS §17.6: a partial list is worse than none).
        let limit = wanted.map_or(usize::MAX, usize::from);
        let from = placed.partition_point(|p| p.marker.sentence <= found.sentence);
        let run = run(&placed, &claimed, from, start.start_ms, limit, false, cfg);
        claim(&mut claimed, run.clone());
        let markers = placed.get(run).unwrap_or(&[]);
        let Some(count) = wanted.filter(|count| markers.len() == usize::from(*count)) else {
            continue;
        };
        let mut score = cfg.list_base + cfg.list_all_markers_bonus;
        if markers.iter().all(|p| p.marker.ordinal_word) {
            score += cfg.list_ordinal_bonus;
        }
        let header = format!("{count} {}", noun.to_uppercase());
        out.extend(candidate(
            toks,
            t,
            cfg,
            Some(&found),
            markers,
            score,
            header,
        ));
    }

    let mut at = 0;
    while let Some(first) = placed.get(at) {
        let starts = first.marker.ordinal == 1
            && first.marker.ordinal_word
            && !claimed.get(at).is_some_and(|c| *c);
        let run = if starts {
            let most = usize::from(cfg.list_count_max);
            run(&placed, &claimed, at, first.marker.at, most, true, cfg)
        } else {
            at..at
        };
        if run.len() < LIST_MARKER_ONLY_MIN {
            at += 1;
            continue;
        }
        at = run.end;
        let markers = placed.get(run).unwrap_or(&[]);
        let steady = markers.windows(2).all(|pair| {
            matches!(pair, [a, b] if b.marker.at.get().saturating_sub(a.marker.at.get()) <= cfg.list_marker_gap_ms)
        });
        let mut score = cfg.list_marker_only_base;
        if steady {
            score += cfg.list_marker_only_bonus;
        }
        out.extend(candidate(toks, t, cfg, None, markers, score, String::new()));
    }
    out
}

/// A count of one or more with no unit, right before a list noun of its
/// sentence: "three reasons", "3 tips".
fn announcer<'a>(toks: &'a [Token], t: &Transcript, found: &Found) -> Option<(f64, &'a str)> {
    let count = found.quantity.value;
    if found.quantity.unit != Unit::None || count < 1.0 || count.fract() != 0.0 {
        return None;
    }
    let noun = toks.get(found.tokens.end)?;
    let end = t.sentences.get(found.sentence.get() as usize)?.words.end;
    let named = noun.kind == TokenKind::Word && noun.word < end && listed(LIST_NOUNS, &noun.text);
    named.then_some((count, noun.text.as_str()))
}

/// The markers from `from` on that name the places 1, 2, 3, … one after the
/// other, none claimed by another list, each starting before `start` plus the
/// list window; at most `limit`. Markers are one per sentence, so each is in a
/// later sentence than the one before.
fn run(
    placed: &[Placed],
    claimed: &[bool],
    from: usize,
    start: TimeMs,
    limit: usize,
    ordinal_words: bool,
    cfg: &DetectorConfig,
) -> Range<usize> {
    let until = start.get().saturating_add(cfg.list_window_ms);
    let mut end = from;
    let mut place: u8 = 1;
    while end - from < limit
        && let Some(p) = placed.get(end)
    {
        let fits = claimed.get(end) == Some(&false)
            && p.marker.ordinal == place
            && p.marker.at.get() < until
            && (p.marker.ordinal_word || !ordinal_words);
        if !fits {
            break;
        }
        end += 1;
        place = place.saturating_add(1);
    }
    from..end
}

fn claim(claimed: &mut [bool], run: Range<usize>) {
    for taken in claimed.get_mut(run).unwrap_or(&mut []) {
        *taken = true;
    }
}

/// The candidate of a list: from the announcer's count (or the first marker)
/// to the last word of the last item.
fn candidate(
    toks: &[Token],
    t: &Transcript,
    cfg: &DetectorConfig,
    announcer: Option<&Found>,
    markers: &[Placed],
    score: f32,
    header: String,
) -> Option<Candidate> {
    let (first, last) = (markers.first()?, markers.last()?);
    let mut items = Vec::new();
    let mut end = last.marker.words.end;
    for p in markers {
        let (text, last_word) = item(toks, t, cfg, p);
        if let Some(word) = last_word {
            end = end.max(after(word));
        }
        items.push(ListItem {
            ordinal: p.marker.ordinal,
            text,
            at: p.marker.at,
        });
    }
    let start = announcer.map_or(first.marker.words.start, |found| found.words.start);
    let words = t.words.get(start.get() as usize..end.get() as usize)?;
    // A list is never more certain than the least certain of its words (TS §17.4).
    let heard = words
        .iter()
        .map(|word| word.confidence.get())
        .fold(1.0f32, f32::min);
    let mut parts: Vec<WordRange> = announcer.iter().map(|found| found.words).collect();
    parts.extend(markers.iter().map(|p| p.marker.words));
    let sentence = announcer.map_or(first.marker.sentence, |found| found.sentence);
    Some(Candidate {
        kind: EventKind::ListReveal,
        anchors: WordRange { start, end },
        span: Span::new(words.first()?.start_ms, words.last()?.end_ms)?,
        score: score.min(1.0) * heard,
        params: EventParams::ListReveal {
            count: u8::try_from(markers.len()).ok()?,
            header,
            items,
        },
        parts,
        sentence,
    })
}

/// The text of a marker's item and its last word (D-107): up to
/// `list_item_max_words` words of its sentence after the marker, leading
/// fillers dropped, ending at `. ? ! ; :` or at a comma once two words are
/// taken. A number in digits is a word of the item: "we saved $10,000".
fn item(
    toks: &[Token],
    t: &Transcript,
    cfg: &DetectorConfig,
    p: &Placed,
) -> (String, Option<WordIdx>) {
    let end = t
        .sentences
        .get(p.marker.sentence.get() as usize)
        .map_or(p.marker.words.end, |s| s.words.end);
    let mut taken: Vec<&Token> = Vec::new();
    for token in toks
        .get(p.item_from..)
        .unwrap_or(&[])
        .iter()
        .take_while(|token| token.word < end)
    {
        if taken.len() >= cfg.list_item_max_words {
            break;
        }
        let text = token.text.as_str();
        match token.kind {
            TokenKind::Punct
                if ITEM_STOPS.contains(&text)
                    || (text == COMMA && taken.len() >= LIST_ITEM_COMMA_WORDS) =>
            {
                break;
            }
            TokenKind::Punct => {}
            TokenKind::Word if taken.is_empty() && listed(FILLERS, text) => {}
            TokenKind::Word | TokenKind::Digits => taken.push(token),
        }
    }
    let text: Vec<&str> = taken.iter().map(|token| token.text.as_str()).collect();
    (text.join(" "), taken.last().map(|token| token.word))
}

fn listed(words: &[&str], word: &str) -> bool {
    words.binary_search(&word).is_ok()
}

fn after(word: WordIdx) -> WordIdx {
    WordIdx::new(word.get().saturating_add(1))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use offcut_text::{RawWord, normalize_transcript, tokenize};
    use offcut_types::WordProsody;

    use super::*;

    /// Each piece's words start at its time, 400 ms apart, 300 ms long.
    fn timed(pieces: &[(&str, u32)]) -> Transcript {
        let raw = pieces
            .iter()
            .flat_map(|(text, from)| {
                (0u32..)
                    .zip(text.split_whitespace())
                    .map(move |(i, word)| RawWord {
                        text: word.to_owned(),
                        start_ms: from + i * 400,
                        end_ms: from + i * 400 + 300,
                        confidence: 1.0,
                    })
            })
            .collect();
        normalize_transcript(raw, "test")
    }

    fn lists_of(t: &Transcript) -> Vec<Candidate> {
        let flat = Prosody {
            per_word: vec![
                WordProsody {
                    energy_z: 0.0,
                    pitch_z: 0.0,
                };
                t.words.len()
            ],
        };
        let toks = tokenize(&t.words, &BTreeMap::new());
        find(&toks, t, &flat, &DetectorConfig::default())
    }

    fn lists(text: &str) -> Vec<Candidate> {
        lists_of(&timed(&[(text, 0)]))
    }

    fn markers_of(text: &str) -> Vec<Marker> {
        let t = timed(&[(text, 0)]);
        markers(&tokenize(&t.words, &BTreeMap::new()), &t)
    }

    fn shown(c: &Candidate) -> (u8, &str, Vec<&str>) {
        match &c.params {
            EventParams::ListReveal {
                count,
                header,
                items,
            } => (
                *count,
                header.as_str(),
                items.iter().map(|item| item.text.as_str()).collect(),
            ),
            EventParams::NumberReveal { .. }
            | EventParams::FromTo { .. }
            | EventParams::KeywordPop { .. } => panic!("{:?}", c.params),
        }
    }

    fn range(start: u32, end: u32) -> WordRange {
        WordRange {
            start: WordIdx::new(start),
            end: WordIdx::new(end),
        }
    }

    const SCRIPT_A: &str = "There are three reasons it worked. First the captions write themselves. \
         Second the numbers animate automatically. Third everything runs on my laptop.";

    #[test]
    fn script_a_is_one_announced_list_of_three_reasons() {
        let found = lists(SCRIPT_A);
        assert_eq!(found.len(), 1);
        let list = &found[0];
        assert_eq!(list.kind, EventKind::ListReveal);
        assert_eq!(list.score, 1.0);
        assert_eq!(
            shown(list),
            (
                3,
                "3 REASONS",
                vec![
                    "the captions write themselves",
                    "the numbers animate automatically",
                    "everything runs on my laptop",
                ]
            )
        );
        // From "three" to "laptop."; the parts are the count and the three markers.
        assert_eq!(list.anchors, range(2, 22));
        assert_eq!(
            list.parts,
            [range(2, 3), range(6, 7), range(11, 12), range(16, 17)]
        );
        assert_eq!((list.span.start.get(), list.span.end.get()), (800, 8_700));
        match &list.params {
            EventParams::ListReveal { items, .. } => {
                let at: Vec<(u8, u32)> = items.iter().map(|i| (i.ordinal, i.at.get())).collect();
                assert_eq!(at, [(1, 2_400), (2, 4_400), (3, 6_400)]);
            }
            EventParams::NumberReveal { .. }
            | EventParams::FromTo { .. }
            | EventParams::KeywordPop { .. } => panic!("{:?}", list.params),
        }
    }

    #[test]
    fn three_reasons_without_a_third_marker_in_time_is_no_list() {
        let two = "There are three reasons it worked. First the captions write themselves. \
                   Second the numbers animate automatically.";
        assert_eq!(lists(two), []);
        // The third starts 45 s after "three": outside the window.
        let late = timed(&[(two, 0), ("Third everything runs on my laptop.", 45_800)]);
        assert_eq!(lists_of(&late), []);
        let in_time = timed(&[(two, 0), ("Third everything runs on my laptop.", 45_700)]);
        assert_eq!(lists_of(&in_time).len(), 1);
    }

    #[test]
    fn three_ordinal_markers_alone_are_a_list_without_a_header() {
        let found = lists("First, open the app. Second, drop a clip. Third, read the result.");
        assert_eq!(found.len(), 1);
        assert!((found[0].score - 0.85).abs() < 1e-6);
        assert_eq!(
            shown(&found[0]),
            (
                3,
                "",
                vec!["open the app", "drop a clip", "read the result"]
            )
        );
        assert_eq!(found[0].parts, [range(0, 1), range(4, 5), range(8, 9)]);
        // A gap of more than 15 s between two markers loses the bonus.
        let slow = timed(&[
            ("First, open the app. Second, drop a clip.", 0),
            ("Third, read the result.", 22_000),
        ]);
        let found = lists_of(&slow);
        assert_eq!(found.len(), 1);
        assert!((found[0].score - 0.65).abs() < 1e-6);
        // Two markers alone, or markers that are not ordinal words, are none.
        assert_eq!(lists("First, open the app. Second, drop a clip."), []);
        assert_eq!(
            lists("1, open the app. 2, drop a clip. 3, read the result."),
            []
        );
    }

    #[test]
    fn a_marker_is_only_the_first_word_of_a_sentence() {
        assert_eq!(markers_of("It was the first time I tried."), []);
        let found = markers_of("First, open it. It was the first time I tried.");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].ordinal, found[0].words), (1, range(0, 1)));
        // After fillers and punctuation: "So, second, …".
        let found = markers_of("Okay. So, second, we wait.");
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].ordinal, found[0].words), (2, range(2, 3)));
        assert_eq!(found[0].sentence, SentenceIdx::new(1));
    }

    #[test]
    fn digits_and_number_words_are_markers_of_their_own_kind() {
        let found = markers_of("Number two, eat. 3: ship. Seven. eight, rest. 3 apples.");
        let read: Vec<(u8, WordRange, bool)> = found
            .iter()
            .map(|m| (m.ordinal, m.words, m.ordinal_word))
            .collect();
        // "Seven." is its own sentence and a marker; 8 is no place a list has;
        // "3 apples" has no mark after the 3.
        assert_eq!(
            read,
            [
                (2, range(0, 2), false),
                (3, range(3, 4), false),
                (7, range(5, 6), false),
            ]
        );
    }

    #[test]
    fn three_tips_with_digit_markers_scores_without_the_ordinal_bonus() {
        let found = lists("I have 3 tips. 1, sleep. 2, eat. 3, ship.");
        assert_eq!(found.len(), 1);
        assert!((found[0].score - 0.90).abs() < 1e-6);
        assert_eq!(
            shown(&found[0]),
            (3, "3 TIPS", vec!["sleep", "eat", "ship"])
        );
        assert_eq!(found[0].anchors, range(2, 10));
    }

    #[test]
    fn eight_reasons_is_no_list_and_keeps_its_markers_from_another() {
        let markers = "First, we start. Second, we plan. Third, we build. Fourth, we test. \
                       Fifth, we ship. Sixth, we learn. Seventh, we rest. Eighth, we repeat.";
        assert_eq!(lists(&format!("There are eight reasons. {markers}")), []);
        // Without the announcer the same seven ordinal markers are a list.
        let alone = lists(markers);
        assert_eq!(alone.len(), 1);
        assert_eq!(shown(&alone[0]).0, 7);
    }

    #[test]
    fn an_item_drops_fillers_and_stops_at_a_comma_only_after_two_words() {
        let found = lists(
            "First, uh, we, then open the app now. Second, drop it, and wait. \
             Third, so, we saved $10,000 there today.",
        );
        assert_eq!(found.len(), 1);
        assert_eq!(
            shown(&found[0]).2,
            [
                "we then open the app",
                "drop it",
                "we saved $10,000 there today"
            ]
        );
    }

    #[test]
    fn an_announced_list_keeps_its_markers_from_a_second_one() {
        let found =
            lists("Here are two tips. First, sleep. Second, eat. Third, ship. Then the next part.");
        // Two tips, then a marker 3 that starts no list of its own.
        assert_eq!(found.len(), 1);
        assert_eq!(shown(&found[0]), (2, "2 TIPS", vec!["sleep", "eat"]));
    }
}
