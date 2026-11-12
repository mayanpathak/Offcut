//! V6.2: what the AI plan's tests share (`plan_route.rs`, `ai_cap.rs`,
//! `ai_quota.rs`, `log_redaction.rs`): a person with an access token, a
//! Creator, a request, and the month's counts.

// Each test binary uses a part of this.
#![allow(dead_code)]

use ed25519_dalek::SigningKey;
use offcut_api::auth::access_token::{self, ACCESS_TOKEN_TTL_SECS, AccessClaims};
use offcut_api::db::ai_spend;
use offcut_api::db::ai_usage::{self, AiKind};
use offcut_api_types::{PlanRequest, PlanWord};
use offcut_plan::CATALOG_VERSION;
use offcut_types::{DurMs, TimeMs, UnixSecs, UserId, WordIdx, WordRange};
use serde_json::json;
use time::{Date, Duration, OffsetDateTime};
use uuid::Uuid;

use super::{TestApp, webhook};

/// A person in the database, and an access token signed with the tests' key
/// (`test_config`'s), as a sign-in would give them.
pub async fn person(app: &TestApp) -> (UserId, String) {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, email, email_normalized) VALUES ($1, $2, $2)")
        .bind(id)
        .bind(format!("{id}@example.com"))
        .execute(&app.pool)
        .await
        .unwrap();
    let user = UserId::new(id);
    let exp = OffsetDateTime::now_utc().unix_timestamp() + ACCESS_TOKEN_TTL_SECS;
    let claims = AccessClaims {
        sub: user,
        sid: Uuid::new_v4(),
        exp: UnixSecs::new(exp),
    };
    let token = access_token::sign(&claims, &SigningKey::from_bytes(&[2; 32]));
    (user, token)
}

/// Makes `user` a Creator through the fake merchant's webhook.
pub async fn make_creator(app: &TestApp, user: UserId) {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let body = json!({
        "id": format!("evt_{}", Uuid::now_v7()),
        "user": user.get(),
        "customer": format!("cus_{}", user.get()),
        "subscription": format!("sub_{}", user.get()),
        "kind": "activated",
        "interval": "monthly",
        "period_end": now + 30 * 86_400,
        "at": now,
    });
    assert!(webhook(app, &body).await.status().is_success());
}

fn word(text: &str, start: u32, end: u32) -> PlanWord {
    PlanWord {
        text: text.to_owned(),
        start: TimeMs::new(start),
        end: TimeMs::new(end),
        energy_z: 0.5,
        pitch_z: -0.5,
    }
}

/// A short clip's transcript: two sentences, one number.
pub fn request() -> PlanRequest {
    let words = [
        ("I", 100, 200),
        ("spent", 250, 600),
        ("3000", 650, 1_100),
        ("hours", 1_150, 1_500),
        ("editing.", 1_550, 2_100),
        ("It", 2_600, 2_700),
        ("was", 2_750, 2_900),
        ("brutal.", 2_950, 3_500),
    ];
    let range = |start: u32, end: u32| WordRange {
        start: WordIdx::new(start),
        end: WordIdx::new(end),
    };
    PlanRequest {
        catalog: CATALOG_VERSION,
        duration: DurMs::new(4_000),
        words: words.iter().map(|(t, s, e)| word(t, *s, *e)).collect(),
        sentences: vec![range(0, 5), range(5, 8)],
    }
}

/// `POST /plan` with a bearer (when given) and a JSON body.
pub async fn post(
    app: &TestApp,
    token: Option<&str>,
    body: &impl serde::Serialize,
) -> reqwest::Response {
    let mut request = app.client.post(format!("{}/plan", app.base_url)).json(body);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    request.send().await.unwrap()
}

/// This UTC month, as the route keys its rows.
pub fn this_month() -> Date {
    ai_usage::month_of(OffsetDateTime::now_utc())
}

/// Last UTC month.
pub fn last_month() -> Date {
    ai_usage::month_of(OffsetDateTime::now_utc().replace_day(1).unwrap() - Duration::days(1))
}

/// The person's plans counted in `month`.
pub async fn plans_counted(app: &TestApp, user: UserId, month: Date) -> i32 {
    ai_usage::count(&app.pool, user, month, AiKind::Plan)
        .await
        .unwrap()
}

/// Sets the person's count for `month` to `count`, as that many successes would.
pub async fn seed_plans(app: &TestApp, user: UserId, month: Date, count: i32) {
    sqlx::query(
        "INSERT INTO ai_usage (user_id, month, kind, count) VALUES ($1, $2, 'plan', $3) \
         ON CONFLICT (user_id, month, kind) DO UPDATE SET count = EXCLUDED.count",
    )
    .bind(user.get())
    .bind(month)
    .bind(count)
    .execute(&app.pool)
    .await
    .unwrap();
}

/// The month's spend so far, in micro-dollars.
pub async fn spent(app: &TestApp, month: Date) -> i64 {
    ai_spend::total(&app.pool, month).await.unwrap()
}
