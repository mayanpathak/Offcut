#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! TS §22.5's table, with v6implementation D-276's and D-301's rows: every
//! (from, kind) cell, through the real route and a real database. The expected
//! rows are written here by hand, apart from the code.

mod common;

use common::{TestApp, webhook};
use reqwest::StatusCode;
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

const BEFORE: i64 = 1_760_000_000;
const AT: i64 = BEFORE + 100;
const OLD_END: i64 = BEFORE + 1_000_000;
const NEW_END: i64 = BEFORE + 2_000_000;

/// A subscription row as the tests read it, with its times in seconds.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    customer: String,
    subscription: String,
    interval: String,
    status: String,
    cancel_at_period_end: bool,
    period_end: i64,
    last_event_at: i64,
}

fn seeded(status: &str, cancel_at_period_end: bool) -> Row {
    Row {
        customer: "cus_old".to_owned(),
        subscription: "sub_old".to_owned(),
        interval: "monthly".to_owned(),
        status: status.to_owned(),
        cancel_at_period_end,
        period_end: OLD_END,
        last_event_at: BEFORE,
    }
}

async fn new_user(app: &TestApp) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, email, email_normalized) VALUES ($1, $2, $2)")
        .bind(id)
        .bind(format!("{id}@example.com"))
        .execute(&app.pool)
        .await
        .unwrap();
    id
}

fn at(secs: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(secs).unwrap()
}

/// `provider_subscription_id` is unique, so each seeded row's id is made unique
/// here and read back through `row_of`.
async fn seed(app: &TestApp, user: Uuid, row: &Row) {
    sqlx::query(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, cancel_at_period_end, current_period_end, last_event_at)
         VALUES ($1, $2, $3, 'creator', $4, $5, $6, $7, $8)",
    )
    .bind(user)
    .bind(format!("{}_{user}", row.customer))
    .bind(format!("{}_{user}", row.subscription))
    .bind(&row.interval)
    .bind(&row.status)
    .bind(row.cancel_at_period_end)
    .bind(at(row.period_end))
    .bind(at(row.last_event_at))
    .execute(&app.pool)
    .await
    .unwrap();
}

async fn row_of(app: &TestApp, user: Uuid) -> Option<Row> {
    let row: Option<(
        String,
        String,
        String,
        String,
        bool,
        OffsetDateTime,
        OffsetDateTime,
    )> = sqlx::query_as(
        "SELECT provider_customer_id, provider_subscription_id, billing_interval, status,
                    cancel_at_period_end, current_period_end, last_event_at
             FROM subscriptions WHERE user_id = $1",
    )
    .bind(user)
    .fetch_optional(&app.pool)
    .await
    .unwrap();
    let suffix = format!("_{user}");
    row.map(|r| Row {
        customer: r.0.trim_end_matches(&suffix).to_owned(),
        subscription: r.1.trim_end_matches(&suffix).to_owned(),
        interval: r.2,
        status: r.3,
        cancel_at_period_end: r.4,
        period_end: r.5.unix_timestamp(),
        last_event_at: r.6.unix_timestamp(),
    })
}

/// An event about `user`, from the merchant, carrying everything a kind can.
fn event(user: Uuid, kind: &str) -> Value {
    json!({
        "id": format!("evt_{}", Uuid::now_v7()),
        "user": user,
        "customer": format!("cus_new_{user}"),
        "subscription": format!("sub_old_{user}"),
        "kind": kind,
        "interval": "annual",
        "period_end": NEW_END,
        "at": AT,
    })
}

async fn send(app: &TestApp, body: &Value) {
    let response = webhook(app, body).await;
    assert_eq!(response.status(), StatusCode::OK, "{body}");
}

const KINDS: [&str; 8] = [
    "activated",
    "renewed",
    "past_due",
    "cancel_scheduled",
    "canceled",
    "refunded",
    "resumed",
    "plan_changed",
];

