mod extra;

use std::collections::HashMap;
use std::env;
use slug::slugify;
use extra::{palindromize, mixed_case, swap_pairs, no_spaces, caesar};
use lazy_static::lazy_static;


type ResErr<T> = Result<T, String>;

/// run is an entry-point for this library.
/// It return either modified string or empty error string (requesting for help print int stdout)
/// or non-empty error string (requesting error report to the user).
pub fn run() -> ResErr<String> {
    let args = env::args().skip(1).collect::<Vec<String>>();
    let action = parse_args(args)?;
    let mut text = String::new();
    std::io::stdin().read_line(&mut text).map_err(|err|err.to_string())?;
    let result = modify_text(&action, &text.trim())?;
    Ok(result)
}

/// help print help text on stdout.
pub fn help() {
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

/// parse_args process the given command line arguments (in a form of vector of strings) and return
/// either the requested action or an error message to be present to the user.
fn parse_args(args: Vec<String>) -> ResErr<&'static Action> {
    if args.is_empty() {
        return Err("".to_string());     // help will be present on empty error message
    }

    if args.len() > 1 {
        return Err("Found too many command line options. Just one was expected.".to_string());
    }

    let arg = args[0].trim().to_lowercase();
    for (k, v) in (&CMD_MAP).iter() {
        if &arg == k {
            return Ok(v)
        }
    }
    Err(format!(r#"Found invalid action request: "{}"!"#, arg))
}

/// modify_text try to modify text by the function mapped to the given action.
fn modify_text(action: &Action, text: &str) -> ResErr<String> {
    match action {
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
