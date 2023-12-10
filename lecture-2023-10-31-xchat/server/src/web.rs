use std::sync::Arc;

use axum::{Router, routing::get, response::Html, Extension};
use axum::{extract::Query};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::ServerError;
use crate::db_queries::{fetch_chat_messages, fetch_users};
use shared::concat;


struct AppState {
    db_pool: SqlitePool,
    host: String,
}


#[derive(Deserialize)]
struct LoginFilter {
    login: Option<String>,
}


/// `user_list`
async fn user_list(
    state: Extension<Arc<AppState>>,
    login_filter: Query<LoginFilter>,
) -> Html<String> {
    let db_result = fetch_chat_messages(
        &state.db_pool,
        &login_filter.login,
    ).await;
    if let Err(_) = db_result {
        return Html("Failed to fetch user list!".to_string())
    }
    let chat_messages = db_result.unwrap();

    let db_result = fetch_users(&state.db_pool).await;
    if let Err(_) = db_result {
        return Html("Failed to fetch user list!".to_string())
    }
    let users = db_result.unwrap();

    let mut page: Vec<String> = vec![
        String::new(),
        "<table>".to_string(),
        " <tr>".to_string(),
        "  <th>".to_string(),
        "   timestamp".to_string(),
        "  </th>".to_string(),
        "  <th>".to_string(),
        "   user".to_string(),
        "  </th>".to_string(),
        "  <th>".to_string(),
        "   message".to_string(),
        "  </th>".to_string(),
        " </tr>".to_string(),
    ];

    for chat_message in chat_messages {
        let mut line: Vec<String> = vec![
            " <tr>".to_string(),
            "  <td>".to_string(),
            format!("   {}", chat_message.timestamp),
            "  </td>".to_string(),
            "  <td>".to_string(),
            format!("   {}", chat_message.login),
            "  </td>".to_string(),
            "  <td>".to_string(),
            format!("   {}", chat_message.text),
            "  </td>".to_string(),
            " </tr>".to_string(),
        ];

        page.append(&mut line);
    }

    page.push("</table>".to_string());

    let mut login_links: Vec<String> = vec![
        format!("Login filter: <a href='http://{}/'>all</a>", state.host),
    ];
    for user in users {
        login_links.push(format!(
            ", <a href='http://{}/?login={}'>{}</a>",
            state.host,
            user.login,  // TODO: URL-safe encoding
            user.login,
        ));
    }
    page[0].push_str(concat(&login_links).as_str());

    Html(concat(&page))
}


/// `start_web_server`
pub async fn start_web_server(
    port_number: u16,
    pool: SqlitePool,
) -> Result<(), ServerError> {
    let address = format!("localhost:{}", port_number);

    let state = Arc::new(AppState {
        db_pool: pool,
        host: address.clone(),
    });

    let router = Router::new()
        .route("/", get(user_list))
        .layer(Extension(state));

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(err) => Err(ServerError::WebServerError(err.to_string()))?,
    };

    match axum::serve(listener, router.into_make_service()).await {
        Ok(_) => Ok(()),
        Err(err) => Err(ServerError::WebServerError(err.to_string())),
    }
}
