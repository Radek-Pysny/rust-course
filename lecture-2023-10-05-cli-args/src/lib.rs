mod extra;

use std::error::Error;
use std::collections::HashMap;
use std::env;
use slug::slugify;
use extra::{palindromize, mixed_case, swap_pairs, no_spaces, caesar, csv};
use lazy_static::lazy_static;


type Result = std::result::Result<String, Box<dyn Error>>;


/// run is an entry-point for this program.
/// It return either modified string or empty error string (requesting for help print int stdout)
/// or non-empty error string (requesting error report to the user).
pub fn run(command: &str, options: Vec<&str>) -> Result {
    let action = parse_action(command)?;
    act(action, options)
}

/// help print help text on stdout.
pub fn print_help() {
    let bin_name = &env::args().take(1).collect::<Vec<String>>()[0];
    let action_list = (&CMD_MAP).iter().map(|(k, _)| k.to_string()).collect::<Vec<_>>();

    println!("Usage:");
    println!("   {} <action>", bin_name);
    println!("        where action is one of: {}.", action_list.join(", "));
}

/// Action represent all the available actions over the given input string.
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
    ]);
}

/// parse_action process the given command and return either the requested action or an error
/// message to be present to the user.
fn parse_action(command: &str) -> std::result::Result<&'static Action, String> {
    let key = command.trim().to_lowercase();
    let key = key.as_str();
    match CMD_MAP.get(key) {
        Some(action) => Ok(action),
        None => Err(format!(r#"Invalid action request: "{}"!"#, key)),
    }
}

/// act process the given action.
fn act(action: &Action, options: Vec<&str>) -> Result {
    if let Action::Csv = action {
        return csv();
    }

    if !options.is_empty() {
        return Err("The given action does not support any options!")?;
    }

    let mut text: String = String::new();
    std::io::stdin().read_line(&mut text)?;
    let text = text.trim();

    match action {
        Action::Csv => Err("Action csv shall already be processed!").unwrap(),
        Action::UpperCase => Ok(text.to_uppercase()),
        Action::LowerCase => Ok(text.to_lowercase()),
        // Action::NoSpaces => Ok(text.replace(" ", "")),
        Action::NoSpaces => Ok(no_spaces(text)),
        Action::Slugify => Ok(slugify(text)),
        Action::Palindromize => Ok(palindromize(text)),
        Action::MixedCase {first_lowercase} => Ok(mixed_case(text, *first_lowercase)),
        Action::SwapPairs => Ok(swap_pairs(text)),
        Action::Caesar => Ok(caesar(text))
    }
}
