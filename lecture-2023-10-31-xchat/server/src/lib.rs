mod db_queries;

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr};
use std::sync::{Arc, atomic};
use std::sync::atomic::Ordering::Relaxed;
use std::thread::{sleep};
use std::time::{self, SystemTime};

use sqlx::sqlite::{SqlitePool};
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use thiserror::Error;

use db_queries::{insert_login, insert_chat_message};
use shared::{Message, timestamp_to_string};


#[derive(Error, Debug)]
pub enum ServerError {
    #[error("failed port binding: {0}")]
    PortBindError(String),
    #[error("failed to establish client connection: {0}")]
    ClientConnectionError(String),
    #[error("failed to get peer address after client connection: {0}")]
    ClientPeerAddressError(String),
    #[error("failed stream configuration after client connection: {0}")]
    ClientStreamConfigError(String),
    #[error("internal error: detected poisoned mutex")]
    SharedMutexPoisonedError,
    #[error("failed to forward message to {address}: {detail} ")]
    ForwardMessageError{ address: String, detail: String },
    #[error("DB error: {0}")]
    DBError(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}


type ClientMap = HashMap::<SocketAddr, (TcpStream, String)>;
type Clients = Arc<Mutex<ClientMap>>;


/// `start_server` is entrypoint of server. It starts main processing loop in a separate thread
/// while main thread keep8s track on managing new client connections.
pub async fn start_server(
        address: &str,
        db_url: &str,
) -> Result<(), ServerError> {
    let client_map: ClientMap = HashMap::new();
    let clients: Clients = Arc::new(Mutex::new(client_map));

    let pool = match SqlitePool::connect(db_url).await {
        Ok(pool) => pool,
        Err(err) => Err(ServerError::DBError(err.to_string()))?,
    };

    let finish_flag = Arc::new(atomic::AtomicBool::new(false));

    let task_clients = clients.clone();
    let task_ok = finish_flag.clone();
    let mut chat_task = Some(tokio::spawn(async move {
        chat(task_clients, task_ok, &pool).await
    }));


    let task_address = address.to_string();
    let task_clients = clients.clone();
    let task_finish_flag = finish_flag.clone();
    let mut server_task = Some(tokio::spawn(async move {
        listen_and_accept(task_address, task_clients, task_finish_flag).await
    }));

    let res: Result<(), ServerError> = Ok(());
    loop {
        if chat_task.is_none() && server_task.is_none() {
            break
        }

        if let Some(handle) = chat_task.as_ref() {
            if handle.is_finished() {
                match tokio::try_join!(chat_task.take().unwrap()) {
                    Ok((Ok(_),)) => {},
                    Ok((Err(err),)) => eprintln!("CHAT THREAD ERROR: {}", err),
                    Err(err) => {
                        eprintln!("CHAT THREAD PANIC: {}", err.to_string())
                    },
                };
                finish_flag.store(true, Relaxed);
            }
        }

        if let Some(handle) = server_task.as_ref() {
            if handle.is_finished() {
                match tokio::try_join!(server_task.take().unwrap()) {
                    Ok((Ok(_),)) => {},
                    Ok((Err(err),)) => eprintln!("SERVER THREAD ERROR: {}", err),
                    Err(err) => {
                        eprintln!("SERVER THREAD PANIC: {}", err.to_string());
                    }
                }
                finish_flag.store(true, Relaxed);
            }
        }
    }

    res
}


/// `listen_and_accept` take care of connection of new client connections.
async fn listen_and_accept(
        address: String,
        clients: Clients,
        finish_flag: Arc<atomic::AtomicBool>,
) -> Result<(), ServerError> {
    let mut connections = 0_u64;

    let listener = match TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(err) => Err(ServerError::PortBindError(err.to_string()))?,
    };

    loop {
        let (stream, address) = match listener.accept().await {
            Ok((stream, address)) => (stream, address),
            Err(err) => Err(ServerError::ClientConnectionError(err.to_string()))?,
        };

        // Detection of error from the other thread.
        // TODO: incomming is blocking, so we would detect it currently only on any new client conn.
        if finish_flag.load(Relaxed) {
            eprintln!("SERVER THREAD: got finish signal");
            break
        };

        connections += 1;
        let temporary_login = format!("unknown {}", connections.to_string());
        clients.lock().await.insert(address, (stream, temporary_login));
    }

    Ok(())
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
        finish_flag: Arc<atomic::AtomicBool>,
        pool: &SqlitePool,
)  -> Result<(), ServerError> {
    let delay = time::Duration::from_micros(10);
    let mut message_queue: Vec<(SocketAddr, Message, String)> = vec![];
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
            for (address, (stream, login)) in client_map.iter_mut() {
                let message = Message::receive(stream).await;
                match message {
                    Ok(Some(Message::Login {login, pass})) => {
                        // Empty password is simulation of wrong password - using timeout on side
                        // of client to forcibly disconnect.
                        if !pass.is_empty() {
                            let welcome_message = format!("Welcome to x-chat {}!", login);
                            rename_queue.push((address.clone(), login));
                            let response = Message::Welcome {
                                motd: welcome_message,
                            };
                            if let Err(err) = response.send(stream).await {
                                eprintln!("failed to send welcome message: {}", err.to_string());
                            }
                        }
                    },
                    Ok(Some(message)) =>
                        message_queue.push((address.clone(), message, login.clone())),
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
            for (address, message, login) in message_queue.iter_mut() {
                match send_to_everyone_else(&clients, address, message, login, &pool).await {
                    Ok(_) => {},
                    Err(err) => eprintln!("sending failed: {}", err.to_string()),
                }
            }
        }

        // Removal of disconnected clients (writing also login/address for better debugging).
        if !close_queue.is_empty() {
            for address in close_queue.iter() {
                let mut clients = clients.lock().await;

                if let Some((_stream, login)) = clients.get(address) {
                    println!("Disconnected client {}/{}", login, address.to_string());
                    clients.remove(address);
                }
            }
        }

        // Renaming of authenticated clients.
        if !rename_queue.is_empty() {
            for (address, login) in rename_queue.iter() {
                clients.lock().await.entry(*address).and_modify(|client| {
                    client.1 = login.clone();
                });

                // Saving a row into DB.
                let timestamp = timestamp_to_string(SystemTime::now());
                if let Err(err) = insert_login(pool, login, &timestamp).await {
                    eprintln!("saving login entry failed: {}", err.to_string());
                }
            }
        }

        sleep(delay);
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

    for (address, (stream, _login)) in clients.lock().await.iter_mut() {
        if address == source_address {
            continue
        }

        if let Err(err) = message.send(stream).await {
            Err(ServerError::ForwardMessageError{
                address: address.to_string(),
                detail: err.to_string(),
            })?;
        }
    }

    Ok(())
}
