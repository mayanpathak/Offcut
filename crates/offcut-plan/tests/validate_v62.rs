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

// --- P186: each kind end to end, with the catalog's five rows.

use offcut_plan::from_rules;
use offcut_types::{
    Confidence, EmojiId, EmojiPlace, EmphasisSize, EventId, EventKind, EventParams, IconId,
    ItemParams, PlanItem, Quantity, QuantityRef, Treatment, Unit, WordIdx, WordRange, ZoomMotion,
    ZoomScale,
};

/// "I saved 10 hours a week with one trick. Here it is. You batch every
/// edit on Sunday.": 18 words, word `i` from `500 + 700 i` for 300 ms.
fn three_sentences() -> Transcript {
    let words = [
        "I", "saved", "10", "hours", "a", "week", "with", "one", "trick.", "Here", "it", "is.",
        "You", "batch", "every", "edit", "on", "Sunday.",
    ];
    timed(&words)
}

fn timed(words: &[&str]) -> Transcript {
    let words = (0u32..)
        .zip(words)
        .map(|(i, text)| RawWord {
            text: (*text).to_owned(),
            start_ms: 500 + i * 700,
            end_ms: 800 + i * 700,
            confidence: 1.0,
        })
        .collect();
    normalize_transcript(words, "test")
}

fn at(start: u32, end: u32) -> WordRange {
    WordRange {
        start: WordIdx::new(start),
        end: WordIdx::new(end),
    }
}

fn item(id: u64, kind: EventKind, anchors: WordRange, params: ItemParams) -> PlanItem {
    PlanItem {
        id: EventId::new(id),
        kind,
        anchors,
        params,
        confidence: None,
    }
}

fn emphasis(id: u64, word: u32, size: EmphasisSize) -> PlanItem {
    let params = ItemParams::Emphasis {
        word: WordIdx::new(word),
        treatment: Treatment::Colour,
        size,
    };
    item(id, EventKind::Emphasis, at(word, word + 1), params)
}

fn icon(id: u64, word: u32) -> PlanItem {
    let params = ItemParams::IconPop {
        word: WordIdx::new(word),
        icon: IconId::Calendar,
    };
    item(id, EventKind::IconPop, at(word, word + 1), params)
}

fn zoom(id: u64, anchors: WordRange, motion: ZoomMotion) -> PlanItem {
    let params = ItemParams::PunchIn {
        scale: ZoomScale::Mild,
        motion,
    };
    item(id, EventKind::PunchIn, anchors, params)
}

fn emoji(id: u64, word: u32) -> PlanItem {
    let params = ItemParams::Emoji {
        word: WordIdx::new(word),
        emoji: EmojiId::Fire,
        place: EmojiPlace::Under,
    };
    item(id, EventKind::Emoji, at(word, word + 1), params)
}

fn hook_title(id: u64, accent: u8, anchors: WordRange) -> PlanItem {
    let params = ItemParams::HookTitle { accent };
    item(id, EventKind::HookTitle, anchors, params)
}

fn plan_of(items: Vec<PlanItem>, hook: Option<&str>) -> EditPlan {
    let mut plan = model();
    plan.items = items;
    plan.extras.hook = hook.map(label);
    plan
}

fn run_on(t: &Transcript, plan: EditPlan) -> (offcut_plan::ValidPlan, Vec<Dropped>) {
    validate(plan, t, &BTreeMap::new(), &PlanConfig::default()).unwrap()
}

fn reasons(dropped: &[Dropped]) -> Vec<(Option<EventKind>, DropReason)> {
    dropped.iter().map(|d| (d.kind, d.reason)).collect()
}

#[test]
fn each_new_kind_is_kept_and_resolved() {
    let t = three_sentences();
    let items = vec![
        emphasis(1, 1, EmphasisSize::Normal),
        zoom(2, at(2, 4), ZoomMotion::Snap),
        icon(3, 5),
        emoji(4, 13),
        hook_title(5, 1, at(0, 18)),
    ];
    let (valid, dropped) = run_on(&t, plan_of(items, Some("One trick saves hours")));
    assert_eq!(dropped, []);
    let params: Vec<&EventParams> = valid.items().iter().map(|i| &i.params).collect();
    assert_eq!(
        params,
        [
            &EventParams::HookTitle {
                text: "One trick saves hours".to_owned(),
                accent: 1
            },
            &EventParams::Emphasis {
                word: WordIdx::new(1),
                treatment: Treatment::Colour,
                size: EmphasisSize::Normal
            },
            &EventParams::PunchIn {
                scale: ZoomScale::Mild,
                motion: ZoomMotion::Snap
            },
            &EventParams::IconPop {
                word: WordIdx::new(5),
                icon: IconId::Calendar
            },
            &EventParams::Emoji {
                word: WordIdx::new(13),
                emoji: EmojiId::Fire,
                place: EmojiPlace::Under
            },
        ]
    );
    assert_eq!(
        valid.extras().hook.as_deref(),
        Some("One trick saves hours")
    );
}

