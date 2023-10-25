use std::error::Error;

/// `get_stdin_line` read single line from stdin.
fn get_stdin_line() -> Result {
    let mut text: String = String::new();
    std::io::stdin().read_line(&mut text)?;

    Ok(text.trim().to_string())
}

/// `check_empty_options` predicate check that caller function takes an empty options vector.
fn check_empty_options(options: Vec<&str>) -> std::result::Result<(), Box<dyn Error>> {
    if !options.is_empty() {
        return Err("The given action did not expect any option.")?;
    }

    Ok(())
}


type Result = std::result::Result<String, Box<dyn Error>>;


pub fn upper_case(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    super::upper_case(&text)
}

pub fn lower_case(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    super::lower_case(&text)
}

pub fn mixed_case(first_lower_case: bool, options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    super::mixed_case(&text, first_lower_case)
}

pub fn no_spaces(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    super::no_spaces(&text)
}

pub fn slugify(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    super::slugify(&text)
}

pub fn palindromize(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    super::palindromize(&text)
}

pub fn swap_pairs(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    let text = get_stdin_line()?;
    super::swap_pairs(&text)
}

pub const DEFAULT_CAESAR_SHIFT: u32 = 3;

/// `caesar` convert the line read from stdin using Caesar cipher with optional shift size.
/// If size is not specifies, default size is 3.
/// One can take implementation of this method as example of working with optional cli options.
pub fn caesar(options: Vec<&str>) -> Result {
    let shift = if options.is_empty() {
        DEFAULT_CAESAR_SHIFT
    } else {
        if options.len() == 1 {
            options[0].parse::<u32>()?
        } else {
            Err(format!("Invalid number of options {} (expected 0 or 1)!", options.len()))?
        }
    };

    let text = get_stdin_line()?;
    super::caesar(&text, shift)
}

pub fn csv(options: Vec<&str>) -> Result {
    check_empty_options(options)?;
    super::_csv()
}
