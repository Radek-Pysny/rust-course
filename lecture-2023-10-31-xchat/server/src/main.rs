use std::process::exit;

use server::start_server;


#[tokio::main]
async fn main() {
    let mut comm_hostname = "localhost".to_string();
    let mut comm_port = 11111_u16;
    let mut db_url = "sqlite:data.db".to_string();
    let mut web_port = 8080_u16;

    parse_arguments(&mut comm_hostname, &mut comm_port, &mut db_url, &mut web_port);

    let address = format!("{}:{}", comm_hostname, comm_port);

    if let Err(err) = start_server(&address, &db_url, web_port).await {
        eprintln!("{}", err);
    }
}


/// `parse_arguments` uses [argparse](https://crates.io/crates/argparse) crate to parse command-line
/// options.
fn parse_arguments(
    comm_hostname: &mut String,
    comm_port: &mut u16,
    db_url: &mut String,
    web_port: &mut u16,
) {
    use argparse::{ArgumentParser, Store};

    let mut _comm_port = comm_port.to_string();
    let mut _web_port = web_port.to_string();

    // Extra limited scope where argparse operates.
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Client for chat communication service.");

        ap.refer(comm_hostname)
            .add_option(
                &["-h", "--host"],
                Store,
                "Communication server hostname (e.g. `localhost`).",
            );

        ap.refer(&mut _comm_port)
            .add_option(
                &["-p", "--port"],
                Store,
                "Communication server port number (e.g. `11111`).",
            );

        ap.refer(&mut _web_port)
            .add_option(
                &["--web-port"],
                Store,
                "Web server port number (e.g. 8080).",
            );

        ap.refer(db_url)
            .add_option(&["--db-url"], Store, "DB URL (e.g. `sqlite:data.db`).");

        if let Err(error_code) = ap.parse_args() {
            exit(error_code);
        }
    }

    _ensure_port_number(comm_port, &_comm_port, "comm_port");
    _ensure_port_number(web_port, &_web_port, "web_port");
}


fn _ensure_port_number(target: &mut u16, source: &str, arg_name: &str) {
    match source.parse::<u16>() {
        Ok(port_number) => *target = port_number,
        Err(_) => {
            eprintln!("failed to parse port number {}", arg_name);
            exit(1);
        }
    }
}
