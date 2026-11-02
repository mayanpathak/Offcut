//! The `usage_receipts` table: one row per export, holding no content.

use offcut_api_types::ExportedReceipt;
use offcut_types::{ExportId, Plan, UnixSecs, UserId};
use sqlx::PgPool;
use time::OffsetDateTime;

/// How many exports made on the Free plan the user has reported since `since`.
pub async fn free_used_since(
    pool: &PgPool,
    user: UserId,
    since: OffsetDateTime,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT count(*) AS "used!" FROM usage_receipts
           WHERE user_id = $1 AND plan_at_export = 'free' AND created_at >= $2"#,
        user.get(),
        since,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.used)
}

/// How many exports made on the Creator plan the user has reported since `since`.
pub async fn creator_used_since(
    pool: &PgPool,
    user: UserId,
    since: OffsetDateTime,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT count(*) AS "used!" FROM usage_receipts
           WHERE user_id = $1 AND plan_at_export = 'creator' AND created_at >= $2"#,
        user.get(),
        since,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.used)
}

/// One row per export id. A repeat, or an id another user reported, changes nothing.
pub async fn record(
    pool: &PgPool,
    export_id: ExportId,
    user: UserId,
    plan: Plan,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO usage_receipts (export_id, user_id, plan_at_export) VALUES ($1, $2, $3)
         ON CONFLICT (export_id) DO NOTHING",
        export_id.get(),
        user.get(),
        plan_text(plan),
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// A person's receipts, oldest first, for their export.
pub async fn exported(pool: &PgPool, user: UserId) -> Result<Vec<ExportedReceipt>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT export_id, created_at, plan_at_export FROM usage_receipts
         WHERE user_id = $1 ORDER BY created_at",
        user.get(),
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(ExportedReceipt {
                export_id: ExportId::new(row.export_id),
                created_at: UnixSecs::new(row.created_at.unix_timestamp()),
                plan_at_export: plan_of(&row.plan_at_export).ok_or(sqlx::Error::RowNotFound)?,
            })
        })
        .collect()
}

/// The value of the `plan_at_export` column: one of its `CHECK`.
fn plan_text(plan: Plan) -> &'static str {
    match plan {
        Plan::Free => "free",
        Plan::Creator => "creator",
    }
}

fn plan_of(text: &str) -> Option<Plan> {
    match text {
        "free" => Some(Plan::Free),
        "creator" => Some(Plan::Creator),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plan_reads_back_as_it_was_written() {
        for plan in [Plan::Free, Plan::Creator] {
            assert_eq!(plan_of(plan_text(plan)), Some(plan));
        }
        assert_eq!(plan_of("pro"), None);
    }
}
