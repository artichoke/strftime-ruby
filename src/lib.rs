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

#[cfg(test)]
mod tests;

use core::fmt;

/// Error type returned by the `strftime` functions.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    /// Provided format string is ended by an unterminated format specifier.
    InvalidFormatString,
    /// Formatted string is too large and could cause an out-of-memory error.
    FormattedStringTooLarge,
    /// Provided buffer for the [`buffered::strftime`] function is too small for the formatted string.
    ///
    /// This corresponds to the [`std::io::ErrorKind::WriteZero`] variant.
    WriteZero,
    /// Formatting error, corresponding to [`core::fmt::Error`].
    FmtError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidFormatString => write!(f, "invalid format string"),
            Error::FormattedStringTooLarge => write!(f, "formatted string too large"),
            Error::WriteZero => write!(f, "failed to write the whole buffer"),
            Error::FmtError => write!(f, "formatter error"),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Error {}

/// Common methods needed for formatting _time_.
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

/// Provides a buffered `strftime` implementation using a format string with arbitrary bytes.
pub mod buffered {
    use super::{Error, Time};
    use crate::format::TimeFormatter;

    /// Format a _time_ implementation with the specified format byte string,
    /// writing in the provided buffer and returning the written subslice.
    ///
    /// See the [crate-level documentation](crate) for a complete description of possible format specifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use strftime::buffered::strftime;
    /// use strftime::Time;
    ///
    /// // Not shown: create a time implementation with the year 1970
    /// // let time = ...;
    /// # include!("mock.rs.in");
    /// # fn main() -> Result<(), strftime::Error> {
    /// # let time = MockTime { year: 1970, ..Default::default() };
    /// assert_eq!(time.year(), 1970);
    ///
    /// let mut buf = [0u8; 8];
    /// assert_eq!(strftime(&time, b"%Y", &mut buf)?, b"1970");
    /// assert_eq!(buf, *b"1970\0\0\0\0");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Can produce an [`Error`](crate::Error) when the formatting fails.
    pub fn strftime<'a>(
        time: &impl Time,
        format: &[u8],
        buf: &'a mut [u8],
    ) -> Result<&'a mut [u8], Error> {
        let len = buf.len();

        let mut cursor = &mut buf[..];
        TimeFormatter::new(time, format).fmt(&mut cursor)?;
        let remaining_len = cursor.len();

        Ok(&mut buf[..len - remaining_len])
    }
}

/// Provides a `strftime` implementation using a format string with arbitrary bytes.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod bytes {
    use super::{Error, Time};
    use crate::format::TimeFormatter;

    /// Format a _time_ implementation with the specified format byte string.
    ///
    /// See the [crate-level documentation](crate) for a complete description of possible format specifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use strftime::bytes::strftime;
    /// use strftime::Time;
    ///
    /// // Not shown: create a time implementation with the year 1970
    /// // let time = ...;
    /// # include!("mock.rs.in");
    /// # fn main() -> Result<(), strftime::Error> {
    /// # let time = MockTime { year: 1970, ..Default::default() };
    /// assert_eq!(time.year(), 1970);
    ///
    /// assert_eq!(strftime(&time, b"%Y")?, b"1970");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Can produce an [`Error`](crate::Error) when the formatting fails.
    pub fn strftime(time: &impl Time, format: &[u8]) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        TimeFormatter::new(time, format).fmt(&mut buf)?;
        Ok(buf)
    }
}

/// Provides a `strftime` implementation using a UTF-8 format string.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub mod string {
    use super::{Error, Time};
    use crate::format::TimeFormatter;

    /// Format a _time_ implementation with the specified UTF-8 format string.
    ///
    /// See the [crate-level documentation](crate) for a complete description of possible format specifiers.
    ///
    /// # Examples
    ///
    /// ```
    /// use strftime::string::strftime;
    /// use strftime::Time;
    ///
    /// // Not shown: create a time implementation with the year 1970
    /// // let time = ...;
    /// # include!("mock.rs.in");
    /// # fn main() -> Result<(), strftime::Error> {
    /// # let time = MockTime { year: 1970, ..Default::default() };
    /// assert_eq!(time.year(), 1970);
    ///
    /// assert_eq!(strftime(&time, "%Y")?, "1970");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Can produce an [`Error`](crate::Error) when the formatting fails.
    pub fn strftime(time: &impl Time, format: &str) -> Result<String, Error> {
        let mut buf = Vec::new();
        TimeFormatter::new(time, format).fmt(&mut buf)?;
        Ok(String::from_utf8(buf).expect("formatted string should be valid UTF-8"))
    }
}
