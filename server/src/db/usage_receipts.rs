//! The `usage_receipts` table: one row per export, holding no content.

use offcut_types::UserId;
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
