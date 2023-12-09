mod db_queries;
mod web;
mod error;

use std::collections::HashMap;
use std::io::{ErrorKind, Write};
use std::net::{SocketAddr};
use std::sync::{Arc};
use std::time::{SystemTime};
use axum::serve::Serve;

use sqlx::sqlite::{SqlitePool};
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;

use db_queries::{insert_login, insert_chat_message};
use shared::{Message, timestamp_to_string};
use crate::error::ServerError;

struct ChatMessage {
    address: SocketAddr,
    message: Message,
    login: String,
}

struct CloseMessage {
    address: SocketAddr,
}

struct RenameMessage {
    address: SocketAddr,
    login: String,
}

struct ClientRecord {
    login: Option<String>,
    sender: flume::Sender<ChatMessage>,
}

type ClientMap = HashMap::<SocketAddr, ClientRecord>;
type Clients = Arc<Mutex<ClientMap>>;


/// `start_server` is entrypoint of server. It starts main processing loop in a separate thread
/// while main thread keep8s track on managing new client connections.
pub async fn start_server(
        address: &str,
        db_url: &str,
        web_port: u16,
) -> Result<(), ServerError> {
    let mut join_set = JoinSet::new();

    let client_map: ClientMap = HashMap::new();
    let clients: Clients = Arc::new(Mutex::new(client_map));

    let (message_sender, message_receiver) =
        flume::unbounded::<ChatMessage>();
    let (close_sender, close_receiver) =
        flume::unbounded::<CloseMessage>();
    let (rename_sender, rename_receiver) =
        flume::unbounded::<RenameMessage>();

    let pool = match SqlitePool::connect(db_url).await {
        Ok(pool) => pool,
        Err(err) => Err(ServerError::DBError(err.to_string()))?,
    };

    // rename task
    let task_clients = clients.clone();
    let task_pool = pool.clone();
    join_set.spawn(async move {
        loop {
            match rename_receiver.recv_async().await {
                Ok(ref rename_message) => {
                    task_clients.lock().await.entry(rename_message.address).and_modify(|client| {
                        client.login = Some(rename_message.login.clone());
                    });

                    // Saving a row into DB.
                    let timestamp = timestamp_to_string(SystemTime::now());
                    if let Err(err) = insert_login(&task_pool, &rename_message.login, &timestamp).await {
                        Err(ServerError::UnspecifiedError(err.to_string()))?;
                    }
                },
                Err(err) => Err(ServerError::UnspecifiedError(err.to_string()))?,
            }
        }
    });

    // rename task
    let task_clients = clients.clone();
    join_set.spawn(async move {
        loop {
            match close_receiver.recv_async().await {
                Ok(close_message) => {
                    let mut clients = task_clients.lock().await;

                    if let Some(client_record) = clients.get(&close_message.address) {
                        let name = match &client_record.login {
                            Some(login) => login.to_string(),
                            None => "not authorized client".to_string(),
                        };
                        println!("Disconnected client {}/{}", name, close_message.address.to_string());
                        clients.remove(&close_message.address);
                    }
                }
                Err(err) => Err(ServerError::UnspecifiedError(err.to_string()))?,
            }
        }
    });

    // chat task
    let task_clients = clients.clone();
    let task_message_receiver = message_receiver.clone();
    join_set.spawn(async move {
        chat(task_clients, &pool, task_message_receiver).await
    });

    // server task
    let task_address = address.to_string();
    let task_clients = clients.clone();
    let task_message_sender = message_sender.clone();
    let task_close_sender = close_sender.clone();
    let task_rename_sender = rename_sender.clone();
    join_set.spawn(async move {
        listen_and_accept(
            task_address,
            task_clients,
            task_message_sender,
            task_close_sender,
            task_rename_sender,
        ).await
    });

    // web task
    join_set.spawn(async move {
        web::start_web_server(web_port).await
    });

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(_)) => {},
            Ok(Err(err)) => eprint!("server error: {}", err.to_string()),
            Err(err) => eprint!("join error: {}", err.to_string()),
        }
    };

    Ok(())
}


/// `listen_and_accept` take care of connection of new client connections.
async fn listen_and_accept(
        address: String,
        clients: Clients,
        message_sender: flume::Sender<ChatMessage>,
        close_sender: flume::Sender<CloseMessage>,
        rename_sender: flume::Sender<RenameMessage>,
) -> Result<(), ServerError> {
    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(err) => Err(ServerError::PortBindError(err.to_string()))?,
    };

    loop {
        let (stream, address) = match listener.accept().await {
            Ok((stream, address)) => (stream, address),
            Err(err) => Err(ServerError::ClientConnectionError(err.to_string()))?,
        };

        let (chat_sender, chat_receiver) = flume::unbounded::<ChatMessage>();
        let client_record = ClientRecord {
            login: None,
            sender: chat_sender,
        };
        clients.lock().await.insert(address, client_record);

        let task_message_sender = message_sender.clone();
        let task_close_sender = close_sender.clone();
        let task_rename_sender = rename_sender.clone();
        let task_clients = clients.clone();
        tokio::spawn(async move {
            client(
                address,
                stream,
                task_clients,
                task_message_sender,
                task_close_sender,
                task_rename_sender,
                chat_receiver,
            ).await
        });
    }
}


