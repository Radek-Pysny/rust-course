// Produce mixed case string variant of the given input string.
pub fn mixed_case(text: &str, first_lowercase: bool) -> String {
    let upcase_class = if first_lowercase { 1 } else { 0 };
    text.chars().enumerate().map(
        |(index, char)| if index % 2 == upcase_class {
            char.to_uppercase().collect::<String>()
        } else {
            char.to_lowercase().collect::<String>()
        }
    ).collect::<String>()
}

// Produce string with swapped characters per each pair.
pub fn swap_pairs(text: &str) -> String {
    text.chars().collect::<Vec<_>>().chunks(2).map(
        |cs| if cs.len() == 2 {
            [cs[1], cs[0]].into_iter().collect::<String>()
        } else {
            cs[0].to_string()
        }
    ).collect::<String>()
}
