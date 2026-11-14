//! The model's text to an `EditPlan` (v6.2implementation D-460), one item at
//! a time (checkpoint 1, 2026-11-12): an item that does not read, such as
//! one naming an emoji outside the 72, is left out and counted, and the rest
//! of the plan stands. TE-15 lost three whole plans to one unknown name each
//! (`experiments.md`). Everything outside `items` must read, or the plan
//! fails and the route takes its one retry.
//!
//! The server sets `format`, `catalog` and `source` itself and never reads
//! them from the model (D-457, D-460). `validate` stays the gate (INV-21):
//! nothing here checks an item against the transcript.

use offcut_plan::CATALOG_VERSION;
use offcut_types::{EditPlan, PLAN_FORMAT, PlanItem, PlanSource};
use serde_json::{Map, Value};

/// A plan read from the model, and how many of its items did not read.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedPlan {
    pub plan: EditPlan,
    /// Logged as a count (D-464); a count alone holds nothing of the text.
    pub items_skipped: u32,
}

/// Which part of the answer did not read. Safe to log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseFailureKind {
    /// Nothing but white space (DeepSeek's documented blank answer, D-500).
    Blank,
    /// Not JSON.
    NotJson,
    /// JSON, but not one object.
    NotAnObject,
    /// An object, but not an `EditPlan` outside its items.
    NotAPlan,
}

/// Why the answer is not a plan. `detail` is the parser's own message, which
/// may quote the model's text, so it goes back to the model on the retry
/// (D-460) and never to a log: `Debug` and `Display` show the kind alone.
#[derive(Clone, PartialEq, Eq)]
pub struct ParseFailure {
    pub kind: ParseFailureKind,
    detail: String,
}

impl ParseFailure {
    fn new(kind: ParseFailureKind, detail: impl ToString) -> ParseFailure {
        ParseFailure {
            kind,
            detail: detail.to_string(),
        }
    }

    /// What the one retry appends to the user message (D-460).
    pub fn retry_note(&self) -> String {
        format!(
            "\n\nYour last answer could not be read as the plan: {}. Answer again with one JSON \
             object that matches the schema.",
            self.detail
        )
    }
}

impl std::fmt::Debug for ParseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseFailure")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for ParseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the model's answer is not a plan ({:?})", self.kind)
    }
}

impl std::error::Error for ParseFailure {}

/// The plan in `text`, with `source` and this build's format and catalog.
pub fn parse_plan(text: &str, source: PlanSource) -> Result<ParsedPlan, ParseFailure> {
    if text.trim().is_empty() {
        return Err(ParseFailure::new(
            ParseFailureKind::Blank,
            "the answer was empty",
        ));
    }
    let value: Value = serde_json::from_str(text)
        .map_err(|error| ParseFailure::new(ParseFailureKind::NotJson, error))?;
    let Value::Object(mut fields) = value else {
        return Err(ParseFailure::new(
            ParseFailureKind::NotAnObject,
            "the answer is not one JSON object",
        ));
    };
    let written = match fields.remove("items") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items,
        Some(_) => {
            return Err(ParseFailure::new(
                ParseFailureKind::NotAPlan,
                "items is not a list",
            ));
        }
    };
    set_by_the_server(&mut fields, source)?;
    let mut plan: EditPlan = serde_json::from_value(Value::Object(fields))
        .map_err(|error| ParseFailure::new(ParseFailureKind::NotAPlan, error))?;
    let mut items_skipped = 0u32;
    for item in written {
        match serde_json::from_value::<PlanItem>(item) {
            Ok(item) => plan.items.push(item),
            Err(_) => items_skipped = items_skipped.saturating_add(1),
        }
    }
    Ok(ParsedPlan {
        plan,
        items_skipped,
    })
}

