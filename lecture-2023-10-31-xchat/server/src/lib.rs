mod db_queries;
mod web;
mod error;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr};
use std::sync::{Arc, atomic};
use std::sync::atomic::Ordering::Relaxed;
use std::time::{SystemTime};

use sqlx::sqlite::{SqlitePool};
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

use db_queries::{insert_login, insert_chat_message, fetch_user_by_login_and_password};
use shared::{Message, timestamp_to_string};
use crate::error::ServerError;


struct ClientRecord {
    stream: TcpStream,
    login: Option<String>,
    user_id: Option<i64>,
}


type ClientMap = HashMap::<SocketAddr, ClientRecord>;
type Clients = Arc<Mutex<ClientMap>>;


struct MessageRecord {
    address: SocketAddr,
    message: Message,
    login: String,
    user_id: i64,
}


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

    let pool = match SqlitePool::connect(db_url).await {
        Ok(pool) => pool,
        Err(err) => Err(ServerError::DBError(err.to_string()))?,
    };

    let finish_flag = Arc::new(atomic::AtomicBool::new(false));

    // chat task
    let task_clients = clients.clone();
    let task_ok = finish_flag.clone();
    let task_pool = pool.clone();
    join_set.spawn(async move {
        chat(task_clients, task_ok, &task_pool).await
    });

    // server task
    let task_address = address.to_string();
    let task_clients = clients.clone();
    let task_finish_flag = finish_flag.clone();
    join_set.spawn(async move {
        listen_and_accept(task_address, task_clients, task_finish_flag).await
    });

    // web task
    let task_pool = pool.clone();
    join_set.spawn(async move {
        web::start_web_server(web_port, task_pool).await
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
        finish_flag: Arc<atomic::AtomicBool>,
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

        // Detection of error from the other thread.s
        if finish_flag.load(Relaxed) {
            eprintln!("SERVER THREAD: got finish signal");
            break
        };

        let client_record = ClientRecord{
            stream: stream,
            login: None,
            user_id: None,
        };
        clients.lock().await.insert(address, client_record);
    }

    Ok(())
}


/// `chat` implement main processing loop.
///
///  It is divided into the following phases:
///   1. try to receive a message from each of connected clients
///   2. broadcast each received message to each other connected client
///   3. removal of disconnected clients from the internal client map
async fn chat(
        // Map of clients shared across threads.
        clients: Clients,
        finish_flag: Arc<atomic::AtomicBool>,
        pool: &SqlitePool,
)  -> Result<(), ServerError> {
    let mut message_queue: Vec<MessageRecord> = vec![];
    let mut close_queue: Vec<SocketAddr> = vec![];
    let mut rename_queue: Vec<(SocketAddr, String)> = vec![];

    loop {
        // Detection of error from the other thread.
        if finish_flag.load(Relaxed) {
            return Ok(());
        }

        message_queue.clear();
        close_queue.clear();
        rename_queue.clear();

        {
            let mut client_map = clients.lock().await;

            // Receiving messages from clients and storing them into `message_queue`.
            for (address, client_record) in client_map.iter_mut() {
                let message = Message::receive(&mut client_record.stream).await;
                match message {
                    Ok(Some(Message::Login {login, pass})) => {
                        // Searching for login & password in the DB as a part of authorization.
                        match fetch_user_by_login_and_password(pool, &login, &pass).await {
                            Ok(Some(user)) => {
                                let welcome_message = format!("Welcome to x-chat {}!", login);

                                client_record.login = Some(login);
                                client_record.user_id = Some(user.id);

                                let timestamp = timestamp_to_string(SystemTime::now());
                                if let Err(err) = insert_login(pool, user.id, &timestamp).await {
                                    eprintln!("saving login entry failed: {}", err.to_string());
                                }
                                // rename_queue.push((address.clone(), login));

                                let response = Message::Welcome {
                                    motd: welcome_message,
                                };

                                if let Err(err) = response.send(&mut client_record.stream).await {
                                    eprintln!("failed to send welcome message: {}", err.to_string());
                                }
                            },
                            Ok(None) => continue, // no response -> client is not authorized in timeout
                            Err(err) => Err(err)?,
                        };
                    },
                    Ok(Some(message)) => {
                        if let Some(login) = &client_record.login {
                            if let Some(user_id) = &client_record.user_id {
                                let message_record = MessageRecord{
                                    user_id: user_id.clone(),
                                    login: login.clone(),
                                    message: message,
                                    address: address.clone(),
                                };
                                message_queue.push(message_record);
                            }
                        }
                    }
                    Ok(None) =>
                        continue,
                    Err(err) => match err.downcast_ref::<std::io::Error>() {
                        // Detected a disconnected client.
                        Some(err) if err.kind() == ErrorKind::UnexpectedEof =>
                            close_queue.push(address.clone()),

                        Some(err) => eprintln!(
                            "I/O error: {}; kind: {}",
                            err.to_string(),
                            err.kind()
                        ),

                        None => eprintln!("not I/O error"),
                    }
                }
            }
        }

        // Broadcasting messages stored in `message_queue`.
        if !message_queue.is_empty() {
            for message_record in message_queue.drain(..) {
                match send_to_everyone_else(&clients, message_record, &pool).await {
                    Ok(_) => {},
                    Err(err) => eprintln!("sending failed: {}", err.to_string()),
                }
            }
        }

        // Removal of disconnected clients (writing also login/address for better debugging).
        if !close_queue.is_empty() {
            for address in close_queue.iter() {
                let mut clients = clients.lock().await;

                if let Some(client_record) = clients.get(address) {
                    let unknown_name = "unknown".to_string();
                    println!(
                        "Disconnected client {}/{}",
                        client_record.login.as_ref().unwrap_or(&unknown_name),
                        address.to_string(),
                    );
                    clients.remove(address);
                }
            }
        }
        sleep(Duration::from_millis(10)).await;
    }
}


/// `send_to_everyone_else` process sending of message to every client other to the message sender.
async fn send_to_everyone_else(
        clients: &Clients,
        mut message_record: MessageRecord,
        pool: &SqlitePool,
) -> Result<(), ServerError> {
    if let Message::Text(text) = &mut message_record.message {
        // Saving a row into DB.
        let timestamp = timestamp_to_string(SystemTime::now());
        let result = insert_chat_message(
            pool,
            message_record.user_id,
            &timestamp,
            &text,
        ).await;
        if let Err(err) = result {
            eprintln!("saving chat message entry failed: {}", err.to_string());
        };

        message_record.message = Message::Text(format!("{}: {}", message_record.login, text));
    }

    for (address, client_record) in clients.lock().await.iter_mut() {
        if address == &message_record.address {
            continue
        }

        if let Err(err) = message_record.message.send(&mut client_record.stream).await {
            Err(ServerError::ForwardMessageError{
                address: address.to_string(),
                detail: err.to_string(),
            })?;
        }
    }

    Ok(())
}
