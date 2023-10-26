mod extra;
mod text_processing;
mod types;

use std::error::Error;
use std::{env};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::JoinHandle;

use strum::IntoEnumIterator;

use types::Action;


type Result = std::result::Result<String, Box<dyn Error>>;


/// `run` is an entry-point for non-interactive mode of this program.
/// It return either modified string or empty error string (requesting for help print int stdout)
/// or non-empty error string (requesting error report to the user).
pub fn run(command: &str, options: Vec<&str>) -> Result {
    let action = Action::from_str(command)?;
    action.single_shot_act(options)
}

/// `run_interactive` is an entry point for interactive mode of this program.
/// It spins up two threads (one for processing of input and one for text processing itself).
pub fn run_interactive() -> std::result::Result<(), Box<dyn Error>> {
    const ERROR_PREFIX: &str = "ERROR: ";

    let (tx, rx) = channel::<(Action, String)>();

    // Input thread takes care of reading from stdio, parsing `Action` and sending it together with
    // the rest text on the line over the channel to the processing thread.
    let input_thread_handle = thread::spawn(move || {
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

            let mut parts = text
                .trim()
                .splitn(2, char::is_whitespace)
                .map(str::to_string)
                .collect::<Vec<String>>();

            if parts.len() != 2 {
                eprintln!("{ERROR_PREFIX}missing action specification");
                continue
            }

            let input_text = parts.pop().unwrap();
            let action_text = parts.pop().unwrap();

            match Action::from_str(&action_text) {
                Ok(action) => tx.send((action, input_text)).unwrap(),
                Err(err) => eprintln!("{ERROR_PREFIX}{err}"),
            }
        }
    });

    // Processing thread awaits a tuples (with action and text to be processed) from the input
    // channel, process the input text and prints output to the stdout.
    let processing_thread_handle = thread::spawn(move || {
        while let Ok((action, text)) = rx.recv() {
            match action.act(&text) {
                Ok(result) => println!("{}", result),
                Err(err) => eprintln!("{ERROR_PREFIX}{err}"),
            }

        }

        Ok::<(), String>(())
    });

    // Trial to do there some reasonable clean up after the threads has finished their work.
    join_thread(input_thread_handle, "input thread")?;
    join_thread(processing_thread_handle, "processing thread")
}

/// `print_help` print help text on stdout.
pub fn print_help() {
    let bin_name = &env::args().take(1).collect::<Vec<String>>()[0];
    let action_list = Action::iter().map(|a|a.to_string()).collect::<Vec<_>>();

    println!("Usage:");
    println!("   {} <action>", bin_name);
    println!("        where action is one of: {}.", action_list.join(", "));
}

/// `join_thread` is just a helper function converting possible errors from thread panicking.
fn join_thread<T>(handle: JoinHandle<T>, thread_name: &str) -> std::result::Result<(), Box<dyn Error>> {
    if let Err(err) = handle.join() {
        let error_message = match err.downcast_ref::<&str>() {
            Some(err) => *err,
            None => "unknown error",
        };
        return Err(format!("{} panicked: {}", thread_name, error_message))?;
    };
    Ok(())
}