/// `format`, `catalog` and `source` as the server sets them, whatever the
/// model wrote; `items` empty until each is read.
fn set_by_the_server(
    fields: &mut Map<String, Value>,
    source: PlanSource,
) -> Result<(), ParseFailure> {
    let source = serde_json::to_value(source)
        .map_err(|error| ParseFailure::new(ParseFailureKind::NotAPlan, error))?;
    fields.insert("format".to_owned(), Value::from(PLAN_FORMAT));
    fields.insert("catalog".to_owned(), Value::from(CATALOG_VERSION));
    fields.insert("source".to_owned(), source);
    fields.insert("items".to_owned(), Value::Array(Vec::new()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn source() -> PlanSource {
        PlanSource::Model {
            provider: "deepseek".to_owned(),
            model: "deepseek-flash".to_owned(),
        }
    }

    fn emoji(id: &str, name: &str) -> Value {
        json!({ "id": id, "kind": "emoji", "anchors": { "start": 3, "end": 4 },
                "params": { "kind": "emoji", "params": { "word": 3, "emoji": name, "place": "under" } },
                "confidence": null })
    }

    fn answer(items: Vec<Value>) -> String {
        json!({ "format": 1, "catalog": 1, "cuts": [], "items": items, "chapters": [],
                "extras": { "hook": "A hook" }, "music": null })
        .to_string()
    }

    #[test]
    fn an_unknown_name_leaves_out_its_item_and_keeps_the_rest() {
        let text = answer(vec![
            emoji("1", "fire"),
            emoji("2", "cat"),
            emoji("3", "rocket"),
        ]);
        let parsed = parse_plan(&text, source()).unwrap();
        let ids: Vec<u64> = parsed.plan.items.iter().map(|i| i.id.get()).collect();
        assert_eq!(ids, [1, 3]);
        assert_eq!(parsed.items_skipped, 1);
        assert_eq!(
            parsed.plan.extras.hook.as_ref().map(|h| h.as_str()),
            Some("A hook")
        );
    }

    #[test]
    fn the_server_sets_format_catalog_and_source_whatever_the_model_wrote() {
        let mut value: Value = serde_json::from_str(&answer(vec![])).unwrap();
        value["format"] = json!(99);
        value["catalog"] = json!({});
        value["source"] = json!({ "kind": "rules" });
        let parsed = parse_plan(&value.to_string(), source()).unwrap();
        assert_eq!(parsed.plan.format, PLAN_FORMAT);
        assert_eq!(parsed.plan.catalog, CATALOG_VERSION);
        assert_eq!(parsed.plan.source, source());
        // Left out by the model (DeepSeek leaves out `source`, D-500): set all the same.
        let mut bare: Value = serde_json::from_str(&answer(vec![])).unwrap();
        for key in ["format", "catalog", "items"] {
            bare.as_object_mut().unwrap().remove(key);
        }
        let parsed = parse_plan(&bare.to_string(), source()).unwrap();
        assert!(parsed.plan.items.is_empty() && parsed.items_skipped == 0);
    }

    #[test]
    fn each_unreadable_answer_fails_with_its_kind() {
        for (text, kind) in [
            ("", ParseFailureKind::Blank),
            ("  \n\n ", ParseFailureKind::Blank),
            ("{\"items\": [", ParseFailureKind::NotJson),
            ("[1, 2]", ParseFailureKind::NotAnObject),
            ("{\"items\": {}}", ParseFailureKind::NotAPlan),
            (
                "{\"cuts\": [], \"chapters\": [], \"extras\": {}, \"music\": null, \"mood\": 1}",
                ParseFailureKind::NotAPlan,
            ),
            (
                "{\"cuts\": [], \"chapters\": [], \"extras\": {\"hashtags\": 3}, \"music\": null}",
                ParseFailureKind::NotAPlan,
            ),
        ] {
            assert_eq!(
                parse_plan(text, source()).unwrap_err().kind,
                kind,
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_failure_shows_its_kind_in_a_log_and_its_detail_only_to_the_model() {
        let failure = parse_plan("{\"markermodeltext\": [", source()).unwrap_err();
        for shown in [format!("{failure:?}"), failure.to_string()] {
            assert!(
                !shown.contains("markermodeltext") && !shown.contains("line"),
                "{shown}"
            );
            assert!(shown.contains("NotJson"), "{shown}");
        }
        let unknown = answer(vec![]).replace("\"music\":null", "\"music\":null,\"markerfield\":1");
        let note = parse_plan(&unknown, source()).unwrap_err().retry_note();
        assert!(
            note.contains("markerfield"),
            "the model is told what failed: {note}"
        );
    }
}
