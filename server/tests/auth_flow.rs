//! Sign-in end to end, against a real Postgres with a fake mailer (TS §22.1,
//! §22.2; v6implementation §7, §15.1).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use std::sync::atomic::Ordering;

use common::TestApp;
use ed25519_dalek::SigningKey;
use offcut_api::auth::access_token;
use offcut_api::auth::session::token_hash;
use offcut_api_types::{
    ApiError, ApiErrorCode, BillingInterval, MeResponse, SessionResponse, SubscriptionStatus,
};
use offcut_entitlement::verify_token;
use offcut_types::{Plan, UnixSecs};
use reqwest::{Response, StatusCode};
use serde_json::json;
use time::OffsetDateTime;

const EMAIL: &str = "Someone@Example.com ";
const NORMALIZED: &str = "someone@example.com";

/// The test app's keys (`test_config`).
fn access_key() -> SigningKey {
    SigningKey::from_bytes(&[2; 32])
}

fn entitlement_key() -> SigningKey {
    SigningKey::from_bytes(&[1; 32])
}

fn now() -> UnixSecs {
    UnixSecs::new(OffsetDateTime::now_utc().unix_timestamp())
}

async fn request_link(app: &TestApp, email: &str) -> Response {
    let request = app.client.post(format!("{}/auth/magic-link", app.base_url));
    request
        .json(&json!({ "email": email }))
        .send()
        .await
        .unwrap()
}

async fn post_verify(app: &TestApp, token: &str) -> Response {
    let request = app.client.post(format!("{}/auth/verify", app.base_url));
    request
        .json(&json!({ "token": token }))
        .send()
        .await
        .unwrap()
}

/// The token of the last link the fake mailer was given.
fn last_token(app: &TestApp) -> String {
    let sent = app.mailer.sent.lock().unwrap();
    let (_, link) = sent.last().expect("a mail was sent");
    link.rsplit_once("token=").unwrap().1.to_owned()
}

async fn signed_up(app: &TestApp) -> String {
    let response = request_link(app, EMAIL).await;
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    last_token(app)
}

async fn error_code(response: Response) -> ApiErrorCode {
    response.json::<ApiError>().await.unwrap().code
}

async fn count(app: &TestApp, table: &str) -> i64 {
    let sql = format!("SELECT count(*) FROM {table}");
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(&app.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_link_is_mailed_to_the_normalized_address_and_only_its_hash_is_stored() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;

    let sent = app.mailer.sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, NORMALIZED);
    assert_eq!(
        sent[0].1,
        format!("http://localhost:5173/auth/callback?token={token}")
    );
    assert_eq!(token.len(), 43);

    let (hash, email): (Vec<u8>, String) =
        sqlx::query_as("SELECT token_hash, email_normalized FROM magic_links")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(hash, token_hash(&token));
    assert_ne!(hash, token.as_bytes());
    assert_eq!(email, NORMALIZED);
}

#[tokio::test]
async fn a_request_answers_alike_for_a_known_and_an_unknown_address() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    assert_eq!(post_verify(&app, &token).await.status(), StatusCode::OK);

    let known = request_link(&app, NORMALIZED).await;
    let unknown = request_link(&app, "nobody@example.com").await;
    assert_eq!(known.status(), StatusCode::ACCEPTED);
    assert_eq!(unknown.status(), StatusCode::ACCEPTED);
    assert_eq!(known.text().await.unwrap(), "");
    assert_eq!(unknown.text().await.unwrap(), "");
    assert_eq!(count(&app, "users").await, 1);
}

#[tokio::test]
async fn what_is_not_an_address_is_a_bad_request_and_sends_nothing() {
    let app = TestApp::spawn().await;
    for email in ["", "no-at-sign", "two@@example.com"] {
        let response = request_link(&app, email).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{email}");
    }
    assert!(app.mailer.sent.lock().unwrap().is_empty());
    assert_eq!(count(&app, "magic_links").await, 0);
}

