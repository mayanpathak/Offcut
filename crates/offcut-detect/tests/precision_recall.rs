//! The precision gate (v3implementation §19.12, §20.3): the detector against
//! the labeled set of `fixtures/labeled/`, written before its rules (D-87).
//! Overall precision at least `DETECTOR_PRECISION_TARGET` over at least 30
//! files; every prediction at or above its kind's threshold (INV-6). The
//! table, the sweep and the false positives are printed on every run.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use offcut_detect::{DetectorConfig, detect};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{
    DETECTOR_PRECISION_TARGET, DetectedEvent, EventKind, EventParams, Prosody, Quantity,
    Transcript, Unit, WordIdx, WordProsody, WordRange,
};
use serde::Deserialize;

// --- The file format (fixtures/labeled/SCHEMA.md). ---------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabeledFile {
    schema: u32,
    id: String,
    source: Source,
    #[serde(rename = "author")]
    _author: Author,
    #[serde(rename = "frozen")]
    _frozen: String,
    sentences: Option<Vec<String>>,
    words: Option<Vec<AsrWord>>,
    prosody: Option<Prosody>,
    expected: Vec<Expected>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    edits: BTreeMap<String, String>,
    #[serde(rename = "notes")]
    _notes: String,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Source {
    Written,
    Asr,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Author {
    Agent,
    Founder,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AsrWord {
    text: String,
    #[serde(rename = "startMs")]
    start_ms: u32,
    #[serde(rename = "endMs")]
    end_ms: u32,
    confidence: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    kind: EventKind,
    say: String,
    occurrence: usize,
    value: Option<f64>,
    unit: Option<String>,
    from: Option<Amount>,
    to: Option<Amount>,
    count: Option<u32>,
    label: Option<String>,
    header: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Amount {
    value: f64,
    unit: String,
}

/// SCHEMA's unit strings.
fn unit(text: &str, file: &str) -> Unit {
    match text {
        "none" => Unit::None,
        "usd" => Unit::Usd,
        "eur" => Unit::Eur,
        "gbp" => Unit::Gbp,
        "inr" => Unit::Inr,
        "percent" => Unit::Percent,
        "times" => Unit::Times,
        "milliseconds" => Unit::Milliseconds,
        "seconds" => Unit::Seconds,
        "minutes" => Unit::Minutes,
        "hours" => Unit::Hours,
        "days" => Unit::Days,
        "weeks" => Unit::Weeks,
        "months" => Unit::Months,
        "years" => Unit::Years,
        "bytes" => Unit::Bytes,
        "kilobytes" => Unit::Kilobytes,
        "megabytes" => Unit::Megabytes,
        "gigabytes" => Unit::Gigabytes,
        other => match other.strip_prefix("count:") {
            Some(noun) if !noun.is_empty() => Unit::Count {
                noun: noun.to_owned(),
            },
            Some(_) | None => panic!("{file}: unknown unit {other:?}"),
        },
    }
}

// --- Timing synthesis (§17.5), the same rule as rules_table.rs. ------------

/// A word lasts `clamp(120 + 45 x letters, 150, 700)` ms; 80 ms between the
/// words of a sentence, 400 ms after a word ending in `.` `?` `!`; the first
/// word starts at 500 ms. `*word*` is emphasized: energy_z and pitch_z 2.0
/// (§23 item 27), every other word 0.0. Confidence 1.0 (TE-2).
fn synthesize(sentences: &[String]) -> (Vec<RawWord>, Prosody) {
    let mut raw = Vec::new();
    let mut per_word = Vec::new();
    let mut at = 500u32;
    for text in sentences {
        for piece in text.split_whitespace() {
            let stressed = piece.starts_with('*')
                && piece.trim_end_matches(['.', ',', '?', '!']).ends_with('*');
            let word = piece.replace('*', "");
            let letters = word.chars().filter(|c| c.is_alphanumeric()).count() as u32;
            let length = (120 + 45 * letters).clamp(150, 700);
            raw.push(RawWord {
                text: word.clone(),
                start_ms: at,
                end_ms: at + length,
                confidence: 1.0,
            });
            let z = if stressed { 2.0 } else { 0.0 };
            per_word.push(WordProsody {
                energy_z: z,
                pitch_z: z,
            });
            at += length
                + if word.ends_with(['.', '?', '!']) {
                    400
                } else {
                    80
                };
        }
    }
    (raw, Prosody { per_word })
}

// --- A file ready to detect. ------------------------------------------------

/// What a person expects, at the words `say` names.
struct Want {
    kind: EventKind,
    anchors: WordRange,
    say: String,
    values: Values,
    label: Option<String>,
    header: Option<String>,
}

/// The values a true positive must equal (TS §17.6): a NumberReveal's and
/// both ends of a FromTo; none for a ListReveal or a KeywordPop.
#[derive(Debug, PartialEq)]
enum Values {
    Number(f64, Unit),
    FromTo((f64, Unit), (f64, Unit)),
    None,
}

struct Case {
    id: String,
    written: bool,
    t: Transcript,
    p: Prosody,
    edits: BTreeMap<WordIdx, String>,
    wants: Vec<Want>,
    tags: Vec<String>,
}

/// SCHEMA's `say` normalisation: lower-case; letters, digits, `$` and `%`
/// kept; every other character a space; spaces collapsed.
fn normalized(text: &str) -> Vec<String> {
    let kept: String = text
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '$' || c == '%' {
                c
            } else {
                ' '
            }
        })
        .collect();
    kept.split_whitespace().map(str::to_owned).collect()
}

/// The words of the `occurrence`-th run of the transcript, as written, that
/// reads `say`: it starts on a word's first piece and ends on a word's last.
fn find(t: &Transcript, say: &str, occurrence: usize, file: &str) -> WordRange {
    let want = normalized(say);
    // Each piece of each word, with its word and whether it starts or ends it.
    let mut pieces: Vec<(String, u32, bool, bool)> = Vec::new();
    for (index, word) in (0u32..).zip(&t.words) {
        let parts = normalized(&word.text);
        let last = parts.len().saturating_sub(1);
        for (i, part) in parts.into_iter().enumerate() {
            pieces.push((part, index, i == 0, i == last));
        }
    }
    let mut found = Vec::new();
    if !want.is_empty() {
        for start in 0..pieces.len() {
            let Some(run) = pieces.get(start..start + want.len()) else {
                break;
            };
            let reads = run.iter().zip(&want).all(|(piece, w)| &piece.0 == w);
            if reads && run[0].2 && run[run.len() - 1].3 {
                found.push(WordRange {
                    start: WordIdx::new(run[0].1),
                    end: WordIdx::new(run[run.len() - 1].1 + 1),
                });
            }
        }
    }
    *found.get(occurrence.wrapping_sub(1)).unwrap_or_else(|| {
        panic!(
            "{file}: \"{say}\" occurrence {occurrence} is not in the transcript ({} found)",
            found.len()
        )
    })
}

fn want(t: &Transcript, e: Expected, file: &str) -> Want {
    let anchors = find(t, &e.say, e.occurrence, file);
    let amount = |a: &Amount| (a.value, unit(&a.unit, file));
    let values = match e.kind {
        EventKind::NumberReveal => Values::Number(
            e.value
                .unwrap_or_else(|| panic!("{file}: \"{}\" has no value", e.say)),
            unit(
                e.unit
                    .as_deref()
                    .unwrap_or_else(|| panic!("{file}: \"{}\" has no unit", e.say)),
                file,
            ),
        ),
        EventKind::FromTo => Values::FromTo(
            amount(
                e.from
                    .as_ref()
                    .unwrap_or_else(|| panic!("{file}: \"{}\" has no from", e.say)),
            ),
            amount(
                e.to.as_ref()
                    .unwrap_or_else(|| panic!("{file}: \"{}\" has no to", e.say)),
            ),
        ),
        EventKind::ListReveal => {
            assert!(e.count.is_some(), "{file}: \"{}\" has no count", e.say);
            Values::None
        }
        EventKind::KeywordPop => Values::None,
    };
    Want {
        kind: e.kind,
        anchors,
        say: e.say,
        values,
        label: e.label,
        header: e.header,
    }
}

fn labeled_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/labeled")
}

