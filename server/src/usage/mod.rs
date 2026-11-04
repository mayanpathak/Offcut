//! `POST /usage/receipts` (TS §22.4). A receipt is never refused for being over
//! a limit: the export has already happened. Limits are conversion mechanisms
//! (PS §11), so this counts and does not judge.

use std::collections::HashSet;
use std::sync::{Mutex, PoisonError};

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use offcut_api_types::{EntitlementResponse, UsageReceiptRequest};
use offcut_types::UserId;
use offcut_types::limits::CREATOR_FAIR_USE_PER_MONTH;
use time::{Date, OffsetDateTime};

use crate::auth::extract::AuthUser;
use crate::db::{usage_receipts, users};
use crate::entitlement::issue::{Issued, issue, plan_now, utc_month};
use crate::error::{AppError, json_rejection};
use crate::log::user_tag;
use crate::rate_limit::{RateKey, RouteGroup, limit};
use crate::state::AppState;

/// Who has been logged above the fair-use ceiling today (v6implementation
/// D-269). In memory: a restart may log someone twice, which harms nothing.
#[derive(Default)]
pub struct FairUseLog {
    today: Mutex<Option<(Date, HashSet<UserId>)>>,
}

impl FairUseLog {
    /// Whether this is the first time today, in UTC, that `user` is reported.
    fn first_today(&self, user: UserId, today: Date) -> bool {
        let mut seen = self.today.lock().unwrap_or_else(PoisonError::into_inner);
        match seen.as_mut() {
            Some((day, users)) if *day == today => users.insert(user),
            Some(_) | None => {
                *seen = Some((today, HashSet::from([user])));
                true
            }
        }
    }
}

pub async fn post_receipt(
    State(state): State<AppState>,
    auth: AuthUser,
    body: Result<Json<UsageReceiptRequest>, JsonRejection>,
) -> Result<Json<EntitlementResponse>, AppError> {
    limit(&state, RouteGroup::UsageReceipts, RateKey::User(auth.user))?;
    let Json(request) = body.map_err(json_rejection)?;
    // Before the insert: a row must not fail on a user who is gone.
    users::email(&state.db, auth.user)
        .await?
        .ok_or(AppError::SessionExpired)?;

    let now = OffsetDateTime::now_utc();
    let plan = plan_now(&state.db, auth.user, now).await?;
    usage_receipts::record(&state.db, request.export_id, auth.user, plan).await?;
    warn_above_fair_use(&state, auth.user, now).await?;

    let key = &state.config.entitlement_signing_key;
    let Issued { token, .. } = issue(&state.db, key, auth.user, now).await?;
    Ok(Json(EntitlementResponse {
        entitlement_token: token,
    }))
}

/// One log line a day for a Creator above the soft ceiling. No behaviour changes.
async fn warn_above_fair_use(
    state: &AppState,
    user: UserId,
    now: OffsetDateTime,
) -> Result<(), AppError> {
    let (start, _) = utc_month(now).ok_or(AppError::Internal)?;
    let used = usage_receipts::creator_used_since(&state.db, user, start).await?;
    let ceiling = i64::from(CREATOR_FAIR_USE_PER_MONTH.get());
    if used > ceiling && state.fair_use.first_today(user, now.date()) {
        tracing::warn!(user = user_tag(user), "fair_use_exceeded");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use time::macros::date;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn a_user_is_reported_once_a_day() {
        let log = FairUseLog::default();
        let (one, two) = (
            UserId::new(Uuid::from_u128(1)),
            UserId::new(Uuid::from_u128(2)),
        );
        let day = date!(2026 - 11 - 02);
        assert!(log.first_today(one, day));
        assert!(!log.first_today(one, day));
        assert!(log.first_today(two, day));
        let next = date!(2026 - 11 - 03);
        assert!(log.first_today(one, next));
        assert!(!log.first_today(one, next));
    }
}
