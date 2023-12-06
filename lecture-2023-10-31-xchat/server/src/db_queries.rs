use std::result::Result;

use sqlx::query;
use sqlx::sqlite::{SqlitePool};

use crate::ServerError;


/// `insert_login` inserts a single complete row into the `client_logins` table.
/// Internal ID comes from a internal DB sequence.
pub async fn insert_login(
        pool: &SqlitePool,
        login: &str,
        timestamp: &str,
) -> Result<(), ServerError> {
    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(err) => Err(ServerError::DBError(err.to_string()))?,
    };

    match query!(
        r#"
INSERT INTO client_logins
(login, timestamp)
VALUES
(?1, ?2);
        "#,
        login,
        timestamp,
    ).execute(&mut *conn).await {
        Ok(_) => Ok(()),
        Err(err) => Err(ServerError::DBError(err.to_string())),
    }
}


/// `insert_chat_message` insert a single complete row into the `chat_messages` table.
/// Internal ID comes from a internal DB sequence.
pub async fn insert_chat_message(
        pool: &SqlitePool,
        login: &str,
        timestamp: &str,
        text: &str,
) -> Result<(), ServerError> {
    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(err) => Err(ServerError::DBError(err.to_string()))?,
    };

    match query!(
        r#"
INSERT INTO chat_messages
(login, timestamp, text)
VALUES
(?1, ?2, ?3);
        "#,
        login,
        timestamp,
        text,
    ).execute(&mut *conn).await {
        Ok(_) => Ok(()),
        Err(err) => Err(ServerError::DBError(err.to_string())),
    }
}
