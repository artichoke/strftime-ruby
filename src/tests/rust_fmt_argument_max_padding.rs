//! These tests exist to validate padding behavior when using dynamic widths in
//! format strings.  As of rust-lang/rust#136932 (part of #99012), the Rust
//! standard library now restricts width and precision fields in format strings
//! to `u16::MAX`, to improve memory layout and prevent silent truncation bugs
//! on cross-compilation targets. This change caused previously valid dynamic
//! width values exceeding `u16::MAX` to panic at runtime. (See
//! rust-lang/rust#136932 and artichoke/strftime-ruby#2f0ab4a).
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
fn test_rust_136932_reduce_fmt_argument_width_and_precision_fuzzer_failures() {
    let test_cases: [(MockTime<'_>, &str); 8] = [
        (
            MockTime::new(-184549375, 93, 71, 166, 14, 1, 2788140547, 166, 65535, 376621239690988031, false, -1499059802, "Z\t\u{3}\u{1}9999"),
            "=99\0\0\0Y%66067c\0\0"
        ),
        (
            MockTime::new(12255231, 202, 175, 0, 2, 0, 3284320768, 251, 0, -5836660741859442688, false, 65538, ""),
            "\0%666666%%3%"
        ),
        (
            MockTime::new(1737472, 0, 0, 0, 0, 11, 0, 0, 0, -256, false, 64256, ""),
            "%111111C1111111111111117\u{1}\0X\0XXD"
        ),
        (
            MockTime::new(3825760, 0, 0, 51, 0, 0, 1044381696, 37, 122, 6584368208339936574, false, 0, ""),
            "@%1111111111z\0\0"
        ),
        (
            MockTime::new(1090397537, 254, 255, 2, 0, 0, 4294639365, 255, 65535, 10455414982377471, false, -33540827, ""),
            "%%\n%5555555%55555555555u555"
        ),
        (
            MockTime::new(-21761, 255, 58, 0, 37, 37, 620822272, 255, 43690, 50396945, false, 3, ""),
            "%R%S%00000400000R%RR%%%\u{3}\0%\0\u{1b}\u{1b}\u{1b}"
        ),
        (
            MockTime::new(13566191, 1, 0, 34, 35, 37, 623190565, 0, 8741, 2457877026632491810, true, 606414171, ""),
            "\0\0\0\0\0\0\0\"%2536361%172049423024124\"2\"\"9\"%^^^YN^\""
        ),
        (
            MockTime::new(1749024784, 96, 0, 96, 104, 96, 620796776, 0, 15968, 0, false, 6307376, ""),
            "%u\0\n\0\0\u{10}\0`\u{31b}@\0\0\x002>`\0%33333333s33333#33333333333333333333333333333333333\u{13}33333333333u\0\0\0\0\0%u\u{4}\0\0\0\u{10}\0`"
        ),
    ];

    for (time, format) in test_cases {
        let err = get_format_err(&time, format);
        assert!(matches!(err, Error::WriteZero));
    }
}

#[test]
#[rustfmt::skip]
fn test_rust_136932_reduce_fmt_argument_width_and_precision_fuzzer_failures_bytes() {
    let test_cases: [(MockTime<'_>, &[u8]); 8] = [
        (
            MockTime::new(674049317, 244, 42, 40, 180, 106, 4096628178, 1, 0, 3239384020795663872, false, 1734567725, ""),
            b"\xd6\x00\x00\x00\x00\xda\xc7\xda%\xc1B%-666666F\xff\xff\xff"
        ),
        (
            MockTime::new(167837442, 64, 255, 10, 96, 255, 4294901760, 255, 65535, -1, true, 235864006, ""),
            b"\x88%%%%%%%%%%%%%777777%%%\x0e"
        ),
        (
            MockTime::new(1375731722, 82, 82, 82, 85, 82, 1392508927, 82, 20992, -48935437123235246, false, -1145324613, ""),
            b"\xbb\xbb%\x00\x00RR%\xbb\xbb\x00\x00\x00B%00124261%R\xff\x00\xbb\xbb\xbb\xbb\x00\x00\x00\x00\x00RRR\xa6RR"
        ),
        (
            MockTime::new(0, 0, 0, 0, 92, 0, 704643072, 0, 242, 5548434740937171968, false, 0, ""),
            b"\x00\x00%7777773%"
        ),
        (
            MockTime::new(458557, 0, 64, 128, 128, 128, 4278222976, 255, 65535, -205, true, 65535, "\0\0\0"),
            b"\x00%00324248X14\xff"
        ),
        (
            MockTime::new(-774831746, 209, 209, 209, 13, 13, 218959117, 13, 3341, 0, false, 218955776, ":\r\r"),
            b"\xff\x00\xff\xff\x08%%%%%%%%%%%%%5%%%%3333333%%"
        ),
        (
            MockTime::new(130826, 3, 223, 29, 37, 37, 623191333, 93, 13861, 2892759177136963579, true, 623191333, "!I\u{1}\0\u{3}"),
            b"\xdf\x1d%\x1d?\xff\xdb\xfe#\x15I%$\x00\x00\x00%]%86995%%6\xfb"
        ),
        (
            MockTime::new(37454602, 0, 0, 0, 0, 9, 1048576, 255, 65535, -1, false, 255, "\0\0\0\0\0/\0"),
            b"y%999999%q)%%z"
        ),
    ];

    for (time, format) in test_cases {
        let err = get_format_err_bytes(&time, format);
        assert!(matches!(err, Error::WriteZero));
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
