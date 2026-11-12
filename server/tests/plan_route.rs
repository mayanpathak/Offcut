#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! `POST /plan` (v6.2implementation §7.2, §7.3, D-460, D-494) on a fake
//! provider and a real Postgres: each step's refusal, and what each one
//! leaves sent, counted and spent.

mod common;

use common::plan::{
    last_month, make_creator, person, plans_counted, post, request, seed_plans, spent, this_month,
};
use common::{FAKE_PLAN, FAKE_USAGE, TestApp, TestOptions, plan_settings};
use offcut_api::plan::provider::{PlanProviderError, Usage};
use offcut_api::plan::spend::SpendCap;
use offcut_api_types::{ApiError, ApiErrorCode, PlanResponse};
use offcut_plan::CATALOG_VERSION;
use offcut_plan::prompt::system_prompt;
use offcut_types::{PLAN_FORMAT, PlanSource};
use reqwest::StatusCode;
use serde_json::{Value, json};

/// One call's cost at the tests' prices.
fn cost() -> i64 {
    SpendCap::new(&plan_settings(5_000_000)).cost(FAKE_USAGE)
}

async fn refused(response: reqwest::Response, status: StatusCode, code: ApiErrorCode) {
    assert_eq!(response.status(), status);
    let body: ApiError = response.json().await.unwrap();
    assert_eq!(body.code, code);
}

fn queue(app: &TestApp, answers: Vec<Result<String, PlanProviderError>>) {
    app.plan.answers.lock().unwrap().extend(answers);
}

fn sent(app: &TestApp) -> usize {
    app.plan.received.lock().unwrap().len()
}

#[tokio::test]
async fn a_plan_is_the_models_answer_with_the_servers_source_counted_and_spent() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    let response = post(&app, Some(&token), &request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body: PlanResponse = response.json().await.unwrap();
    assert_eq!(body.plan.format, PLAN_FORMAT);
    assert_eq!(body.plan.catalog, CATALOG_VERSION);
    assert_eq!(
        body.plan.source,
        PlanSource::Model {
            provider: "fake".to_owned(),
            model: "fake-model".to_owned(),
        }
    );
    assert_eq!(body.plan.items.len(), 1);
    assert_eq!(body.plans_remaining, Some(39));
    assert_eq!(plans_counted(&app, user, this_month()).await, 1);
    assert_eq!(spent(&app, this_month()).await, cost());

    // What the model was sent: the prompt, and the ASR words as the prompt writes them.
    let received = app.plan.received.lock().unwrap();
    let (system, user_text) = &received[0];
    assert_eq!(system, &system_prompt());
    assert!(user_text.starts_with("DURATION 4000\n0\tI\t100\t200\t0.5\t-0.5\n"));
    assert!(user_text.contains("2\t3000\t650\t1100\t0.5\t-0.5\n"));
    assert!(user_text.contains("PAUSE 500\n5\tIt\t2600"));
}

/// D-460 as read: the server sets `format`, `catalog` and `source` and never
/// reads them from the model, so an answer claiming catalog 1 and the rules
/// still comes back as this catalog's model plan.
#[tokio::test]
async fn the_server_sets_the_source_whatever_the_model_wrote() {
    let app = TestApp::spawn().await;
    let (_, token) = person(&app).await;
    let mut claimed: Value = serde_json::from_str(FAKE_PLAN).unwrap();
    claimed["catalog"] = json!(1);
    claimed["format"] = json!(7);
    claimed["source"] = json!({ "kind": "rules" });
    queue(&app, vec![Ok(claimed.to_string())]);
    let body: PlanResponse = post(&app, Some(&token), &request())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body.plan.catalog, CATALOG_VERSION);
    assert_eq!(body.plan.format, PLAN_FORMAT);
    assert!(matches!(body.plan.source, PlanSource::Model { .. }));
}

#[tokio::test]
async fn without_a_bearer_it_is_401_before_the_body_and_nothing_is_sent_or_counted() {
    let app = TestApp::spawn().await;
    let (user, _) = person(&app).await;
    // Not even JSON: a 401, not a 400, so the body was not read.
    let response = app
        .client
        .post(format!("{}/plan", app.base_url))
        .header("content-type", "application/json")
        .body("{ not json")
        .send()
        .await
        .unwrap();
    refused(
        response,
        StatusCode::UNAUTHORIZED,
        ApiErrorCode::Unauthorized,
    )
    .await;
    refused(
        post(&app, Some("not.a-token"), &request()).await,
        StatusCode::UNAUTHORIZED,
        ApiErrorCode::Unauthorized,
    )
    .await;
    assert_eq!(sent(&app), 0);
    assert_eq!(plans_counted(&app, user, this_month()).await, 0);
    assert_eq!(spent(&app, this_month()).await, 0);
}

