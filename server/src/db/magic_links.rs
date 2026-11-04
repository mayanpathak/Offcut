//! The `magic_links` table. It holds a hash of each link's token, never the token.

use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

pub struct MagicLink {
    pub email_normalized: String,
    pub expires_at: OffsetDateTime,
    pub consumed_at: Option<OffsetDateTime>,
}

/// Stores a new link and, in the same statement, spends the address's earlier
/// unused ones (TS §22.1).
pub async fn replace(
    pool: &PgPool,
    token_hash: &[u8],
    email_normalized: &str,
    expires_at: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "WITH spent AS (
           UPDATE magic_links SET consumed_at = now()
           WHERE email_normalized = $2 AND consumed_at IS NULL
         )
         INSERT INTO magic_links (token_hash, email_normalized, expires_at) VALUES ($1, $2, $3)",
        token_hash,
        email_normalized,
        expires_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, token_hash: &[u8]) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM magic_links WHERE token_hash = $1", token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// The link of this hash, locked until the transaction ends.
pub async fn for_update(
    conn: &mut PgConnection,
    token_hash: &[u8],
) -> Result<Option<MagicLink>, sqlx::Error> {
    sqlx::query_as!(
        MagicLink,
        "SELECT email_normalized, expires_at, consumed_at FROM magic_links
         WHERE token_hash = $1 FOR UPDATE",
        token_hash,
    )
    .fetch_optional(conn)
    .await
}

pub async fn consume(conn: &mut PgConnection, token_hash: &[u8]) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE magic_links SET consumed_at = now() WHERE token_hash = $1",
        token_hash,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete_for_email(
    conn: &mut PgConnection,
    email_normalized: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM magic_links WHERE email_normalized = $1",
        email_normalized,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Deletes the links made more than `days` days ago and returns how many.
pub async fn purge_older_than(pool: &PgPool, days: u32) -> Result<u64, sqlx::Error> {
    let days = i32::try_from(days).unwrap_or(i32::MAX);
    let deleted = sqlx::query!(
        "DELETE FROM magic_links WHERE created_at < now() - make_interval(days => $1)",
        days,
    )
    .execute(pool)
    .await?;
    Ok(deleted.rows_affected())
}
