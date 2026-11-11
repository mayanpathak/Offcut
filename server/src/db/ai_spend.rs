//! The `ai_spend` table (TS §52, v6.2implementation D-463): what the AI
//! provider has cost this UTC month, in micro-dollars, across every person.
//! No content and no person: the spend cap reads it.

use sqlx::PgPool;
use time::Date;

/// Adds one call's cost to `month` and returns the month's total after it.
pub async fn add(pool: &PgPool, month: Date, micro_usd: i64) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"INSERT INTO ai_spend (month, micro_usd) VALUES ($1, $2)
           ON CONFLICT (month) DO UPDATE SET micro_usd = ai_spend.micro_usd + EXCLUDED.micro_usd
           RETURNING micro_usd"#,
        month,
        micro_usd,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.micro_usd)
}

/// The month's total, 0 before its first call.
pub async fn total(pool: &PgPool, month: Date) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!("SELECT micro_usd FROM ai_spend WHERE month = $1", month)
        .fetch_optional(pool)
        .await?;
    Ok(row.map_or(0, |row| row.micro_usd))
}