/// What a cell should leave, from a row with this state. `None` is "ignored".
fn expected(from: Option<&Row>, kind: &str) -> Option<Row> {
    let moved = |change: &dyn Fn(&mut Row)| {
        let mut row = from.expect("a cell with a row").clone();
        row.last_event_at = AT;
        change(&mut row);
        Some(row)
    };
    let status = from.map(|row| row.status.as_str());
    match (status, kind) {
        // none, canceled + Activated, and D-276's past_due + Activated: everything set.
        (None | Some("canceled" | "past_due"), "activated") => Some(Row {
            customer: "cus_new".to_owned(),
            subscription: "sub_old".to_owned(),
            interval: "annual".to_owned(),
            status: "active".to_owned(),
            cancel_at_period_end: false,
            period_end: NEW_END,
            last_event_at: AT,
        }),
        (Some("active" | "past_due"), "renewed") => moved(&|row| {
            row.status = "active".to_owned();
            row.period_end = NEW_END;
        }),
        (Some("active"), "past_due") => moved(&|row| row.status = "past_due".to_owned()),
        (Some("active" | "past_due"), "cancel_scheduled") => {
            moved(&|row| row.cancel_at_period_end = true)
        }
        (Some("active" | "past_due"), "canceled") => moved(&|row| {
            row.status = "canceled".to_owned();
            row.period_end = NEW_END;
        }),
        // "any" row; the refund's time is the period's end.
        (Some(_), "refunded") => moved(&|row| {
            row.status = "canceled".to_owned();
            row.period_end = AT;
        }),
        // D-301.
        (Some("active" | "past_due"), "resumed") => moved(&|row| row.cancel_at_period_end = false),
        (Some("active" | "past_due"), "plan_changed") => moved(&|row| {
            row.interval = "annual".to_owned();
            row.period_end = NEW_END;
        }),
        _ => None,
    }
}

#[tokio::test]
async fn every_cell_of_the_table() {
    let app = TestApp::spawn().await;
    let mut applied = 0;
    let mut ignored = 0;

    for from in [None, Some("active"), Some("past_due"), Some("canceled")] {
        for kind in KINDS {
            let user = new_user(&app).await;
            let before = from.map(|status| seeded(status, true));
            if let Some(row) = &before {
                seed(&app, user, row).await;
            }
            send(&app, &event(user, kind)).await;

            let cell = format!("from {from:?}, {kind}");
            let want = expected(before.as_ref(), kind);
            let got = row_of(&app, user).await;
            match want {
                Some(row) => {
                    assert_eq!(got.as_ref(), Some(&row), "{cell}");
                    applied += 1;
                }
                // An ignored cell changes nothing at all.
                None => {
                    assert_eq!(got, before, "{cell}");
                    ignored += 1;
                }
            }
        }
    }
    // The table's size: a wrong `expected` would move these.
    assert_eq!((applied, ignored), (17, 15));
}

#[tokio::test]
async fn resumed_clears_the_scheduled_cancel_and_cancel_scheduled_sets_it() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    seed(&app, user, &seeded("active", false)).await;

    send(&app, &event(user, "cancel_scheduled")).await;
    assert!(row_of(&app, user).await.unwrap().cancel_at_period_end);
    send(&app, &event(user, "resumed")).await;
    let row = row_of(&app, user).await.unwrap();
    assert!(!row.cancel_at_period_end);
    assert_eq!(row.status, "active");
}

#[tokio::test]
async fn an_event_older_than_the_last_one_applied_is_ignored() {
    let app = TestApp::spawn().await;
    for kind in KINDS {
        let user = new_user(&app).await;
        let before = seeded("past_due", false);
        seed(&app, user, &before).await;
        let mut old = event(user, kind);
        old["at"] = json!(BEFORE - 1);
        send(&app, &old).await;
        assert_eq!(row_of(&app, user).await.unwrap(), before, "{kind}");
    }
}

#[tokio::test]
async fn an_event_at_the_same_second_as_the_last_one_is_applied() {
    // A first payment sends several events in one second (TS §22.5: "older than").
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    seed(&app, user, &seeded("active", false)).await;
    let mut same = event(user, "renewed");
    same["at"] = json!(BEFORE);
    send(&app, &same).await;
    assert_eq!(row_of(&app, user).await.unwrap().period_end, NEW_END);
}

