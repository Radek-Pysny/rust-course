use std::time::{SystemTime};

use chrono::offset::Local;
use chrono::DateTime;


/// `timestamp_to_string` transforms system time (e.g. from `SystemTime::now()`) into string
/// of form ISO 8601 (YYYY-MM-DDThh:mm:ss).
pub fn timestamp_to_string(timestamp: SystemTime) -> String {
    let timestamp: DateTime<Local> = timestamp.into();
    let timestamp = timestamp.format("%Y-%m-%dT%H:%M:%S");
    format!("{}", timestamp)
}