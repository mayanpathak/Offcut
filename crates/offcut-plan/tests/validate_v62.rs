//! V6.2's validation (v6.2implementation §6.3, §13.1): what can be checked
//! through `validate` before the catalog has the five rows (P185). The text
//! extras (D-454), one by one; DC-7 (D-449). P186 adds each new kind end to
//! end, V-3, V-9 and D-452 through `validate`, and the rules plan's exemption.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeMap;

use offcut_plan::{
    CATALOG_VERSION, DropReason, Dropped, PlanConfig, RejectReason, ValidExtras, validate,
};
use offcut_text::{RawWord, normalize_transcript};
use offcut_types::{EditPlan, Label, PlanSource, Transcript};

/// "I saved 10 hours a week with one trick." and a second sentence.
fn transcript() -> Transcript {
    let words = [
        "I", "saved", "10", "hours", "a", "week", "with", "one", "trick.", "Here", "it", "is.",
    ];
    let words = (0u32..)
        .zip(words)
        .map(|(i, text)| RawWord {
            text: text.to_owned(),
            start_ms: 500 + i * 400,
            end_ms: 800 + i * 400,
            confidence: 1.0,
        })
        .collect();
    normalize_transcript(words, "test")
}

fn label(text: &str) -> Label {
    serde_json::from_value(serde_json::json!(text)).unwrap()
}

fn model() -> EditPlan {
    let mut plan = EditPlan::empty_rules(CATALOG_VERSION);
    plan.source = PlanSource::Model {
        provider: "deepseek".to_owned(),
        model: "test".to_owned(),
    };
    plan
}

fn run(plan: EditPlan) -> (ValidExtras, Vec<Dropped>) {
    let (valid, dropped) = validate(
        plan,
        &transcript(),
        &BTreeMap::new(),
        &PlanConfig::default(),
    )
    .unwrap();
    (valid.extras().clone(), dropped)
}

fn extra(reason: DropReason) -> Dropped {
    Dropped { kind: None, reason }
}

#[test]
fn d454_the_hook_is_2_to_9_words_of_at_most_48_characters() {
    let hook = |text: &str| {
        let mut plan = model();
        plan.extras.hook = Some(label(text));
        run(plan)
    };
    let (extras, dropped) = hook("I saved 10 hours a week");
    assert_eq!(extras.hook.as_deref(), Some("I saved 10 hours a week"));
    assert_eq!(dropped, []);
    for bad in [
        "Saved",
        "one two three four five six seven eight nine ten",
        "This hook is far too long for the top of a short",
    ] {
        let (extras, dropped) = hook(bad);
        assert_eq!(extras.hook, None, "{bad}");
        assert_eq!(dropped, [extra(DropReason::LabelTooLong)], "{bad}");
    }
    // V-4: a digit the speaker did not say.
    let (extras, dropped) = hook("I saved 20 hours");
    assert_eq!(extras.hook, None);
    assert_eq!(dropped, [extra(DropReason::LabelDigits)]);
}

#[test]
fn d454_the_post_caption_is_at_most_2200_characters_and_any_digits() {
    let caption = |text: String| {
        let mut plan = model();
        plan.extras.post_caption = Some(text);
        run(plan)
    };
    let (extras, dropped) = caption("Saved 25 hours this month".to_owned());
    assert_eq!(
        extras.post_caption.as_deref(),
        Some("Saved 25 hours this month")
    );
    assert_eq!(dropped, []);
    let (extras, _) = caption("é".repeat(2_200));
    assert_eq!(extras.post_caption.map(|c| c.chars().count()), Some(2_200));
    let (extras, dropped) = caption("a".repeat(2_201));
    assert_eq!(extras.post_caption, None);
    assert_eq!(dropped, [extra(DropReason::LabelTooLong)]);
}

#[test]
fn d454_hashtags_are_at_most_8_each_checked_alone() {
    let tags = |tags: &[&str]| {
        let mut plan = model();
        plan.extras.hashtags = tags.iter().map(|t| label(t)).collect();
        run(plan)
    };
    let (extras, dropped) = tags(&["#productivity", "#time_saving", "#20x", "#10x"]);
    assert_eq!(extras.hashtags, ["#productivity", "#time_saving", "#10x"]);
    assert_eq!(dropped, [extra(DropReason::LabelDigits)]);

    let (extras, dropped) = tags(&[
        "editing",
        "#",
        "#two words",
        "#a-b",
        "#abcdefghijklmnopqrstuvwxyz12345",
        "#ok",
    ]);
    assert_eq!(extras.hashtags, ["#ok"]);
    let reasons: Vec<DropReason> = dropped.iter().map(|d| d.reason).collect();
    assert_eq!(
        reasons,
        [
            DropReason::ParamsMismatch,
            DropReason::ParamsMismatch,
            DropReason::ParamsMismatch,
            DropReason::ParamsMismatch,
            DropReason::LabelTooLong,
        ]
    );
    assert!(dropped.iter().all(|d| d.kind.is_none()));

    let nine: Vec<String> = (b'a'..=b'i').map(|c| format!("#{}", c as char)).collect();
    let nine: Vec<&str> = nine.iter().map(String::as_str).collect();
    let (extras, dropped) = tags(&nine);
    assert_eq!(extras.hashtags.len(), 8);
    assert_eq!(dropped, [extra(DropReason::OverLimit)]);
}

#[test]
fn a_plan_of_the_rules_has_no_extras() {
    let (valid, dropped) = validate(
        EditPlan::empty_rules(CATALOG_VERSION),
        &transcript(),
        &BTreeMap::new(),
        &PlanConfig::default(),
    )
    .unwrap();
    assert_eq!(valid.extras(), &ValidExtras::default());
    assert_eq!(dropped, []);
}

#[test]
fn dc7_a_stored_plan_of_an_older_catalog_still_validates() {
    let t = transcript();
    let cfg = PlanConfig::default();
    for catalog in 0..=CATALOG_VERSION {
        let mut plan = model();
        plan.catalog = catalog;
        assert!(
            validate(plan, &t, &BTreeMap::new(), &cfg).is_ok(),
            "{catalog}"
        );
    }
    for newer in [CATALOG_VERSION + 1, CATALOG_VERSION + 2] {
        let mut plan = model();
        plan.catalog = newer;
        let refused = validate(plan, &t, &BTreeMap::new(), &cfg).unwrap_err();
        assert_eq!(refused.reason, RejectReason::Catalog);
    }
}
