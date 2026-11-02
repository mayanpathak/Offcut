//! The `usage_receipts` table: one row per export, holding no content.

use offcut_types::{ExportId, Plan, UserId};
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

/// The value of the `plan_at_export` column: one of its `CHECK`.
fn plan_text(plan: Plan) -> &'static str {
    match plan {
        Plan::Free => "free",
        Plan::Creator => "creator",
    }
}
