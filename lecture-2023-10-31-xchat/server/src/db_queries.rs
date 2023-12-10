use std::result::Result;

use sqlx::{query, query_as};
use sqlx::sqlite::{SqlitePool};

use crate::ServerError;


#[derive(Clone, Debug)]
pub struct DbUser {
    pub id: i64,
    pub login: String,
    #[allow(dead_code)]
    password: String,
}

pub struct DbChatMessage {
    pub login: String,
    pub timestamp: String,
    pub text: String,
}


/// `fetch_user_by_login_and_password` receives a user from the `users` table.
pub async fn fetch_user_by_login_and_password(
        pool: &SqlitePool,
        login: &str,
        password: &str,
) -> Result<Option<DbUser>, ServerError> {
    match query_as!(
        DbUser,
        r#"
SELECT *
FROM users
WHERE
    login = ?1
    AND
    LOWER(password) = LOWER(?2)
;"#,
        login,
        password,
    ).fetch_one(pool).await {
        Ok(user) => Ok(Some(user)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(err) => Err(ServerError::DBError(err.to_string())),
    }
}


/// `fetch_chat_messages` fetch chat messages with optional filtering.
pub async fn fetch_chat_messages(
    pool: &SqlitePool,
    filter: &Option<String>,
) -> Result<Vec<DbChatMessage>, ServerError> {
    match filter {
        None => {
            match query_as!(
                DbChatMessage,
                r#"
SELECT
    u.login AS login,
    cm.timestamp AS timestamp,
    cm.text AS text
FROM
    chat_messages AS cm
    JOIN users AS u ON u.id = cm.user_id
ORDER BY timestamp DESC
;"#,
            ).fetch_all(pool).await {
                Ok(chat_messages) => Ok(chat_messages),
                Err(sqlx::Error::RowNotFound) => Ok(vec![]),
                Err(err) => Err(ServerError::DBError(err.to_string())),
            }
        },
        Some(login) => {
            match query_as!(
                DbChatMessage,
                r#"
SELECT
    u.login AS login,
    cm.timestamp AS timestamp,
    cm.text AS text
FROM
    chat_messages AS cm
    JOIN users AS u ON u.id = cm.user_id
WHERE u.login = ?1
ORDER BY timestamp DESC
;"#,
                login,
            ).fetch_all(pool).await {
                Ok(chat_messages) => Ok(chat_messages),
                Err(sqlx::Error::RowNotFound) => Ok(vec![]),
                Err(err) => Err(ServerError::DBError(err.to_string())),
            }
        },
    }
}


/// `fetch_users` fetch all users.
pub async fn fetch_users(
    pool: &SqlitePool,
) -> Result<Vec<DbUser>, ServerError> {
    match query_as!(
        DbUser,
        r#"
SELECT *
FROM users
ORDER BY login ASC
;"#,
    ).fetch_all(pool).await {
        Ok(users) => Ok(users),
        Err(sqlx::Error::RowNotFound) => Ok(vec![]),
        Err(err) => Err(ServerError::DBError(err.to_string())),
    }
}


/// `insert_login` inserts a single complete row into the `client_logins` table.
/// Internal ID comes from a internal DB sequence.
pub async fn insert_login(
        pool: &SqlitePool,
        user_id: i64,
        timestamp: &str,
) -> Result<(), ServerError> {
    let mut conn = match pool.acquire().await {
        Ok(conn) => conn,
        Err(err) => Err(ServerError::DBError(err.to_string()))?,
    };

    match query!(
        r#"
INSERT INTO client_logins
(user_id, timestamp)
VALUES
(?1, ?2)
;"#,
        user_id,
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
        user_id: i64,
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
(user_id, timestamp, text)
VALUES
(?1, ?2, ?3)
;"#,
        user_id,
        timestamp,
        text,
    ).execute(&mut *conn).await {
        Ok(_) => Ok(()),
        Err(err) => Err(ServerError::DBError(err.to_string())),
    }
}
