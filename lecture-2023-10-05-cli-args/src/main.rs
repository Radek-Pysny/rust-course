use std::env;
use lecture::{print_help, run};


fn main() {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let first_arg = args.get(0);
    let options = args.iter().skip(1).map(String::as_str).collect::<Vec<&str>>();
    match first_arg {
        None => print_help(),
        Some(command) => {
            match run(command, options) {
                Ok(text) => println!("{}", text),
                Err(error_text) => eprintln!("{}", error_text)
            }
        },
    }
}
