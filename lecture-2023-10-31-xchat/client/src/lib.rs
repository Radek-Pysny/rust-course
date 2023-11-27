mod commands;

use std::io::Write;
use std::fs::{File, create_dir_all};
use std::path::Path;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::mpsc::{channel, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time;

#[cfg(debug_assertions)]
use color_eyre::eyre;
#[cfg(not(debug_assertions))]
use ::anyhow as eyre;
use eyre::{anyhow, Result, Context};

use commands::{Command, MessageType};
use shared::{Message, panic_to_text};


#[repr(u8)]
enum OutputType {
    StandardOutput,
    ErrorOutput,
}


/// `run_interactive` is an entry point for interactive mode of this program.
/// It spins up two threads (one for processing of input and one for text processing itself).
pub fn run_interactive(address: &str) -> Result<()> {
    #[cfg(debug_assertions)]
    color_eyre::install()?;

    const ERROR_PREFIX: &str = "ERROR: ";

    let mut stream = match TcpStream::connect(address) {
        Ok(stream) => stream,
        Err(err) => Err(format!("failed to connect: {}", err.to_string()))?,
    };
    if let Err(err) = stream.set_nonblocking(true) {
        Err(format!("failed to set stream attribute: {}", err.to_string()))?;
    }

    // Channel for sending of commands from input thread to processing thread.
    let (tx_cmd, rx_cmd) =
        channel::<(MessageType, Option<String>, Option<Vec<u8>>)>();

    // Channel for accepting of any messages to be print out on a stdout or stderr.
    let (tx_print, rx_print) = channel::<(OutputType, String)>();

    // Input thread takes care of reading from stdio, parsing `Action` and sending it together with
    // the rest text on the line over the channel to the processing thread.
    let tx_print_for_input_thread = tx_print.clone();
    let input_thread_handle = thread::spawn(move || {
        let tx_print = tx_print_for_input_thread;
        let mut text: String = String::new();
        loop {
            text.clear();

            let count = std::io::stdin().read_line(&mut text);
            if count.is_err() {
                return Err(count.err().unwrap().to_string());
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

    // Processing thread awaits a tuples (with action and text to be processed) from the input
    // channel, process the input text and prints output to the stdout.
    let processing_thread_handle = thread::spawn(move || {
        let tx_print = tx_print;    // takes ownership
        let mut processed = (false, false);
        let delay = time::Duration::from_millis(10);

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

                    match message.send(&mut stream) {
                        Ok(_) => {},
                        Err(err) => tx_print.
                            send((OutputType::ErrorOutput,format!("{}", err.to_string()))).
                            unwrap(),
                    }
                }
                Err(TryRecvError::Empty) => processed.0 = false, // nothing to be send to server
                Err(TryRecvError::Disconnected) => break,
            }

            // Processing messages received from the server.
            processed.1 = true;
            match Message::receive(&mut stream) {
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

                    if let Err(err) = save_image(payload) {
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

                    if let Err(err) = save_file(&filename, payload) {
                        let error_message = format!("Failed to save file: {}", err);
                        tx_print
                            .send((OutputType::ErrorOutput, error_message))
                            .unwrap();
                    }
                },

                // write error message for any error that could possibly occur
                Err(err) => {
                    tx_print.send((OutputType::ErrorOutput, err.to_string())).unwrap();
                    return Err(err.to_string());
                }
            }

            // Optional sleep that takes part in case of nothing being processed at this loop round.
            if let (false, false) = processed {
                thread::sleep(delay);
            }
        }

        Ok::<(), String>(())
    });

    // Printing thread takes care of any prints to stdout or stderr.
    let printing_thread_handle = thread::spawn(move || {
        while let Ok(request) = rx_print.recv() {
            match request {
                (OutputType::StandardOutput, text) => println!("{}", text),
                (OutputType::ErrorOutput, text) => eprintln!("{}{}", ERROR_PREFIX, text),
            }
        }

       Ok::<(), String>(())
    });

    // Trial to do there some reasonable clean up after the threads has finished their work.
    join_thread(input_thread_handle).context("input thread")?;
    join_thread(processing_thread_handle).context("processing thread")?;
    join_thread(printing_thread_handle).context("printing thread")
}


/// `print_help` print help text on stdout.
// pub fn print_help() {
//     let bin_name = &env::args().take(1).collect::<Vec<String>>()[0];
//     let action_list = Action::iter().map(|a|a.to_string()).collect::<Vec<_>>();
//
//     println!("Usage:");
//     println!("   {} <action>", bin_name);
//     println!("        where action is one of: {}.", action_list.join(", "));
// }


/// `join_thread` is just a helper function converting possible errors from thread panicking.
fn join_thread<T: Display>(handle: JoinHandle<T>) -> Result<()> {
    match handle.join() {
        Ok(ok) => {},
        Err(err) => anyhow!("thread panicked: {}", panic_to_text(err)),
    };
    Ok(())
}


fn save_image(payload: Vec<u8>) -> Result<()> {
    use chrono::offset::Local;
    use chrono::DateTime;
    use std::time::SystemTime;

    let now: DateTime<Local> = SystemTime::now().into();
    let timestamp = now.format("%Y-%m-%dT%H:%I");

    let filepath_str = format!("./images/{}.png", timestamp);
    let filepath = Path::new(filepath_str.as_str());

    _save_file(&filepath, payload).with_context(||
        format!("saving image: {}", filepath_str)
    )
}


fn save_file(filename: &String, payload: Vec<u8>) -> Result<()> {
    let filepath_str = format!("./files/{}", filename);
    let filepath = Path::new(filepath_str.as_str());

    _save_file(&filepath, payload).with_context(||
        format!("saving file: {}", filepath_str)
    )
}


fn _save_file(filepath: &Path, content: Vec<u8>) -> Result<()> {
    // create needed directories on path to the target file (if needed)
    if let Err(err) = create_dir_all(filepath.parent().unwrap()) {
        anyhow!("failed to prepare directories: {}", err.to_string());
    }

    // create a new file (possibly truncating any already existing)
    let mut f = match File::create(&filepath) {
        Ok(file) => file,
        Err(err) => anyhow!("failed to create file: {}", err.to_string()),
    };

    // write all the binary data into an empty file open for writing
    match f.write_all(&content) {
        Ok(_) => Ok(()),
        Err(err) => anyhow!("failed to write into file: {}", err.to_string()),
    }
}