async fn client(
    address: SocketAddr,
    mut stream: TcpStream,
    clients: Clients,
    message_sender: flume::Sender<ChatMessage>,
    close_sender: flume::Sender<CloseMessage>,
    rename_sender: flume::Sender<RenameMessage>,
    chat_receiver: flume::Receiver<ChatMessage>,
) -> Result<(), ServerError> {
    loop {
        tokio::select! {
            message = Message::blocking_receive(&mut stream) => {
                match message {
                    Ok(Some(Message::Login { login, pass })) => {
                        // Empty password is simulation of wrong password - using timeout on side
                        // of client to forcibly disconnect.
                        if !pass.is_empty() {
                            let welcome_message = format!("Welcome to x-chat {}!", login);
                            let rename_message = RenameMessage {
                                address: address.clone(),
                                login,
                            };
                            match rename_sender.send(rename_message) {
                                Ok(_) => {
                                    let response = Message::Welcome {
                                        motd: welcome_message,
                                    };
                                    if let Err(err) = response.send(&mut stream).await {
                                        Err(ServerError::ClientSendError(err.to_string()))?;
                                    }
                                },
                                Err(err) => Err(ServerError::UnspecifiedError(err.to_string()))?,
                            }
                        }
                    },
                    Ok(Some(received_message)) => {
                        if let Some(Some(current_login)) = clients.lock().await.get(&address).map(
                            |client_record| client_record.login.clone()
                        ) { // not yet authenticated user (without login) cannot send any message
                            let message_to_send = ChatMessage {
                                address: address.clone(),
                                message: received_message,
                                login: current_login,
                            };
                            match message_sender.send(message_to_send) {
                                Ok(_) => {},
                                Err(err) => eprintln!("failed pass sent message to chat: {}", err),
                            };
                        }
                    }
                    Ok(None) =>
                        continue,
                    Err(err) => match err.downcast_ref::<std::io::Error>() {
                        // Detected a disconnected client.
                        Some(err) if err.kind() == ErrorKind::UnexpectedEof => {
                            let close_message = CloseMessage{address: address.clone()};
                            match close_sender.send(close_message) {
                                Ok(_) => {},
                                Err(err) => Err(ServerError::UnspecifiedError(err.to_string()))?,
                            };
                        },

                        Some(err) => eprintln!(
                            "I/O error: {}; kind: {}",
                            err.to_string(),
                            err.kind()
                        ),

                        None => eprintln!("not I/O error"),
                    }
                }
            }

            result = chat_receiver.recv_async() => {
                match result {
                    Ok(received_message) =>
                        if let Err(err) = received_message.message.send(&mut stream).await {
                            Err(ServerError::ClientSendError(err.to_string()))?;
                        },
                    Err(err) => Err(ServerError::UnspecifiedError(err.to_string()))?,
                }
            }
        }
    }
}


/// `chat` implement main processing loop.
///
///  It is divided into the following phases:
///   1. try to receive a message from each of connected clients
///   2. broadcast each received message to each other connected client
///   3. removal of disconnected clients from the internal client map
///   4. renaming of authenticated clients (by default they have name in a form `unknown N`, where
///         `N` is a number from internal running sequence and they got the name/login based on
///         the content of the `Login` message received from client during authentication phase).
async fn chat(
        // Map of clients shared across threads.
        clients: Clients,
        pool: &SqlitePool,
        message_receiver: flume::Receiver<ChatMessage>,
)  -> Result<(), ServerError> {
    loop {
        match message_receiver.recv() {
            Ok(mut received_message) => {
                if let Err(err) = send_to_everyone_else(
                    &clients,
                    &received_message.address,
                    &mut received_message.message,
                    &received_message.login,
                    &pool,
                ).await {
                    Err(ServerError::UnspecifiedError(err.to_string()))?;
                }
            },
            Err(err) => Err(ServerError::UnspecifiedError(err.to_string()))?,
        }
    }
}


/// `send_to_everyone_else` process sending of message to every client other to the message sender.
async fn send_to_everyone_else(
        clients: &Clients,
        source_address: &SocketAddr,
        message: &mut Message,
        login: &String,
        pool: &SqlitePool,
) -> Result<(), ServerError> {
    if let Message::Text(text) = message {
        // Saving a row into DB.
        let timestamp = timestamp_to_string(SystemTime::now());
        let result = insert_chat_message(pool, login, &timestamp, &text).await;
        if let Err(err) = result {
            eprintln!("saving chat message entry failed: {}", err.to_string());
        }

        *message = Message::Text(format!("{}: {}", login, text));
    }

    for (address, client_record) in clients.lock().await.iter_mut() {
        if address == source_address {
            continue
        }

        let message_to_send = ChatMessage{
            address: address.clone(),
            message: message.clone(),
            login: login.clone(),
        };

        if let Err(err) = client_record.sender.send(message_to_send) {
            Err(ServerError::ForwardMessageError{
                address: address.to_string(),
                detail: err.to_string(),
            })?;
        }
    }

    Ok(())
}