#[tokio::test]
async fn a_second_request_spends_the_first_link() {
    let app = TestApp::spawn().await;
    let first = signed_up(&app).await;
    let second = signed_up(&app).await;
    assert_ne!(first, second);

    let old = post_verify(&app, &first).await;
    assert_eq!(old.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(old).await, ApiErrorCode::LinkInvalid);
    assert_eq!(post_verify(&app, &second).await.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_mail_that_fails_is_503_and_leaves_no_link() {
    let app = TestApp::spawn().await;
    app.mailer.fail.store(true, Ordering::SeqCst);
    let response = request_link(&app, EMAIL).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error_code(response).await, ApiErrorCode::EmailUnavailable);
    assert_eq!(count(&app, "magic_links").await, 0);
}

#[tokio::test]
async fn an_expired_link_says_so_and_makes_nothing() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    sqlx::query("UPDATE magic_links SET expires_at = now() - interval '1 second'")
        .execute(&app.pool)
        .await
        .unwrap();

    let response = post_verify(&app, &token).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(response).await, ApiErrorCode::LinkExpired);
    assert_eq!(count(&app, "users").await, 0);
    assert_eq!(count(&app, "sessions").await, 0);
}

#[tokio::test]
async fn an_unknown_or_used_link_is_invalid() {
    let app = TestApp::spawn().await;
    let unknown = post_verify(&app, "never-issued").await;
    assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(unknown).await, ApiErrorCode::LinkInvalid);

    let token = signed_up(&app).await;
    assert_eq!(post_verify(&app, &token).await.status(), StatusCode::OK);
    let again = post_verify(&app, &token).await;
    assert_eq!(again.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error_code(again).await, ApiErrorCode::LinkInvalid);
    assert_eq!(count(&app, "sessions").await, 1);
}

#[tokio::test]
async fn two_racing_verifies_of_one_link_give_one_success() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    let (one, two) = tokio::join!(post_verify(&app, &token), post_verify(&app, &token));
    let mut statuses = [one.status(), two.status()];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::BAD_REQUEST]);
    assert_eq!(count(&app, "users").await, 1);
    assert_eq!(count(&app, "sessions").await, 1);
}

#[tokio::test]
async fn verify_makes_the_user_once_and_finds_it_after_that() {
    let app = TestApp::spawn().await;
    let first = signed_up(&app).await;
    assert_eq!(post_verify(&app, &first).await.status(), StatusCode::OK);
    let second = signed_up(&app).await;
    assert_eq!(post_verify(&app, &second).await.status(), StatusCode::OK);

    assert_eq!(count(&app, "users").await, 1);
    assert_eq!(count(&app, "sessions").await, 2);
    // The address as normalized: the link keeps no other (D-260).
    let (email, normalized): (String, String) =
        sqlx::query_as("SELECT email, email_normalized FROM users")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(
        (email.as_str(), normalized.as_str()),
        (NORMALIZED, NORMALIZED)
    );
}

