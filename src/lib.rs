#![forbid(unsafe_code)]
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

//! This crate provides a Ruby 3.1.2 compatible `strftime` function, which
//! formats time according to the directives in the given format string.
//!
//! The directives begin with a percent `%` character. Any text not listed as a
//! directive will be passed through to the output string.
//!
//! Each directive consists of a percent `%` character, zero or more flags,
//! optional minimum field width, optional modifier and a conversion specifier
//! as follows:
//!
//! ```text
//! %<flags><width><modifier><conversion>
//! ```
//!
//! ## Flags
//!
//! | Flag | Description                                                                            |
//! |------|----------------------------------------------------------------------------------------|
//! |  `-` | Use left padding, ignoring width and removing all other padding options in most cases. |
//! |  `_` | Use spaces for padding.                                                                |
//! |  `0` | Use zeros for padding.                                                                 |
//! |  `^` | Convert the resulting string to uppercase.                                             |
//! |  `#` | Change case of the resulting string.                                                   |
//!
//!
//! ## Width
//!
//! The minimum field width specifies the minimum width.
//!
//! ## Modifiers
//!
//! The modifiers are `E` and `O`. They are ignored.
//!
//! ## Specifiers
//!
//! | Specifier  | Example       | Description                                                                                                           |
//! |------------|---------------|-----------------------------------------------------------------------------------------------------------------------|
//! |    `%Y`    | `-2001`       | Year with century if provided, zero-padded to at least 4 digits plus the possible negative sign.                      |
//! |    `%C`    | `-21`         | `Year / 100` using Euclidean division, zero-padded to at least 2 digits.                                              |
//! |    `%y`    | `99`          | `Year % 100` in `00..=99`, using Euclidean remainder, zero-padded to 2 digits.                                        |
//! |    `%m`    | `01`          | Month of the year in `01..=12`, zero-padded to 2 digits.                                                              |
//! |    `%B`    | `July`        | Locale independent full month name.                                                                                   |
//! | `%b`, `%h` | `Jul`         | Locale independent abbreviated month name, using the first 3 letters.                                                 |
//! |    `%d`    | `01`          | Day of the month in `01..=31`, zero-padded to 2 digits.                                                               |
//! |    `%e`    | ` 1`          | Day of the month in ` 1..=31`, blank-padded to 2 digits.                                                              |
//! |    `%j`    | `001`         | Day of the year in `001..=366`, zero-padded to 3 digits.                                                              |
//! |    `%H`    | `00`          | Hour of the day (24-hour clock) in `00..=23`, zero-padded to 2 digits.                                                |
//! |    `%k`    | ` 0`          | Hour of the day (24-hour clock) in ` 0..=23`, blank-padded to 2 digits.                                               |
//! |    `%I`    | `01`          | Hour of the day (12-hour clock) in `01..=12`, zero-padded to 2 digits.                                                |
//! |    `%l`    | ` 1`          | Hour of the day (12-hour clock) in ` 1..=12`, blank-padded to 2 digits.                                               |
//! |    `%P`    | `am`          | Lowercase meridian indicator (`"am"` or `"pm"`).                                                                      |
//! |    `%p`    | `AM`          | Uppercase meridian indicator (`"AM"` or `"PM"`).                                                                      |
//! |    `%M`    | `00`          | Minute of the hour in `00..=59`, zero-padded to 2 digits.                                                             |
//! |    `%S`    | `00`          | Second of the minute in `00..=60`, zero-padded to 2 digits.                                                           |
//! |    `%L`    | `123`         | Truncated fractional seconds digits, with 3 digits by default. Number of digits is specified by the width field.      |
//! |    `%N`    | `123456789`   | Truncated fractional seconds digits, with 9 digits by default. Number of digits is specified by the width field.      |
//! |    `%z`    | `+0200`       | Zero-padded signed time zone UTC hour and minute offsets (`+hhmm`).                                                   |
//! |    `%:z`   | `+02:00`      | Zero-padded signed time zone UTC hour and minute offsets with colons (`+hh:mm`).                                      |
//! |    `%::z`  | `+02:00:00`   | Zero-padded signed time zone UTC hour, minute and second offsets with colons (`+hh:mm:ss`).                           |
//! |    `%:::z` | `+02`         | Zero-padded signed time zone UTC hour offset, with optional minute and second offsets with colons (`+hh[:mm[:ss]]`).  |
//! |    `%Z`    | `CEST`        | Platform-dependent abbreviated time zone name.                                                                        |
//! |    `%A`    | `Sunday`      | Locale independent full weekday name.                                                                                 |
//! |    `%a`    | `Sun`         | Locale independent abbreviated weekday name, using the first 3 letters.                                               |
//! |    `%u`    | `1`           | Day of the week from Monday in `1..=7`, zero-padded to 1 digit.                                                       |
//! |    `%w`    | `0`           | Day of the week from Sunday in `0..=6`, zero-padded to 1 digit.                                                       |
//! |    `%G`    | `-2001`       | Same as `%Y`, but using the ISO 8601 week-based year. [^1]                                                            |
//! |    `%g`    | `99`          | Same as `%y`, but using the ISO 8601 week-based year. [^1]                                                            |
//! |    `%V`    | `01`          | ISO 8601 week number in `01..=53`, zero-padded to 2 digits. [^1]                                                      |
//! |    `%U`    | `00`          | Week number from Sunday in `00..=53`, zero-padded to 2 digits. The week `1` starts with the first Sunday of the year. |
//! |    `%W`    | `00`          | Week number from Monday in `00..=53`, zero-padded to 2 digits. The week `1` starts with the first Monday of the year. |
//! |    `%s`    | `86400`       | Number of seconds since `1970-01-01 00:00:00 UTC`, zero-padded to at least 1 digit.                                   |
//! |    `%n`    | `\n`          | Newline character `'\n'`.                                                                                             |
//! |    `%t`    | `\t`          | Tab character `'\t'`.                                                                                                 |
//! |    `%%`    | `%`           | Literal `'%'` character.                                                                                              |
//! |    `%c`    | `Sun Jul  8 00:23:45 2001` | Date and time, equivalent to `"%a %b %e %H:%M:%S %Y"`.                                                   |
//! | `%D`, `%x` | `07/08/01`    | Date, equivalent to `"%m/%d/%y"`.                                                                                     |
//! |    `%F`    | `2001-07-08`  | ISO 8601 date, equivalent to `"%Y-%m-%d"`.                                                                            |
//! |    `%v`    | ` 8-JUL-2001` | VMS date, equivalent to `"%e-%^b-%4Y"`.                                                                               |
//! |    `%r`    | `12:23:45 AM` | 12-hour time, equivalent to `"%I:%M:%S %p"`.                                                                          |
//! |    `%R`    | `00:23`       | 24-hour time without seconds, equivalent to `"%H:%M"`.                                                                |
//! | `%T`, `%X` | `00:23:45`    | 24-hour time, equivalent to `"%H:%M:%S"`.                                                                             |
//!
//! [^1]: `%G`, `%g`, `%V`: Week 1 of ISO 8601 is the first week with at least 4
//! days in that year. The days before the first week are in the last week of
//! the previous year.

