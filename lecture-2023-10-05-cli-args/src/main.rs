use lecture_2023_10_05_cli_args::{run, help};

fn main() {
    match run() {
        Ok(s) => println!("{}", s),
        Err(err) => {
            help();
            if !err.is_empty() {
                eprintln!("ERROR: {}", err);
            }
        },
    }
}
