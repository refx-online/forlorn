use anyhow::Result;

use crate::{infrastructure::database::DbPoolManager, models::ClientError};

pub async fn insert(db: &DbPoolManager, error: &ClientError) -> Result<()> {
    sqlx::query(
        "insert into client_errors (user_id, username, feedback, exception, stacktrace) values (?, ?, ?, ?, ?)",
    )
    .bind(error.user_id)
    .bind(&error.username)
    .bind(&error.feedback)
    .bind(&error.exception)
    .bind(&error.stacktrace)
    .execute(db.as_ref())
    .await?;

    Ok(())
}