#![doc(html_root_url = "https://docs.rs/strftime-ruby/0.1.0")]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
use alloc::collections::TryReserveError;

// Ensure code blocks in `README.md` compile
#[cfg(all(doctest, feature = "std"))]
#[doc = include_str!("../README.md")]
mod readme {}

mod format;

#[cfg(test)]
mod functional_tests;

use core::fmt;

/// Error type returned by the `strftime` functions.
#[derive(Debug, Clone, Eq, PartialEq)]
// To ensure the API is the same for all feature combinations, do not derive
// `Copy`. The `OutOfMemory` variant (when it is enabled by `alloc`) contains a
// member that is not `Copy`.
#[non_exhaustive]
#[allow(missing_copy_implementations)]
pub enum Error {
    /// Provided time implementation returns invalid values.
    InvalidTime,
    /// Provided format string is ended by an unterminated format specifier.
    InvalidFormatString,
    /// Formatted string is too large and could cause an out-of-memory error.
    FormattedStringTooLarge,
    /// Provided buffer for the [`buffered::strftime`] function is too small for
    /// the formatted string.
    ///
    /// This corresponds to the [`std::io::ErrorKind::WriteZero`] variant.
    ///
    /// [`std::io::ErrorKind::WriteZero`]: <https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.WriteZero>
    WriteZero,
    /// Formatting error, corresponding to [`core::fmt::Error`].
    FmtError,
    /// An allocation failure has occurred in either [`bytes::strftime`] or
    /// [`string::strftime`].
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    OutOfMemory(TryReserveError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidTime => f.write_str("invalid time"),
            Error::InvalidFormatString => f.write_str("invalid format string"),
            Error::FormattedStringTooLarge => f.write_str("formatted string too large"),
            Error::WriteZero => f.write_str("failed to write the whole buffer"),
            Error::FmtError => f.write_str("formatter error"),
            #[cfg(feature = "alloc")]
            Error::OutOfMemory(..) => f.write_str("allocation failure"),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OutOfMemory(ref inner) => Some(inner),
            _ => None,
        }
    }
}

