#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::cast_possible_truncation)]
#![allow(unknown_lints)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! WIP

#![doc(html_root_url = "https://docs.rs/strftime-ruby/0.1.0")]

// Ensure code blocks in `README.md` compile
#[cfg(all(doctest, feature = "std"))]
#[doc = include_str!("../README.md")]
mod readme {}

mod assert;
mod format;
mod utils;
mod week;
mod write;

use format::TimeFormatter;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    InvalidFormat,
    WriteZero,
    FmtError,
}

pub trait Time {
    /// Returns the year for _time_ (including the century).
    fn year(&self) -> i32;
    /// Returns the month of the year in `1..=12` for _time_.
    fn month(&self) -> u8;
    /// Returns the day of the month in `1..=31` for _time_.
    fn day(&self) -> u8;
    /// Returns the hour of the day in `0..=23` for _time_.
    fn hour(&self) -> u8;
    /// Returns the minute of the hour in `0..=59` for _time_.
    fn minute(&self) -> u8;
    /// Returns the second of the minute in `0..=60` for _time_.
    fn second(&self) -> u8;
    /// Returns the number of nanoseconds in `0..=999_999_999` for _time_.
    fn nanoseconds(&self) -> u32;
    /// Returns an integer representing the day of the week in `0..=6`, with `Sunday == 0`.
    fn day_of_week(&self) -> u8;
    /// Returns an integer representing the day of the year in `1..=366`.
    fn day_of_year(&self) -> u16;
    /// Returns the number of seconds as a signed integer since the Epoch.
    fn to_int(&self) -> i64;
    /// Returns true if the time zone is UTC.
    fn is_utc(&self) -> bool;
    /// Returns the offset in seconds between the timezone of _time_ and UTC.
    fn utc_offset(&self) -> i32;
    /// Returns the name of the time zone as a string.
    fn time_zone(&self) -> &str;
}

pub mod buffered {
    use super::{Error, Time, TimeFormatter};

    pub fn strftime(time: &impl Time, format: &[u8], mut buf: &mut [u8]) -> Result<(), Error> {
        TimeFormatter::new(time, format).fmt(&mut buf)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod bytes {
    use super::{Error, Time, TimeFormatter};

    pub fn strftime(time: &impl Time, format: &[u8]) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        TimeFormatter::new(time, format).fmt(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod string {
    use super::{Error, Time, TimeFormatter};

    pub fn strftime(time: &impl Time, format: &str) -> Result<String, Error> {
        let mut buf = Vec::new();
        TimeFormatter::new(time, format).fmt(&mut buf)?;
        Ok(String::from_utf8(buf).expect("resulting string should be valid UTF-8"))
    }
}