/// Every file with data, by name, and the ids of those still without. A file
/// that does not parse fails here with its name.
fn cases() -> (Vec<Case>, Vec<String>) {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(labeled_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    let mut cases = Vec::new();
    let mut skipped = Vec::new();
    for path in &paths {
        let name = stem(path);
        let text = std::fs::read_to_string(path).unwrap();
        let file: LabeledFile = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("{name}: does not parse: {error}"));
        assert_eq!(file.schema, 1, "{name}: schema");
        assert_eq!(file.id, name, "{name}: id");
        let (raw, p) = match (&file.source, file.sentences, file.words) {
            (Source::Written, Some(sentences), None) => synthesize(&sentences),
            (Source::Asr, None, Some(words)) => {
                let raw = words
                    .into_iter()
                    .map(|w| RawWord {
                        text: w.text,
                        start_ms: w.start_ms,
                        end_ms: w.end_ms,
                        confidence: w.confidence,
                    })
                    .collect();
                let p = file
                    .prosody
                    .unwrap_or_else(|| panic!("{name}: words without prosody"));
                (raw, p)
            }
            (Source::Asr, None, None) => {
                skipped.push(name);
                continue;
            }
            (Source::Written, _, _) | (Source::Asr, _, _) => {
                panic!("{name}: sentences and words do not match its source")
            }
        };
        let t = normalize_transcript(raw, "labeled");
        let edits = file
            .edits
            .into_iter()
            .map(|(word, text)| {
                let word: u32 = word
                    .parse()
                    .unwrap_or_else(|_| panic!("{name}: edit key {word:?}"));
                (WordIdx::new(word), text)
            })
            .collect();
        let wants = file
            .expected
            .into_iter()
            .map(|e| want(&t, e, &name))
            .collect();
        cases.push(Case {
            id: name,
            written: file.source == Source::Written,
            t,
            p,
            edits,
            wants,
            tags: file.tags,
        });
    }
    (cases, skipped)
}

