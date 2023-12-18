mod message;
mod panic;
mod timestamp;

pub use message::Message;
pub use panic::panic_to_text;
pub use timestamp::timestamp_to_string;


pub fn concat(strings: &Vec<String>) -> String {
    use std::borrow::{Borrow};

    if strings.is_empty() {
        return String::new();
    }

    // `len` calculation may overflow but push_str will check boundaries
    let len = strings.iter().map(|s| <String as Borrow<String>>::borrow(s).len()).sum();
    let mut result = String::with_capacity(len);

    for s in strings {
        result.push_str(s.borrow())
    }

    result
}
