//! Checks for a time implementation.

use crate::{Error, Time};

/// Wrapper trait for validating a time implementation.
pub(crate) trait CheckedTime {
    /// No checks.
    fn year(&self) -> i32;
    /// Checks if the month is in `1..=12`.
    fn month(&self) -> Result<u8, Error>;
    /// Checks if the day of the month is in `1..=31`.
    fn day(&self) -> Result<u8, Error>;
    /// Checks if the hour of the day is in `0..=23`.
    fn hour(&self) -> Result<u8, Error>;
    /// Checks if the minute of the hour is in `0..=59`.
    fn minute(&self) -> Result<u8, Error>;
    /// Checks if the second of the minute is in `0..=60`.
    fn second(&self) -> Result<u8, Error>;
    /// Checks if the number of nanoseconds is in `0..=999_999_999`.
    fn nanoseconds(&self) -> Result<u32, Error>;
    /// Checks if the day of the week is in `0..=6`.
    fn day_of_week(&self) -> Result<u8, Error>;
    /// Checks if the day of the year is in `1..=366`.
    fn day_of_year(&self) -> Result<u16, Error>;
    /// No checks.
    fn to_int(&self) -> i64;
    /// No checks.
    fn is_utc(&self) -> bool;
    /// No checks.
    fn utc_offset(&self) -> i32;
    /// Checks if the name of the time zone is valid ASCII.
    fn time_zone(&self) -> Result<&str, Error>;
}

impl<T: Time> CheckedTime for T {
    fn year(&self) -> i32 {
        self.year()
    }

    fn month(&self) -> Result<u8, Error> {
        match self.month() {
            month @ 1..=12 => Ok(month),
            _ => Err(Error::InvalidTime),
        }
    }

    fn day(&self) -> Result<u8, Error> {
        match self.day() {
            day @ 1..=31 => Ok(day),
            _ => Err(Error::InvalidTime),
        }
    }

    fn hour(&self) -> Result<u8, Error> {
        match self.hour() {
            hour @ 0..=23 => Ok(hour),
            _ => Err(Error::InvalidTime),
        }
    }

    fn minute(&self) -> Result<u8, Error> {
        match self.minute() {
            minute @ 0..=59 => Ok(minute),
            _ => Err(Error::InvalidTime),
        }
    }

    fn second(&self) -> Result<u8, Error> {
        match self.second() {
            second @ 0..=60 => Ok(second),
            _ => Err(Error::InvalidTime),
        }
    }

    fn nanoseconds(&self) -> Result<u32, Error> {
        match self.nanoseconds() {
            nanoseconds @ 0..=999_999_999 => Ok(nanoseconds),
            _ => Err(Error::InvalidTime),
        }
    }

    fn day_of_week(&self) -> Result<u8, Error> {
        match self.day_of_week() {
            day_of_week @ 0..=6 => Ok(day_of_week),
            _ => Err(Error::InvalidTime),
        }
    }

    fn day_of_year(&self) -> Result<u16, Error> {
        match self.day_of_year() {
            day_of_year @ 1..=366 => Ok(day_of_year),
            _ => Err(Error::InvalidTime),
        }
    }

    fn to_int(&self) -> i64 {
        self.to_int()
    }

    fn is_utc(&self) -> bool {
        self.is_utc()
    }

    fn utc_offset(&self) -> i32 {
        self.utc_offset()
    }

    fn time_zone(&self) -> Result<&str, Error> {
        match self.time_zone() {
            time_zone if time_zone.is_ascii() => Ok(time_zone),
            _ => Err(Error::InvalidTime),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    include!("../mock.rs.in");

    fn check<T>(ok: bool, result: &Result<T, Error>) {
        if ok {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(Error::InvalidTime)));
        }
    }

    #[test]
    fn test_checked_time() {
        #[rustfmt::skip]
        let times = [
            MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, ""),
            MockTime::new(1970, 0, 0, 99, 99, 99, 1_000_000_000, 9, 999, 0, false, 0, "€"),
        ];

        check(true, &CheckedTime::month(&times[0]));
        check(true, &CheckedTime::day(&times[0]));
        check(true, &CheckedTime::hour(&times[0]));
        check(true, &CheckedTime::minute(&times[0]));
        check(true, &CheckedTime::second(&times[0]));
        check(true, &CheckedTime::nanoseconds(&times[0]));
        check(true, &CheckedTime::day_of_week(&times[0]));
        check(true, &CheckedTime::day_of_year(&times[0]));
        check(true, &CheckedTime::time_zone(&times[0]));

        check(false, &CheckedTime::month(&times[1]));
        check(false, &CheckedTime::day(&times[1]));
        check(false, &CheckedTime::hour(&times[1]));
        check(false, &CheckedTime::minute(&times[1]));
        check(false, &CheckedTime::second(&times[1]));
        check(false, &CheckedTime::nanoseconds(&times[1]));
        check(false, &CheckedTime::day_of_week(&times[1]));
        check(false, &CheckedTime::day_of_year(&times[1]));
        check(false, &CheckedTime::time_zone(&times[1]));
    }
}
