//! What a model is told and must answer with (TS §40.3, v4implementation
//! §7.3, D-123): the system prompt and the output schema, both generated from
//! the catalog, and the user message, generated from a transcript. Written
//! and snapshot-tested in V4; nothing calls them until V6.1. Behind the
//! feature `prompt`, so the render bundle carries none of it (D-122).

use std::fmt::Write as _;

use offcut_types::{DurMs, PLAN_FORMAT, Sentence, Word, WordProsody};
use serde_json::{Map, Value, json};

use crate::catalog::{AnchorRule, CATALOG_VERSION, ItemSpec, ParamSpec, ParamTy, catalog};

/// A pause at least this long is marked in the user message (assumption, V6.1).
pub const PAUSE_MARK_MS: u32 = 300;

/// A transcript as the user message needs it. V6.1 fills it from the request.
pub struct PromptInput<'a> {
    pub duration: DurMs,
    pub words: &'a [Word],
    pub prosody: &'a [WordProsody],
    pub sentences: &'a [Sentence],
}

const PREAMBLE: &str = "\
You plan the edit of a short talking-head video from its transcript. The \
speaker talks to the camera; you choose the moments to show as visuals over \
the video, using only the kinds of item listed below, and you point at the \
words of the transcript where each one belongs. Leave out an item rather \
than force one: a plan with few good items is better than a busy one.";

const TRANSCRIPT_FORMAT: &str = "\
The transcript arrives as plain text. Its first line is DURATION and the \
length of the clip in milliseconds. Then each word has a line with six \
fields separated by tabs: its index, its text, its start and its end in \
milliseconds, and how much louder (energy) and how much higher (pitch) it \
was said than the speaker's average, in standard deviations. A line PAUSE \
and a length in milliseconds comes before a word that follows a silence at \
least that long. A blank line ends a sentence.";

const RULES: [&str; 7] = [
    "Never write a number: point at the words that say it.",
    "Every item points at word indexes of the transcript.",
    "Every item has an id, a string of digits no other item of the plan has, and a confidence of null.",
    "A range of words is written {\"start\": first word index, \"end\": the index after the last word}; an item's anchors are the words it belongs to, and every word or range it points at lies inside them.",
    "A label repeats or shortens what the speaker says; it holds no digit the speaker did not say.",
    "Two items never show in the same zone at the same time, and two items of one kind keep their distance.",
    "Return JSON matching the schema; no prose.",
];

fn anchor_text(anchor: AnchorRule) -> String {
    match anchor {
        AnchorRule::Words { min, max } => format!("from {min} to {max} words in a row"),
        AnchorRule::Point => "exactly one word".to_owned(),
        AnchorRule::ClipWide => "the whole clip".to_owned(),
    }
}

fn param_text(param: &ParamSpec) -> String {
    let what = match param.ty {
        ParamTy::Label { max_chars } => format!("text of at most {max_chars} characters"),
        ParamTy::QuantityRef => "the range of words that says the number".to_owned(),
        ParamTy::WordIdx => "the index of one word".to_owned(),
        ParamTy::ListPoints { max, label_chars } => format!(
            "at most {max} points, each the index of the word where it is said and a label of at most {label_chars} characters"
        ),
        ParamTy::Enum(values) => format!("one of {}", values.join(", ")),
        ParamTy::Bool => "true or false".to_owned(),
    };
    let need = if param.required {
        "required"
    } else {
        "optional"
    };
    format!("{} ({need}): {what}", param.name)
}

fn row_text(row: &ItemSpec) -> String {
    let params: Vec<String> = row.params.iter().map(param_text).collect();
    format!(
        "{}: {} Anchors: {}. Params: {}. Zone: {}. At most {} in a clip, at least {} ms apart.",
        row.name,
        row.purpose,
        anchor_text(row.anchor),
        params.join("; "),
        row.zone,
        row.max_per_clip,
        row.min_gap_ms,
    )
}