#[tokio::test]
async fn verify_answers_a_cookie_an_access_token_and_a_free_token() {
    let app = TestApp::spawn().await;
    let token = signed_up(&app).await;
    let response = post_verify(&app, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");

    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .to_owned();
    let refresh = cookie
        .strip_prefix("offcut_rt=")
        .and_then(|rest| rest.split_once(';'))
        .unwrap()
        .0
        .to_owned();
    assert_eq!(
        cookie,
        format!(
            "offcut_rt={refresh}; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=5184000"
        )
    );

    // Only the refresh token's hash is stored.
    let (hash,): (Vec<u8>,) = sqlx::query_as("SELECT refresh_token_hash FROM sessions")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(hash, token_hash(&refresh));

    let body: SessionResponse = response.json().await.unwrap();
    let claims =
        access_token::verify(&body.access_token, &access_key().verifying_key(), now()).unwrap();
    assert_eq!(claims.exp, body.access_expires_at);
    let left = claims.exp.get() - now().get();
    assert!((890..=900).contains(&left), "{left}");

    let public = entitlement_key().verifying_key().to_bytes();
    let entitlement = verify_token(&body.entitlement_token, &[public]).unwrap();
    assert_eq!(entitlement.sub, claims.sub);
    assert_eq!(entitlement.plan, Plan::Free);
    assert_eq!(entitlement.free_exports_remaining.get(), 3);
    assert_eq!(entitlement.exp.get() - entitlement.iat.get(), 7 * 24 * 3600);
}

#[tokio::test]
async fn the_routes_take_small_json_bodies_only() {
    let app = TestApp::spawn().await;
    let long = "a".repeat(2_000);
    let response = post_verify(&app, &long).await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let request = app.client.post(format!("{}/auth/verify", app.base_url));
    let unknown_field = json!({ "token": "x", "extra": 1 });
    let response = request.json(&unknown_field).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// --- Sessions: the table of v6implementation §7.3 --------------------------

const ORIGIN: &str = "http://localhost:5173";
const CLEARED: &str = "offcut_rt=; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=0";

/// A session, signed in through the real routes: its access token and its
/// refresh token.
async fn signed_in(app: &TestApp) -> (SessionResponse, String) {
    let token = signed_up(app).await;
    let response = post_verify(app, &token).await;
    assert_eq!(response.status(), StatusCode::OK);
    let refresh = refresh_of(&response).expect("verify sets the cookie");
    (response.json().await.unwrap(), refresh)
}

/// The refresh token a response set, if it set one.
fn refresh_of(response: &Response) -> Option<String> {
    let cookie = response.headers().get("set-cookie")?.to_str().unwrap();
    let (token, _) = cookie.strip_prefix("offcut_rt=")?.split_once(';')?;
    Some(token.to_owned())
}

fn set_cookie(response: &Response) -> Option<String> {
    let cookie = response.headers().get("set-cookie")?;
    Some(cookie.to_str().unwrap().to_owned())
}

async fn post_refresh(app: &TestApp, refresh: &str) -> Response {
    app.client
        .post(format!("{}/auth/refresh", app.base_url))
        .header("cookie", format!("offcut_rt={refresh}"))
        .header("x-offcut-csrf", "1")
        .header("origin", ORIGIN)
        .send()
        .await
        .unwrap()
}

async fn post_logout(app: &TestApp, refresh: &str) -> Response {
    app.client
        .post(format!("{}/auth/logout", app.base_url))
        .header("cookie", format!("offcut_rt={refresh}"))
        .header("x-offcut-csrf", "1")
        .header("origin", ORIGIN)
        .send()
        .await
        .unwrap()
}

async fn get_me(app: &TestApp, access: &str) -> Response {
    app.client
        .get(format!("{}/me", app.base_url))
        .bearer_auth(access)
        .send()
        .await
        .unwrap()
}

/// What the tests read of a session's row.
#[derive(Debug, PartialEq, sqlx::FromRow)]
struct SessionRow {
    refresh_token_hash: Vec<u8>,
    prev_token_hash: Option<Vec<u8>>,
    prev_valid_until: Option<OffsetDateTime>,
    last_used_at: OffsetDateTime,
    revoked_at: Option<OffsetDateTime>,
}

async fn session_row(app: &TestApp) -> SessionRow {
    sqlx::query_as(
        "SELECT refresh_token_hash, prev_token_hash, prev_valid_until, last_used_at, revoked_at
         FROM sessions",
    )
    .fetch_one(&app.pool)
    .await
    .unwrap()
}

/// The previous token's 30 seconds are over: moves the row's time, with no sleep.
async fn grace_is_over(app: &TestApp) {
    sqlx::query("UPDATE sessions SET prev_valid_until = now() - interval '1 second'")
        .execute(&app.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn row_1_the_current_token_rotates_and_the_old_one_becomes_previous() {
    let app = TestApp::spawn().await;
    let (_, first) = signed_in(&app).await;
    let before = session_row(&app).await;
    assert_eq!(before.refresh_token_hash, token_hash(&first));
    assert_eq!(before.prev_token_hash, None);

    let started = OffsetDateTime::now_utc();
    let response = post_refresh(&app, &first).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let second = refresh_of(&response).expect("a new cookie");
    assert_ne!(second, first);
    assert_eq!(
        set_cookie(&response).unwrap(),
        format!(
            "offcut_rt={second}; HttpOnly; Secure; SameSite=Strict; Path=/api/v1/auth; Max-Age=5184000"
        )
    );

    let row = session_row(&app).await;
    assert_eq!(row.refresh_token_hash, token_hash(&second));
    assert_eq!(row.prev_token_hash, Some(token_hash(&first).to_vec()));
    let grace = row.prev_valid_until.unwrap() - started;
    assert!(
        (29..=31).contains(&grace.whole_seconds()),
        "{grace}: the previous token works for 30 s"
    );
    assert!(row.last_used_at >= before.last_used_at);
    assert_eq!(row.revoked_at, None);

    // A session's answer: an access token for the same user, and a good token.
    let body: SessionResponse = response.json().await.unwrap();
    let claims =
        access_token::verify(&body.access_token, &access_key().verifying_key(), now()).unwrap();
    assert_eq!(claims.exp, body.access_expires_at);
    let public = entitlement_key().verifying_key().to_bytes();
    let entitlement = verify_token(&body.entitlement_token, &[public]).unwrap();
    assert_eq!(entitlement.sub, claims.sub);
}

#[tokio::test]
async fn after_a_rotation_the_new_cookie_refreshes_again() {
    let app = TestApp::spawn().await;
    let (_, mut refresh) = signed_in(&app).await;
    for _ in 0..3 {
        let response = post_refresh(&app, &refresh).await;
        assert_eq!(response.status(), StatusCode::OK);
        refresh = refresh_of(&response).expect("a new cookie");
    }
    assert_eq!(session_row(&app).await.revoked_at, None);
}

#[tokio::test]
async fn row_2_the_previous_token_in_its_grace_rotates_and_leaves_the_previous_as_it_was() {
    let app = TestApp::spawn().await;
    let (_, first) = signed_in(&app).await;
    let response = post_refresh(&app, &first).await;
    let second = refresh_of(&response).unwrap();
    let after_first = session_row(&app).await;

    // As after a lost response: the browser still holds the first token.
    let response = post_refresh(&app, &first).await;
    assert_eq!(response.status(), StatusCode::OK);
    let third = refresh_of(&response).expect("a retry must get a new cookie");
    assert_ne!(third, first);
    assert_ne!(third, second);

    let row = session_row(&app).await;
    assert_eq!(row.refresh_token_hash, token_hash(&third));
    assert_eq!(row.prev_token_hash, after_first.prev_token_hash);
    assert_eq!(row.prev_valid_until, after_first.prev_valid_until);
    assert_eq!(row.prev_token_hash, Some(token_hash(&first).to_vec()));
    assert_eq!(row.revoked_at, None);

    // The next refresh with that new cookie succeeds.
    let next = post_refresh(&app, &third).await;
    assert_eq!(next.status(), StatusCode::OK);
    assert!(refresh_of(&next).is_some());
}

#[tokio::test]
async fn row_3_the_previous_token_after_its_grace_revokes_the_session() {
    let app = TestApp::spawn().await;
    let (_, first) = signed_in(&app).await;
    let second = refresh_of(&post_refresh(&app, &first).await).unwrap();
    grace_is_over(&app).await;

    let response = post_refresh(&app, &first).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(set_cookie(&response).as_deref(), Some(CLEARED));
    assert_eq!(error_code(response).await, ApiErrorCode::SessionExpired);

    let row = session_row(&app).await;
    assert!(row.revoked_at.is_some());
    assert_eq!(row.refresh_token_hash, token_hash(&second));

    // The session is over, for its current token too.
    let response = post_refresh(&app, &second).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn row_4_no_cookie_or_an_unknown_token_is_401_with_no_cookie_and_revokes_nothing() {
    let app = TestApp::spawn().await;
    let (_, refresh) = signed_in(&app).await;
    let before = session_row(&app).await;

    let unknown = post_refresh(&app, "never-issued").await;
    assert_eq!(unknown.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(set_cookie(&unknown), None);
    assert_eq!(error_code(unknown).await, ApiErrorCode::SessionExpired);

    let none = app
        .client
        .post(format!("{}/auth/refresh", app.base_url))
        .header("x-offcut-csrf", "1")
        .header("origin", ORIGIN)
        .send()
        .await
        .unwrap();
    assert_eq!(none.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(set_cookie(&none), None);
    assert_eq!(error_code(none).await, ApiErrorCode::SessionExpired);

    assert_eq!(session_row(&app).await, before);
    assert_eq!(post_refresh(&app, &refresh).await.status(), StatusCode::OK);
}

#[tokio::test]
async fn row_4_a_revoked_or_expired_session_is_401_with_no_cookie_and_stays_as_it_was() {
    let app = TestApp::spawn().await;
    let (_, refresh) = signed_in(&app).await;

    sqlx::query("UPDATE sessions SET expires_at = now() - interval '1 second'")
        .execute(&app.pool)
        .await
        .unwrap();
    let before = session_row(&app).await;
    let expired = post_refresh(&app, &refresh).await;
    assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(set_cookie(&expired), None);
    assert_eq!(session_row(&app).await, before);

    sqlx::query("UPDATE sessions SET expires_at = now() + interval '1 day', revoked_at = now()")
        .execute(&app.pool)
        .await
        .unwrap();
    let before = session_row(&app).await;
    let revoked = post_refresh(&app, &refresh).await;
    assert_eq!(revoked.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(set_cookie(&revoked), None);
    assert_eq!(session_row(&app).await, before);
}

#[tokio::test]
async fn three_concurrent_refreshes_with_one_cookie_all_succeed_and_the_last_token_refreshes() {
    let app = TestApp::spawn().await;
    let (_, first) = signed_in(&app).await;

    let (a, b, c) = tokio::join!(
        post_refresh(&app, &first),
        post_refresh(&app, &first),
        post_refresh(&app, &first)
    );
    for response in [&a, &b, &c] {
        assert_eq!(response.status(), StatusCode::OK);
    }
    let tokens = [&a, &b, &c].map(|response| refresh_of(response).expect("a new cookie"));
    assert_ne!(tokens[0], tokens[1]);
    assert_ne!(tokens[1], tokens[2]);
    assert_ne!(tokens[0], tokens[2]);

    let row = session_row(&app).await;
    assert_eq!(row.revoked_at, None);
    assert_eq!(row.prev_token_hash, Some(token_hash(&first).to_vec()));
    let last = tokens
        .iter()
        .find(|token| token_hash(token).to_vec() == row.refresh_token_hash)
        .expect("one of the three is current");
    assert_eq!(post_refresh(&app, last).await.status(), StatusCode::OK);
}

#[tokio::test]
async fn a_token_rotated_away_used_after_the_grace_is_401_and_revokes_nothing() {
    let app = TestApp::spawn().await;
    let (_, first) = signed_in(&app).await;
    // Two answers crossed: `second` is neither current nor previous now.
    let second = refresh_of(&post_refresh(&app, &first).await).unwrap();
    let third = refresh_of(&post_refresh(&app, &first).await).unwrap();
    grace_is_over(&app).await;
    let before = session_row(&app).await;

    let response = post_refresh(&app, &second).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(set_cookie(&response), None);
    assert_eq!(error_code(response).await, ApiErrorCode::SessionExpired);
    assert_eq!(session_row(&app).await, before);

    // The current token still works.
    let response = post_refresh(&app, &third).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(session_row(&app).await.revoked_at, None);
}

#[tokio::test]
async fn the_csrf_header_and_the_origin_are_both_required_on_the_cookie_routes() {
    let app = TestApp::spawn().await;
    let (_, refresh) = signed_in(&app).await;
    let before = session_row(&app).await;
    let cookie = format!("offcut_rt={refresh}");

    for route in ["refresh", "logout"] {
        let url = format!("{}/auth/{route}", app.base_url);
        let cases = [
            ("no header", None, Some(ORIGIN)),
            ("a wrong header", Some("0"), Some(ORIGIN)),
            ("no origin", Some("1"), None),
            (
                "a wrong origin",
                Some("1"),
                Some("https://evil.example.com"),
            ),
        ];
        for (case, csrf, origin) in cases {
            let mut request = app.client.post(&url).header("cookie", &cookie);
            if let Some(csrf) = csrf {
                request = request.header("x-offcut-csrf", csrf);
            }
            if let Some(origin) = origin {
                request = request.header("origin", origin);
            }
            let response = request.send().await.unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{route}: {case}");
            assert_eq!(set_cookie(&response), None, "{route}: {case}");
            assert_eq!(error_code(response).await, ApiErrorCode::Forbidden);
        }
    }
    assert_eq!(session_row(&app).await, before);
}

#[tokio::test]
async fn the_cookie_is_found_among_others() {
    let app = TestApp::spawn().await;
    let (_, refresh) = signed_in(&app).await;
    let response = app
        .client
        .post(format!("{}/auth/refresh", app.base_url))
        .header("cookie", format!("other=1; offcut_rt={refresh}; more=2"))
        .header("x-offcut-csrf", "1")
        .header("origin", ORIGIN)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn logout_revokes_clears_the_cookie_and_can_be_repeated() {
    let app = TestApp::spawn().await;
    let (_, refresh) = signed_in(&app).await;

    let response = post_logout(&app, &refresh).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(set_cookie(&response).as_deref(), Some(CLEARED));
    let revoked = session_row(&app).await.revoked_at;
    assert!(revoked.is_some());
    assert_eq!(
        post_refresh(&app, &refresh).await.status(),
        StatusCode::UNAUTHORIZED
    );

    // Again, and with an unknown token: the same answer, and nothing moves.
    let again = post_logout(&app, &refresh).await;
    assert_eq!(again.status(), StatusCode::NO_CONTENT);
    assert_eq!(session_row(&app).await.revoked_at, revoked);
    let unknown = post_logout(&app, "never-issued").await;
    assert_eq!(unknown.status(), StatusCode::NO_CONTENT);
    assert_eq!(set_cookie(&unknown).as_deref(), Some(CLEARED));
}

#[tokio::test]
async fn logout_with_the_previous_token_ends_the_session_too() {
    let app = TestApp::spawn().await;
    let (_, first) = signed_in(&app).await;
    let second = refresh_of(&post_refresh(&app, &first).await).unwrap();

    assert_eq!(
        post_logout(&app, &first).await.status(),
        StatusCode::NO_CONTENT
    );
    assert!(session_row(&app).await.revoked_at.is_some());
    assert_eq!(
        post_refresh(&app, &second).await.status(),
        StatusCode::UNAUTHORIZED
    );
}

// --- GET /me and the extractor ----------------------------------------------

async fn subscribe(app: &TestApp, status: &str, period_end_sql: &str) {
    let sql = format!(
        "INSERT INTO subscriptions (user_id, provider_customer_id, provider_subscription_id, plan,
           billing_interval, status, cancel_at_period_end, current_period_end, last_event_at)
         SELECT id, 'cus_1', 'sub_1', 'creator', 'annual', '{status}', true, {period_end_sql}, now()
         FROM users"
    );
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .execute(&app.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn me_answers_the_address_the_free_plan_and_a_fresh_token() {
    let app = TestApp::spawn().await;
    let (session, _) = signed_in(&app).await;

    let response = get_me(&app, &session.access_token).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let me: MeResponse = response.json().await.unwrap();
    assert_eq!(me.email, NORMALIZED);
    assert_eq!(me.plan, Plan::Free);
    assert_eq!(me.subscription, None);

    let public = entitlement_key().verifying_key().to_bytes();
    let entitlement = verify_token(&me.entitlement_token, &[public]).unwrap();
    assert_eq!(entitlement.plan, Plan::Free);
}

#[tokio::test]
async fn me_answers_creator_in_a_paid_period_with_its_summary() {
    let app = TestApp::spawn().await;
    let (session, _) = signed_in(&app).await;
    subscribe(&app, "past_due", "now() + interval '10 days'").await;

    let me: MeResponse = get_me(&app, &session.access_token)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(me.plan, Plan::Creator);
    let summary = me.subscription.expect("a summary");
    assert_eq!(summary.status, SubscriptionStatus::PastDue);
    assert_eq!(summary.interval, BillingInterval::Annual);
    assert!(summary.cancel_at_period_end);

    let public = entitlement_key().verifying_key().to_bytes();
    let entitlement = verify_token(&me.entitlement_token, &[public]).unwrap();
    assert_eq!(entitlement.plan, Plan::Creator);
    assert_eq!(entitlement.period_end, summary.current_period_end);
}

#[tokio::test]
async fn me_answers_free_after_the_paid_period_and_still_shows_the_summary() {
    let app = TestApp::spawn().await;
    let (session, _) = signed_in(&app).await;
    subscribe(&app, "canceled", "now() - interval '1 day'").await;

    let me: MeResponse = get_me(&app, &session.access_token)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(me.plan, Plan::Free);
    assert_eq!(
        me.subscription.unwrap().status,
        SubscriptionStatus::Canceled
    );
}

#[tokio::test]
async fn me_refuses_a_missing_forged_or_expired_access_token() {
    let app = TestApp::spawn().await;
    let (session, _) = signed_in(&app).await;

    let none = app
        .client
        .get(format!("{}/me", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(none.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(none).await, ApiErrorCode::Unauthorized);

    let basic = app
        .client
        .get(format!("{}/me", app.base_url))
        .header("authorization", format!("Basic {}", session.access_token))
        .send()
        .await
        .unwrap();
    assert_eq!(basic.status(), StatusCode::UNAUTHORIZED);

    let claims =
        access_token::verify(&session.access_token, &access_key().verifying_key(), now()).unwrap();
    let other_key = SigningKey::from_bytes(&[9; 32]);
    let forged = get_me(&app, &access_token::sign(&claims, &other_key)).await;
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(forged).await, ApiErrorCode::Unauthorized);

    let mut late = claims;
    late.exp = UnixSecs::new(now().get() - 1);
    let expired = get_me(&app, &access_token::sign(&late, &access_key())).await;
    assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(expired).await, ApiErrorCode::Unauthorized);

    // An entitlement token is not an access token: the keys differ.
    let wrong = get_me(&app, &session.entitlement_token).await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_for_a_deleted_user_is_401() {
    let app = TestApp::spawn().await;
    let (session, _) = signed_in(&app).await;
    sqlx::query("DELETE FROM users")
        .execute(&app.pool)
        .await
        .unwrap();

    let response = get_me(&app, &session.access_token).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(response).await, ApiErrorCode::SessionExpired);
}

#[tokio::test]
async fn the_session_routes_refuse_the_wrong_method() {
    let app = TestApp::spawn().await;
    for path in ["auth/refresh", "auth/logout"] {
        let url = format!("{}/{path}", app.base_url);
        let response = app.client.get(url).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
    let response = app
        .client
        .post(format!("{}/me", app.base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
