use std::env;
use lecture::{print_help, run, run_interactive};


fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let first_arg = args.get(0).map(String::as_str);
    let options = args.iter().skip(1).map(String::as_str).collect::<Vec<&str>>();
    match first_arg {
        None => match run_interactive() {
            Ok(_) => {},
            Err(error_message) => eprintln!("{}", error_message),
        },
        Some("-h") | Some("--help") => print_help(),
        Some(command) => match run(command, options) {
            Ok(text) => println!("{}", text),
            Err(error_message) => eprintln!("{}", error_message),
        },
    }
}
