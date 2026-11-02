//! `POST /billing/webhook` (C-19; TS §22.5; v6implementation §10.3). The signature
//! first, then one transaction holds the dedupe insert and the change, so a change
//! that fails leaves no event row and the merchant's retry applies it.

use axum::body::Bytes;
use axum::extract::State;
use axum::extract::rejection::BytesRejection;
use axum::http::{HeaderMap, StatusCode};
use offcut_api_types::SubscriptionStatus;
use offcut_types::{UnixSecs, UserId};
use time::OffsetDateTime;

use crate::billing::provider::{BillingError, BillingEvent, BillingEventKind, EventOwner};
use crate::db::subscriptions::{self, SubscriptionRow};
use crate::db::{users, webhook_events};
use crate::error::AppError;
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<StatusCode, AppError> {
    let body = body.map_err(|rejection| {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            AppError::PayloadTooLarge
        } else {
            AppError::BadRequest
        }
    })?;
    let now = UnixSecs::new(OffsetDateTime::now_utc().unix_timestamp());
    state
        .billing
        .verify_signature(&headers, &body, now)
        .map_err(|_| AppError::Unauthorized)?;

    let event = match state.billing.parse_event(&headers, &body) {
        Ok(Some(event)) => event,
        Ok(None) => return Ok(StatusCode::OK),
        // Signed, and unreadable: the merchant's retries wait for a fix.
        Err(BillingError::BadEvent) => return Err(AppError::BadRequest),
        Err(BillingError::BadSignature | BillingError::Unavailable) => {
            return Err(AppError::Unauthorized);
        }
    };

    let mut tx = state.db.begin().await?;
    let new_event =
        webhook_events::record(&mut tx, &event.provider_event_id, kind_text(event.kind)).await?;
    if !new_event {
        return Ok(StatusCode::OK);
    }
    match owner_of(&mut tx, &event.owner).await? {
        Some(user) => {
            let row = subscriptions::for_update(&mut tx, user).await?;
            match next(row.as_ref(), &event) {
                Some(changed) => {
                    subscriptions::upsert(&mut tx, user, &changed).await?;
                    // A refund does not stop the subscription at the merchant (read
                    // in test mode), and the policy says it ends the plan. Inside
                    // the transaction: if this fails, nothing is recorded, and the
                    // merchant's retry tries again.
                    if event.kind == BillingEventKind::Refunded && row.as_ref().is_some_and(is_live)
                    {
                        state
                            .billing
                            .cancel_subscription(&changed.subscription_id)
                            .await
                            .map_err(|_| AppError::BillingUnavailable)?;
                    }
                }
                None => ignored(event.kind),
            }
        }
        // A user who is gone, or a customer nobody holds (D-272, D-284). The
        // event stays recorded: there is nothing for the merchant to retry.
        None => ignored(event.kind),
    }
    tx.commit().await?;
    Ok(StatusCode::OK)
}

/// The kind only: an event's ids belong to a person (TS §24.4).
fn ignored(kind: BillingEventKind) {
    tracing::info!(kind = kind_text(kind), "billing_event_ignored");
}

/// The user an event is about, still there. The row stays locked until the
/// commit, so a delete cannot take the user from under the change.
async fn owner_of(
    conn: &mut sqlx::PgConnection,
    owner: &EventOwner,
) -> Result<Option<UserId>, sqlx::Error> {
    match owner {
        EventOwner::User(user) => users::lock(conn, *user).await,
        EventOwner::Customer(customer) => subscriptions::user_of_customer(conn, customer).await,
    }
}

/// The value of the `event_type` column.
fn kind_text(kind: BillingEventKind) -> &'static str {
    match kind {
        BillingEventKind::Activated => "activated",
        BillingEventKind::Renewed => "renewed",
        BillingEventKind::PastDue => "past_due",
        BillingEventKind::CancelScheduled => "cancel_scheduled",
        BillingEventKind::Canceled => "canceled",
        BillingEventKind::Refunded => "refunded",
        BillingEventKind::Resumed => "resumed",
        BillingEventKind::PlanChanged => "plan_changed",
    }
}

fn is_live(row: &SubscriptionRow) -> bool {
    row.status == SubscriptionStatus::Active || row.status == SubscriptionStatus::PastDue
}

fn time_of(secs: UnixSecs) -> Option<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp(secs.get()).ok()
}

/// TS §22.5's table with D-276's and D-301's rows: what an event makes of a
/// subscription. `None` is "ignored": no row changes. Pure, so the table is read
/// in one place.
fn next(from: Option<&SubscriptionRow>, event: &BillingEvent) -> Option<SubscriptionRow> {
    let occurred = time_of(event.occurred_at)?;
    let period_end = event.current_period_end.and_then(time_of);

    if let Some(row) = from {
        if occurred < row.last_event_at {
            return None;
        }
        // A late event of an old subscription must not touch a new one (D-299).
        let other_subscription = event
            .subscription_id
            .as_ref()
            .is_some_and(|id| *id != row.subscription_id);
        if other_subscription && event.kind != BillingEventKind::Activated {
            return None;
        }
    }

    let live = from.filter(|row| is_live(row));
    let changed = |change: &dyn Fn(&mut SubscriptionRow), row: &SubscriptionRow| {
        let mut row = row.clone();
        row.last_event_at = occurred;
        change(&mut row);
        row
    };

    match event.kind {
        BillingEventKind::Activated => {
            // Active already: a repeat of what is known. Past due: a recovery (D-276).
            if from.is_some_and(|row| row.status == SubscriptionStatus::Active) {
                return None;
            }
            Some(SubscriptionRow {
                customer_id: event.customer_id.clone(),
                subscription_id: event.subscription_id.clone()?,
                interval: event.interval?,
                status: SubscriptionStatus::Active,
                cancel_at_period_end: false,
                current_period_end: period_end?,
                last_event_at: occurred,
            })
        }
        BillingEventKind::Renewed => {
            let end = period_end?;
            Some(changed(
                &|row| {
                    row.status = SubscriptionStatus::Active;
                    row.current_period_end = end;
                },
                live?,
            ))
        }
        BillingEventKind::PastDue => {
            let active = from.filter(|row| row.status == SubscriptionStatus::Active)?;
            Some(changed(
                &|row| row.status = SubscriptionStatus::PastDue,
                active,
            ))
        }
        BillingEventKind::CancelScheduled => {
            Some(changed(&|row| row.cancel_at_period_end = true, live?))
        }
        // `subscription.updated` fires on any change, and carries this flag each
        // time (read in test mode). With no cancel scheduled it says nothing, and
        // must not move `last_event_at`, or it would hide a retried older event.
        BillingEventKind::Resumed => {
            let scheduled = live.filter(|row| row.cancel_at_period_end)?;
            Some(changed(&|row| row.cancel_at_period_end = false, scheduled))
        }
        BillingEventKind::Canceled => Some(changed(
            &|row| {
                row.status = SubscriptionStatus::Canceled;
                // A cancel that names no date leaves paid time as it was.
                if let Some(end) = period_end {
                    row.current_period_end = end;
                }
            },
            live?,
        )),
        BillingEventKind::Refunded => Some(changed(
            &|row| {
                row.status = SubscriptionStatus::Canceled;
                row.current_period_end = occurred;
            },
            from?,
        )),
        BillingEventKind::PlanChanged => Some(changed(
            &|row| {
                if let Some(interval) = event.interval {
                    row.interval = interval;
                }
                if let Some(end) = period_end {
                    row.current_period_end = end;
                }
            },
            live?,
        )),
    }
}