#[cfg(feature = "alloc")]
impl From<TryReserveError> for Error {
    fn from(err: TryReserveError) -> Self {
        Self::OutOfMemory(err)
    }
}

/// Common methods needed for formatting _time_.
///
/// This should be implemented for structs representing a _time_.
///
/// All the `strftime` functions take as input an implementation of this trait.
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
    /// Returns an integer representing the day of the week in `0..=6`, with
    /// `Sunday == 0`.
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

// Check that the Time trait is object-safe
const _: Option<&dyn Time> = None;

/// Provides a buffered `strftime` implementation using a format string with
/// arbitrary bytes.
pub mod buffered {
    use super::{Error, Time};
    use crate::format::TimeFormatter;

    /// Format a _time_ implementation with the specified format byte string,
    /// writing in the provided buffer and returning the written subslice.
    ///
    /// See the [crate-level documentation](crate) for a complete description of
    /// possible format specifiers.
    ///
    /// # Allocations
    ///
    /// This `strftime` implementation makes no heap allocations and is usable
    /// in a `no_std` context.
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
    use alloc::vec::Vec;

    use super::{Error, Time};
    use crate::format::TimeFormatter;

    /// Format a _time_ implementation with the specified format byte string.
    ///
    /// See the [crate-level documentation](crate) for a complete description of
    /// possible format specifiers.
    ///
    /// # Allocations
    ///
    /// This `strftime` implementation writes its output to a heap-allocated
    /// [`Vec`]. The implementation exclusively uses fallible allocation APIs
    /// like [`Vec::try_reserve`]. This function will return [`Error::OutOfMemory`]
    /// if there is an allocation failure.
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
    use alloc::string::String;
    use alloc::vec::Vec;

    use super::{Error, Time};
    use crate::format::TimeFormatter;

    /// Format a _time_ implementation with the specified UTF-8 format string.
    ///
    /// See the [crate-level documentation](crate) for a complete description of
    /// possible format specifiers.
    ///
    /// # Allocations
    ///
    /// This `strftime` implementation writes its output to a heap-allocated
    /// [`Vec`]. The implementation exclusively uses fallible allocation APIs
    /// like [`Vec::try_reserve`]. This function will return [`Error::OutOfMemory`]
    /// if there is an allocation failure.
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

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "alloc")]
    fn error_display_is_non_empty() {
        use alloc::string::String;
        use core::fmt::Write;

        use super::Error;

        let try_reserve_error = {
            let mut s = String::with_capacity(1);
            s.try_reserve(usize::MAX).unwrap_err()
        };

        let test_cases = [
            Error::InvalidTime,
            Error::InvalidFormatString,
            Error::FormattedStringTooLarge,
            Error::WriteZero,
            Error::FmtError,
            Error::OutOfMemory(try_reserve_error),
        ];
        for err in test_cases {
            let mut buf = String::new();
            write!(&mut buf, "{err}").unwrap();
            assert!(!buf.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn error_cause_returns_inner_error() {
        use alloc::collections::TryReserveError;
        use alloc::string::String;
        use std::error::Error as _;

        use super::Error;

        let try_reserve_error = {
            let mut s = String::with_capacity(1);
            s.try_reserve(usize::MAX).unwrap_err()
        };

        // Errors variants with no inner error
        let test_cases = [
            Error::InvalidTime,
            Error::InvalidFormatString,
            Error::FormattedStringTooLarge,
            Error::WriteZero,
            Error::FmtError,
        ];
        for err in test_cases {
            assert!(err.source().is_none());
        }

        // Error variants with an inner error
        let err = Error::OutOfMemory(try_reserve_error.clone());
        let err_source = err.source().unwrap();
        assert_eq!(
            err_source
                .downcast_ref::<TryReserveError>()
                .unwrap()
                .clone(),
            try_reserve_error
        );
    }
}
