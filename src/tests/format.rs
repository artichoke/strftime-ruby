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
#[rustfmt::skip]
fn test_format_year_4_digits() {
    let times = [
        MockTime { year: -1111, ..Default::default() },
        MockTime { year: -11,   ..Default::default() },
        MockTime { year: 1,     ..Default::default() },
        MockTime { year: 1111,  ..Default::default() },
    ];

    check_all(&times, "'%Y'",    &["'-1111'", "'-0011'", "'0001'",  "'1111'"]);
    check_all(&times, "'%1Y'",   &["'-1111'", "'-11'",   "'1'",     "'1111'"]);
    check_all(&times, "'%4Y'",   &["'-1111'", "'-011'",  "'0001'",  "'1111'"]);
    check_all(&times, "'%-_5Y'", &["'-1111'", "'-11'",   "'1'",     "'1111'"]);
    check_all(&times, "'%-05Y'", &["'-1111'", "'-11'",   "'1'",     "'1111'"]);
    check_all(&times, "'%0_5Y'", &["'-1111'", "'  -11'", "'    1'", "' 1111'"]);
    check_all(&times, "'%_05Y'", &["'-1111'", "'-0011'", "'00001'", "'01111'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_year_div_100() {
    let times = [
        MockTime { year: -1111, ..Default::default() },
        MockTime { year: -11,   ..Default::default() },
        MockTime { year: 1,     ..Default::default() },
        MockTime { year: 1111,  ..Default::default() },
    ];

    check_all(&times, "'%C'",    &["'-12'",  "'-1'",   "'00'",   "'11'"]);
    check_all(&times, "'%1C'",   &["'-12'",  "'-1'",   "'0'",    "'11'"]);
    check_all(&times, "'%4C'",   &["'-012'", "'-001'", "'0000'", "'0011'"]);
    check_all(&times, "'%-_4C'", &["'-12'",  "'-1'",   "'0'",    "'11'"]);
    check_all(&times, "'%-04C'", &["'-12'",  "'-1'",   "'0'",    "'11'"]);
    check_all(&times, "'%0_4C'", &["' -12'", "'  -1'", "'   0'", "'  11'"]);
    check_all(&times, "'%_04C'", &["'-012'", "'-001'", "'0000'", "'0011'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_year_rem_100() {
    let times = [
        MockTime { year: -1111, ..Default::default() },
        MockTime { year: -11,   ..Default::default() },
        MockTime { year: 1,     ..Default::default() },
        MockTime { year: 1111,  ..Default::default() },
    ];

    check_all(&times, "'%y'",   &["'89'",   "'89'",   "'01'",   "'11'"]);
    check_all(&times, "'%1y'",  &["'89'",   "'89'",   "'1'",    "'11'"]);
    check_all(&times, "'%4y'",  &["'0089'", "'0089'", "'0001'", "'0011'"]);
    check_all(&times, "'%-_y'", &["'89'",   "'89'",   "'1'",    "'11'"]);
    check_all(&times, "'%-0y'", &["'89'",   "'89'",   "'1'",    "'11'"]);
    check_all(&times, "'%0_y'", &["'89'",   "'89'",   "' 1'",   "'11'"]);
    check_all(&times, "'%_0y'", &["'89'",   "'89'",   "'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_month() {
    let times = [
        MockTime { month: 1,  ..Default::default() },
        MockTime { month: 11, ..Default::default() },
    ];

    check_all(&times, "'%m'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1m'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4m'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_m'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0m'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_m'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0m'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_name() {
    let times = [MockTime { month: 7, ..Default::default() }];

    check_all(&times, "'%B'",      &["'July'"]);
    check_all(&times, "'%1B'",     &["'July'"]);
    check_all(&times, "'%6B'",     &["'  July'"]);
    check_all(&times, "'%-_#^6B'", &["'JULY'"]);
    check_all(&times, "'%-0^6B'",  &["'JULY'"]);
    check_all(&times, "'%0_#6B'",  &["'  JULY'"]);
    check_all(&times, "'%_06B'",   &["'00July'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_name_abbr() {
    let times = [MockTime { month: 7, ..Default::default() }];

    check_all(&times, "'%b'",      &["'Jul'"]);
    check_all(&times, "'%1b'",     &["'Jul'"]);
    check_all(&times, "'%6b'",     &["'   Jul'"]);
    check_all(&times, "'%-_#^6b'", &["'JUL'"]);
    check_all(&times, "'%-0^6b'",  &["'JUL'"]);
    check_all(&times, "'%0_#6b'",  &["'   JUL'"]);
    check_all(&times, "'%_06b'",   &["'000Jul'"]);

    check_all(&times, "'%h'",      &["'Jul'"]);
    check_all(&times, "'%1h'",     &["'Jul'"]);
    check_all(&times, "'%6h'",     &["'   Jul'"]);
    check_all(&times, "'%-_#^6h'", &["'JUL'"]);
    check_all(&times, "'%-0^6h'",  &["'JUL'"]);
    check_all(&times, "'%0_#6h'",  &["'   JUL'"]);
    check_all(&times, "'%_06h'",   &["'000Jul'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_day_zero() {
    let times = [
        MockTime { day: 1,  ..Default::default() },
        MockTime { day: 11, ..Default::default() },
    ];

    check_all(&times, "'%d'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1d'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4d'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_d'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0d'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_d'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0d'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_day_space() {
    let times = [
        MockTime { day: 1,  ..Default::default() },
        MockTime { day: 11, ..Default::default() },
    ];

    check_all(&times, "'%e'",   &["' 1'",   "'11'"]);
    check_all(&times, "'%1e'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4e'",  &["'   1'", "'  11'"]);
    check_all(&times, "'%-_e'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0e'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_e'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0e'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_year_day() {
    let times = [
        MockTime { day_of_year: 1,   ..Default::default() },
        MockTime { day_of_year: 300, ..Default::default() },
    ];

    check_all(&times, "'%j'",   &["'001'",  "'300'"]);
    check_all(&times, "'%1j'",  &["'1'",    "'300'"]);
    check_all(&times, "'%4j'",  &["'0001'", "'0300'"]);
    check_all(&times, "'%-_j'", &["'1'",    "'300'"]);
    check_all(&times, "'%-0j'", &["'1'",    "'300'"]);
    check_all(&times, "'%0_j'", &["'  1'",  "'300'"]);
    check_all(&times, "'%_0j'", &["'001'",  "'300'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_24h_zero() {
    let times = [
        MockTime { hour: 1,  ..Default::default() },
        MockTime { hour: 11, ..Default::default() },
    ];

    check_all(&times, "'%H'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1H'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4H'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_H'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0H'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_H'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0H'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_24h_space() {
    let times = [
        MockTime { hour: 1,  ..Default::default() },
        MockTime { hour: 11, ..Default::default() },
    ];

    check_all(&times, "'%k'",   &["' 1'",   "'11'"]);
    check_all(&times, "'%1k'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4k'",  &["'   1'", "'  11'"]);
    check_all(&times, "'%-_k'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0k'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_k'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0k'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_12h_zero() {
    let times = [
        MockTime { hour: 13, ..Default::default() },
        MockTime { hour: 0,  ..Default::default() },
    ];

    check_all(&times, "'%I'",   &["'01'",   "'12'"]);
    check_all(&times, "'%1I'",  &["'1'",    "'12'"]);
    check_all(&times, "'%4I'",  &["'0001'", "'0012'"]);
    check_all(&times, "'%-_I'", &["'1'",    "'12'"]);
    check_all(&times, "'%-0I'", &["'1'",    "'12'"]);
    check_all(&times, "'%0_I'", &["' 1'",   "'12'"]);
    check_all(&times, "'%_0I'", &["'01'",   "'12'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_12h_space() {
    let times = [
        MockTime { hour: 13, ..Default::default() },
        MockTime { hour: 0,  ..Default::default() },
    ];

    check_all(&times, "'%l'",   &["' 1'",   "'12'"]);
    check_all(&times, "'%1l'",  &["'1'",    "'12'"]);
    check_all(&times, "'%4l'",  &["'   1'", "'  12'"]);
    check_all(&times, "'%-_l'", &["'1'",    "'12'"]);
    check_all(&times, "'%-0l'", &["'1'",    "'12'"]);
    check_all(&times, "'%0_l'", &["' 1'",   "'12'"]);
    check_all(&times, "'%_0l'", &["'01'",   "'12'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_meridian_lower() {
    let times = [
        MockTime { hour: 11, ..Default::default() },
        MockTime { hour: 12, ..Default::default() },
    ];

    check_all(&times, "'%P'",      &["'am'",   "'pm'"]);
    check_all(&times, "'%1P'",     &["'am'",   "'pm'"]);
    check_all(&times, "'%4P'",     &["'  am'", "'  pm'"]);
    check_all(&times, "'%-_#^4P'", &["'AM'",   "'PM'"]);
    check_all(&times, "'%-0^4P'",  &["'AM'",   "'PM'"]);
    check_all(&times, "'%0_#4P'",  &["'  AM'", "'  PM'"]);
    check_all(&times, "'%_04P'",   &["'00am'", "'00pm'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_meridian_upper() {
    let times = [
        MockTime { hour: 11, ..Default::default() },
        MockTime { hour: 12, ..Default::default() },
    ];

    check_all(&times, "'%p'",      &["'AM'",   "'PM'"]);
    check_all(&times, "'%1p'",     &["'AM'",   "'PM'"]);
    check_all(&times, "'%4p'",     &["'  AM'", "'  PM'"]);
    check_all(&times, "'%-_#^4p'", &["'am'",   "'pm'"]);
    check_all(&times, "'%-0^4p'",  &["'AM'",   "'PM'"]);
    check_all(&times, "'%0_#4p'",  &["'  am'", "'  pm'"]);
    check_all(&times, "'%_04p'",   &["'00AM'", "'00PM'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_minute() {
    let times = [
        MockTime { minute: 1,  ..Default::default() },
        MockTime { minute: 11, ..Default::default() },
    ];

    check_all(&times, "'%M'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1M'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4M'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_M'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0M'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_M'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0M'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_second() {
    let times = [
        MockTime { second: 1,  ..Default::default() },
        MockTime { second: 11, ..Default::default() },
    ];

    check_all(&times, "'%S'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1S'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4S'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_S'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0S'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_S'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0S'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_milli_second() {
    let times = [
        MockTime { nanoseconds: 1,           ..Default::default() },
        MockTime { nanoseconds: 123_456_789, ..Default::default() },
    ];

    check_all(&times, "'%L'",    &["'000'",          "'123'"]);
    check_all(&times, "'%00L'",  &["'000'",          "'123'"]);
    check_all(&times, "'%0L'",   &["'000'",          "'123'"]);
    check_all(&times, "'%1L'",   &["'0'",            "'1'"]);
    check_all(&times, "'%2L'",   &["'00'",           "'12'"]);
    check_all(&times, "'%3L'",   &["'000'",          "'123'"]);
    check_all(&times, "'%4L'",   &["'0000'",         "'1234'"]);
    check_all(&times, "'%5L'",   &["'00000'",        "'12345'"]);
    check_all(&times, "'%6L'",   &["'000000'",       "'123456'"]);
    check_all(&times, "'%7L'",   &["'0000000'",      "'1234567'"]);
    check_all(&times, "'%8L'",   &["'00000000'",     "'12345678'"]);
    check_all(&times, "'%9L'",   &["'000000001'",    "'123456789'"]);
    check_all(&times, "'%12L'",  &["'000000001000'", "'123456789000'"]);
    check_all(&times, "'%-12L'", &["'000000001000'", "'123456789000'"]);
    check_all(&times, "'%_12L'", &["'000000001000'", "'123456789000'"]);
    check_all(&times, "'%012L'", &["'000000001000'", "'123456789000'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_fractional_second() {
    let times = [
        MockTime { nanoseconds: 1,           ..Default::default() },
        MockTime { nanoseconds: 123_456_789, ..Default::default() },
    ];

    check_all(&times, "'%N'",    &["'000000001'",    "'123456789'"]);
    check_all(&times, "'%00N'",  &["'000000001'",    "'123456789'"]);
    check_all(&times, "'%0N'",   &["'000000001'",    "'123456789'"]);
    check_all(&times, "'%1N'",   &["'0'",            "'1'"]);
    check_all(&times, "'%2N'",   &["'00'",           "'12'"]);
    check_all(&times, "'%3N'",   &["'000'",          "'123'"]);
    check_all(&times, "'%4N'",   &["'0000'",         "'1234'"]);
    check_all(&times, "'%5N'",   &["'00000'",        "'12345'"]);
    check_all(&times, "'%6N'",   &["'000000'",       "'123456'"]);
    check_all(&times, "'%7N'",   &["'0000000'",      "'1234567'"]);
    check_all(&times, "'%8N'",   &["'00000000'",     "'12345678'"]);
    check_all(&times, "'%9N'",   &["'000000001'",    "'123456789'"]);
    check_all(&times, "'%12N'",  &["'000000001000'", "'123456789000'"]);
    check_all(&times, "'%-12N'", &["'000000001000'", "'123456789000'"]);
    check_all(&times, "'%_12N'", &["'000000001000'", "'123456789000'"]);
    check_all(&times, "'%012N'", &["'000000001000'", "'123456789000'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_time_zone_offset_hour_minute() {
    let times = [
        MockTime { is_utc: true,  utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 561,  ..Default::default() },
        MockTime { is_utc: false, utc_offset: 3600, ..Default::default() },
    ];

    check_all(&times, "'%z'",    &["'+0000'",  "'+0000'",  "'+0009'",  "'+0100'"]);
    check_all(&times, "'%1z'",   &["'+0000'",  "'+0000'",  "'+0009'",  "'+0100'"]);
    check_all(&times, "'%6z'",   &["'+00000'", "'+00000'", "'+00009'", "'+00100'"]);
    check_all(&times, "'%-6z'",  &["'-00000'", "'+00000'", "'+00009'", "'+00100'"]);
    check_all(&times, "'%-_6z'", &["'  -000'", "'  +000'", "'  +009'", "'  +100'"]);
    check_all(&times, "'%-06z'", &["'-00000'", "'+00000'", "'+00009'", "'+00100'"]);
    check_all(&times, "'%0_6z'", &["'  +000'", "'  +000'", "'  +009'", "'  +100'"]);
    check_all(&times, "'%_06z'", &["'+00000'", "'+00000'", "'+00009'", "'+00100'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_time_zone_offset_hour_minute_colon() {
    let times = [
        MockTime { is_utc: true,  utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 561,  ..Default::default() },
        MockTime { is_utc: false, utc_offset: 3600, ..Default::default() },
    ];

    check_all(&times, "'%:z'",    &["'+00:00'",  "'+00:00'",  "'+00:09'",  "'+01:00'"]);
    check_all(&times, "'%1:z'",   &["'+00:00'",  "'+00:00'",  "'+00:09'",  "'+01:00'"]);
    check_all(&times, "'%7:z'",   &["'+000:00'", "'+000:00'", "'+000:09'", "'+001:00'"]);
    check_all(&times, "'%-7:z'",  &["'-000:00'", "'+000:00'", "'+000:09'", "'+001:00'"]);
    check_all(&times, "'%-_7:z'", &["'  -0:00'", "'  +0:00'", "'  +0:09'", "'  +1:00'"]);
    check_all(&times, "'%-07:z'", &["'-000:00'", "'+000:00'", "'+000:09'", "'+001:00'"]);
    check_all(&times, "'%0_7:z'", &["'  +0:00'", "'  +0:00'", "'  +0:09'", "'  +1:00'"]);
    check_all(&times, "'%_07:z'", &["'+000:00'", "'+000:00'", "'+000:09'", "'+001:00'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_time_zone_offset_hour_minute_second_colon() {
    let times = [
        MockTime { is_utc: true,  utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 561,  ..Default::default() },
        MockTime { is_utc: false, utc_offset: 3600, ..Default::default() },
    ];

    check_all(&times, "'%::z'",     &["'+00:00:00'",  "'+00:00:00'",  "'+00:09:21'",  "'+01:00:00'"]);
    check_all(&times, "'%1::z'",    &["'+00:00:00'",  "'+00:00:00'",  "'+00:09:21'",  "'+01:00:00'"]);
    check_all(&times, "'%10::z'",   &["'+000:00:00'", "'+000:00:00'", "'+000:09:21'", "'+001:00:00'"]);
    check_all(&times, "'%-10::z'",  &["'-000:00:00'", "'+000:00:00'", "'+000:09:21'", "'+001:00:00'"]);
    check_all(&times, "'%-_10::z'", &["'  -0:00:00'", "'  +0:00:00'", "'  +0:09:21'", "'  +1:00:00'"]);
    check_all(&times, "'%-010::z'", &["'-000:00:00'", "'+000:00:00'", "'+000:09:21'", "'+001:00:00'"]);
    check_all(&times, "'%0_10::z'", &["'  +0:00:00'", "'  +0:00:00'", "'  +0:09:21'", "'  +1:00:00'"]);
    check_all(&times, "'%_010::z'", &["'+000:00:00'", "'+000:00:00'", "'+000:09:21'", "'+001:00:00'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_time_zone_offset_colon_minimal() {
    let times = [
        MockTime { is_utc: true,  utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 0,    ..Default::default() },
        MockTime { is_utc: false, utc_offset: 540,  ..Default::default() },
        MockTime { is_utc: false, utc_offset: 561,  ..Default::default() },
        MockTime { is_utc: false, utc_offset: 3600, ..Default::default() },
    ];

    check_all(&times, "'%:::z'",     &["'+00'",        "'+00'",        "'+00:09'",     "'+00:09:21'",  "'+01'"]);
    check_all(&times, "'%1:::z'",    &["'+00'",        "'+00'",        "'+00:09'",     "'+00:09:21'",  "'+01'"]);
    check_all(&times, "'%10:::z'",   &["'+000000000'", "'+000000000'", "'+000000:09'", "'+000:09:21'", "'+000000001'"]);
    check_all(&times, "'%-10:::z'",  &["'-000000000'", "'+000000000'", "'+000000:09'", "'+000:09:21'", "'+000000001'"]);
    check_all(&times, "'%-_10:::z'", &["'        -0'", "'        +0'", "'     +0:09'", "'  +0:09:21'", "'        +1'"]);
    check_all(&times, "'%-010:::z'", &["'-000000000'", "'+000000000'", "'+000000:09'", "'+000:09:21'", "'+000000001'"]);
    check_all(&times, "'%0_10:::z'", &["'        +0'", "'        +0'", "'     +0:09'", "'  +0:09:21'", "'        +1'"]);
    check_all(&times, "'%_010:::z'", &["'+000000000'", "'+000000000'", "'+000000:09'", "'+000:09:21'", "'+000000001'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_time_zone_name() {
    let times = [
        MockTime { time_zone: "",      ..Default::default() },
        MockTime { time_zone: "UTC",   ..Default::default() },
        MockTime { time_zone: "+0000", ..Default::default() },
    ];

    check_all(&times, "'%Z'",      &["''", "'UTC'"   , "'+0000'"]);
    check_all(&times, "'%1Z'",     &["''", "'UTC'"   , "'+0000'"]);
    check_all(&times, "'%6Z'",     &["''", "'   UTC'", "' +0000'"]);
    check_all(&times, "'%-_#^6Z'", &["''", "'utc'"   , "'+0000'"]);
    check_all(&times, "'%-0^6Z'",  &["''", "'UTC'"   , "'+0000'"]);
    check_all(&times, "'%0_#6Z'",  &["''", "'   utc'", "' +0000'"]);
    check_all(&times, "'%_06Z'",   &["''", "'000UTC'", "'0+0000'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_name() {
    let times = [MockTime { day_of_week: 1, ..Default::default() }];

    check_all(&times, "'%A'",      &["'Monday'"]);
    check_all(&times, "'%1A'",     &["'Monday'"]);
    check_all(&times, "'%8A'",     &["'  Monday'"]);
    check_all(&times, "'%-_#^8A'", &["'MONDAY'"]);
    check_all(&times, "'%-0^8A'",  &["'MONDAY'"]);
    check_all(&times, "'%0_#8A'",  &["'  MONDAY'"]);
    check_all(&times, "'%_08A'",   &["'00Monday'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_name_abbr() {
    let times = [MockTime { day_of_week: 1, ..Default::default() }];

    check_all(&times, "'%a'",      &["'Mon'"]);
    check_all(&times, "'%1a'",     &["'Mon'"]);
    check_all(&times, "'%8a'",     &["'     Mon'"]);
    check_all(&times, "'%-_#^8a'", &["'MON'"]);
    check_all(&times, "'%-0^8a'",  &["'MON'"]);
    check_all(&times, "'%0_#8a'",  &["'     MON'"]);
    check_all(&times, "'%_08a'",   &["'00000Mon'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_from_1() {
    let times = [MockTime { day_of_week: 0, ..Default::default() }];

    check_all(&times, "'%u'",    &["'7'"]);
    check_all(&times, "'%1u'",   &["'7'"]);
    check_all(&times, "'%4u'",   &["'0007'"]);
    check_all(&times, "'%-_4u'", &["'7'"]);
    check_all(&times, "'%-04u'", &["'7'"]);
    check_all(&times, "'%0_4u'", &["'   7'"]);
    check_all(&times, "'%_04u'", &["'0007'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_from_0() {
    let times = [MockTime { day_of_week: 0, ..Default::default() }];

    check_all(&times, "'%w'",    &["'0'"]);
    check_all(&times, "'%1w'",   &["'0'"]);
    check_all(&times, "'%4w'",   &["'0000'"]);
    check_all(&times, "'%-_4w'", &["'0'"]);
    check_all(&times, "'%-04w'", &["'0'"]);
    check_all(&times, "'%0_4w'", &["'   0'"]);
    check_all(&times, "'%_04w'", &["'0000'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_year_iso_8601() {
    let times = [
        MockTime { year: -1111, day_of_year: 30, ..Default::default() },
        MockTime { year: -11,   day_of_year: 30, ..Default::default() },
        MockTime { year: 1,     day_of_year: 30, ..Default::default() },
        MockTime { year: 1111,  day_of_year: 30, ..Default::default() },
    ];

    check_all(&times, "'%G'",    &["'-1111'", "'-0011'", "'0001'",  "'1111'"]);
    check_all(&times, "'%1G'",   &["'-1111'", "'-11'",   "'1'",     "'1111'"]);
    check_all(&times, "'%4G'",   &["'-1111'", "'-011'",  "'0001'",  "'1111'"]);
    check_all(&times, "'%-_5G'", &["'-1111'", "'-11'",   "'1'",     "'1111'"]);
    check_all(&times, "'%-05G'", &["'-1111'", "'-11'",   "'1'",     "'1111'"]);
    check_all(&times, "'%0_5G'", &["'-1111'", "'  -11'", "'    1'", "' 1111'"]);
    check_all(&times, "'%_05G'", &["'-1111'", "'-0011'", "'00001'", "'01111'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_year_iso_8601_rem_100() {
    let times = [
        MockTime { year: -1111, day_of_year: 30, ..Default::default() },
        MockTime { year: -11,   day_of_year: 30, ..Default::default() },
        MockTime { year: 1,     day_of_year: 30, ..Default::default() },
        MockTime { year: 1111,  day_of_year: 30, ..Default::default() },
    ];

    check_all(&times, "'%g'",   &["'89'",   "'89'",   "'01'",   "'11'"]);
    check_all(&times, "'%1g'",  &["'89'",   "'89'",   "'1'",    "'11'"]);
    check_all(&times, "'%4g'",  &["'0089'", "'0089'", "'0001'", "'0011'"]);
    check_all(&times, "'%-_g'", &["'89'",   "'89'",   "'1'",    "'11'"]);
    check_all(&times, "'%-0g'", &["'89'",   "'89'",   "'1'",    "'11'"]);
    check_all(&times, "'%0_g'", &["'89'",   "'89'",   "' 1'",   "'11'"]);
    check_all(&times, "'%_0g'", &["'89'",   "'89'",   "'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_number_iso_8601() {
    let times = [
        MockTime { year: 2000, day_of_year: 7,  ..Default::default() },
        MockTime { year: 2000, day_of_year: 80, ..Default::default() },
    ];

    check_all(&times, "'%V'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1V'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4V'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_V'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0V'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_V'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0V'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_number_from_sunday() {
    let times = [
        MockTime { year: 2000, day_of_year: 7,  ..Default::default() },
        MockTime { year: 2000, day_of_year: 77, ..Default::default() },
    ];

    check_all(&times, "'%U'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1U'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4U'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_U'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0U'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_U'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0U'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_number_from_monday() {
    let times = [
        MockTime { year: 2000, day_of_year: 7,  ..Default::default() },
        MockTime { year: 2000, day_of_year: 77, ..Default::default() },
    ];

    check_all(&times, "'%W'",   &["'01'",   "'11'"]);
    check_all(&times, "'%1W'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4W'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_W'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0W'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_W'", &["' 1'",   "'11'"]);
    check_all(&times, "'%_0W'", &["'01'",   "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_seconds_since_epoch() {
    let times = [
        MockTime { to_int: 1,  ..Default::default() },
        MockTime { to_int: 11, ..Default::default() },
    ];

    check_all(&times, "'%s'",   &["'1'",    "'11'"]);
    check_all(&times, "'%1s'",  &["'1'",    "'11'"]);
    check_all(&times, "'%4s'",  &["'0001'", "'0011'"]);
    check_all(&times, "'%-_s'", &["'1'",    "'11'"]);
    check_all(&times, "'%-0s'", &["'1'",    "'11'"]);
    check_all(&times, "'%0_s'", &["'1'",    "'11'"]);
    check_all(&times, "'%_0s'", &["'1'",    "'11'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_newline() {
    let times = [MockTime::default()];

    check_all(&times, "'%n'",      &["'\n'"]);
    check_all(&times, "'%1n'",     &["'\n'"]);
    check_all(&times, "'%6n'",     &["'     \n'"]);
    check_all(&times, "'%-_#^6n'", &["'\n'"]);
    check_all(&times, "'%-0^6n'",  &["'\n'"]);
    check_all(&times, "'%0_#6n'",  &["'     \n'"]);
    check_all(&times, "'%_06n'",   &["'00000\n'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_tabulation() {
    let times = [MockTime::default()];

    check_all(&times, "'%t'",      &["'\t'"]);
    check_all(&times, "'%1t'",     &["'\t'"]);
    check_all(&times, "'%6t'",     &["'     \t'"]);
    check_all(&times, "'%-_#^6t'", &["'\t'"]);
    check_all(&times, "'%-0^6t'",  &["'\t'"]);
    check_all(&times, "'%0_#6t'",  &["'     \t'"]);
    check_all(&times, "'%_06t'",   &["'00000\t'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_percent() {
    let times = [MockTime::default()];

    check_all(&times, "'%%'",      &["'%'"]);
    check_all(&times, "'%1%'",     &["'%'"]);
    check_all(&times, "'%6%'",     &["'     %'"]);
    check_all(&times, "'%-_#^6%'", &["'%'"]);
    check_all(&times, "'%-0^6%'",  &["'%'"]);
    check_all(&times, "'%0_#6%'",  &["'     %'"]);
    check_all(&times, "'%_06%'",   &["'00000%'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_date_time() {
    let times = [
        MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, ""),
        MockTime::new(-1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, ""),
    ];

    check_all(&times, "'%c'",       &["'Thu Jan  1 00:00:00 1970'",       "'Thu Jan  1 00:00:00 -1970'"]);
    check_all(&times, "'%1c'",      &["'Thu Jan  1 00:00:00 1970'",       "'Thu Jan  1 00:00:00 -1970'"]);
    check_all(&times, "'%30c'",     &["'      Thu Jan  1 00:00:00 1970'", "'     Thu Jan  1 00:00:00 -1970'"]);
    check_all(&times, "'%-^_#30c'", &["'      THU JAN  1 00:00:00 1970'", "'     THU JAN  1 00:00:00 -1970'"]);
    check_all(&times, "'%-0^30c'",  &["'000000THU JAN  1 00:00:00 1970'", "'00000THU JAN  1 00:00:00 -1970'"]);
    check_all(&times, "'%0_#30c'",  &["'      Thu Jan  1 00:00:00 1970'", "'     Thu Jan  1 00:00:00 -1970'"]);
    check_all(&times, "'%_030c'",   &["'000000Thu Jan  1 00:00:00 1970'", "'00000Thu Jan  1 00:00:00 -1970'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_date() {
    let times = [
        MockTime { year: 1234,  month: 5, day: 6, ..Default::default() },
        MockTime { year: -1234, month: 5, day: 6, ..Default::default() },
    ];

    check_all(&times, "'%D'",       &["'05/06/34'",   "'05/06/66'"]);
    check_all(&times, "'%1D'",      &["'05/06/34'",   "'05/06/66'"]);
    check_all(&times, "'%10D'",     &["'  05/06/34'", "'  05/06/66'"]);
    check_all(&times, "'%-^_#10D'", &["'  05/06/34'", "'  05/06/66'"]);
    check_all(&times, "'%-0^10D'",  &["'0005/06/34'", "'0005/06/66'"]);
    check_all(&times, "'%0_#10D'",  &["'  05/06/34'", "'  05/06/66'"]);
    check_all(&times, "'%_010D'",   &["'0005/06/34'", "'0005/06/66'"]);

    check_all(&times, "'%x'",       &["'05/06/34'",   "'05/06/66'"]);
    check_all(&times, "'%1x'",      &["'05/06/34'",   "'05/06/66'"]);
    check_all(&times, "'%10x'",     &["'  05/06/34'", "'  05/06/66'"]);
    check_all(&times, "'%-^_#10x'", &["'  05/06/34'", "'  05/06/66'"]);
    check_all(&times, "'%-0^10x'",  &["'0005/06/34'", "'0005/06/66'"]);
    check_all(&times, "'%0_#10x'",  &["'  05/06/34'", "'  05/06/66'"]);
    check_all(&times, "'%_010x'",   &["'0005/06/34'", "'0005/06/66'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_iso_8601() {
    let times = [
        MockTime { year: 1234,  month: 5, day: 6, ..Default::default() },
        MockTime { year: -1234, month: 5, day: 6, ..Default::default() },
    ];

    check_all(&times, "'%F'",       &["'1234-05-06'",   "'-1234-05-06'"]);
    check_all(&times, "'%1F'",      &["'1234-05-06'",   "'-1234-05-06'"]);
    check_all(&times, "'%12F'",     &["'  1234-05-06'", "' -1234-05-06'"]);
    check_all(&times, "'%-^_#12F'", &["'  1234-05-06'", "' -1234-05-06'"]);
    check_all(&times, "'%-0^12F'",  &["'001234-05-06'", "'0-1234-05-06'"]);
    check_all(&times, "'%0_#12F'",  &["'  1234-05-06'", "' -1234-05-06'"]);
    check_all(&times, "'%_012F'",   &["'001234-05-06'", "'0-1234-05-06'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_vms_date() {
    let times = [
        MockTime { year: 1234,  month: 7, day: 6, ..Default::default() },
        MockTime { year: -1234, month: 7, day: 6, ..Default::default() },
    ];

    check_all(&times, "'%v'",       &["' 6-JUL-1234'",   "' 6-JUL--1234'"]);
    check_all(&times, "'%1v'",      &["' 6-JUL-1234'",   "' 6-JUL--1234'"]);
    check_all(&times, "'%13v'",     &["'   6-JUL-1234'", "'  6-JUL--1234'"]);
    check_all(&times, "'%-^_#13v'", &["'   6-JUL-1234'", "'  6-JUL--1234'"]);
    check_all(&times, "'%-0^13v'",  &["'00 6-JUL-1234'", "'0 6-JUL--1234'"]);
    check_all(&times, "'%0_#13v'",  &["'   6-JUL-1234'", "'  6-JUL--1234'"]);
    check_all(&times, "'%_013v'",   &["'00 6-JUL-1234'", "'0 6-JUL--1234'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_time_12h() {
    let times = [
        MockTime { hour: 11, minute: 2, second: 3, ..Default::default() },
        MockTime { hour: 12, minute: 2, second: 3, ..Default::default() },
    ];

    check_all(&times, "'%r'",       &["'11:02:03 AM'",   "'12:02:03 PM'"]);
    check_all(&times, "'%1r'",      &["'11:02:03 AM'",   "'12:02:03 PM'"]);
    check_all(&times, "'%13r'",     &["'  11:02:03 AM'", "'  12:02:03 PM'"]);
    check_all(&times, "'%-^_#13r'", &["'  11:02:03 AM'", "'  12:02:03 PM'"]);
    check_all(&times, "'%-0^13r'",  &["'0011:02:03 AM'", "'0012:02:03 PM'"]);
    check_all(&times, "'%0_#13r'",  &["'  11:02:03 AM'", "'  12:02:03 PM'"]);
    check_all(&times, "'%_013r'",   &["'0011:02:03 AM'", "'0012:02:03 PM'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_hour_minute_24h() {
    let times = [MockTime { hour: 13, minute: 2, ..Default::default() }];

    check_all(&times, "'%R'",      &["'13:02'"]);
    check_all(&times, "'%1R'",     &["'13:02'"]);
    check_all(&times, "'%7R'",     &["'  13:02'"]);
    check_all(&times, "'%-^_#7R'", &["'  13:02'"]);
    check_all(&times, "'%-0^7R'",  &["'0013:02'"]);
    check_all(&times, "'%0_#7R'",  &["'  13:02'"]);
    check_all(&times, "'%_07R'",   &["'0013:02'"]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_time_24h() {
    let times = [MockTime { hour: 13, minute: 2, second: 3, ..Default::default() }];

    check_all(&times, "'%T'",       &["'13:02:03'"]);
    check_all(&times, "'%1T'",      &["'13:02:03'"]);
    check_all(&times, "'%10T'",     &["'  13:02:03'"]);
    check_all(&times, "'%-^_#10T'", &["'  13:02:03'"]);
    check_all(&times, "'%-0^10T'",  &["'0013:02:03'"]);
    check_all(&times, "'%0_#10T'",  &["'  13:02:03'"]);
    check_all(&times, "'%_010T'",   &["'0013:02:03'"]);

    check_all(&times, "'%X'",       &["'13:02:03'"]);
    check_all(&times, "'%1X'",      &["'13:02:03'"]);
    check_all(&times, "'%10X'",     &["'  13:02:03'"]);
    check_all(&times, "'%-^_#10X'", &["'  13:02:03'"]);
    check_all(&times, "'%-0^10X'",  &["'0013:02:03'"]);
    check_all(&times, "'%0_#10X'",  &["'  13:02:03'"]);
    check_all(&times, "'%_010X'",   &["'0013:02:03'"]);
}

#[test]
fn test_format_invalid() {
    let time = MockTime::default();

    for format in ["%", "%-4", "%-", "%-_"] {
        let err = get_format_err(&time, format);
        assert!(matches!(err, Error::InvalidFormatString));
    }
}

#[test]
fn test_format_literal() {
    let time = MockTime::default();

    check_format(&time, "% ", "% ");
    check_format(&time, "%-4 ", "%-4 ");
    check_format(&time, "%- ", "%- ");
    check_format(&time, "%-_ ", "%-_ ");

    check_format(&time, "'%:'", "'%:'");
    check_format(&time, "'%::'", "'%::'");
    check_format(&time, "'%:::'", "'%:::'");
    check_format(&time, "'%:::m'", "'%:::m'");
    check_format(&time, "'%::::z'", "'%::::z'");
}

#[test]
fn test_format_with_modifiers() {
    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    check_format(&time, "%EY, %Oy, %EE, %OO", "1970, 70, %EE, %OO");
}

#[test]
fn test_format_large_width() {
    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    check_format(&time, "%-100000000m", "1");
    check_format(&time, "%2147483648m", "%2147483648m");

    let err = get_format_err(&time, "%2147483647m");
    assert!(matches!(err, Error::WriteZero));
}

#[cfg(feature = "alloc")]
#[test]
fn test_format_formatted_string_too_large() {
    use alloc::vec::Vec;

    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    let mut buf = Vec::new();
    let result = TimeFormatter::new(&time, "%4718593m").fmt(&mut buf);

    assert_eq!(buf.len(), 4_718_592);
    assert!(matches!(result, Err(Error::FormattedStringTooLarge)));
}

#[test]
fn test_format_small_buffer() {
    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    let mut buf = [0u8; 3];
    let result = TimeFormatter::new(&time, "%Y").fmt(&mut &mut buf[..]);
    assert!(matches!(result, Err(Error::WriteZero)));
}

#[test]
fn test_format_empty() {
    let time = MockTime::default();

    check_format(&time, "", "");
}