#[test]
fn the_rules_plan_is_of_catalog_2_and_cannot_hold_the_new_kinds() {
    let t = three_sentences();
    assert_eq!(from_rules(&[], &t).catalog, 2);
    let mut rules = EditPlan::empty_rules(CATALOG_VERSION);
    rules.items = vec![emphasis(1, 1, EmphasisSize::Normal), icon(2, 5)];
    let (valid, dropped) = run_on(&t, rules);
    assert!(valid.is_empty());
    assert_eq!(
        reasons(&dropped),
        [
            (Some(EventKind::Emphasis), DropReason::WrongProducer),
            (Some(EventKind::IconPop), DropReason::WrongProducer),
        ]
    );
}

#[test]
fn d453_the_hook_title_needs_the_hook_an_accent_in_it_and_the_whole_clip() {
    let t = three_sentences();
    let one =
        |item: PlanItem, hook: Option<&str>| reasons(&run_on(&t, plan_of(vec![item], hook)).1);
    let hook_missing = [(Some(EventKind::HookTitle), DropReason::HookMissing)];
    assert_eq!(one(hook_title(1, 0, at(0, 18)), None), hook_missing);
    // A bad hook is dropped as an extra, and the title with it.
    assert_eq!(
        one(hook_title(1, 0, at(0, 18)), Some("Hours")),
        [
            (None, DropReason::LabelTooLong),
            (Some(EventKind::HookTitle), DropReason::HookMissing)
        ]
    );
    assert_eq!(
        one(hook_title(1, 4, at(0, 18)), Some("One trick saves hours")),
        [(Some(EventKind::HookTitle), DropReason::ParamsMismatch)]
    );
    assert_eq!(
        one(hook_title(1, 0, at(0, 17)), Some("One trick saves hours")),
        [(Some(EventKind::HookTitle), DropReason::AnchorRule)]
    );
}

#[test]
fn a_word_outside_its_anchors_is_dropped_for_each_word_kind() {
    let t = three_sentences();
    let outside = |mut item: PlanItem| {
        item.anchors = at(9, 10);
        reasons(&run_on(&t, plan_of(vec![item], None)).1)
    };
    for (kind, built) in [
        (EventKind::Emphasis, emphasis(1, 1, EmphasisSize::Normal)),
        (EventKind::IconPop, icon(1, 1)),
        (EventKind::Emoji, emoji(1, 1)),
    ] {
        assert_eq!(outside(built), [(Some(kind), DropReason::AnchorRule)]);
    }
}

#[test]
fn v3_a_zoom_fits_its_motion_and_one_sentence_and_a_sentence_has_one() {
    let t = three_sentences();
    let one = |item: PlanItem| reasons(&run_on(&t, plan_of(vec![item], None)).1);
    let anchor_rule = [(Some(EventKind::PunchIn), DropReason::AnchorRule)];
    assert_eq!(
        one(zoom(1, at(0, 4), ZoomMotion::Snap)),
        anchor_rule,
        "4 words"
    );
    assert_eq!(
        one(zoom(1, at(1, 2), ZoomMotion::Push)),
        anchor_rule,
        "1 word"
    );
    assert_eq!(
        one(zoom(1, at(7, 11), ZoomMotion::Push)),
        anchor_rule,
        "two sentences"
    );
    assert_eq!(one(zoom(1, at(12, 18), ZoomMotion::Push)), []);
    // Two in the first sentence, far enough apart for V-6: the second is V-3's.
    let items = vec![
        zoom(1, at(1, 2), ZoomMotion::Snap),
        zoom(2, at(6, 7), ZoomMotion::Snap),
        zoom(3, at(13, 14), ZoomMotion::Snap),
    ];
    let (valid, dropped) = run_on(&t, plan_of(items, None));
    assert_eq!(valid.items().len(), 2);
    assert_eq!(
        reasons(&dropped),
        [(Some(EventKind::PunchIn), DropReason::TooClose)]
    );
}

