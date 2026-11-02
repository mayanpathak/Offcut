//! The `sessions` table. It holds hashes of refresh tokens, never a token.

use offcut_api_types::ExportedSession;
use offcut_types::{UnixSecs, UserId};
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;
use uuid::Uuid;

/// What deciding a refresh needs of a session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: Vec<u8>,
    pub prev_token_hash: Option<Vec<u8>>,
    pub prev_valid_until: Option<OffsetDateTime>,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
}

pub async fn create(
    conn: &mut PgConnection,
    id: Uuid,
    user: UserId,
    refresh_token_hash: &[u8],
    expires_at: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO sessions (id, user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3, $4)",
        id,
        user.get(),
        refresh_token_hash,
        expires_at,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// The session whose current or previous hash this is, locked until the
/// transaction ends. A request that waits for the lock sees the row as the
/// holder left it, so requests with one cookie take their turns.
pub async fn for_update(
    conn: &mut PgConnection,
    token_hash: &[u8],
) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionRow,
        "SELECT id, user_id, refresh_token_hash, prev_token_hash, prev_valid_until, expires_at,
                revoked_at
         FROM sessions
         WHERE refresh_token_hash = $1 OR prev_token_hash = $1
         FOR UPDATE",
        token_hash,
    )
    .fetch_optional(conn)
    .await
}

/// The current token was used: it becomes the previous one, good until `until`.
pub async fn rotate_current(
    conn: &mut PgConnection,
    id: Uuid,
    new_hash: &[u8],
    until: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sessions
         SET prev_token_hash = refresh_token_hash, prev_valid_until = $3,
             refresh_token_hash = $2, last_used_at = now()
         WHERE id = $1",
        id,
        new_hash,
        until,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// The previous token was used in its grace. The previous hash and its time do
/// not move (v6implementation D-258).
pub async fn rotate_in_grace(
    conn: &mut PgConnection,
    id: Uuid,
    new_hash: &[u8],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sessions SET refresh_token_hash = $2, last_used_at = now() WHERE id = $1",
        id,
        new_hash,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn revoke(conn: &mut PgConnection, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("UPDATE sessions SET revoked_at = now() WHERE id = $1", id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Ends the session of this hash, if there is one still running.
pub async fn revoke_by_token(pool: &PgPool, token_hash: &[u8]) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sessions SET revoked_at = now()
         WHERE (refresh_token_hash = $1 OR prev_token_hash = $1) AND revoked_at IS NULL",
        token_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// A person's sessions, for their export: times only, no hash.
pub async fn exported(pool: &PgPool, user: UserId) -> Result<Vec<ExportedSession>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT created_at, last_used_at, expires_at FROM sessions
         WHERE user_id = $1 ORDER BY created_at",
        user.get(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ExportedSession {
            created_at: UnixSecs::new(row.created_at.unix_timestamp()),
            last_used_at: UnixSecs::new(row.last_used_at.unix_timestamp()),
            expires_at: UnixSecs::new(row.expires_at.unix_timestamp()),
        })
        .collect())
}

/// Deletes the sessions that can never be used again and returns how many.
pub async fn purge_finished(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let deleted =
        sqlx::query!("DELETE FROM sessions WHERE revoked_at IS NOT NULL OR expires_at < now()")
            .execute(pool)
            .await?;
    Ok(deleted.rows_affected())
}
