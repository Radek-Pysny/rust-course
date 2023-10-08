pub mod palindrome;

pub use palindrome::palindromize;

/// mixed_case produce a string with changing lowercase and uppercase letters per odd/even character
/// position. One can select whether the first character should be lowercase or uppercase.
pub fn mixed_case(text: &str, first_lowercase: bool) -> String {
    let uppercase_class = if first_lowercase { 1 } else { 0 };
    text.chars().enumerate().map(
        |(index, char)| if index % 2 == uppercase_class {
            char.to_uppercase().collect::<String>()
        } else {
            char.to_lowercase().collect::<String>()
        }
    ).collect::<String>()
}

/// swap_pairs produce string with swapped characters per each pair.
pub fn swap_pairs(text: &str) -> String {
    text.chars().collect::<Vec<_>>().chunks(2).map(
        |cs| if cs.len() == 2 {
            [cs[1], cs[0]].into_iter().collect::<String>()
        } else {
            cs[0].to_string()
        }
    ).collect::<String>()
}

/// no_space removes all basic whitespace characters.
pub fn no_spaces(text: &str) -> String {
    text.chars().filter(|char| !char.is_whitespace()).collect::<String>()
}

/// caesar is implementing Caesar's cipher.
pub fn caesar(text: &str) -> String {
    const SHIFT: u32 = 3;

    text.chars().map(|char| {
        if char.is_ascii_alphabetic() {
            let ord_a;
            let ord_z;
            if char.is_ascii_lowercase() {
                ord_a = 'a' as u32;
                ord_z = 'z' as u32;
            } else {
                ord_a = 'A' as u32;
                ord_z = 'Z' as u32;
            }
            let ord = (char as u32 - ord_a + SHIFT) % (ord_z - ord_a + 1) + ord_a;
            if let Some(result) = char::from_u32(ord) {
                result
            } else {
                char
            }
        } else {
            char
        }}).collect::<String>()
}