#[test]
fn d452_a_punch_word_keeps_6_s_from_the_last() {
    let t = three_sentences();
    let items = vec![
        emphasis(1, 1, EmphasisSize::Punch),
        emphasis(2, 4, EmphasisSize::Normal),
        emphasis(3, 9, EmphasisSize::Punch),
        emphasis(4, 12, EmphasisSize::Punch),
    ];
    let (valid, dropped) = run_on(&t, plan_of(items, None));
    let words: Vec<u32> = valid
        .items()
        .iter()
        .map(|i| i.anchors.start.get())
        .collect();
    // Word 9 starts at 6,800 ms, under 6 s after word 1's end (1,500); word 12 at 8,900 is not.
    assert_eq!(words, [1, 4, 12]);
    assert_eq!(
        reasons(&dropped),
        [(Some(EventKind::Emphasis), DropReason::TooClose)]
    );
}

#[test]
fn v9_a_models_visuals_start_2_5_s_apart() {
    let t = three_sentences();
    // An icon at 4,000 ms, an emoji at 5,400 ms: the emoji is dropped.
    let (valid, dropped) = run_on(&t, plan_of(vec![icon(1, 5), emoji(2, 7)], None));
    assert_eq!(valid.items().len(), 1);
    assert_eq!(
        reasons(&dropped),
        [(Some(EventKind::Emoji), DropReason::Density)]
    );
    // The hook title starts with the first word: an icon 1.4 s later is dropped,
    // while a zoom and an emphasis there are not visuals.
    let items = vec![
        hook_title(1, 0, at(0, 18)),
        icon(2, 2),
        zoom(3, at(2, 3), ZoomMotion::Snap),
        emphasis(4, 3, EmphasisSize::Normal),
    ];
    let (valid, dropped) = run_on(&t, plan_of(items, Some("One trick saves hours")));
    assert_eq!(valid.items().len(), 3);
    assert_eq!(
        reasons(&dropped),
        [(Some(EventKind::IconPop), DropReason::Density)]
    );
}

#[test]
fn v9_a_plan_of_the_rules_is_exempt() {
    // "500" at 1,900-2,200 ms and "900" at 4,000-4,300 ms: 2.1 s apart, clear of
    // V-6 (the zone frees at 3,400, the gap ends at 3,700).
    let t = timed(&[
        "We", "made", "500", "dollars", "and", "900", "dollars", "later.",
    ]);
    let detected = |value: f64| {
        ItemParams::Detected(EventParams::NumberReveal {
            value: Quantity {
                value,
                unit: Unit::Usd,
                display: format!("${value}"),
            },
            label: None,
        })
    };
    let number = |id, word: u32, params| PlanItem {
        confidence: Confidence::new(0.9),
        ..item(id, EventKind::NumberReveal, at(word, word + 1), params)
    };
    let mut rules = EditPlan::empty_rules(CATALOG_VERSION);
    rules.items = vec![number(1, 2, detected(500.0)), number(2, 5, detected(900.0))];
    let (valid, dropped) = run_on(&t, rules);
    assert_eq!((valid.items().len(), dropped.len()), (2, 0));
    // The same two numbers from a model: the second is V-9's.
    let pointer = |word: u32| ItemParams::NumberReveal {
        value: QuantityRef {
            words: at(word, word + 1),
        },
        label: None,
    };
    let items = vec![
        item(1, EventKind::NumberReveal, at(2, 3), pointer(2)),
        item(2, EventKind::NumberReveal, at(5, 6), pointer(5)),
    ];
    let (valid, dropped) = run_on(&t, plan_of(items, None));
    assert_eq!(valid.items().len(), 1);
    assert_eq!(
        reasons(&dropped),
        [(Some(EventKind::NumberReveal), DropReason::Density)]
    );
}

// --- D-503 (P206c): a model's valid hook is always shown ----------------------

