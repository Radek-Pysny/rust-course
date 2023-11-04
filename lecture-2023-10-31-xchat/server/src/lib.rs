use std::collections::HashMap;
use std::error::Error;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{spawn, sleep};
use std::time;

use shared::Message;


type ClientMap = HashMap::<SocketAddr, TcpStream>;
type Clients = Arc<Mutex<ClientMap>>;


/// `start_server` is entrypoint of server. It starts main processing loop in a separate thread
/// while main thread keeps track on managing new client connections.
pub fn start_server(address: &str) -> Result<(), Box<dyn Error>> {
    let client_map: ClientMap = HashMap::new();
    let clients: Clients = Arc::new(Mutex::new(client_map));

    let thread_clients = clients.clone();
    let handle = spawn(|| chat(thread_clients) );

    listen_and_accept(address, clients.clone())?;

    let _ = handle.join();

    Ok(())
}


/// `listen_and_accept` take care of connection of new client connections.
fn listen_and_accept(address: &str, clients: Clients) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
        let stream = stream;
        if stream.as_ref().is_err() {
            continue
        }
        let stream = stream.unwrap();

        let address = stream.peer_addr();
        if address.as_ref().is_err() {
            continue
        }
        let address = address.unwrap();

        if let Err(_) = stream.set_nonblocking(true) {
            continue
        }

        let clients = clients.lock();
        if clients.as_ref().is_err() {
            continue
        }
        clients.unwrap().insert(address, stream);
    }

    Ok(())
}


/// `chat` implement main processing loop divided into three phases:
///   1. try to receive a message from each of connected clients
///   2. broadcast each received message to each other connected client
///   3. removal of disconnected clients from the internal map
fn chat(clients: Clients) {
    let delay = time::Duration::from_micros(10);
    let mut message_queue: Vec<(SocketAddr, Message)> = vec![];
    let mut close_queue: Vec<SocketAddr> = vec![];

    loop {
        let mut client_map = clients.lock().unwrap();
        message_queue.clear();
        close_queue.clear();

        // Receiving messages from clients and storing them into `message_queue`.
        for (address, stream) in client_map.iter_mut(){
            let message = Message::receive(stream);
            match message {
                Ok(message) => match message {
                    Some(message) => message_queue.push((address.clone(), message)),
                    None => continue,
                },
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

        // Broadcasting messages stored in `message_queue`.
        for (address, message) in message_queue.iter_mut() {
            match send_to_everyone_else(&mut client_map, address, message) {
                Ok(_) => {},
                Err(err) => eprintln!("sending failed: {}", err.to_string()),
            }
        }

        // Removal of disconnected clients.
        for address in close_queue.iter() {
            println!("Disconnected client {}", address.to_string());
            client_map.remove(address);
        }

        sleep(delay);
    }
}


/// `send_to_everyone_else` process sending of message to every client other to the message sender.
fn send_to_everyone_else(
    client_map: &mut MutexGuard<ClientMap>,
    source_address: &SocketAddr,
    message: &Message,
) -> Result<(), Box<dyn Error>> {
    for (address, stream) in client_map.iter_mut() {
        if address == source_address {
            continue
        }

        message.send(stream)?;
    }

    Ok(())
}
