mod error;
mod format;
mod rust_fmt_argument_max_padding;

use crate::format::TimeFormatter;
use crate::{Error, Time};

include!("../mock.rs.in");

fn get_format_err(time: &MockTime<'_>, format: &str) -> Error {
    TimeFormatter::new(time, format)
        .fmt(&mut &mut [0u8; 100][..])
        .unwrap_err()
}

fn check_format(time: &MockTime<'_>, format: &str, expected: &str) {
    const SIZE: usize = 100;
    let mut buf = [0u8; SIZE];
    let mut cursor = &mut buf[..];

    TimeFormatter::new(time, format).fmt(&mut cursor).unwrap();
    let written = SIZE - cursor.len();
    let data = core::str::from_utf8(&buf[..written]).unwrap();

    assert_eq!(data, expected);
}

fn check_all(times: &[MockTime<'_>], format: &str, all_expected: &[&str]) {
    assert_eq!(times.len(), all_expected.len());
    for (time, expected) in times.iter().zip(all_expected) {
        check_format(time, format, expected);
    }
}

#[test]
#[should_panic = "assertion `left == right` failed"]
#[rustfmt::skip]
fn test_check_format_panics_on_error() {
    let time = MockTime { year: 1111,  ..Default::default() };

    check_format(&time, "'%Y'", "'1112'");
}

#[test]
#[should_panic = "assertion `left == right` failed"]
#[rustfmt::skip]
fn test_check_all_panics_on_error() {
    let times = [
        MockTime { year: -1111, ..Default::default() },
        MockTime { year: -11,   ..Default::default() },
        MockTime { year: 1,     ..Default::default() },
        MockTime { year: 1111,  ..Default::default() },
    ];

    check_all(&times, "'%Y'", &["'-1111'", "'-0011'", "'0001'",  "'1112'"]);
}
