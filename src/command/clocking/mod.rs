mod start;
mod status;
mod stop;

pub use start::*;
pub use status::*;
pub use stop::*;

use chrono::{Datelike, NaiveDateTime, Timelike};
use std::fmt;

pub struct FormatNativeDateTime(NaiveDateTime);

impl fmt::Display for FormatNativeDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:0>4}-{:0>2}-{:0>2} {} {:0>2}:{:0>2}]",
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.weekday(),
            self.0.hour(),
            self.0.minute(),
        )
    }
}
