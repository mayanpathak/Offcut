//! The `ai_usage` table (TS §52, v6.2implementation D-461): per person, UTC
//! month and kind, how many AI answers succeeded. Counted only on success;
//! no content. Also the export's AI rows, with `ai_shorts_grants` by month
//! only (its hash stays in the database, S-10).

use offcut_types::UserId;
use serde::Serialize;
use sqlx::PgPool;
use time::{Date, OffsetDateTime};

/// What an AI answer was for: the `kind` column's values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiKind {
    Plan,
    Shorts,
    Chat,
}

impl AiKind {
    fn text(self) -> &'static str {
        match self {
            AiKind::Plan => "plan",
            AiKind::Shorts => "shorts",
            AiKind::Chat => "chat",
        }
    }
}

/// The first day of `at`'s UTC month: the `month` of a row.
pub fn month_of(at: OffsetDateTime) -> Date {
    let utc = at.to_offset(time::UtcOffset::UTC);
    utc.date().replace_day(1).unwrap_or(utc.date())
}

/// Counts one success and returns the month's count after it.
pub async fn increment(
    pool: &PgPool,
    user: UserId,
    month: Date,
    kind: AiKind,
) -> Result<i32, sqlx::Error> {
    let row = sqlx::query!(
        r#"INSERT INTO ai_usage (user_id, month, kind, count) VALUES ($1, $2, $3, 1)
           ON CONFLICT (user_id, month, kind) DO UPDATE SET count = ai_usage.count + 1
           RETURNING count"#,
        user.get(),
        month,
        kind.text(),
    )
    .fetch_one(pool)
    .await?;
    Ok(row.count)
}

/// The month's count, 0 with no row.
pub async fn count(
    pool: &PgPool,
    user: UserId,
    month: Date,
    kind: AiKind,
) -> Result<i32, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT count FROM ai_usage WHERE user_id = $1 AND month = $2 AND kind = $3",
        user.get(),
        month,
        kind.text(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map_or(0, |row| row.count))
}

/// One month's count of one kind, as the export shows it: `"2026-11"`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExportedAiUsage {
    pub month: String,
    pub kind: String,
    pub count: i32,
}

fn month_text(month: Date) -> String {
    format!("{:04}-{:02}", month.year(), u8::from(month.month()))
}

/// The person's counts, oldest month first.
pub async fn exported(pool: &PgPool, user: UserId) -> Result<Vec<ExportedAiUsage>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT month, kind, count FROM ai_usage WHERE user_id = $1 ORDER BY month, kind",
        user.get(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ExportedAiUsage {
            month: month_text(row.month),
            kind: row.kind,
            count: row.count,
        })
        .collect())
}

/// The months in which the person was granted a long video (V6.4), oldest
/// first, one entry per grant; never the grant's hash.
pub async fn exported_grant_months(
    pool: &PgPool,
    user: UserId,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT month FROM ai_shorts_grants WHERE user_id = $1 ORDER BY month",
        user.get(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| month_text(row.month)).collect())
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    #[test]
    fn a_month_is_its_first_utc_day() {
        let month = month_of(datetime!(2026-11-30 23:30 -02:00));
        assert_eq!(month_text(month), "2026-12");
        assert_eq!(month.day(), 1);
        assert_eq!(
            month_text(month_of(datetime!(2026-01-01 00:00 UTC))),
            "2026-01"
        );
        let kinds = [AiKind::Plan, AiKind::Shorts, AiKind::Chat].map(AiKind::text);
        assert_eq!(kinds, ["plan", "shorts", "chat"]);
    }
}
