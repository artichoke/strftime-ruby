use crate::format::TimeFormatter;
use crate::{Error, Time};

include!("mock.rs.in");

fn check_format(time: &MockTime<'_>, format: &str, expected: Result<&str, Error>) {
    const SIZE: usize = 100;
    let mut buf = [0u8; SIZE];
    let mut cursor = &mut buf[..];

    let result = TimeFormatter::new(time, format).fmt(&mut cursor);
    let written = SIZE - cursor.len();
    let data = core::str::from_utf8(&buf[..written]).unwrap();

    assert_eq!(result.map(|_| data), expected);
}

fn check_all(times: &[MockTime<'_>], format: &str, all_expected: &[Result<&str, Error>]) {
    assert_eq!(times.len(), all_expected.len());
    for (time, &expected) in times.iter().zip(all_expected) {
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

    check_all(&times, "'%Y'",    &[Ok("'-1111'"), Ok("'-0011'"), Ok("'0001'"),  Ok("'1111'")]);
    check_all(&times, "'%1Y'",   &[Ok("'-1111'"), Ok("'-11'"),   Ok("'1'"),     Ok("'1111'")]);
    check_all(&times, "'%4Y'",   &[Ok("'-1111'"), Ok("'-011'"),  Ok("'0001'"),  Ok("'1111'")]);
    check_all(&times, "'%-_5Y'", &[Ok("'-1111'"), Ok("'-11'"),   Ok("'1'"),     Ok("'1111'")]);
    check_all(&times, "'%-05Y'", &[Ok("'-1111'"), Ok("'-11'"),   Ok("'1'"),     Ok("'1111'")]);
    check_all(&times, "'%0_5Y'", &[Ok("'-1111'"), Ok("'  -11'"), Ok("'    1'"), Ok("' 1111'")]);
    check_all(&times, "'%_05Y'", &[Ok("'-1111'"), Ok("'-0011'"), Ok("'00001'"), Ok("'01111'")]);
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

    check_all(&times, "'%C'",    &[Ok("'-12'"),  Ok("'-1'"),   Ok("'00'"),   Ok("'11'")]);
    check_all(&times, "'%1C'",   &[Ok("'-12'"),  Ok("'-1'"),   Ok("'0'"),    Ok("'11'")]);
    check_all(&times, "'%4C'",   &[Ok("'-012'"), Ok("'-001'"), Ok("'0000'"), Ok("'0011'")]);
    check_all(&times, "'%-_4C'", &[Ok("'-12'"),  Ok("'-1'"),   Ok("'0'"),    Ok("'11'")]);
    check_all(&times, "'%-04C'", &[Ok("'-12'"),  Ok("'-1'"),   Ok("'0'"),    Ok("'11'")]);
    check_all(&times, "'%0_4C'", &[Ok("' -12'"), Ok("'  -1'"), Ok("'   0'"), Ok("'  11'")]);
    check_all(&times, "'%_04C'", &[Ok("'-012'"), Ok("'-001'"), Ok("'0000'"), Ok("'0011'")]);
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

    check_all(&times, "'%y'",   &[Ok("'89'"),   Ok("'89'"),   Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1y'",  &[Ok("'89'"),   Ok("'89'"),   Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4y'",  &[Ok("'0089'"), Ok("'0089'"), Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_y'", &[Ok("'89'"),   Ok("'89'"),   Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0y'", &[Ok("'89'"),   Ok("'89'"),   Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_y'", &[Ok("'89'"),   Ok("'89'"),   Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0y'", &[Ok("'89'"),   Ok("'89'"),   Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_month() {
    let times = [
        MockTime { month: 1,  ..Default::default() },
        MockTime { month: 11, ..Default::default() },
    ];

    check_all(&times, "'%m'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1m'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4m'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_m'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0m'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_m'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0m'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_name() {
    let times = [MockTime { month: 7, ..Default::default() }];

    check_all(&times, "'%B'",      &[Ok("'July'")]);
    check_all(&times, "'%1B'",     &[Ok("'July'")]);
    check_all(&times, "'%6B'",     &[Ok("'  July'")]);
    check_all(&times, "'%-_#^6B'", &[Ok("'JULY'")]);
    check_all(&times, "'%-0^6B'",  &[Ok("'JULY'")]);
    check_all(&times, "'%0_#6B'",  &[Ok("'  JULY'")]);
    check_all(&times, "'%_06B'",   &[Ok("'00July'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_name_abbr() {
    let times = [MockTime { month: 7, ..Default::default() }];

    check_all(&times, "'%b'",      &[Ok("'Jul'")]);
    check_all(&times, "'%1b'",     &[Ok("'Jul'")]);
    check_all(&times, "'%6b'",     &[Ok("'   Jul'")]);
    check_all(&times, "'%-_#^6b'", &[Ok("'JUL'")]);
    check_all(&times, "'%-0^6b'",  &[Ok("'JUL'")]);
    check_all(&times, "'%0_#6b'",  &[Ok("'   JUL'")]);
    check_all(&times, "'%_06b'",   &[Ok("'000Jul'")]);

    check_all(&times, "'%h'",      &[Ok("'Jul'")]);
    check_all(&times, "'%1h'",     &[Ok("'Jul'")]);
    check_all(&times, "'%6h'",     &[Ok("'   Jul'")]);
    check_all(&times, "'%-_#^6h'", &[Ok("'JUL'")]);
    check_all(&times, "'%-0^6h'",  &[Ok("'JUL'")]);
    check_all(&times, "'%0_#6h'",  &[Ok("'   JUL'")]);
    check_all(&times, "'%_06h'",   &[Ok("'000Jul'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_day_zero() {
    let times = [
        MockTime { day: 1,  ..Default::default() },
        MockTime { day: 11, ..Default::default() },
    ];

    check_all(&times, "'%d'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1d'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4d'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_d'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0d'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_d'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0d'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_month_day_space() {
    let times = [
        MockTime { day: 1,  ..Default::default() },
        MockTime { day: 11, ..Default::default() },
    ];

    check_all(&times, "'%e'",   &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%1e'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4e'",  &[Ok("'   1'"), Ok("'  11'")]);
    check_all(&times, "'%-_e'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0e'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_e'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0e'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_year_day() {
    let times = [
        MockTime { day_of_year: 1,   ..Default::default() },
        MockTime { day_of_year: 300, ..Default::default() },
    ];

    check_all(&times, "'%j'",   &[Ok("'001'"),  Ok("'300'")]);
    check_all(&times, "'%1j'",  &[Ok("'1'"),    Ok("'300'")]);
    check_all(&times, "'%4j'",  &[Ok("'0001'"), Ok("'0300'")]);
    check_all(&times, "'%-_j'", &[Ok("'1'"),    Ok("'300'")]);
    check_all(&times, "'%-0j'", &[Ok("'1'"),    Ok("'300'")]);
    check_all(&times, "'%0_j'", &[Ok("'  1'"),  Ok("'300'")]);
    check_all(&times, "'%_0j'", &[Ok("'001'"),  Ok("'300'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_24h_zero() {
    let times = [
        MockTime { hour: 1,  ..Default::default() },
        MockTime { hour: 11, ..Default::default() },
    ];

    check_all(&times, "'%H'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1H'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4H'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_H'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0H'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_H'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0H'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_24h_space() {
    let times = [
        MockTime { hour: 1,  ..Default::default() },
        MockTime { hour: 11, ..Default::default() },
    ];

    check_all(&times, "'%k'",   &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%1k'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4k'",  &[Ok("'   1'"), Ok("'  11'")]);
    check_all(&times, "'%-_k'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0k'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_k'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0k'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_12h_zero() {
    let times = [
        MockTime { hour: 13, ..Default::default() },
        MockTime { hour: 0,  ..Default::default() },
    ];

    check_all(&times, "'%I'",   &[Ok("'01'"),   Ok("'12'")]);
    check_all(&times, "'%1I'",  &[Ok("'1'"),    Ok("'12'")]);
    check_all(&times, "'%4I'",  &[Ok("'0001'"), Ok("'0012'")]);
    check_all(&times, "'%-_I'", &[Ok("'1'"),    Ok("'12'")]);
    check_all(&times, "'%-0I'", &[Ok("'1'"),    Ok("'12'")]);
    check_all(&times, "'%0_I'", &[Ok("' 1'"),   Ok("'12'")]);
    check_all(&times, "'%_0I'", &[Ok("'01'"),   Ok("'12'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_hour_12h_space() {
    let times = [
        MockTime { hour: 13, ..Default::default() },
        MockTime { hour: 0,  ..Default::default() },
    ];

    check_all(&times, "'%l'",   &[Ok("' 1'"),   Ok("'12'")]);
    check_all(&times, "'%1l'",  &[Ok("'1'"),    Ok("'12'")]);
    check_all(&times, "'%4l'",  &[Ok("'   1'"), Ok("'  12'")]);
    check_all(&times, "'%-_l'", &[Ok("'1'"),    Ok("'12'")]);
    check_all(&times, "'%-0l'", &[Ok("'1'"),    Ok("'12'")]);
    check_all(&times, "'%0_l'", &[Ok("' 1'"),   Ok("'12'")]);
    check_all(&times, "'%_0l'", &[Ok("'01'"),   Ok("'12'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_meridian_lower() {
    let times = [
        MockTime { hour: 11, ..Default::default() },
        MockTime { hour: 12, ..Default::default() },
    ];

    check_all(&times, "'%P'",      &[Ok("'am'"),   Ok("'pm'")]);
    check_all(&times, "'%1P'",     &[Ok("'am'"),   Ok("'pm'")]);
    check_all(&times, "'%4P'",     &[Ok("'  am'"), Ok("'  pm'")]);
    check_all(&times, "'%-_#^4P'", &[Ok("'AM'"),   Ok("'PM'")]);
    check_all(&times, "'%-0^4P'",  &[Ok("'AM'"),   Ok("'PM'")]);
    check_all(&times, "'%0_#4P'",  &[Ok("'  AM'"), Ok("'  PM'")]);
    check_all(&times, "'%_04P'",   &[Ok("'00am'"), Ok("'00pm'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_meridian_upper() {
    let times = [
        MockTime { hour: 11, ..Default::default() },
        MockTime { hour: 12, ..Default::default() },
    ];

    check_all(&times, "'%p'",      &[Ok("'AM'"),   Ok("'PM'")]);
    check_all(&times, "'%1p'",     &[Ok("'AM'"),   Ok("'PM'")]);
    check_all(&times, "'%4p'",     &[Ok("'  AM'"), Ok("'  PM'")]);
    check_all(&times, "'%-_#^4p'", &[Ok("'am'"),   Ok("'pm'")]);
    check_all(&times, "'%-0^4p'",  &[Ok("'AM'"),   Ok("'PM'")]);
    check_all(&times, "'%0_#4p'",  &[Ok("'  am'"), Ok("'  pm'")]);
    check_all(&times, "'%_04p'",   &[Ok("'00AM'"), Ok("'00PM'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_minute() {
    let times = [
        MockTime { minute: 1,  ..Default::default() },
        MockTime { minute: 11, ..Default::default() },
    ];

    check_all(&times, "'%M'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1M'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4M'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_M'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0M'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_M'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0M'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_second() {
    let times = [
        MockTime { second: 1,  ..Default::default() },
        MockTime { second: 11, ..Default::default() },
    ];

    check_all(&times, "'%S'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1S'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4S'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_S'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0S'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_S'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0S'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_milli_second() {
    let times = [
        MockTime { nanoseconds: 1,           ..Default::default() },
        MockTime { nanoseconds: 123_456_789, ..Default::default() },
    ];

    check_all(&times, "'%L'",    &[Ok("'000'"),          Ok("'123'")]);
    check_all(&times, "'%00L'",  &[Ok("'000'"),          Ok("'123'")]);
    check_all(&times, "'%0L'",   &[Ok("'000'"),          Ok("'123'")]);
    check_all(&times, "'%1L'",   &[Ok("'0'"),            Ok("'1'")]);
    check_all(&times, "'%2L'",   &[Ok("'00'"),           Ok("'12'")]);
    check_all(&times, "'%3L'",   &[Ok("'000'"),          Ok("'123'")]);
    check_all(&times, "'%4L'",   &[Ok("'0000'"),         Ok("'1234'")]);
    check_all(&times, "'%5L'",   &[Ok("'00000'"),        Ok("'12345'")]);
    check_all(&times, "'%6L'",   &[Ok("'000000'"),       Ok("'123456'")]);
    check_all(&times, "'%7L'",   &[Ok("'0000000'"),      Ok("'1234567'")]);
    check_all(&times, "'%8L'",   &[Ok("'00000000'"),     Ok("'12345678'")]);
    check_all(&times, "'%9L'",   &[Ok("'000000001'"),    Ok("'123456789'")]);
    check_all(&times, "'%12L'",  &[Ok("'000000001000'"), Ok("'123456789000'")]);
    check_all(&times, "'%-12L'", &[Ok("'000000001000'"), Ok("'123456789000'")]);
    check_all(&times, "'%_12L'", &[Ok("'000000001000'"), Ok("'123456789000'")]);
    check_all(&times, "'%012L'", &[Ok("'000000001000'"), Ok("'123456789000'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_fractional_second() {
    let times = [
        MockTime { nanoseconds: 1,           ..Default::default() },
        MockTime { nanoseconds: 123_456_789, ..Default::default() },
    ];

    check_all(&times, "'%N'",    &[Ok("'000000001'"),    Ok("'123456789'")]);
    check_all(&times, "'%00N'",  &[Ok("'000000001'"),    Ok("'123456789'")]);
    check_all(&times, "'%0N'",   &[Ok("'000000001'"),    Ok("'123456789'")]);
    check_all(&times, "'%1N'",   &[Ok("'0'"),            Ok("'1'")]);
    check_all(&times, "'%2N'",   &[Ok("'00'"),           Ok("'12'")]);
    check_all(&times, "'%3N'",   &[Ok("'000'"),          Ok("'123'")]);
    check_all(&times, "'%4N'",   &[Ok("'0000'"),         Ok("'1234'")]);
    check_all(&times, "'%5N'",   &[Ok("'00000'"),        Ok("'12345'")]);
    check_all(&times, "'%6N'",   &[Ok("'000000'"),       Ok("'123456'")]);
    check_all(&times, "'%7N'",   &[Ok("'0000000'"),      Ok("'1234567'")]);
    check_all(&times, "'%8N'",   &[Ok("'00000000'"),     Ok("'12345678'")]);
    check_all(&times, "'%9N'",   &[Ok("'000000001'"),    Ok("'123456789'")]);
    check_all(&times, "'%12N'",  &[Ok("'000000001000'"), Ok("'123456789000'")]);
    check_all(&times, "'%-12N'", &[Ok("'000000001000'"), Ok("'123456789000'")]);
    check_all(&times, "'%_12N'", &[Ok("'000000001000'"), Ok("'123456789000'")]);
    check_all(&times, "'%012N'", &[Ok("'000000001000'"), Ok("'123456789000'")]);
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

    check_all(&times, "'%z'",    &[Ok("'+0000'"),  Ok("'+0000'"),  Ok("'+0009'"),  Ok("'+0100'")]);
    check_all(&times, "'%1z'",   &[Ok("'+0000'"),  Ok("'+0000'"),  Ok("'+0009'"),  Ok("'+0100'")]);
    check_all(&times, "'%6z'",   &[Ok("'+00000'"), Ok("'+00000'"), Ok("'+00009'"), Ok("'+00100'")]);
    check_all(&times, "'%-6z'",  &[Ok("'-00000'"), Ok("'+00000'"), Ok("'+00009'"), Ok("'+00100'")]);
    check_all(&times, "'%-_6z'", &[Ok("'  -000'"), Ok("'  +000'"), Ok("'  +009'"), Ok("'  +100'")]);
    check_all(&times, "'%-06z'", &[Ok("'-00000'"), Ok("'+00000'"), Ok("'+00009'"), Ok("'+00100'")]);
    check_all(&times, "'%0_6z'", &[Ok("'  +000'"), Ok("'  +000'"), Ok("'  +009'"), Ok("'  +100'")]);
    check_all(&times, "'%_06z'", &[Ok("'+00000'"), Ok("'+00000'"), Ok("'+00009'"), Ok("'+00100'")]);
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

    check_all(&times, "'%:z'",    &[Ok("'+00:00'"),  Ok("'+00:00'"),  Ok("'+00:09'"),  Ok("'+01:00'")]);
    check_all(&times, "'%1:z'",   &[Ok("'+00:00'"),  Ok("'+00:00'"),  Ok("'+00:09'"),  Ok("'+01:00'")]);
    check_all(&times, "'%7:z'",   &[Ok("'+000:00'"), Ok("'+000:00'"), Ok("'+000:09'"), Ok("'+001:00'")]);
    check_all(&times, "'%-7:z'",  &[Ok("'-000:00'"), Ok("'+000:00'"), Ok("'+000:09'"), Ok("'+001:00'")]);
    check_all(&times, "'%-_7:z'", &[Ok("'  -0:00'"), Ok("'  +0:00'"), Ok("'  +0:09'"), Ok("'  +1:00'")]);
    check_all(&times, "'%-07:z'", &[Ok("'-000:00'"), Ok("'+000:00'"), Ok("'+000:09'"), Ok("'+001:00'")]);
    check_all(&times, "'%0_7:z'", &[Ok("'  +0:00'"), Ok("'  +0:00'"), Ok("'  +0:09'"), Ok("'  +1:00'")]);
    check_all(&times, "'%_07:z'", &[Ok("'+000:00'"), Ok("'+000:00'"), Ok("'+000:09'"), Ok("'+001:00'")]);
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

    check_all(&times, "'%::z'",     &[Ok("'+00:00:00'"),  Ok("'+00:00:00'"),  Ok("'+00:09:21'"),  Ok("'+01:00:00'")]);
    check_all(&times, "'%1::z'",    &[Ok("'+00:00:00'"),  Ok("'+00:00:00'"),  Ok("'+00:09:21'"),  Ok("'+01:00:00'")]);
    check_all(&times, "'%10::z'",   &[Ok("'+000:00:00'"), Ok("'+000:00:00'"), Ok("'+000:09:21'"), Ok("'+001:00:00'")]);
    check_all(&times, "'%-10::z'",  &[Ok("'-000:00:00'"), Ok("'+000:00:00'"), Ok("'+000:09:21'"), Ok("'+001:00:00'")]);
    check_all(&times, "'%-_10::z'", &[Ok("'  -0:00:00'"), Ok("'  +0:00:00'"), Ok("'  +0:09:21'"), Ok("'  +1:00:00'")]);
    check_all(&times, "'%-010::z'", &[Ok("'-000:00:00'"), Ok("'+000:00:00'"), Ok("'+000:09:21'"), Ok("'+001:00:00'")]);
    check_all(&times, "'%0_10::z'", &[Ok("'  +0:00:00'"), Ok("'  +0:00:00'"), Ok("'  +0:09:21'"), Ok("'  +1:00:00'")]);
    check_all(&times, "'%_010::z'", &[Ok("'+000:00:00'"), Ok("'+000:00:00'"), Ok("'+000:09:21'"), Ok("'+001:00:00'")]);
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

    check_all(&times, "'%:::z'",     &[Ok("'+00'"),        Ok("'+00'"),        Ok("'+00:09'"),     Ok("'+00:09:21'"),  Ok("'+01'")]);
    check_all(&times, "'%1:::z'",    &[Ok("'+00'"),        Ok("'+00'"),        Ok("'+00:09'"),     Ok("'+00:09:21'"),  Ok("'+01'")]);
    check_all(&times, "'%10:::z'",   &[Ok("'+000000000'"), Ok("'+000000000'"), Ok("'+000000:09'"), Ok("'+000:09:21'"), Ok("'+000000001'")]);
    check_all(&times, "'%-10:::z'",  &[Ok("'-000000000'"), Ok("'+000000000'"), Ok("'+000000:09'"), Ok("'+000:09:21'"), Ok("'+000000001'")]);
    check_all(&times, "'%-_10:::z'", &[Ok("'        -0'"), Ok("'        +0'"), Ok("'     +0:09'"), Ok("'  +0:09:21'"), Ok("'        +1'")]);
    check_all(&times, "'%-010:::z'", &[Ok("'-000000000'"), Ok("'+000000000'"), Ok("'+000000:09'"), Ok("'+000:09:21'"), Ok("'+000000001'")]);
    check_all(&times, "'%0_10:::z'", &[Ok("'        +0'"), Ok("'        +0'"), Ok("'     +0:09'"), Ok("'  +0:09:21'"), Ok("'        +1'")]);
    check_all(&times, "'%_010:::z'", &[Ok("'+000000000'"), Ok("'+000000000'"), Ok("'+000000:09'"), Ok("'+000:09:21'"), Ok("'+000000001'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_time_zone_name() {
    let times = [
        MockTime { time_zone: "",      ..Default::default() },
        MockTime { time_zone: "UTC",   ..Default::default() },
        MockTime { time_zone: "+0000", ..Default::default() },
    ];

    check_all(&times, "'%Z'",      &[Ok("''"), Ok("'UTC'")   , Ok("'+0000'")]);
    check_all(&times, "'%1Z'",     &[Ok("''"), Ok("'UTC'")   , Ok("'+0000'")]);
    check_all(&times, "'%6Z'",     &[Ok("''"), Ok("'   UTC'"), Ok("' +0000'")]);
    check_all(&times, "'%-_#^6Z'", &[Ok("''"), Ok("'utc'")   , Ok("'+0000'")]);
    check_all(&times, "'%-0^6Z'",  &[Ok("''"), Ok("'UTC'")   , Ok("'+0000'")]);
    check_all(&times, "'%0_#6Z'",  &[Ok("''"), Ok("'   utc'"), Ok("' +0000'")]);
    check_all(&times, "'%_06Z'",   &[Ok("''"), Ok("'000UTC'"), Ok("'0+0000'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_name() {
    let times = [MockTime { day_of_week: 1, ..Default::default() }];

    check_all(&times, "'%A'",      &[Ok("'Monday'")]);
    check_all(&times, "'%1A'",     &[Ok("'Monday'")]);
    check_all(&times, "'%8A'",     &[Ok("'  Monday'")]);
    check_all(&times, "'%-_#^8A'", &[Ok("'MONDAY'")]);
    check_all(&times, "'%-0^8A'",  &[Ok("'MONDAY'")]);
    check_all(&times, "'%0_#8A'",  &[Ok("'  MONDAY'")]);
    check_all(&times, "'%_08A'",   &[Ok("'00Monday'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_name_abbr() {
    let times = [MockTime { day_of_week: 1, ..Default::default() }];

    check_all(&times, "'%a'",      &[Ok("'Mon'")]);
    check_all(&times, "'%1a'",     &[Ok("'Mon'")]);
    check_all(&times, "'%8a'",     &[Ok("'     Mon'")]);
    check_all(&times, "'%-_#^8a'", &[Ok("'MON'")]);
    check_all(&times, "'%-0^8a'",  &[Ok("'MON'")]);
    check_all(&times, "'%0_#8a'",  &[Ok("'     MON'")]);
    check_all(&times, "'%_08a'",   &[Ok("'00000Mon'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_from_1() {
    let times = [MockTime { day_of_week: 7, ..Default::default() }];

    check_all(&times, "'%u'",   &[Ok("'7'")]);
    check_all(&times, "'%1u'",  &[Ok("'7'")]);
    check_all(&times, "'%4u'",  &[Ok("'0007'")]);
    check_all(&times, "'%-_4u'", &[Ok("'7'")]);
    check_all(&times, "'%-04u'", &[Ok("'7'")]);
    check_all(&times, "'%0_4u'", &[Ok("'   7'")]);
    check_all(&times, "'%_04u'", &[Ok("'0007'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_day_from_0() {
    let times = [MockTime { day_of_week: 0, ..Default::default() }];

    check_all(&times, "'%w'",   &[Ok("'0'")]);
    check_all(&times, "'%1w'",  &[Ok("'0'")]);
    check_all(&times, "'%4w'",  &[Ok("'0000'")]);
    check_all(&times, "'%-_4w'", &[Ok("'0'")]);
    check_all(&times, "'%-04w'", &[Ok("'0'")]);
    check_all(&times, "'%0_4w'", &[Ok("'   0'")]);
    check_all(&times, "'%_04w'", &[Ok("'0000'")]);
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

    check_all(&times, "'%G'",    &[Ok("'-1111'"), Ok("'-0011'"), Ok("'0001'"),  Ok("'1111'")]);
    check_all(&times, "'%1G'",   &[Ok("'-1111'"), Ok("'-11'"),   Ok("'1'"),     Ok("'1111'")]);
    check_all(&times, "'%4G'",   &[Ok("'-1111'"), Ok("'-011'"),  Ok("'0001'"),  Ok("'1111'")]);
    check_all(&times, "'%-_5G'", &[Ok("'-1111'"), Ok("'-11'"),   Ok("'1'"),     Ok("'1111'")]);
    check_all(&times, "'%-05G'", &[Ok("'-1111'"), Ok("'-11'"),   Ok("'1'"),     Ok("'1111'")]);
    check_all(&times, "'%0_5G'", &[Ok("'-1111'"), Ok("'  -11'"), Ok("'    1'"), Ok("' 1111'")]);
    check_all(&times, "'%_05G'", &[Ok("'-1111'"), Ok("'-0011'"), Ok("'00001'"), Ok("'01111'")]);
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

    check_all(&times, "'%g'",   &[Ok("'89'"),   Ok("'89'"),   Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1g'",  &[Ok("'89'"),   Ok("'89'"),   Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4g'",  &[Ok("'0089'"), Ok("'0089'"), Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_g'", &[Ok("'89'"),   Ok("'89'"),   Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0g'", &[Ok("'89'"),   Ok("'89'"),   Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_g'", &[Ok("'89'"),   Ok("'89'"),   Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0g'", &[Ok("'89'"),   Ok("'89'"),   Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_number_iso_8601() {
    let times = [
        MockTime { year: 2000, day_of_year: 7,  ..Default::default() },
        MockTime { year: 2000, day_of_year: 80, ..Default::default() },
    ];

    check_all(&times, "'%V'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1V'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4V'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_V'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0V'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_V'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0V'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_number_from_sunday() {
    let times = [
        MockTime { year: 2000, day_of_year: 7,  ..Default::default() },
        MockTime { year: 2000, day_of_year: 77, ..Default::default() },
    ];

    check_all(&times, "'%U'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1U'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4U'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_U'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0U'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_U'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0U'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_week_number_from_monday() {
    let times = [
        MockTime { year: 2000, day_of_year: 7,  ..Default::default() },
        MockTime { year: 2000, day_of_year: 77, ..Default::default() },
    ];

    check_all(&times, "'%W'",   &[Ok("'01'"),   Ok("'11'")]);
    check_all(&times, "'%1W'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4W'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_W'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0W'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_W'", &[Ok("' 1'"),   Ok("'11'")]);
    check_all(&times, "'%_0W'", &[Ok("'01'"),   Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_seconds_since_epoch() {
    let times = [
        MockTime { to_int: 1,  ..Default::default() },
        MockTime { to_int: 11, ..Default::default() },
    ];

    check_all(&times, "'%s'",   &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%1s'",  &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%4s'",  &[Ok("'0001'"), Ok("'0011'")]);
    check_all(&times, "'%-_s'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%-0s'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%0_s'", &[Ok("'1'"),    Ok("'11'")]);
    check_all(&times, "'%_0s'", &[Ok("'1'"),    Ok("'11'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_newline() {
    let times = [MockTime::default()];

    check_all(&times, "'%n'",      &[Ok("'\n'")]);
    check_all(&times, "'%1n'",     &[Ok("'\n'")]);
    check_all(&times, "'%6n'",     &[Ok("'     \n'")]);
    check_all(&times, "'%-_#^6n'", &[Ok("'\n'")]);
    check_all(&times, "'%-0^6n'",  &[Ok("'\n'")]);
    check_all(&times, "'%0_#6n'",  &[Ok("'     \n'")]);
    check_all(&times, "'%_06n'",   &[Ok("'00000\n'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_tabulation() {
    let times = [MockTime::default()];

    check_all(&times, "'%t'",      &[Ok("'\t'")]);
    check_all(&times, "'%1t'",     &[Ok("'\t'")]);
    check_all(&times, "'%6t'",     &[Ok("'     \t'")]);
    check_all(&times, "'%-_#^6t'", &[Ok("'\t'")]);
    check_all(&times, "'%-0^6t'",  &[Ok("'\t'")]);
    check_all(&times, "'%0_#6t'",  &[Ok("'     \t'")]);
    check_all(&times, "'%_06t'",   &[Ok("'00000\t'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_percent() {
    let times = [MockTime::default()];

    check_all(&times, "'%%'",      &[Ok("'%'")]);
    check_all(&times, "'%1%'",     &[Ok("'%'")]);
    check_all(&times, "'%6%'",     &[Ok("'     %'")]);
    check_all(&times, "'%-_#^6%'", &[Ok("'%'")]);
    check_all(&times, "'%-0^6%'",  &[Ok("'%'")]);
    check_all(&times, "'%0_#6%'",  &[Ok("'     %'")]);
    check_all(&times, "'%_06%'",   &[Ok("'00000%'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_date_time() {
    let times = [
        MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, ""),
        MockTime::new(-1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, ""),
    ];

    check_all(&times, "'%c'",       &[Ok("'Thu Jan  1 00:00:00 1970'"),       Ok("'Thu Jan  1 00:00:00 -1970'")]);
    check_all(&times, "'%1c'",      &[Ok("'Thu Jan  1 00:00:00 1970'"),       Ok("'Thu Jan  1 00:00:00 -1970'")]);
    check_all(&times, "'%30c'",     &[Ok("'      Thu Jan  1 00:00:00 1970'"), Ok("'     Thu Jan  1 00:00:00 -1970'")]);
    check_all(&times, "'%-^_#30c'", &[Ok("'      THU JAN  1 00:00:00 1970'"), Ok("'     THU JAN  1 00:00:00 -1970'")]);
    check_all(&times, "'%-0^30c'",  &[Ok("'000000THU JAN  1 00:00:00 1970'"), Ok("'00000THU JAN  1 00:00:00 -1970'")]);
    check_all(&times, "'%0_#30c'",  &[Ok("'      Thu Jan  1 00:00:00 1970'"), Ok("'     Thu Jan  1 00:00:00 -1970'")]);
    check_all(&times, "'%_030c'",   &[Ok("'000000Thu Jan  1 00:00:00 1970'"), Ok("'00000Thu Jan  1 00:00:00 -1970'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_date() {
    let times = [
        MockTime { year: 1234,  month: 5, day: 6, ..Default::default() },
        MockTime { year: -1234, month: 5, day: 6, ..Default::default() },
    ];

    check_all(&times, "'%D'",       &[Ok("'05/06/34'"),   Ok("'05/06/66'")]);
    check_all(&times, "'%1D'",      &[Ok("'05/06/34'"),   Ok("'05/06/66'")]);
    check_all(&times, "'%10D'",     &[Ok("'  05/06/34'"), Ok("'  05/06/66'")]);
    check_all(&times, "'%-^_#10D'", &[Ok("'  05/06/34'"), Ok("'  05/06/66'")]);
    check_all(&times, "'%-0^10D'",  &[Ok("'0005/06/34'"), Ok("'0005/06/66'")]);
    check_all(&times, "'%0_#10D'",  &[Ok("'  05/06/34'"), Ok("'  05/06/66'")]);
    check_all(&times, "'%_010D'",   &[Ok("'0005/06/34'"), Ok("'0005/06/66'")]);

    check_all(&times, "'%x'",       &[Ok("'05/06/34'"),   Ok("'05/06/66'")]);
    check_all(&times, "'%1x'",      &[Ok("'05/06/34'"),   Ok("'05/06/66'")]);
    check_all(&times, "'%10x'",     &[Ok("'  05/06/34'"), Ok("'  05/06/66'")]);
    check_all(&times, "'%-^_#10x'", &[Ok("'  05/06/34'"), Ok("'  05/06/66'")]);
    check_all(&times, "'%-0^10x'",  &[Ok("'0005/06/34'"), Ok("'0005/06/66'")]);
    check_all(&times, "'%0_#10x'",  &[Ok("'  05/06/34'"), Ok("'  05/06/66'")]);
    check_all(&times, "'%_010x'",   &[Ok("'0005/06/34'"), Ok("'0005/06/66'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_iso_8601() {
    let times = [
        MockTime { year: 1234,  month: 5, day: 6, ..Default::default() },
        MockTime { year: -1234, month: 5, day: 6, ..Default::default() },
    ];

    check_all(&times, "'%F'",       &[Ok("'1234-05-06'"),   Ok("'-1234-05-06'")]);
    check_all(&times, "'%1F'",      &[Ok("'1234-05-06'"),   Ok("'-1234-05-06'")]);
    check_all(&times, "'%12F'",     &[Ok("'  1234-05-06'"), Ok("' -1234-05-06'")]);
    check_all(&times, "'%-^_#12F'", &[Ok("'  1234-05-06'"), Ok("' -1234-05-06'")]);
    check_all(&times, "'%-0^12F'",  &[Ok("'001234-05-06'"), Ok("'0-1234-05-06'")]);
    check_all(&times, "'%0_#12F'",  &[Ok("'  1234-05-06'"), Ok("' -1234-05-06'")]);
    check_all(&times, "'%_012F'",   &[Ok("'001234-05-06'"), Ok("'0-1234-05-06'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_vms_date() {
    let times = [
        MockTime { year: 1234,  month: 7, day: 6, ..Default::default() },
        MockTime { year: -1234, month: 7, day: 6, ..Default::default() },
    ];

    check_all(&times, "'%v'",       &[Ok("' 6-JUL-1234'"),   Ok("' 6-JUL--1234'")]);
    check_all(&times, "'%1v'",      &[Ok("' 6-JUL-1234'"),   Ok("' 6-JUL--1234'")]);
    check_all(&times, "'%13v'",     &[Ok("'   6-JUL-1234'"), Ok("'  6-JUL--1234'")]);
    check_all(&times, "'%-^_#13v'", &[Ok("'   6-JUL-1234'"), Ok("'  6-JUL--1234'")]);
    check_all(&times, "'%-0^13v'",  &[Ok("'00 6-JUL-1234'"), Ok("'0 6-JUL--1234'")]);
    check_all(&times, "'%0_#13v'",  &[Ok("'   6-JUL-1234'"), Ok("'  6-JUL--1234'")]);
    check_all(&times, "'%_013v'",   &[Ok("'00 6-JUL-1234'"), Ok("'0 6-JUL--1234'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_time_12h() {
    let times = [
        MockTime { hour: 11, minute: 2, second: 3, ..Default::default() },
        MockTime { hour: 12, minute: 2, second: 3, ..Default::default() },
    ];

    check_all(&times, "'%r'",       &[Ok("'11:02:03 AM'"),   Ok("'12:02:03 PM'")]);
    check_all(&times, "'%1r'",      &[Ok("'11:02:03 AM'"),   Ok("'12:02:03 PM'")]);
    check_all(&times, "'%13r'",     &[Ok("'  11:02:03 AM'"), Ok("'  12:02:03 PM'")]);
    check_all(&times, "'%-^_#13r'", &[Ok("'  11:02:03 AM'"), Ok("'  12:02:03 PM'")]);
    check_all(&times, "'%-0^13r'",  &[Ok("'0011:02:03 AM'"), Ok("'0012:02:03 PM'")]);
    check_all(&times, "'%0_#13r'",  &[Ok("'  11:02:03 AM'"), Ok("'  12:02:03 PM'")]);
    check_all(&times, "'%_013r'",   &[Ok("'0011:02:03 AM'"), Ok("'0012:02:03 PM'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_hour_minute_24h() {
    let times = [MockTime { hour: 13, minute: 2, ..Default::default() }];

    check_all(&times, "'%R'",      &[Ok("'13:02'")]);
    check_all(&times, "'%1R'",     &[Ok("'13:02'")]);
    check_all(&times, "'%7R'",     &[Ok("'  13:02'")]);
    check_all(&times, "'%-^_#7R'", &[Ok("'  13:02'")]);
    check_all(&times, "'%-0^7R'",  &[Ok("'0013:02'")]);
    check_all(&times, "'%0_#7R'",  &[Ok("'  13:02'")]);
    check_all(&times, "'%_07R'",   &[Ok("'0013:02'")]);
}

#[test]
#[rustfmt::skip]
fn test_format_combination_time_24h() {
    let times = [MockTime { hour: 13, minute: 2, second: 3, ..Default::default() }];

    check_all(&times, "'%T'",       &[Ok("'13:02:03'")]);
    check_all(&times, "'%1T'",      &[Ok("'13:02:03'")]);
    check_all(&times, "'%10T'",     &[Ok("'  13:02:03'")]);
    check_all(&times, "'%-^_#10T'", &[Ok("'  13:02:03'")]);
    check_all(&times, "'%-0^10T'",  &[Ok("'0013:02:03'")]);
    check_all(&times, "'%0_#10T'",  &[Ok("'  13:02:03'")]);
    check_all(&times, "'%_010T'",   &[Ok("'0013:02:03'")]);

    check_all(&times, "'%X'",       &[Ok("'13:02:03'")]);
    check_all(&times, "'%1X'",      &[Ok("'13:02:03'")]);
    check_all(&times, "'%10X'",     &[Ok("'  13:02:03'")]);
    check_all(&times, "'%-^_#10X'", &[Ok("'  13:02:03'")]);
    check_all(&times, "'%-0^10X'",  &[Ok("'0013:02:03'")]);
    check_all(&times, "'%0_#10X'",  &[Ok("'  13:02:03'")]);
    check_all(&times, "'%_010X'",   &[Ok("'0013:02:03'")]);
}

#[test]
fn test_format_invalid() {
    let time = MockTime::default();

    check_format(&time, "%", Err(Error::InvalidFormatString));
    check_format(&time, "%-4", Err(Error::InvalidFormatString));
    check_format(&time, "%-", Err(Error::InvalidFormatString));
    check_format(&time, "%-_", Err(Error::InvalidFormatString));
}

#[test]
fn test_format_literal() {
    let time = MockTime::default();

    check_format(&time, "%", Err(Error::InvalidFormatString));
    check_format(&time, "%-4", Err(Error::InvalidFormatString));
    check_format(&time, "%-", Err(Error::InvalidFormatString));
    check_format(&time, "%-_", Err(Error::InvalidFormatString));

    check_format(&time, "% ", Ok("% "));
    check_format(&time, "%-4 ", Ok("%-4 "));
    check_format(&time, "%- ", Ok("%- "));
    check_format(&time, "%-_ ", Ok("%-_ "));

    check_format(&time, "'%:'", Ok("'%:'"));
    check_format(&time, "'%::'", Ok("'%::'"));
    check_format(&time, "'%:::'", Ok("'%:::'"));
    check_format(&time, "'%:::m'", Ok("'%:::m'"));
    check_format(&time, "'%::::z'", Ok("'%::::z'"));
}

#[test]
fn test_format_with_modifiers() {
    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    check_format(&time, "%EY, %Oy, %EE, %OO", Ok("1970, 70, %EE, %OO"));
}

#[test]
fn test_format_large_width() {
    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    check_format(&time, "%2147483647m", Err(Error::WriteZero));
    check_format(&time, "%2147483648m", Ok("%2147483648m"));
    check_format(&time, "%-100000000m", Ok("1"));
}

#[cfg(feature = "alloc")]
#[test]
fn test_format_formatted_string_too_large() {
    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    let mut buf = Vec::new();
    let result = TimeFormatter::new(&time, "%4718593m").fmt(&mut buf);

    assert_eq!(buf.len(), 4_718_592);
    assert_eq!(result, Err(Error::FormattedStringTooLarge));
}

#[test]
fn test_format_small_buffer() {
    let time = MockTime::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

    let mut buf = [0u8; 3];
    let result = TimeFormatter::new(&time, "%Y").fmt(&mut &mut buf[..]);
    assert_eq!(result, Err(Error::WriteZero));
}
