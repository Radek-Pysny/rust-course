use std::net::SocketAddr;

use axum::{Router, routing::get};

use crate::error::ServerError;


async fn hello_world() -> &'static str {
    "Hello, World!"
}


pub async fn start_web_server(port_number: u16) -> Result<(), ServerError> {
    let router = Router::new().route("/", get(hello_world));
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