/// The plan's hook titles: (id, anchors, text, accent).
fn hooks(valid: &offcut_plan::ValidPlan) -> Vec<(u64, WordRange, String, u8)> {
    valid
        .items()
        .iter()
        .filter_map(|item| {
            if let EventParams::HookTitle { text, accent } = &item.params {
                Some((item.id.get(), item.anchors, text.clone(), *accent))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn d503_a_models_valid_hook_with_no_item_gains_one_on_its_longest_word() {
    let t = three_sentences();
    let one = |items: Vec<PlanItem>, hook: &str| run_on(&t, plan_of(items, Some(hook)));
    // The whole clip's words, an id no item has (one more than the largest),
    // nothing reported as dropped.
    let (valid, dropped) = one(
        vec![emphasis(7, 13, EmphasisSize::Normal)],
        "One simple trick saves hours",
    );
    assert_eq!(dropped, []);
    assert_eq!(
        hooks(&valid),
        [(8, at(0, 18), "One simple trick saves hours".to_owned(), 1)]
    );
    assert_eq!(valid.items().len(), 2);
    // The longest word by letters and digits; the first of equal ones.
    for (hook, accent) in [
        ("One trick saves hours", 1),
        ("Saved hours, weekly!", 2),
        ("Hours saved", 0),
    ] {
        assert_eq!(hooks(&one(Vec::new(), hook).0)[0].3, accent, "{hook}");
    }
    // An empty plan: the first id.
    assert_eq!(hooks(&one(Vec::new(), "Hours saved").0)[0].0, 1);
}

#[test]
fn d503_the_added_hook_opens_the_clip_and_v9_drops_a_visual_after_it() {
    let t = three_sentences();
    // As `v9_a_models_visuals_start_2_5_s_apart`, without the model's hook
    // item: the added one drops the icon 1.4 s later, not the zoom or emphasis.
    let items = vec![
        icon(2, 2),
        zoom(3, at(2, 3), ZoomMotion::Snap),
        emphasis(4, 3, EmphasisSize::Normal),
    ];
    let (valid, dropped) = run_on(&t, plan_of(items, Some("One trick saves hours")));
    assert_eq!(valid.items().len(), 3);
    assert_eq!(hooks(&valid).len(), 1);
    assert_eq!(
        reasons(&dropped),
        [(Some(EventKind::IconPop), DropReason::Density)]
    );
    // A visual on the first word sorts before the hook, and gives way to it.
    let (valid, dropped) = run_on(
        &t,
        plan_of(vec![emoji(1, 0)], Some("One trick saves hours")),
    );
    assert_eq!(hooks(&valid).len(), 1);
    assert_eq!(
        reasons(&dropped),
        [(Some(EventKind::Emoji), DropReason::Density)]
    );
}

#[test]
fn d503_a_plan_with_its_own_valid_hook_item_is_unchanged() {
    let t = three_sentences();
    let own = hook_title(1, 3, at(0, 18));
    let (valid, dropped) = run_on(&t, plan_of(vec![own], Some("One trick saves hours")));
    assert_eq!(dropped, []);
    assert_eq!(
        hooks(&valid),
        [(1, at(0, 18), "One trick saves hours".to_owned(), 3)]
    );
}

#[test]
fn d503_a_models_hook_item_that_fails_is_dropped_and_replaced() {
    // TE-15 (P206c): the model wrote the item, with an accent past the hook's
    // last word or anchors short of the clip's end.
    let t = three_sentences();
    for (bad, reason) in [
        (hook_title(5, 4, at(0, 18)), DropReason::ParamsMismatch),
        (hook_title(5, 0, at(0, 17)), DropReason::AnchorRule),
    ] {
        let (valid, dropped) = run_on(&t, plan_of(vec![bad], Some("One trick saves hours")));
        assert_eq!(reasons(&dropped), [(Some(EventKind::HookTitle), reason)]);
        assert_eq!(
            hooks(&valid),
            [(6, at(0, 18), "One trick saves hours".to_owned(), 1)]
        );
    }
}

#[test]
fn d503_no_hook_is_added_without_a_valid_hook_or_to_a_rules_plan() {
    let t = three_sentences();
    // No hook, or one dropped as an extra: nothing to show.
    let (valid, dropped) = run_on(&t, plan_of(vec![icon(1, 12)], None));
    assert_eq!((hooks(&valid).len(), dropped.len()), (0, 0));
    let (valid, dropped) = run_on(&t, plan_of(vec![icon(1, 12)], Some("Hours")));
    assert!(hooks(&valid).is_empty());
    assert_eq!(reasons(&dropped), [(None, DropReason::LabelTooLong)]);
    // A plan of the rules is never given one.
    let mut rules = EditPlan::empty_rules(CATALOG_VERSION);
    rules.extras.hook = Some(label("One trick saves hours"));
    let (valid, _) = run_on(&t, rules);
    assert!(hooks(&valid).is_empty());
}
