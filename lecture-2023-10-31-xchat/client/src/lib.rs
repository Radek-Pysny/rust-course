mod commands;

use std::io;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt};
use tokio::fs::{File, create_dir_all};
use tokio::time::{sleep, Duration};

#[cfg(debug_assertions)]
use color_eyre::eyre;
#[cfg(not(debug_assertions))]
use ::anyhow as eyre;
use eyre::{anyhow, bail, Result, Context};

use commands::{Command, MessageType};
use shared::{Message, timestamp_to_string};


#[repr(u8)]
enum OutputType {
    StandardOutput,
    ErrorOutput,
}


/// `run_interactive` is an entry point for interactive mode of this program.
/// It spins up three async tasks (input processing, server communication, and printing).
pub async fn run_interactive(
        address: &str,
        user_login: &str,
        user_pass: &str,
) -> Result<()> {
    #[cfg(debug_assertions)]
    color_eyre::install()?;

    const ERROR_PREFIX: &str = "ERROR: ";

    let mut stream = match TcpStream::connect(address).await {
        Ok(stream) => stream,
        Err(err) => bail!("failed to connect: {}", err.to_string()),
    };

    // Login process.
    match _login(&mut stream, user_login, user_pass).await {
        Ok(motd) => println!("connected!\n{}", motd),
        Err(err) => bail!("failed to authenticate: {}", err.to_string()),
    }

    // Channel for sending of commands from input task to processing task.
    let (tx_cmd, rx_cmd) =
        flume::unbounded::<(MessageType, Option<String>, Option<Vec<u8>>)>();

    // Channel for accepting of any messages to be print out on a stdout or stderr.
    let (tx_print, rx_print) = flume::unbounded::<(OutputType, String)>();

    // Input task takes care of reading from stdio, parsing `Action` and sending it together with
    // the rest text on the line over the channel to the processing task.
    let tx_print_for_input_task = tx_print.clone();
    let input_task = tokio::spawn(async move {
        let tx_print = tx_print_for_input_task;
        let mut text: String = String::new();
        loop {
            text.clear();

            let count = std::io::stdin().read_line(&mut text);
            if count.is_err() {
                bail!("failed to read from stdin: {}", count.err().unwrap().to_string());
            }
            if let Ok(0) = count { // no \n character -> finished by Ctrl+D
                return Ok(());
            }

            match Command::from_str(&text) {
                Ok(Command::Quit) => return Ok(()),
                Ok(Command::Empty) => continue,
                Ok(command) => match tx_cmd.send(command.extract()) {
                    Ok(_) => {},
                    Err(err) => tx_print.
                        send((OutputType::ErrorOutput, err.to_string())).
                        unwrap(),
                },
                Err(err) => tx_print.send((
                    OutputType::ErrorOutput,
                    format!("{}", err.to_string()),
                )).unwrap(),
            }
        }
    });

    // Processing task awaits a tuples (with action and text to be processed) from the input
    // channel, process the input text and prints output to the stdout.
    let process_task = tokio::spawn(async move {
        let tx_print = tx_print;    // takes ownership
        let mut processed = (false, false);
        let delay = Duration::from_millis(10);

        loop {
            // Processing command for sending a message to the server.
            processed.0 = true;
            match rx_cmd.try_recv() {
                Ok(request) => {
                    let message = match request {
                        (MessageType::File, Some(path), Some(content)) =>
                            Message::File {filename: path, payload: content},
                        (MessageType::Image, _, Some(content)) =>
                            Message::Image(content),
                        (MessageType::Text, Some(text), None) =>
                            Message::Text(text),
                        _ => continue,
                    };

                    match message.send(&mut stream).await {
                        Ok(_) => {},
                        Err(err) => tx_print.
                            send((OutputType::ErrorOutput,format!("{}", err.to_string()))).
                            unwrap(),
                    }
                }
                Err(flume::TryRecvError::Empty) => processed.0 = false, // nothing to be send
                Err(flume::TryRecvError::Disconnected) => break,
            }

            // Processing messages received from the server.
            processed.1 = true;
            match Message::receive(&mut stream).await {
                // nothing incoming from the server
                Ok(None) => processed.1 = false,

                // simply printing out any received text message
                Ok(Some(Message::Text(text))) => {
                    tx_print.send((OutputType::StandardOutput, text)).unwrap();
                },

                // received image should be saved as png file into the images subdirectory
                Ok(Some(Message::Image(payload))) => {
                    tx_print
                        .send((OutputType::StandardOutput, "Receiving image...".to_string()))
                        .unwrap();

                    if let Err(err) = save_image(payload).await {
                        let error_message = format!("Failed to save image: {}", err);
                        tx_print
                            .send((OutputType::ErrorOutput, error_message))
                            .unwrap();
                    }
                },

                // received file should be saved into the files subdirectory
                Ok(Some(Message::File{filename, payload})) => {
                    let info_text = format!("Receiving {}", filename);
                    tx_print.send((OutputType::StandardOutput, info_text)).unwrap();

                    if let Err(err) = save_file(&filename, payload).await {
                        let error_message = format!("Failed to save file: {}", err);
                        tx_print
                            .send((OutputType::ErrorOutput, error_message))
                            .unwrap();
                    }
                },

                Ok(Some(_)) => {
                    tx_print
                        .send((OutputType::ErrorOutput, "invalid message".to_string()))
                        .unwrap();
                },

                // write error message for any error that could possibly occur
                Err(err) => {
                    let error_message = err.to_string();
                    tx_print.send((OutputType::ErrorOutput, error_message.clone())).unwrap();
                    bail!("failed to receive a message from the server: {}", error_message);
                }
            }

            // Optional sleep that takes part in case of nothing being processed at this loop round.
            if let (false, false) = processed {
                sleep(delay).await;
            }
        }

        Ok(())
    });

    // Printing task takes care of any prints to stdout or stderr.
    let print_task = tokio::spawn(async move {
        while let Ok(request) = rx_print.recv() {
            match request {
                (OutputType::StandardOutput, text) => println!("{}", text),
                (OutputType::ErrorOutput, text) => eprintln!("{}{}", ERROR_PREFIX, text),
            }
        }

       Ok::<(), String>(())
    });

    let _ = tokio::try_join!(input_task, process_task, print_task);

    Ok(())
}