/// The instructions, generated from the catalog: the task, the transcript's
/// format, one paragraph per kind, and the rules every plan keeps.
pub fn system_prompt() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{PREAMBLE}\n\n{TRANSCRIPT_FORMAT}\n\nThe kinds of item:"
    );
    for row in catalog() {
        let _ = writeln!(out, "\n{}", row_text(row));
    }
    out.push_str("\nRules:\n");
    for rule in RULES {
        let _ = writeln!(out, "- {rule}");
    }
    out
}

fn word_range_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["start", "end"],
        "properties": {
            "start": { "type": "integer", "minimum": 0 },
            "end": { "type": "integer", "minimum": 0 }
        }
    })
}

fn param_schema(ty: ParamTy) -> Value {
    match ty {
        ParamTy::Label { max_chars } => json!({ "type": "string", "maxLength": max_chars }),
        ParamTy::QuantityRef => json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["words"],
            "properties": { "words": word_range_schema() }
        }),
        ParamTy::WordIdx => json!({ "type": "integer", "minimum": 0 }),
        ParamTy::ListPoints { max, label_chars } => json!({
            "type": "array",
            "minItems": 1,
            "maxItems": max,
            "items": {
                "type": "object",
                "additionalProperties": false,
                "required": ["at", "label"],
                "properties": {
                    "at": { "type": "integer", "minimum": 0 },
                    "label": { "type": "string", "maxLength": label_chars }
                }
            }
        }),
        ParamTy::Enum(values) => json!({ "enum": values }),
        ParamTy::Bool => json!({ "type": "boolean" }),
    }
}

/// One item of a kind: its word-pointing params under `"params"` (the item's
/// `ItemParams`), never the detector's `"detected"` arm.
fn item_schema(row: &ItemSpec) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for param in row.params {
        properties.insert(param.name.to_owned(), param_schema(param.ty));
        if param.required {
            required.push(param.name);
        }
    }
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["id", "kind", "anchors", "params", "confidence"],
        "properties": {
            "id": { "type": "string", "pattern": "^(0|[1-9][0-9]*)$" },
            "kind": { "const": row.name },
            "anchors": word_range_schema(),
            "params": {
                "type": "object",
                "additionalProperties": false,
                "required": ["kind", "params"],
                "properties": {
                    "kind": { "const": row.name },
                    "params": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": required,
                        "properties": properties
                    }
                }
            },
            "confidence": { "type": "null" }
        }
    })
}

/// The JSON Schema of an `EditPlan` as a model may write it: the catalog's
/// kinds only, word-pointing params only, no `source` (the server sets it,
/// TS §40.7), and the fields of later releases held empty.
pub fn output_schema() -> Value {
    let items: Vec<Value> = catalog().iter().map(item_schema).collect();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "EditPlan",
        "type": "object",
        "additionalProperties": false,
        "required": ["format", "catalog", "cuts", "items", "chapters", "extras", "music"],
        "properties": {
            "format": { "const": PLAN_FORMAT },
            "catalog": { "const": CATALOG_VERSION },
            "cuts": { "type": "array", "maxItems": 0 },
            "items": { "type": "array", "items": { "oneOf": items } },
            "chapters": { "type": "array", "maxItems": 0 },
            "extras": { "type": "object", "additionalProperties": false, "properties": {} },
            "music": { "type": "null" }
        }
    })
}

