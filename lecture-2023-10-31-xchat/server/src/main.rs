use server::start_server;


fn main() {
    let mut hostname = "localhost".to_string();
    let mut port = 11111_u16;

    parse_arguments(&mut hostname, &mut port);

    let address = format!("{}:{}", hostname, port);

    if let Err(err) = start_server(&address) {
        eprintln!("{}", err);
    }
}


fn parse_arguments(hostname: &mut String, port: &mut u16) {
    use argparse::{ArgumentParser, Store};
    use std::process::exit;

    let mut _port = port.to_string();

    // Extra limited scope where argparse operates.
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Client for chat communication service.");

        ap.refer(hostname)
            .add_option(&["-h", "--host"], Store, "Hostname (e.g. localhost).");

        ap.refer(&mut _port)
            .add_option(&["-p", "--port"], Store, "Port number (e.g. 11111).");

        if let Err(error_code) = ap.parse_args() {
            exit(error_code);
        }
    }

    // Ensure that read parse number is valid unsigned 16b integer.
    match _port.parse::<u16>() {
        Ok(port_number) => *port = port_number,
        Err(_) => {
            eprintln!("failed to parse port number");
            exit(1);
        }
    }
}