fn stem(path: &Path) -> String {
    path.file_stem().unwrap().to_string_lossy().into_owned()
}

// --- Coverage (§17.5). ------------------------------------------------------

/// The coverage the set must have, checked before anything is measured; a
/// shortfall names what is missing.
fn check_coverage(cases: &[Case]) {
    let mut missing = Vec::new();
    let mut need = |what: &str, have: usize, least: usize| {
        if have < least {
            missing.push(format!("{what}: {have}, at least {least}"));
        }
    };
    need("files read", cases.len(), 30);
    for kind in KINDS {
        let have = cases
            .iter()
            .flat_map(|c| &c.wants)
            .filter(|w| w.kind == kind)
            .count();
        need(&format!("expected {}", name(kind)), have, 8);
    }
    need(
        "files with no expected event",
        cases.iter().filter(|c| c.wants.is_empty()).count(),
        6,
    );
    let tagged = |tag: &str| {
        cases
            .iter()
            .flat_map(|c| &c.tags)
            .filter(|t| t.as_str() == tag)
            .count()
    };
    for (tag, least) in [
        ("X1", 2),
        ("X2", 2),
        ("X3", 2),
        ("X4", 2),
        ("X5", 2),
        ("years_from_to", 1),
        ("partial_list", 1),
        ("announcer", 5),
        ("marker_only", 3),
        ("from_frame", 4),
        ("change_verb", 4),
        ("no_change_pair", 2),
    ] {
        need(&format!("tag {tag}"), tagged(tag), least);
    }
    // An edit that changes a number, and a hidden word: tagged, and in `edits`.
    let edited = |tag: &str, hides: bool| {
        cases
            .iter()
            .filter(|c| c.tags.iter().any(|t| t == tag))
            .filter(|c| c.edits.values().any(|text| text.is_empty() == hides))
            .count()
    };
    need(
        "tag edit_number with a number edit",
        edited("edit_number", false),
        1,
    );
    need(
        "tag hidden_word with a hidden word",
        edited("hidden_word", true),
        1,
    );
    assert!(
        missing.is_empty(),
        "the labeled set lacks coverage (v3implementation §17.5):\n{}",
        missing.join("\n")
    );
}

// --- Matching (TS §17.6). ---------------------------------------------------

const KINDS: [EventKind; 4] = [
    EventKind::NumberReveal,
    EventKind::ListReveal,
    EventKind::FromTo,
    EventKind::KeywordPop,
];

fn name(kind: EventKind) -> &'static str {
    match kind {
        EventKind::NumberReveal => "number_reveal",
        EventKind::ListReveal => "list_reveal",
        EventKind::FromTo => "from_to",
        EventKind::KeywordPop => "keyword_pop",
    }
}

fn iou(a: WordRange, b: WordRange) -> f64 {
    let (a0, a1, b0, b1) = (a.start.get(), a.end.get(), b.start.get(), b.end.get());
    let inter = a1.min(b1).saturating_sub(a0.max(b0));
    let union = a1.max(b1) - a0.min(b0);
    if union == 0 {
        0.0
    } else {
        f64::from(inter) / f64::from(union)
    }
}

fn values(e: &DetectedEvent) -> Values {
    let q = |q: &Quantity| (q.value, q.unit.clone());
    match &e.params {
        EventParams::NumberReveal { value, .. } => {
            let (v, u) = q(value);
            Values::Number(v, u)
        }
        EventParams::FromTo { from, to, .. } => Values::FromTo(q(from), q(to)),
        EventParams::ListReveal { .. } | EventParams::KeywordPop { .. } => Values::None,
    }
}