/// The transcript as the model reads it: the ASR text, never the user's edits
/// (TS §40.7). `DURATION`, then one line per word, a `PAUSE` line before a
/// word after a long silence, and a blank line after each sentence.
pub fn user_message(input: &PromptInput<'_>) -> String {
    let mut out = format!("DURATION {}\n", input.duration.get());
    let mut previous_end = None;
    for (i, word) in (0u32..).zip(input.words) {
        if let Some(end) = previous_end {
            let gap = word.start_ms.duration_since(end).map_or(0, DurMs::get);
            if gap >= PAUSE_MARK_MS {
                let _ = writeln!(out, "PAUSE {gap}");
            }
        }
        let index = usize::try_from(i).unwrap_or(usize::MAX);
        let (energy, pitch) = input
            .prosody
            .get(index)
            .map_or((0.0, 0.0), |p| (p.energy_z, p.pitch_z));
        let _ = writeln!(
            out,
            "{i}\t{}\t{}\t{}\t{energy:.1}\t{pitch:.1}",
            word.text,
            word.start_ms.get(),
            word.end_ms.get()
        );
        if input.sentences.iter().any(|s| s.words.end.get() == i + 1) {
            out.push('\n');
        }
        previous_end = Some(word.end_ms);
    }
    out
}

#[cfg(test)]
mod tests {
    use offcut_types::{Confidence, TimeMs, WordIdx, WordRange};

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
    fn every_output_is_the_same_twice() {
        assert_eq!(system_prompt(), system_prompt());
        assert_eq!(output_schema(), output_schema());
        let words = [word("So", 100, 300), word("I", 700, 800)];
        let sentences = [Sentence {
            words: WordRange {
                start: WordIdx::new(0),
                end: WordIdx::new(2),
            },
            start_ms: TimeMs::new(100),
            end_ms: TimeMs::new(800),
        }];
        let input = PromptInput {
            duration: DurMs::new(1_000),
            words: &words,
            prosody: &[],
            sentences: &sentences,
        };
        assert_eq!(user_message(&input), user_message(&input));
        assert_eq!(
            user_message(&input),
            "DURATION 1000\n0\tSo\t100\t300\t0.0\t0.0\nPAUSE 400\n1\tI\t700\t800\t0.0\t0.0\n\n"
        );
    }

    #[test]
    fn the_prompt_names_every_kind_and_the_rules() {
        let prompt = system_prompt();
        for row in catalog() {
            assert!(prompt.contains(row.name) && prompt.contains(row.purpose));
        }
        assert!(prompt.contains("Never write a number: point at the words that say it."));
        assert!(prompt.contains("Return JSON matching the schema; no prose."));
        let schema = output_schema().to_string();
        assert!(!schema.contains("detected") && !schema.contains("\"source\""));
    }

    /// A plan as the schema describes it, with the `source` the server adds
    /// (TS §40.7), is an `EditPlan`: the schema and the wire format agree.
    #[test]
    fn a_plan_written_to_the_schema_reads_as_an_edit_plan() {
        let range = |start: u32, end: u32| json!({ "start": start, "end": end });
        let item = |id: &str, kind: &str, anchors: Value, params: Value| {
            json!({ "id": id, "kind": kind, "anchors": anchors,
                    "params": { "kind": kind, "params": params }, "confidence": null })
        };
        let mut plan = json!({
            "format": PLAN_FORMAT,
            "catalog": CATALOG_VERSION,
            "cuts": [],
            "items": [
                item("1", "number_reveal", range(3, 5), json!({ "value": { "words": range(3, 5) } })),
                item("2", "list_reveal", range(6, 20),
                     json!({ "items": [{ "at": 8, "label": "captions" }] })),
                item("3", "from_to", range(21, 27),
                     json!({ "from": { "words": range(22, 24) }, "to": { "words": range(25, 27) },
                             "label": "a rough cut" })),
                item("4", "keyword_pop", range(30, 31), json!({ "word": 30 })),
            ],
            "chapters": [],
            "extras": {},
            "music": null
        });
        let kinds: Vec<&str> = catalog().iter().map(|row| row.name).collect();
        assert_eq!(kinds.len(), 4);
        plan["source"] = json!({ "kind": "model", "provider": "p", "model": "m" });
        let read: offcut_types::EditPlan = serde_json::from_value(plan).unwrap();
        assert_eq!(read.items.len(), 4);
        assert!(read.extras.hook.is_none() && read.music.is_none());
    }
}
