//! The `platform_waitlist` table: who asked to be told about what.

use offcut_api_types::Wanted;
use sqlx::PgPool;

/// Adds the pair if it is not there yet. The caller cannot tell which
/// happened, so the waitlist cannot be probed for an address.
pub async fn upsert(
    pool: &PgPool,
    email_normalized: &str,
    wanted: Wanted,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO platform_waitlist (email_normalized, wanted) VALUES ($1, $2)
         ON CONFLICT (email_normalized, wanted) DO NOTHING",
        email_normalized,
        wanted_text(wanted),
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// The value of the `wanted` column. It equals the JSON form of `Wanted` and
/// one of the values of the column's `CHECK` (D-2).
fn wanted_text(wanted: Wanted) -> &'static str {
    match wanted {
        Wanted::Launch => "launch",
        Wanted::Safari => "safari",
        Wanted::Firefox => "firefox",
        Wanted::Mobile => "mobile",
        Wanted::Linux => "linux",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_column_value_is_the_json_form_of_wanted() {
        let all = [
            Wanted::Launch,
            Wanted::Safari,
            Wanted::Firefox,
            Wanted::Mobile,
            Wanted::Linux,
        ];
        for wanted in all {
            let json = serde_json::to_value(wanted).unwrap();
            assert_eq!(json, wanted_text(wanted));
        }
    }
}