/// `save_image` save image as <timestamp>.png file under `images/` subdirectory. It expects, that
/// conversion of any image format was done by the client that sent image.
async fn save_image(payload: Vec<u8>) -> Result<()> {
    let timestamp = timestamp_to_string(SystemTime::now());
    let filepath_str = format!("./images/{}.png", timestamp);
    let filepath = Path::new(filepath_str.as_str());

    _save_file(&filepath, payload).await.with_context(||
        format!("saving image: {}", filepath_str)
    )
}


/// `save_file` save general file into `files/` subdirectory.
async fn save_file(filename: &String, payload: Vec<u8>) -> Result<()> {
    let filepath_str = format!("./files/{}", filename);
    let filepath = Path::new(filepath_str.as_str());

    _save_file(&filepath, payload).await.with_context(||
        format!("saving file: {}", filepath_str)
    )
}


/// `_save_file` is just a helper function that saved what is needed in the given filepath.
async fn _save_file(filepath: &Path, content: Vec<u8>) -> Result<()> {
    // create needed directories on path to the target file (if needed)
    if let Err(err) = create_dir_all(filepath.parent().unwrap()).await {
        bail!("failed to prepare directories: {}", err.to_string());
    }

    // create a new file (possibly truncating any already existing)
    let mut f = match File::create(&filepath).await {
        Ok(file) => file,
        Err(err) => bail!("failed to create file: {}", err.to_string()),
    };

    // write all the binary data into an empty file open for writing
    match f.write_all(&content).await {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!("failed to write into file: {}", err.to_string())),
    }
}


/// `login` take care of client authentication right after establishing a connection to the server.
pub async fn _login(stream: &mut TcpStream, login: &str, pass: &str) -> Result<String> {
    print!("Connection in progress...");
    let _ = io::stdout().flush();

    let message = Message::Login {
        login: login.to_string(),
        pass: pass.to_uppercase(),  // imagine that conversion to uppercase is password hashing
    };

    match message.send(stream).await {
        Ok(_) => {},
        Err(err) => bail!("failed to send authentication: {}", err.to_string()),
    };

    match Message::receive_with_timeout(stream, Duration::from_secs(5)).await {
        Ok(Some(Message::Welcome {motd})) => Ok(motd),
        Ok(_) => Err(anyhow!("authentication failed")),
        Err(err) => Err(err),
    }
}
