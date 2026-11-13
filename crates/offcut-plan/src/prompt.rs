//! What a model is told and must answer with (TS §40.3, v4implementation
//! §7.3, D-123): the system prompt and the output schema, both generated from
//! the catalog, and the user message, generated from a transcript. Written
//! and snapshot-tested in V4; nothing calls them until V6.2. Behind the
//! feature `prompt`, so the render bundle carries none of it (D-122).

use std::fmt::Write as _;

use offcut_types::{DurMs, EventKind, PLAN_FORMAT, Sentence, Word, WordProsody};
use serde_json::{Map, Value, json};

use crate::catalog::{
    AnchorRule, CATALOG_VERSION, EVENT_ZONE, ItemSpec, ParamSpec, ParamTy, catalog, spec,
};
use crate::validate::{
    DENSITY_WINDOW_MS, EVENT_HOLD_MS, HASHTAG_MAX_CHARS, HOOK_MAX_CHARS, HOOK_WORDS, MAX_HASHTAGS,
    MAX_PUNCH, POST_CAPTION_MAX_CHARS, PUNCH_GAP_MS,
};

/// A pause at least this long is marked in the user message (assumption, V6.2).
pub const PAUSE_MARK_MS: u32 = 300;

/// A transcript as the user message needs it. V6.2 fills it from the request.
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
        AnchorRule::ClipWide => {
            "the whole clip, written {\"start\": 0, \"end\": the number of words}".to_owned()
        }
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
        ParamTy::Index { max } => format!("a whole number from 0 to {max}"),
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

/// Seconds from milliseconds, as the prompt writes them (`1.5`, `6`).
fn seconds(ms: u32) -> f64 {
    f64::from(ms) / 1_000.0
}

/// The most words a kind's anchors may hold, from its catalog row.
fn max_anchor_words(kind: EventKind) -> u32 {
    match spec(kind).map(|row| row.anchor) {
        Some(AnchorRule::Words { max, .. }) => max,
        _ => 1,
    }
}

/// V6.2's paragraphs (TS §47, §47.2, §47.3; v6.2implementation D-456): what
/// a finished short looks like, the zooms, the punch words and emoji, and
/// the text extras. Their numbers are `validate`'s, so the two cannot drift.
/// P188b (TE-15, CP 1): the zooms are fewer and cover only their claim, a
/// one-word item is anchored to its word, a number's range is exactly its
/// words, and the event zone is shared.
fn finished_short_text() -> String {
    let density = seconds(DENSITY_WINDOW_MS);
    let punch_gap = seconds(PUNCH_GAP_MS);
    let zoom_gap = seconds(spec(EventKind::PunchIn).map_or(0, |row| row.min_gap_ms));
    let number_words = max_anchor_words(EventKind::NumberReveal);
    let event_hold = seconds(EVENT_HOLD_MS);
    let mut event_kinds: Vec<&str> = catalog()
        .iter()
        .filter(|row| row.zone == EVENT_ZONE)
        .map(|row| row.name)
        .collect();
    let last_event_kind = event_kinds.pop().unwrap_or_default();
    let (hook_min, hook_max) = (HOOK_WORDS.start(), HOOK_WORDS.end());
    let mut out = String::new();
    let _ = writeln!(
        out,
        "A finished short: open with the hook title; zoom only on the sentences that land hardest, so most sentences have no \
         zoom; never stack two visuals, so at most one visual (a number, a list, a from-to, an \
         icon, an emoji or the hook title) starts in any {density} seconds; end on the payoff, \
         not a call to action, unless the speaker asks for one."
    );
    let _ = writeln!(
        out,
        "\nZooms: zoom on the claim, the number or the punchline, never on filler, and only \
         where it earns it: about one sentence in three. A zoom covers only the words of its \
         claim, not its whole sentence: a snap covers one strong word (1 to 3 words); a push \
         covers the few words that build to it (2 to 40 words). A zoom stays inside one \
         sentence, and a sentence has at most one. Two zooms never overlap, and a zoom starts \
         at least {zoom_gap} seconds after the last one ends, so two sentences in a row are \
         never both zoomed."
    );
    let _ = writeln!(
        out,
        "\nPointing at words: an item that marks one word (emphasis, keyword_pop, icon_pop, \
         emoji) is anchored to that word alone, written {{\"start\": w, \"end\": w + 1}} where w \
         is the word's index, never to its sentence. A number's value, and each side of a \
         from-to, is the range of exactly the words that say the number: it starts on the \
         number, and ends on the number or on the one unit word just after it (hours, percent, \
         dollars, users), never on a word such as a, an, of, per or pm: \"$2,400\", not \"cost \
         $2,400\"; \"3000 hours\"; \"$12\" in \"$12 a month\". Both sides of a from-to are \
         numbers the speaker says. A number_reveal is anchored to those words, or to at most \
         {number_words} words around them."
    );
    let _ = writeln!(
        out,
        "\nThe event zone: {} and {last_event_kind} share it, and each holds it until \
         {event_hold} seconds after its last word. Never put an icon on a number, a list or a \
         from-to, or just after one.",
        event_kinds.join(", ")
    );
    let _ = writeln!(
        out,
        "\nEmphasis and emoji: emphasis marks the one word that carries a sentence. Give size \
         punch only to the line that lands hardest: at most one punch word in any {punch_gap} \
         seconds, and at most {MAX_PUNCH} in a clip. Put an emoji under its word, or beside it, \
         only where it says the word better than text. Use only the icon and emoji names \
         listed above."
    );
    let _ = writeln!(
        out,
        "\nText extras, under extras: hook, a title for the short in the speaker's voice, \
         {hook_min} to {hook_max} words and at most {HOOK_MAX_CHARS} characters, with no digit \
         the speaker did not say. Write it always, and always add a hook_title item with it. \
         Its accent is the index of the hook's strongest word, counting the hook's own words \
         from 0, not the transcript's: in \"We cut our bill\", \"bill\" is 3, and in a hook of n \
         words the last is n - 1. Its anchors are every word of the transcript: start 0, and \
         end one more than the index of the transcript's last word. post_caption: the text to \
         post with the short, at most \
         {POST_CAPTION_MAX_CHARS} characters. hashtags: at most {MAX_HASHTAGS}, each # followed \
         by 1 to {HASHTAG_MAX_CHARS} letters, digits or underscores."
    );
    out
}

