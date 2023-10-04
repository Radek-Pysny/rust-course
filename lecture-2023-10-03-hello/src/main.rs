use std::io;

extern crate lecture_2023_10_03_hello;
use lecture_2023_10_03_hello::{mixed_case, swap_pairs};


fn main() {
    println!("Enter you name, please:");

    let mut name: String = String::new();

    match io::stdin().read_line(&mut name) {
        Ok(x) => println!("read {} characters\n", x),
        Err(err) => {
            println!("failed to read line: {}", err);
            return;
        },
    }

    let name = name.trim();

    let mutators = [
        |name| mixed_case(name, true),
        |name| mixed_case(name, false),
        swap_pairs as fn(&str) -> String,
    ];

    mutators.iter().for_each(|f|println!("Hello, {}", f(name)));
}
