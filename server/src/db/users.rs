//! The `users` table.

use offcut_api_types::ExportedUser;
use offcut_types::{UnixSecs, UserId};
use sqlx::{PgConnection, PgPool};
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

/// The user, if still there, held until the transaction ends: a delete waits.
pub async fn lock(conn: &mut PgConnection, user: UserId) -> Result<Option<UserId>, sqlx::Error> {
    let row = sqlx::query!("SELECT id FROM users WHERE id = $1 FOR SHARE", user.get())
        .fetch_optional(conn)
        .await?;
    Ok(row.map(|row| UserId::new(row.id)))
}

/// The address of a user, or `None` when the account is gone.
pub async fn email(pool: &PgPool, user: UserId) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!("SELECT email FROM users WHERE id = $1", user.get())
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|row| row.email))
}

/// Whether the user's row still exists. An access token lives 15 minutes and
/// reads no database (D-261), so a route about to spend money asks (D-262).
pub async fn exists(pool: &PgPool, user: UserId) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE id = $1) AS "found!""#,
        user.get()
    )
    .fetch_one(pool)
    .await?;
    Ok(row.found)
}

/// Removes the user. The foreign keys take the rows of `USER_TABLES` with it.
pub async fn delete(conn: &mut PgConnection, user: UserId) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM users WHERE id = $1", user.get())
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn exported(pool: &PgPool, user: UserId) -> Result<Option<ExportedUser>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT email, created_at FROM users WHERE id = $1",
        user.get()
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| ExportedUser {
        email: row.email,
        created_at: UnixSecs::new(row.created_at.unix_timestamp()),
    }))
}
