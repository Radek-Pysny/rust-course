use std::sync::Arc;

use axum::{Router, routing::get, response::Html, Extension};
use sqlx::SqlitePool;

use crate::error::ServerError;
use crate::db_queries::{fetch_chat_messages};
use shared::concat;


struct AppState {
    db_pool: SqlitePool,
}


async fn user_list(
    state: Extension<Arc<AppState>>,
) -> Html<String> {
    let db_result = fetch_chat_messages(&state.db_pool).await;
    if let Err(_) = db_result {
        return Html("Failed to fetch user list!".to_string())
    }

    let mut page: Vec<String> = vec![
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

    for chat_message in db_result.unwrap() {
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

    Html(concat(&page))
}


pub async fn start_web_server(
    port_number: u16,
    pool: SqlitePool,
) -> Result<(), ServerError> {
    let state = Arc::new(AppState { db_pool: pool });

    let router = Router::new()
        .route("/", get(user_list))
        .layer(Extension(state));

    let address = format!("localhost:{}", port_number);

    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(err) => Err(ServerError::WebServerError(err.to_string()))?,
    };

    match axum::serve(listener, router.into_make_service()).await {
        Ok(_) => Ok(()),
        Err(err) => Err(ServerError::WebServerError(err.to_string())),
    }
}
