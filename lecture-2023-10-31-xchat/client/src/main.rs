use client::run_interactive;

fn main() {
    if let Err(err) = run_interactive("localhost:11111") {
        eprintln!("{}", err.to_string());
    }
}
