//! The `users` table.

use offcut_types::UserId;
use sqlx::PgConnection;
use uuid::Uuid;

/// The user of this address, made with `new_id` if there is none. Two sign-ins
/// at once for a new address give one user.
pub async fn find_or_create(
    conn: &mut PgConnection,
    new_id: Uuid,
    email_normalized: &str,
) -> Result<UserId, sqlx::Error> {
    let row = sqlx::query!(
        "INSERT INTO users (id, email, email_normalized) VALUES ($1, $2, $2)
         ON CONFLICT (email_normalized) DO UPDATE SET email_normalized = EXCLUDED.email_normalized
         RETURNING id",
        new_id,
        email_normalized,
    )
    .fetch_one(conn)
    .await?;
    Ok(UserId::new(row.id))
}
