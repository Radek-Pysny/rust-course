use std::sync::Arc;

use axum::{Router, routing::get, response::Html, Extension};
use axum::{extract::Query};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::ServerError;
use crate::db_queries::{fetch_chat_messages, fetch_users, delete_user_by_id};
use shared::concat;


struct AppState {
    db_pool: SqlitePool,
    host: String,
}


/// `start_web_server` is entrypoint for web server part of server crate.
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
        .route("/delete_user", get(delete_user))
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


#[derive(Deserialize)]
struct LoginFilterParam {
    login: Option<String>,
}


/// `user_list` is main endpoint (aka landing site) for the web server.
async fn user_list(
    state: Extension<Arc<AppState>>,
    login_filter: Query<LoginFilterParam>,
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

    // Preparing a links for filtering by login and user deletion.
    let mut filter_links: Vec<String> = vec![
        format!("<p>Login filter: <a href='http://{}/'>all</a>", state.host),
    ];
    let mut delete_links: Vec<String> = vec![
        "<p>Delete user: ".to_uppercase(),
    ];
    for user in users {
        filter_links.push(format!(
            ", <a href='http://{}/?login={}'>{}</a>",
            state.host,
            user.login,  // TODO: URL-safe encoding
            user.login,
        ));

        delete_links.push(format!(
            ", <a href='http://{}/delete_user?id={}&login={}'>{}</a>",
            state.host,
            user.id,    // TODO: perhaps base64 encoding to obfuscate it a bit...
            user.login, // TODO: URL-safe encoding
            user.login,
        ));
    }
    let mut filter_links_html = concat(&filter_links);
    filter_links_html.push_str("</p>");
    let mut delete_links_html = concat(&delete_links);
    delete_links_html.push_str("</p>");

    // Construction of the top-level page layout.
    let mut page: Vec<String> = vec![
        filter_links_html,
        delete_links_html,
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

    // Construction of table row for each chat message.
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

    Html(concat(&page))
}


#[derive(Deserialize)]
struct UserDeleteParam {
    id: Option<i64>,
    login: Option<String>,
}


/// `delete_user` is a web endpoint that is responsible for deletion of a single user by ID.
async fn delete_user(
    state: Extension<Arc<AppState>>,
    user_delete_id: Query<UserDeleteParam>,
) -> Html<String> {
    let go_back = format!(
        "<a href='http://{}'>Return back to user list.</a>",
        state.host,
    );

    if user_delete_id.id.is_none() && user_delete_id.login.is_none() {
        return Html(format!("Missing id and login parameters. {}", go_back));
    } else if user_delete_id.id.is_none() {
        return Html(format!("Missing id parameter. {}", go_back));
    } else if user_delete_id.login.is_none() {
        return Html(format!("Missing login parameter. {}", go_back));
    };

    let param_user_id = user_delete_id.id.unwrap();
    let param_user_login = user_delete_id.login.clone().unwrap();

    let db_result = fetch_users(&state.db_pool).await;
    if let Err(_) = db_result {
        return Html("Failed to fetch user list.".to_string())
    }
    let users = db_result.unwrap();

    let found = users.iter().any(|user| user.id == param_user_id && user.login == param_user_login);
    if !found {
        return Html(format!("User not found. {}", go_back));
    };

    let db_result = delete_user_by_id(&state.db_pool, param_user_id).await;
    if let Err(_) = db_result {
        return Html(format!("Failed to delete user. {}", go_back));
    };

    Html(format!(
        "Successfully deleted user with id {} and login {}. {}",
        param_user_id,
        param_user_login,
        go_back,
    ))
}