#[tokio::test]
async fn each_check_before_the_provider_refuses_without_sending_or_counting() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;

    // 2. A body that is not a plan request.
    let mut unknown = serde_json::to_value(request()).unwrap();
    unknown["edits"] = json!({ "3": "3,000" });
    refused(
        post(&app, Some(&token), &unknown).await,
        StatusCode::BAD_REQUEST,
        ApiErrorCode::BadRequest,
    )
    .await;
    // 3. Another catalog.
    let mut old = request();
    old.catalog = CATALOG_VERSION - 1;
    refused(
        post(&app, Some(&token), &old).await,
        StatusCode::CONFLICT,
        ApiErrorCode::PlanCatalog,
    )
    .await;
    // 6. A transcript that is not one: a tab would break the user message's lines.
    let mut tab = request();
    tab.words[1].text = "spent\t9".to_owned();
    refused(
        post(&app, Some(&token), &tab).await,
        StatusCode::BAD_REQUEST,
        ApiErrorCode::BadRequest,
    )
    .await;

    assert_eq!(sent(&app), 0);
    assert_eq!(plans_counted(&app, user, this_month()).await, 0);
}

#[tokio::test]
async fn with_the_switch_off_it_is_unavailable_and_nothing_is_sent() {
    let app = TestApp::spawn_with(TestOptions {
        plan_enabled: false,
        ..TestOptions::default()
    })
    .await;
    let (_, token) = person(&app).await;
    refused(
        post(&app, Some(&token), &request()).await,
        StatusCode::BAD_GATEWAY,
        ApiErrorCode::PlanUnavailable,
    )
    .await;
    assert_eq!(sent(&app), 0);
}

#[tokio::test]
async fn a_deleted_account_with_a_live_token_is_401_before_the_provider() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.get())
        .execute(&app.pool)
        .await
        .unwrap();
    refused(
        post(&app, Some(&token), &request()).await,
        StatusCode::UNAUTHORIZED,
        ApiErrorCode::Unauthorized,
    )
    .await;
    assert_eq!(sent(&app), 0);
}

#[tokio::test]
async fn the_41st_free_plan_of_a_month_is_429_and_last_months_count_does_not_matter() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    seed_plans(&app, user, last_month(), 40).await;
    seed_plans(&app, user, this_month(), 39).await;
    let body: PlanResponse = post(&app, Some(&token), &request())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body.plans_remaining, Some(0), "the 40th");
    refused(
        post(&app, Some(&token), &request()).await,
        StatusCode::TOO_MANY_REQUESTS,
        ApiErrorCode::PlanQuota,
    )
    .await;
    assert_eq!(sent(&app), 1);
    assert_eq!(plans_counted(&app, user, this_month()).await, 40);
}

#[tokio::test]
async fn a_creators_3001st_plan_is_blocked_and_the_count_is_not_shown() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    make_creator(&app, user).await;
    seed_plans(&app, user, this_month(), 2_999).await;
    let body: PlanResponse = post(&app, Some(&token), &request())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body.plans_remaining, None);
    refused(
        post(&app, Some(&token), &request()).await,
        StatusCode::TOO_MANY_REQUESTS,
        ApiErrorCode::PlanQuota,
    )
    .await;
    assert_eq!(plans_counted(&app, user, this_month()).await, 3_000);
}

#[tokio::test]
async fn an_answer_that_does_not_parse_is_retried_once_with_the_error() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    queue(&app, vec![Ok("{\"items\": [".to_owned())]);
    let response = post(&app, Some(&token), &request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    let received = app.plan.received.lock().unwrap().clone();
    assert_eq!(received.len(), 2);
    assert!(!received[0].1.contains("could not be read"));
    assert!(received[1].1.starts_with(&received[0].1));
    assert!(
        received[1]
            .1
            .contains("Your last answer could not be read as the plan")
    );
    assert_eq!(
        plans_counted(&app, user, this_month()).await,
        1,
        "one plan, counted once"
    );
    assert_eq!(
        spent(&app, this_month()).await,
        2 * cost(),
        "both calls were paid for"
    );
}

#[tokio::test]
async fn two_answers_that_do_not_parse_are_plan_invalid_spent_and_not_counted() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    queue(&app, vec![Ok(String::new()), Ok("[1, 2]".to_owned())]);
    refused(
        post(&app, Some(&token), &request()).await,
        StatusCode::BAD_GATEWAY,
        ApiErrorCode::PlanInvalid,
    )
    .await;
    assert_eq!(sent(&app), 2);
    assert_eq!(plans_counted(&app, user, this_month()).await, 0);
    assert_eq!(spent(&app, this_month()).await, 2 * cost());
}

