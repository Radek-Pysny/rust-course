mod extra;
mod text_processing;

use std::error::Error;
use std::collections::HashMap;
use std::env;
use lazy_static::lazy_static;


type Result = std::result::Result<String, Box<dyn Error>>;


/// `run` is an entry-point for this program.
/// It return either modified string or empty error string (requesting for help print int stdout)
/// or non-empty error string (requesting error report to the user).
pub fn run(command: &str, options: Vec<&str>) -> Result {
    let action = parse_action(command)?;
    act(action, options)
}

/// `print_help` print help text on stdout.
pub fn print_help() {
    let bin_name = &env::args().take(1).collect::<Vec<String>>()[0];
    let action_list = (&CMD_MAP).iter().map(|(k, _)| k.to_string()).collect::<Vec<_>>();

    println!("Usage:");
    println!("   {} <action>", bin_name);
    println!("        where action is one of: {}.", action_list.join(", "));
}

/// `Action` represent all the available actions over the given input string.
#[derive(Debug, PartialEq, Eq, Hash)]
enum Action {
    LowerCase,
    UpperCase,
    NoSpaces,
    Slugify,
    Palindromize,
    MixedCase{first_lowercase: bool},
    SwapPairs,
    Caesar,
    Csv,
}

lazy_static! {
    static ref CMD_MAP: HashMap<&'static str, Action> = HashMap::from([
        ("lowercase", Action::LowerCase),
        ("uppercase", Action::UpperCase),
        ("no-spaces", Action::NoSpaces),
        ("slugify", Action::Slugify),
        ("palindromize", Action::Palindromize),
        ("lo-mixedcase", Action::MixedCase{first_lowercase: true}),
        ("hi-mixedcase", Action::MixedCase{first_lowercase: false}),
        ("swap-pairs", Action::SwapPairs),
        ("caesar", Action::Caesar),
        ("csv", Action::Csv),
    ]);
}

/// `parse_action` process the given command and return either the requested action or an error
/// message to be present to the user.
fn parse_action(command: &str) -> std::result::Result<&'static Action, String> {
    let key = command.trim().to_lowercase();
    let key = key.as_str();
    match CMD_MAP.get(key) {
        Some(action) => Ok(action),
        None => Err(format!(r#"Invalid action request: "{}"!"#, key)),
    }
}

/// `act` implement mapping from action to the result of applied text-processing function.
fn act(action: &Action, options: Vec<&str>) -> Result {
    use text_processing::*;

    match action {
        Action::UpperCase => upper_case(options),
        Action::LowerCase => lower_case(options),
        Action::NoSpaces => no_spaces(options),
        Action::Slugify => slugify(options),
        Action::Palindromize => palindromize(options),
        Action::MixedCase {first_lowercase} => mixed_case( *first_lowercase, options),
        Action::SwapPairs => swap_pairs(options),
        Action::Caesar => caesar(options),
        Action::Csv => csv(options),
    }
}
