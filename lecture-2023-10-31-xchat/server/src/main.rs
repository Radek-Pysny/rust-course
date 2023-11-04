use server::start_server;


fn main() {
    if let Err(err) = start_server("0.0.0.0:11111") {
        eprintln!("{}", err);
    }
}
