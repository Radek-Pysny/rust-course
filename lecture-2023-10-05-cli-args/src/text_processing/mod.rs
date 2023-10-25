pub mod single_shot;

use slug;
use std::error::Error;

use crate::extra;
use crate::extra::csv::_csv;

pub use single_shot::DEFAULT_CAESAR_SHIFT;


pub fn upper_case(text: &str) -> std::result::Result<String, Box<dyn Error>> {
    Ok(text.to_uppercase())
}

pub fn lower_case(text: &str) -> std::result::Result<String, Box<dyn Error>> {
    Ok(text.to_lowercase())
}

pub fn mixed_case(text: &str, first_lower_case: bool) -> std::result::Result<String, Box<dyn Error>> {
    Ok(extra::mixed_case(&text, first_lower_case))
}

pub fn no_spaces(text: &str) -> std::result::Result<String, Box<dyn Error>> {
    // Ok(text.replace(" ", "");
    Ok(extra::no_spaces(&text))
}

pub fn slugify(text: &str) -> std::result::Result<String, Box<dyn Error>> {
    Ok(slug::slugify(text))
}

pub fn palindromize(text: &str) -> std::result::Result<String, Box<dyn Error>> {
    Ok(extra::palindromize(&text))
}

pub fn swap_pairs(text: &str) -> std::result::Result<String, Box<dyn Error>> {
    Ok(extra::swap_pairs(&text))
}

pub fn caesar(text: &str, shift: u32) -> std::result::Result<String, Box<dyn Error>> {
    Ok(extra::caesar(&text, shift))
}

pub fn csv() -> std::result::Result<String, Box<dyn Error>> {
    _csv()
}
