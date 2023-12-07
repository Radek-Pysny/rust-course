use std::time::{SystemTime};

use chrono::offset::Local;
use chrono::DateTime;


/// `timestamp_to_string` transforms system time (e.g. from `SystemTime::now()`) into string
/// of form ISO 8601 (`YYYY-MM-DDThh:mm:ss`).
pub fn timestamp_to_string(timestamp: SystemTime) -> String {
    let timestamp: DateTime<Local> = timestamp.into();
    let timestamp = timestamp.format("%Y-%m-%dT%H:%M:%S");
    format!("{}", timestamp)
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;

    use chrono::offset::Local;
    use chrono::{Datelike, DateTime, Timelike};
    use regex::Regex;

    use super::timestamp_to_string;

    #[test]
    fn test_timestamp_to_string__really_now() {
        let now= SystemTime::now();
        let timestamp = timestamp_to_string(now);

        let re: Regex = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})$").unwrap();

        assert!(re.is_match(&timestamp));

        let caps =  re.captures(&timestamp);
        assert!(caps.as_ref().is_some());

        let caps = caps.unwrap();
        assert_eq!(caps.len(), 1 + 6);

        let datetime: DateTime<Local> = now.into();
        assert_eq!(datetime.year(), caps[1].parse::<i32>().unwrap());
        assert_eq!(datetime.month(), caps[2].parse::<u32>().unwrap());
        assert_eq!(datetime.day(), caps[3].parse::<u32>().unwrap());
        assert_eq!(datetime.hour(), caps[4].parse::<u32>().unwrap());
        assert_eq!(datetime.minute(), caps[5].parse::<u32>().unwrap());
        assert_eq!(datetime.second(), caps[6].parse::<u32>().unwrap())
    }
}
