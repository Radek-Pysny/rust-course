use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;


/// `Action` represent all the available actions over the given input string.
#[derive(PartialEq, Eq, Hash, EnumIter)]
pub enum Action {
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


impl fmt::Display for Action {
    /// `fmt` enable conversion from `Action` to `&str` via `to_string` method.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let key = match self {
            Action::LowerCase => "lowercase",
            Action::UpperCase => "uppercase",
            Action::NoSpaces => "no-spaces",
            Action::Slugify => "slugify",
            Action::Palindromize => "palindromize",
            Action::MixedCase {first_lowercase: true} => "lo-mixedcase",
            Action::MixedCase {first_lowercase: false} => "hi-mixedcase",
            Action::SwapPairs => "swap-pairs",
            Action::Caesar => "caesar",
            Action::Csv => "csv"
        };

        write!(f, "{}", key)
    }
}


impl FromStr for Action {
    type Err = String;

    /// `from_str` implement conversion from `&str` to `Action` that might fail on invalid text.
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let key = s.trim().to_lowercase();
        let key = key.as_str();

        // chain hack - due to not having there parameterized enum variant with non-default value
        let action_list = Action::iter().chain([Action::MixedCase {first_lowercase: true}]);

        for action in action_list {
            if key == action.to_string().as_str() {
                return Ok(action)
            }
        }

        Err(format!("invalid action: {}", s))
    }
}


impl Action {
    /// `single_shot_act` start to process input from stdin with the given options.
    pub fn single_shot_act(&self, options: Vec<&str>) -> Result<String, Box<dyn Error>> {
        use crate::text_processing::single_shot::*;

        match self {
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

    /// `act` process single line of input from stdin without possibility to pass any options.
    pub fn act(&self, text: &str) -> Result<String, Box<dyn Error>> {
        use crate::text_processing::*;

        match self {
            Action::UpperCase => upper_case(text),
            Action::LowerCase => lower_case(text),
            Action::NoSpaces => no_spaces(text),
            Action::Slugify => slugify(text),
            Action::Palindromize => palindromize(text),
            Action::MixedCase {first_lowercase} => mixed_case( text, *first_lowercase),
            Action::SwapPairs => swap_pairs(text),
            Action::Caesar => caesar(text, DEFAULT_CAESAR_SHIFT),
            Action::Csv => todo!("Not yet implemented"),
        }
    }
}