/// Checkpoint 1: an item naming an emoji outside the 72 is left out; the plan stands.
#[tokio::test]
async fn an_item_with_an_unknown_name_is_left_out_and_the_rest_comes_back() {
    let app = TestApp::spawn().await;
    let (_, token) = person(&app).await;
    let mut plan: Value = serde_json::from_str(FAKE_PLAN).unwrap();
    let emoji = json!({ "id": "2", "kind": "emoji", "anchors": { "start": 7, "end": 8 },
        "params": { "kind": "emoji", "params": { "word": 7, "emoji": "cat", "place": "under" } },
        "confidence": null });
    plan["items"].as_array_mut().unwrap().push(emoji);
    queue(&app, vec![Ok(plan.to_string())]);
    let body: PlanResponse = post(&app, Some(&token), &request())
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body.plan.items.len(), 1);
    assert_eq!(sent(&app), 1, "no retry for one bad item");
}

#[tokio::test]
async fn each_provider_failure_is_unavailable_and_not_counted() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    let blocked = Usage {
        input_tokens: 1_000_000,
        cached_input_tokens: 0,
        output_tokens: 0,
    };
    queue(
        &app,
        vec![
            Err(PlanProviderError::Unavailable),
            Err(PlanProviderError::Timeout),
            Err(PlanProviderError::Blocked(blocked)),
        ],
    );
    let token = Some(token.as_str());
    refused(
        post(&app, token, &request()).await,
        StatusCode::BAD_GATEWAY,
        ApiErrorCode::PlanUnavailable,
    )
    .await;
    refused(
        post(&app, token, &request()).await,
        StatusCode::GATEWAY_TIMEOUT,
        ApiErrorCode::PlanUnavailable,
    )
    .await;
    refused(
        post(&app, token, &request()).await,
        StatusCode::BAD_GATEWAY,
        ApiErrorCode::PlanUnavailable,
    )
    .await;
    assert_eq!(sent(&app), 3, "no retry for a provider failure");
    assert_eq!(plans_counted(&app, user, this_month()).await, 0);
    // Only the blocked answer was billed: a million input tokens at 0.30 USD.
    assert_eq!(spent(&app, this_month()).await, 300_000);
}

#[tokio::test]
async fn an_answer_over_32_kb_is_plan_invalid_and_not_counted() {
    let app = TestApp::spawn().await;
    let (user, token) = person(&app).await;
    let mut plan: Value = serde_json::from_str(FAKE_PLAN).unwrap();
    plan["extras"]["post_caption"] = json!("a".repeat(33 * 1024));
    queue(&app, vec![Ok(plan.to_string())]);
    refused(
        post(&app, Some(&token), &request()).await,
        StatusCode::BAD_GATEWAY,
        ApiErrorCode::PlanInvalid,
    )
    .await;
    assert_eq!(plans_counted(&app, user, this_month()).await, 0);
}

/// D-461: `PlanUser` follows the person across addresses; `PlanIp` holds an
/// address before any bearer is read.
#[tokio::test]
async fn ten_a_minute_for_a_person_and_ten_for_an_address() {
    let app = TestApp::spawn().await;
    let (_, token) = person(&app).await;
    let from = |ip: &str| {
        app.client
            .post(format!("{}/plan", app.base_url))
            .header("x-forwarded-for", ip.to_owned())
            .bearer_auth(&token)
            .json(&request())
    };
    for n in 0..10 {
        let ip = format!("198.51.100.{n}");
        assert_eq!(
            from(&ip).send().await.unwrap().status(),
            StatusCode::OK,
            "plan {n}"
        );
    }
    let response = from("198.51.100.99").send().await.unwrap();
    refused(
        response,
        StatusCode::TOO_MANY_REQUESTS,
        ApiErrorCode::RateLimited,
    )
    .await;

    for n in 0..10 {
        let response = app
            .client
            .post(format!("{}/plan", app.base_url))
            .header("x-forwarded-for", "203.0.113.50")
            .json(&request())
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "request {n}");
    }
    let response = app
        .client
        .post(format!("{}/plan", app.base_url))
        .header("x-forwarded-for", "203.0.113.50")
        .bearer_auth(&token)
        .json(&request())
        .send()
        .await
        .unwrap();
    refused(
        response,
        StatusCode::TOO_MANY_REQUESTS,
        ApiErrorCode::RateLimited,
    )
    .await;
}
