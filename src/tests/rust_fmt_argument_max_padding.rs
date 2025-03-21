//! These tests exist to validate padding behavior when using dynamic widths in
//! format strings.  As of rust-lang/rust#136932 (part of #99012), the Rust
//! standard library now restricts width and precision fields in format strings
//! to `u16::MAX`, to improve memory layout and prevent silent truncation bugs
//! on cross-compilation targets. This change caused previously valid dynamic
//! width values exceeding `u16::MAX` to panic at runtime. (See
//! rust-lang/rust#136932).
//!
//! These test cases specifically target edge conditions revealed by fuzzing
//! strftime-ruby, ensuring we handle excessively large padding values without
//! panicking, aligning behavior closely with CRuby's `Time#strftime`
//! specification and the limitations described in rust-lang/rust#136932.
//!
//! Reference:
//!
//! - <https://github.com/rust-lang/rust/pull/136932>
//! - <https://github.com/rust-lang/rust/pull/136932#issuecomment-2739434542>

#![allow(clippy::unreadable_literal)]

#[cfg(feature = "alloc")]
use {
    crate::format::TimeFormatter,
    alloc::{format, vec},
};

use crate::Error;

use super::{check_all, get_format_err, get_format_err_bytes, MockTime};

#[test]
fn test_larger_than_int_max_formats_are_returned_verbatim() {
    let times = [
        MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, ""),
        MockTime::new(-1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, ""),
    ];

    for format in [
        "%100000000000000000000c",
        "%1000000000000c",
        "%10000000000c",
    ] {
        check_all(&times, format, &[format, format]);
    }
}

#[test]
#[cfg(feature = "alloc")]
fn test_format_specifiers_large_width_success() {
    // List of format specifiers that take a width.
    //
    // For each, we construct a format string with a width of 131,072. The
    // format string is wrapped in single quotes so that we can easily strip
    // them and check that the inner formatted result has exactly the given
    // width.
    let specifiers = [
        "Y", "C", "y", "m", "B", "b", "d", "e", "j", "H", "k", "I", "l", "P", "p", "M", "S", "L",
        "N", "z", ":z", "::z", ":::z", "Z", "A", "a", "u", "w", "G", "g", "V", "U", "W", "s", "n",
        "t", "c", "D", "F", "v", "r", "R", "T", "X",
    ];
    // Some width greater than `u16::MAX`.
    let width = 2 * usize::from(u16::MAX);

    // A valid and interesting MockTime instance that exercises a wide range of
    // specifiers (e.g. year, month, day, time, fractional seconds, week day,
    // time zone, etc.):
    let time = MockTime::new(
        2021,       // year: 2021 (a recent common year)
        12,         // month: December
        31,         // day: 31st (last day of the year)
        23,         // hour: 23 (will yield 11 PM in 12-hour formats)
        59,         // minute: 59
        60,         // second: 60 (testing the leap-second edge case, as spec allows 00..=60)
        987654321,  // nanoseconds: an interesting fraction for testing %L and %N
        5,          // day_of_week: 5 (if 0 = Sunday, then 5 = Friday)
        365,        // day_of_year: December 31 is the 365th day in a non-leap year
        1640995200, // to_int: seconds since epoch (an arbitrary value corresponding roughly to 2022-01-01T00:00:00 UTC)
        false,      // is_utc: false (indicating local time)
        3600,       // utc_offset: +3600 seconds (i.e. UTC+1)
        "CET",      // time_zone: the time zone name (e.g. "CET")
    );

    for spec in specifiers {
        // Build a format string with the given width and specifier.
        // For example, if spec is "Y", the format string will be: "|%65636Y|"
        let fmt_str = format!("|%{width}{spec}|");

        // Allocate a buffer large enough to hold the resulting formatted string.
        // We expect the specifier to produce an output shorter than the given width,
        // so the result should be padded to exactly `width` characters (inside the quotes).
        let mut buf = vec![0u8; width + 2]; // +2 for the surrounding quotes

        let result = TimeFormatter::new(&time, fmt_str.as_bytes()).fmt(&mut buf.as_mut_slice());
        result.unwrap_or_else(|_| panic!("Failed for specifier '{spec}' with width {width}"));

        let output = core::str::from_utf8(&buf).expect("Output not valid UTF-8");
        match &buf[..] {
            [b'|', inner @ .., b'|'] => {
                assert_eq!(
                    inner.len(),
                    width,
                    "bad len for '{spec}': expected {width}, got {got}",
                    got = inner.len()
                );
            }
            _ => panic!("Output not properly quoted for specifier '{spec}': {output}"),
        };
    }
}

#[test]
#[cfg(feature = "alloc")]
fn test_format_specifiers_int_max_fail() {
    // List of format specifiers that take a width.
    //
    // Test that using a width equal to `INT_MAX` (2,147,483,647) causes an
    // error (e.g. due to write buffer limits). We use a small output buffer so
    // that the formatting attempt cannot succeed.
    let specifiers = [
        "Y", "C", "y", "m", "B", "b", "d", "e", "j", "H", "k", "I", "l", "P", "p", "M", "S", "L",
        "N", "z", ":z", "::z", ":::z", "Z", "A", "a", "u", "w", "G", "g", "V", "U", "W", "s", "n",
        "t", "c", "D", "F", "v", "r", "R", "T", "X",
    ];
    let width = usize::try_from(i32::MAX).unwrap();

    // A valid and interesting MockTime instance that exercises a wide range of
    // specifiers (e.g. year, month, day, time, fractional seconds, week day,
    // time zone, etc.):
    let time = MockTime::new(
        2021,       // year: 2021 (a recent common year)
        12,         // month: December
        31,         // day: 31st (last day of the year)
        23,         // hour: 23 (will yield 11 PM in 12-hour formats)
        59,         // minute: 59
        60,         // second: 60 (testing the leap-second edge case, as spec allows 00..=60)
        987654321,  // nanoseconds: an interesting fraction for testing %L and %N
        5,          // day_of_week: 5 (if 0 = Sunday, then 5 = Friday)
        365,        // day_of_year: December 31 is the 365th day in a non-leap year
        1640995200, // to_int: seconds since epoch (an arbitrary value corresponding roughly to 2022-01-01T00:00:00 UTC)
        false,      // is_utc: false (indicating local time)
        3600,       // utc_offset: +3600 seconds (i.e. UTC+1)
        "CET",      // time_zone: the time zone name (e.g. "CET")
    );

    for spec in specifiers {
        let fmt_str = format!("'%{width}{spec}'");
        // Use a very small buffer to force a write failure.
        let mut buf = [0u8; 100];
        let err = TimeFormatter::new(&time, fmt_str.as_bytes())
            .fmt(&mut &mut buf[..])
            .unwrap_err();
        assert!(
            matches!(err, Error::WriteZero),
            "Expected write failure for specifier '{spec}' with width {width} but got unexpected error: {err:?}",
        );
    }
}
