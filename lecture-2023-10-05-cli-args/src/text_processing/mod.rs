use slug;
use std::error::Error;

use crate::extra;
use crate::extra::csv::_csv;


type Result = std::result::Result<String, Box<dyn Error>>;

/// `get_stdin_line` read single line from stdin.
fn get_stdin_line() -> std::result::Result<String, Box<dyn Error>> {
    let mut text: String = String::new();
    std::io::stdin().read_line(&mut text)?;

    Ok(text.trim().to_string())
}

/// `check_empty_options` predicate check that caller function takes an empty options vector.
fn check_empty_options(options: Vec<&str>) -> std::result::Result<(), Box<dyn Error>> {
    if !options.is_empty() {
        return Err("")?;
    }

    Ok(())
}

pub fn upper_case(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    Ok(text.to_uppercase())
}

pub fn lower_case(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    Ok(text.to_lowercase())
}

pub fn mixed_case(first_lower_case: bool, options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    Ok(extra::mixed_case(&text, first_lower_case))
}

pub fn no_spaces(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    // Ok(text.replace(" ", "");
    Ok(extra::no_spaces(&text))
}

pub fn slugify(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    Ok(slug::slugify(text))
}

pub fn palindromize(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    Ok(extra::palindromize(&text))
}

pub fn swap_pairs(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    Ok(extra::swap_pairs(&text))
}

/// `caesar` convert the line read from stdin using Caesar cipher with optional shift size.
/// If size is not specifies, default size is 3.
/// One can take implementation of this method as example of working with optional cli options.
pub fn caesar(options: Vec<&str>) -> Result {
    let shift = if options.is_empty() {
        3   // the default shift size
    } else {
        if options.len() == 1 {
            options[0].parse::<u32>()?
        } else {
            Err(format!("Invalid number of options {} (expected 0 or 1)!", options.len()))?
        }
    };

    let text = get_stdin_line()?;
    Ok(extra::caesar(&text, shift))
}

pub fn csv(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    _csv()
}
