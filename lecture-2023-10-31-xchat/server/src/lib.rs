use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, atomic};
use std::sync::atomic::Ordering::Relaxed;
use std::thread::{spawn, sleep};
use std::time;

use rayon::prelude::*;
use thiserror::Error;

use shared::{Message, panic_to_text};


#[derive(Error, Debug)]
pub enum ServerError {
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
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}


type ClientMap = HashMap::<SocketAddr, TcpStream>;
type Clients = Arc<Mutex<ClientMap>>;


/// `start_server` is entrypoint of server. It starts main processing loop in a separate thread
/// while main thread keep8s track on managing new client connections.
pub fn start_server(address: &str) -> Result<(), ServerError> {
    let client_map: ClientMap = HashMap::new();
    let clients: Clients = Arc::new(Mutex::new(client_map));

    let finish_flag = Arc::new(atomic::AtomicBool::new(false));

    let thread_clients = clients.clone();
    let thread_ok = finish_flag.clone();
    let mut chat_handle = Some(spawn(||
        chat(thread_clients, thread_ok)
    ));


    let thread_address = address.to_string();
    let thread_clients = clients.clone();
    let thread_finish_flag = finish_flag.clone();
    let mut server_handle = Some(spawn(move ||
        listen_and_accept(thread_address, thread_clients, thread_finish_flag)
    ));

    let res: Result<(), ServerError> = Ok(());
    loop {
        if chat_handle.is_none() && server_handle.is_none() {
            break
        }

        if let Some(handle) = chat_handle.as_ref() {
            if handle.is_finished() {
                match chat_handle.take().unwrap().join() {
                    Ok(Ok(_)) => {},
                    Ok(Err(err)) => eprintln!("CHAT THREAD ERROR: {}", err),
                    Err(err) => {
                        eprintln!("CHAT THREAD PANIC: {}", panic_to_text(err))
                    },
                };
                finish_flag.store(true, Relaxed);
            }
        }

        if let Some(handle) = server_handle.as_ref() {
            if handle.is_finished() {
                match server_handle.take().unwrap().join() {
                    Ok(Ok(_)) => {},
                    Ok(Err(err)) => eprintln!("SERVER THREAD ERROR: {}", err),
                    Err(err) => {
                        eprintln!("SERVER THREAD PANIC: {}", panic_to_text(err));
                    }
                }
                finish_flag.store(true, Relaxed);
            }
        }
    }

    res
}


/// `listen_and_accept` take care of connection of new client connections.
fn listen_and_accept(
    address: String,
    clients: Clients,
    finish_flag: Arc<atomic::AtomicBool>,
) -> Result<(), ServerError> {
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
        // Detection of error from the other thread.
        // TODO: incomming is blocking, so we would detect it currently only on any new client conn.
        if finish_flag.load(Relaxed) {
            eprintln!("SERVER THREAD: got finish signal");
            break
        }

        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => Err(ServerError::ClientConnectionError(err.to_string()))?,
        };

        let address = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(err) => Err(ServerError::ClientPeerAddressError(err.to_string()))?,
        };

        // if let Err(err) = stream.set_nonblocking(true) {
        //     ServerError::ClientStreamConfigError(err.to_string())?;
        // }
        if let Err(err) = stream.set_read_timeout(Some(time::Duration::from_nanos(10))) {
            Err(ServerError::ClientStreamConfigError(err.to_string()))?;
        }

        if let Err(err) = stream.set_write_timeout(Some(time::Duration::from_nanos(10))) {
            Err(ServerError::ClientStreamConfigError(err.to_string()))?;
        }

        let mut clients = match clients.lock() {
            Ok(clients) => clients,
            Err(_poisoned) => Err(ServerError::SharedMutexPoisonedError)?,
        };
        clients.insert(address, stream);
    }

    Ok(())
}


/// `chat` implement main processing loop divided into three phases:
///   1. try to receive a message from each of connected clients
///   2. broadcast each received message to each other connected client
///   3. removal of disconnected clients from the internal map
fn chat(
    clients: Clients,
    finish_flag: Arc<atomic::AtomicBool>,
)  -> Result<(), ServerError> {
    let delay = time::Duration::from_micros(10);
    let mut message_queue: Vec<(SocketAddr, Message)> = vec![];
    let mut close_queue: Vec<SocketAddr> = vec![];

    loop {
        // Detection of error from the other thread.
        if finish_flag.load(Relaxed) {
            return Ok(());
        }

        message_queue.clear();
        close_queue.clear();

        {
            let mut client_map = clients.lock().unwrap();

            // Receiving messages from clients and storing them into `message_queue`.
            for (address, stream) in client_map.iter_mut() {
                let message = Message::receive(stream);
                match message {
                    Ok(Some(message)) =>
                        message_queue.push((address.clone(), message)),
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
            message_queue.par_iter().for_each(|(address, message)| {
                let result = send_to_everyone_else(&clients, address, message);
                if let Err(err) = result {
                    eprintln!("{}", err.to_string())
                }
            });
        }

        // Removal of disconnected clients.
        if !close_queue.is_empty() {
            close_queue.par_iter().for_each(|address| {
                println!("Disconnected client {}", address.to_string());
                clients.lock().unwrap().remove(address);
            });
        }

        sleep(delay);
    }
}


/// `send_to_everyone_else` process sending of message to every client other to the message sender.
fn send_to_everyone_else(
    clients: &Clients,
    source_address: &SocketAddr,
    message: &Message,
) -> Result<(), ServerError> {
    for (address, stream) in clients.lock().unwrap().iter_mut() {
        if address == source_address {
            continue
        }

        if let Err(err) = message.send(stream) {
            Err(ServerError::ForwardMessageError{
                address: address.to_string(),
                detail: err.to_string(),
            })?;
        }
    }

    Ok(())
}