/// The instructions, generated from the catalog: the task, the transcript's
/// format, one paragraph per kind, V6.2's paragraphs, and the rules every
/// plan keeps.
pub fn system_prompt() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{PREAMBLE}\n\n{TRANSCRIPT_FORMAT}\n\nThe kinds of item:"
    );
    for row in catalog() {
        let _ = writeln!(out, "\n{}", row_text(row));
    }
    let _ = write!(out, "\n{}", finished_short_text());
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
        ParamTy::Index { max } => json!({ "type": "integer", "minimum": 0, "maximum": max }),
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

/// V6.2 (D-454): the three text extras, each optional.
fn extras_schema() -> Value {
    let hashtag = format!("^#[A-Za-z0-9_]{{1,{HASHTAG_MAX_CHARS}}}$");
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "hook": { "type": "string", "maxLength": HOOK_MAX_CHARS },
            "post_caption": { "type": "string", "maxLength": POST_CAPTION_MAX_CHARS },
            "hashtags": {
                "type": "array",
                "maxItems": MAX_HASHTAGS,
                "items": { "type": "string", "pattern": hashtag }
            }
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
            "extras": extras_schema(),
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
        // P188b: the one-word anchor and the zoom's gap, from the catalog.
        assert!(prompt.contains(r#"{"start": w, "end": w + 1}"#));
        assert!(prompt.contains("at least 1.5 seconds after the last one ends"));
        assert!(prompt.contains("number_reveal, list_reveal, from_to and icon_pop share it"));
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
        assert_eq!(kinds.len(), 9);
        plan["source"] = json!({ "kind": "model", "provider": "p", "model": "m" });
        let read: offcut_types::EditPlan = serde_json::from_value(plan).unwrap();
        assert_eq!(read.items.len(), 4);
        assert!(read.extras.hook.is_none() && read.music.is_none());
    }

    /// V6.2: the five kinds and the extras as the v2 schema describes them
    /// read as an `EditPlan` too, and the schema names what the types read.
    #[test]
    fn the_v2_kinds_and_extras_written_to_the_schema_read_as_an_edit_plan() {
        let range = |start: u32, end: u32| json!({ "start": start, "end": end });
        let item = |id: &str, kind: &str, anchors: Value, params: Value| {
            json!({ "id": id, "kind": kind, "anchors": anchors,
                    "params": { "kind": kind, "params": params }, "confidence": null })
        };
        let plan = json!({
            "format": PLAN_FORMAT,
            "catalog": CATALOG_VERSION,
            "source": { "kind": "model", "provider": "p", "model": "m" },
            "cuts": [],
            "items": [
                item("1", "emphasis", range(2, 3),
                     json!({ "word": 2, "treatment": "highlight", "size": "punch" })),
                item("2", "icon_pop", range(4, 5), json!({ "word": 4, "icon": "trending_up" })),
                item("3", "punch_in", range(6, 9), json!({ "scale": "strong", "motion": "push" })),
                item("4", "emoji", range(10, 11),
                     json!({ "word": 10, "emoji": "money_bag", "place": "beside" })),
                item("5", "hook_title", range(0, 40), json!({ "accent": 2 })),
            ],
            "chapters": [],
            "extras": { "hook": "How I saved hours", "post_caption": "A trick", "hashtags": ["#editing"] },
            "music": null
        });
        let read: offcut_types::EditPlan = serde_json::from_value(plan).unwrap();
        assert_eq!(read.items.len(), 5);
        assert_eq!(read.extras.hashtags.len(), 1);
        let schema = output_schema();
        assert_eq!(
            schema["properties"]["catalog"]["const"],
            json!(CATALOG_VERSION)
        );
        let extras = &schema["properties"]["extras"]["properties"];
        assert_eq!(extras["hook"]["maxLength"], json!(48));
        assert_eq!(extras["hashtags"]["maxItems"], json!(8));
    }
}