#[derive(Clone, Copy, Default)]
struct Tally {
    tp: usize,
    fp: usize,
    fn_: usize,
}

impl Tally {
    fn add(&mut self, other: Self) {
        self.tp += other.tp;
        self.fp += other.fp;
        self.fn_ += other.fn_;
    }

    fn precision(self) -> Option<f64> {
        let found = self.tp + self.fp;
        (found > 0).then(|| self.tp as f64 / found as f64)
    }

    fn recall(self) -> Option<f64> {
        let wanted = self.tp + self.fn_;
        (wanted > 0).then(|| self.tp as f64 / wanted as f64)
    }
}

fn ratio(r: Option<f64>) -> String {
    r.map_or_else(|| "-".to_owned(), |r| format!("{r:.2}"))
}

/// One file's result for one kind: the tally, the false positives, the
/// misses, and each matched pair.
struct Matched<'a> {
    tally: Tally,
    false_positives: Vec<&'a DetectedEvent>,
    missed: Vec<&'a Want>,
    pairs: Vec<(&'a DetectedEvent, &'a Want)>,
}

/// Greedy, in spoken order: each prediction takes the unmatched expectation
/// of its kind with the highest IoU, at least 0.5, and equal values.
fn match_kind<'a>(found: &'a [DetectedEvent], wants: &'a [Want], kind: EventKind) -> Matched<'a> {
    let wants: Vec<&Want> = wants.iter().filter(|w| w.kind == kind).collect();
    let mut taken = vec![false; wants.len()];
    let mut out = Matched {
        tally: Tally::default(),
        false_positives: Vec::new(),
        missed: Vec::new(),
        pairs: Vec::new(),
    };
    for e in found.iter().filter(|e| e.kind == kind) {
        let mine = values(e);
        let best = wants
            .iter()
            .enumerate()
            .filter(|(i, w)| !taken[*i] && w.values == mine)
            .map(|(i, w)| (i, iou(e.anchors, w.anchors)))
            .filter(|(_, overlap)| *overlap >= 0.5)
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)));
        match best {
            Some((i, _)) => {
                taken[i] = true;
                out.tally.tp += 1;
                out.pairs.push((e, wants[i]));
            }
            None => {
                out.tally.fp += 1;
                out.false_positives.push(e);
            }
        }
    }
    for (i, w) in wants.iter().enumerate() {
        if !taken[i] {
            out.tally.fn_ += 1;
            out.missed.push(w);
        }
    }
    out
}

/// What a person sees of an event of a written file: its words. A file of
/// a recording prints only the word numbers (D-112).
fn said(case: &Case, anchors: WordRange) -> String {
    if !case.written {
        return format!("words {}-{}", anchors.start.get(), anchors.end.get() - 1);
    }
    let words: Vec<&str> = case
        .t
        .words
        .get(anchors.start.get() as usize..anchors.end.get() as usize)
        .unwrap_or(&[])
        .iter()
        .map(|w| w.text.as_str())
        .collect();
    format!("\"{}\"", words.join(" "))
}

fn with_threshold(kind: EventKind, threshold: f32) -> DetectorConfig {
    let mut cfg = DetectorConfig::default();
    match kind {
        EventKind::NumberReveal => cfg.threshold_number = threshold,
        EventKind::ListReveal => cfg.threshold_list = threshold,
        EventKind::FromTo => cfg.threshold_from_to = threshold,
        EventKind::KeywordPop => cfg.threshold_keyword = threshold,
    }
    cfg
}

// --- The gate. --------------------------------------------------------------

