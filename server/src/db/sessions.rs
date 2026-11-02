//! The `sessions` table. It holds hashes of refresh tokens, never a token.

use offcut_types::UserId;
use sqlx::PgConnection;
use time::OffsetDateTime;
use uuid::Uuid;

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