/// D-299: `Canceled` for an old subscription must not end a new one.
#[tokio::test]
async fn an_event_for_another_subscription_is_ignored_except_activated() {
    let app = TestApp::spawn().await;
    for kind in KINDS {
        if kind == "activated" {
            continue;
        }
        let user = new_user(&app).await;
        let before = seeded("active", false);
        seed(&app, user, &before).await;
        let mut other = event(user, kind);
        other["subscription"] = json!("sub_an_older_one");
        send(&app, &other).await;
        assert_eq!(row_of(&app, user).await.unwrap(), before, "{kind}");
    }

    // A person whose old subscription ended buys again: the new one takes over.
    let user = new_user(&app).await;
    seed(&app, user, &seeded("canceled", false)).await;
    let mut again = event(user, "activated");
    again["subscription"] = json!(format!("sub_new_{user}"));
    send(&app, &again).await;
    let row = row_of(&app, user).await.unwrap();
    assert_eq!(row.status, "active");
    assert_eq!(row.subscription, "sub_new");
}

#[tokio::test]
async fn a_refund_that_names_no_subscription_still_applies() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    seed(&app, user, &seeded("active", false)).await;
    let mut refund = event(user, "refunded");
    refund["subscription"] = Value::Null;
    send(&app, &refund).await;
    let row = row_of(&app, user).await.unwrap();
    assert_eq!((row.status.as_str(), row.period_end), ("canceled", AT));
}

/// D-275: a refund may name only the customer.
#[tokio::test]
async fn an_event_naming_only_the_customer_finds_its_user_through_the_subscription() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let other = new_user(&app).await;
    seed(&app, user, &seeded("active", false)).await;
    seed(&app, other, &seeded("active", false)).await;

    let mut refund = event(user, "refunded");
    refund["user"] = Value::Null;
    refund["customer_owner"] = json!(format!("cus_old_{user}"));
    refund["customer"] = json!(format!("cus_old_{user}"));
    refund["subscription"] = Value::Null;
    send(&app, &refund).await;

    assert_eq!(row_of(&app, user).await.unwrap().status, "canceled");
    assert_eq!(row_of(&app, other).await.unwrap().status, "active");
}

/// D-284: nothing to apply it to, and nothing for the merchant to retry.
#[tokio::test]
async fn an_event_for_an_unknown_customer_is_200_and_changes_nothing() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let before = seeded("active", false);
    seed(&app, user, &before).await;
    let mut refund = event(user, "refunded");
    refund["user"] = Value::Null;
    refund["customer_owner"] = json!("cus_nobody");
    send(&app, &refund).await;
    assert_eq!(row_of(&app, user).await.unwrap(), before);
}

#[tokio::test]
async fn a_kind_missing_what_it_needs_is_ignored() {
    let app = TestApp::spawn().await;
    for (from, kind, field) in [
        (None, "activated", "period_end"),
        (None, "activated", "interval"),
        (None, "activated", "subscription"),
        (Some("active"), "renewed", "period_end"),
    ] {
        let user = new_user(&app).await;
        let before = from.map(|status| seeded(status, false));
        if let Some(row) = &before {
            seed(&app, user, row).await;
        }
        let mut body = event(user, kind);
        body[field] = Value::Null;
        send(&app, &body).await;
        assert_eq!(row_of(&app, user).await, before, "{kind} with no {field}");
    }
}

#[tokio::test]
async fn a_cancel_or_a_plan_change_that_does_not_say_a_date_keeps_the_one_there_is() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    seed(&app, user, &seeded("active", false)).await;
    let mut change = event(user, "plan_changed");
    change["period_end"] = Value::Null;
    send(&app, &change).await;
    let row = row_of(&app, user).await.unwrap();
    assert_eq!((row.interval.as_str(), row.period_end), ("annual", OLD_END));

    let mut cancel = event(user, "canceled");
    cancel["period_end"] = Value::Null;
    cancel["at"] = json!(AT + 1);
    send(&app, &cancel).await;
    let row = row_of(&app, user).await.unwrap();
    assert_eq!((row.status.as_str(), row.period_end), ("canceled", OLD_END));
}

/// `subscription.updated` fires on any change. With no cancel scheduled it says
/// nothing, and must not move `last_event_at`.
#[tokio::test]
async fn resumed_with_no_scheduled_cancel_changes_nothing_at_all() {
    let app = TestApp::spawn().await;
    let user = new_user(&app).await;
    let before = seeded("active", false);
    seed(&app, user, &before).await;
    send(&app, &event(user, "resumed")).await;
    assert_eq!(row_of(&app, user).await.unwrap(), before);
}