#[test]
fn precision_over_the_labeled_set() {
    let (cases, skipped) = cases();
    let mut out = String::new();
    writeln!(
        out,
        "precision_recall: {} files read, {} skipped",
        cases.len(),
        skipped.len()
    )
    .unwrap();
    for id in &skipped {
        writeln!(out, "{id}: skipped (no data yet)").unwrap();
    }
    check_coverage(&cases);

    // The default config: the gate.
    let cfg = DetectorConfig::default();
    let mut tallies: BTreeMap<&str, Tally> = BTreeMap::new();
    let mut below = Vec::new();
    let mut false_positives = String::new();
    let mut missed = String::new();
    let mut reported = String::new();
    for case in &cases {
        let found = detect(&case.t, &case.p, &case.edits, &cfg);
        for e in &found {
            if e.confidence.get() < cfg.threshold(e.kind) {
                below.push(format!("{}: {e:?}", case.id));
            }
        }
        for kind in KINDS {
            let m = match_kind(&found, &case.wants, kind);
            tallies.entry(name(kind)).or_default().add(m.tally);
            for e in m.false_positives {
                writeln!(
                    false_positives,
                    "{}: {} {} ({:.2})",
                    case.id,
                    name(kind),
                    said(case, e.anchors),
                    e.confidence.get()
                )
                .unwrap();
            }
            for w in m.missed {
                writeln!(missed, "{}: {} \"{}\"", case.id, name(kind), w.say).unwrap();
            }
            // Labels and headers are reported, not matched.
            for (e, w) in m.pairs {
                let (wanted, got) = match &e.params {
                    EventParams::NumberReveal { label, .. } => (&w.label, label.clone()),
                    EventParams::ListReveal { header, .. } => (&w.header, Some(header.clone())),
                    EventParams::FromTo { .. } | EventParams::KeywordPop { .. } => continue,
                };
                if let Some(wanted) = wanted.as_ref().filter(|x| got.as_ref() != Some(*x)) {
                    writeln!(
                        reported,
                        "{}: {} \"{}\": labeled {wanted:?}, detected {got:?}",
                        case.id,
                        name(kind),
                        w.say
                    )
                    .unwrap();
                }
            }
        }
    }
    let mut overall = Tally::default();
    writeln!(out, "\nkind            TP  FP  FN  precision  recall").unwrap();
    for kind in KINDS {
        let t = tallies.get(name(kind)).copied().unwrap_or_default();
        overall.add(t);
        writeln!(
            out,
            "{:<14} {:>3} {:>3} {:>3}  {:>9}  {:>6}",
            name(kind),
            t.tp,
            t.fp,
            t.fn_,
            ratio(t.precision()),
            ratio(t.recall())
        )
        .unwrap();
    }
    writeln!(
        out,
        "{:<14} {:>3} {:>3} {:>3}  {:>9}  {:>6}",
        "overall",
        overall.tp,
        overall.fp,
        overall.fn_,
        ratio(overall.precision()),
        ratio(overall.recall())
    )
    .unwrap();

    // The sweep: one kind's threshold moved, the others at their defaults,
    // `detect` run again, so `resolve` sees what that threshold lets through.
    writeln!(
        out,
        "\nsweep (precision/recall of the kind; its threshold alone moved, defaults {:.2} {:.2} {:.2} {:.2})",
        cfg.threshold_number, cfg.threshold_list, cfg.threshold_from_to, cfg.threshold_keyword
    )
    .unwrap();
    writeln!(
        out,
        "threshold  number_reveal  list_reveal  from_to    keyword_pop"
    )
    .unwrap();
    for step in 10u8..=20 {
        let threshold = f32::from(step * 5) / 100.0;
        let mut row = format!("{threshold:<9.2}");
        for kind in KINDS {
            let swept = with_threshold(kind, threshold);
            let mut t = Tally::default();
            for case in &cases {
                let found = detect(&case.t, &case.p, &case.edits, &swept);
                t.add(match_kind(&found, &case.wants, kind).tally);
            }
            let cell = format!("{}/{}", ratio(t.precision()), ratio(t.recall()));
            write!(row, "  {cell:<11}").unwrap();
        }
        writeln!(out, "{}", row.trim_end()).unwrap();
    }

    writeln!(out, "\nfalse positives (file: kind words (confidence))").unwrap();
    out.push_str(if false_positives.is_empty() {
        "none\n"
    } else {
        &false_positives
    });
    writeln!(out, "\nmissed (file: kind \"say\")").unwrap();
    out.push_str(if missed.is_empty() { "none\n" } else { &missed });
    writeln!(
        out,
        "\nlabels and headers that differ (reported, not matched)"
    )
    .unwrap();
    out.push_str(if reported.is_empty() {
        "none\n"
    } else {
        &reported
    });
    println!("{out}");

    // Assertion 3, then 2 (INV-6), then 1.
    assert!(cases.len() >= 30, "{} files read, at least 30", cases.len());
    assert!(below.is_empty(), "INV-6: {}", below.join("\n"));
    let precision = overall.precision().unwrap_or(0.0);
    assert!(
        precision >= f64::from(DETECTOR_PRECISION_TARGET),
        "overall precision {precision:.3} is under {DETECTOR_PRECISION_TARGET}: read v3implementation \
         §20.3. For each kind under 0.90, the lowest threshold of the sweep above at which it reaches \
         0.90 is the contingency (TS §17); labels are not edited (D-87)"
    );
}
