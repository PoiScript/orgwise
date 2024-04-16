use chrono::{Datelike, NaiveDateTime, Timelike};
use std::fmt;

pub struct FormatActiveTimestamp(pub NaiveDateTime);

impl fmt::Display for FormatActiveTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<{:0>4}-{:0>2}-{:0>2} {} {:0>2}:{:0>2}>",
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.weekday(),
            self.0.hour(),
            self.0.minute()
        )
    }
}

pub struct FormatInactiveTimestamp(pub NaiveDateTime);

impl fmt::Display for FormatInactiveTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:0>4}-{:0>2}-{:0>2} {} {:0>2}:{:0>2}]",
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.weekday(),
            self.0.hour(),
            self.0.minute()
        )
    }
}
