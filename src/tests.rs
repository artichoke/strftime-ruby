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

#[test]
fn test_format() {
    #[rustfmt::skip]
    let times = [
        MockTime::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, true, 0, "UTC"),
        MockTime::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, false, 0, "UTC"),
        MockTime::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, false, 0, "+0000"),
        MockTime::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, false, 0, ""),
        MockTime::new(-94, 1, 2, 13, 18, 19, 9876, 2, 2, -65_133_456_662, false, 561, "LMT"),
        MockTime::new(2094, 1, 2, 13, 18, 19, 9876, 6, 2, 3_913_273_099, false, 3600, "CET"),
    ];

    check_format(&times[4], "%", Err(Error::InvalidFormatString));
    check_format(&times[4], "%-4", Err(Error::InvalidFormatString));
    check_format(&times[4], "%-", Err(Error::InvalidFormatString));
    check_format(&times[4], "%-_", Err(Error::InvalidFormatString));

    check_format(&times[4], "% ", Ok("% "));
    check_format(&times[4], "%-4 ", Ok("%-4 "));
    check_format(&times[4], "%- ", Ok("%- "));
    check_format(&times[4], "%-_ ", Ok("%-_ "));

    check_format(&times[4], "'%4Y'", Ok("'-094'"));
    check_format(&times[4], "'%_Y'", Ok("'  -94'"));
    check_format(&times[4], "'%y'", Ok("'06'"));
    check_format(&times[4], "'%C'", Ok("'-1'"));

    check_format(&times[0], "'%z'", Ok("'+0000'"));
    check_format(&times[1], "'%z'", Ok("'+0000'"));
    check_format(&times[4], "'%z'", Ok("'+0009'"));
    check_format(&times[5], "'%z'", Ok("'+0100'"));

    check_format(&times[0], "'%1z'", Ok("'+0000'"));
    check_format(&times[1], "'%1z'", Ok("'+0000'"));
    check_format(&times[4], "'%1z'", Ok("'+0009'"));
    check_format(&times[5], "'%1z'", Ok("'+0100'"));

    check_format(&times[0], "'%-6z'", Ok("'-00000'"));
    check_format(&times[1], "'%-6z'", Ok("'+00000'"));
    check_format(&times[4], "'%-6z'", Ok("'+00009'"));
    check_format(&times[5], "'%-6z'", Ok("'+00100'"));

    check_format(&times[0], "'%_6z'", Ok("'  +000'"));
    check_format(&times[1], "'%_6z'", Ok("'  +000'"));
    check_format(&times[4], "'%_6z'", Ok("'  +009'"));
    check_format(&times[5], "'%_6z'", Ok("'  +100'"));

    check_format(&times[0], "'%:z'", Ok("'+00:00'"));
    check_format(&times[1], "'%:z'", Ok("'+00:00'"));
    check_format(&times[4], "'%:z'", Ok("'+00:09'"));
    check_format(&times[5], "'%:z'", Ok("'+01:00'"));

    check_format(&times[0], "'%1:z'", Ok("'+00:00'"));
    check_format(&times[1], "'%1:z'", Ok("'+00:00'"));
    check_format(&times[4], "'%1:z'", Ok("'+00:09'"));
    check_format(&times[5], "'%1:z'", Ok("'+01:00'"));

    check_format(&times[0], "'%-7:z'", Ok("'-000:00'"));
    check_format(&times[1], "'%-7:z'", Ok("'+000:00'"));
    check_format(&times[4], "'%-7:z'", Ok("'+000:09'"));
    check_format(&times[5], "'%-7:z'", Ok("'+001:00'"));

    check_format(&times[0], "'%_7:z'", Ok("'  +0:00'"));
    check_format(&times[1], "'%_7:z'", Ok("'  +0:00'"));
    check_format(&times[4], "'%_7:z'", Ok("'  +0:09'"));
    check_format(&times[5], "'%_7:z'", Ok("'  +1:00'"));

    check_format(&times[0], "'%::z'", Ok("'+00:00:00'"));
    check_format(&times[1], "'%::z'", Ok("'+00:00:00'"));
    check_format(&times[4], "'%::z'", Ok("'+00:09:21'"));
    check_format(&times[5], "'%::z'", Ok("'+01:00:00'"));

    check_format(&times[0], "'%1::z'", Ok("'+00:00:00'"));
    check_format(&times[1], "'%1::z'", Ok("'+00:00:00'"));
    check_format(&times[4], "'%1::z'", Ok("'+00:09:21'"));
    check_format(&times[5], "'%1::z'", Ok("'+01:00:00'"));

    check_format(&times[0], "'%-10::z'", Ok("'-000:00:00'"));
    check_format(&times[1], "'%-10::z'", Ok("'+000:00:00'"));
    check_format(&times[4], "'%-10::z'", Ok("'+000:09:21'"));
    check_format(&times[5], "'%-10::z'", Ok("'+001:00:00'"));

    check_format(&times[0], "'%_10::z'", Ok("'  +0:00:00'"));
    check_format(&times[1], "'%_10::z'", Ok("'  +0:00:00'"));
    check_format(&times[4], "'%_10::z'", Ok("'  +0:09:21'"));
    check_format(&times[5], "'%_10::z'", Ok("'  +1:00:00'"));

    check_format(&times[0], "'%1:::z'", Ok("'+00'"));
    check_format(&times[1], "'%1:::z'", Ok("'+00'"));
    check_format(&times[4], "'%1:::z'", Ok("'+00:09:21'"));
    check_format(&times[5], "'%1:::z'", Ok("'+01'"));

    check_format(&times[0], "'%8:::z'", Ok("'+0000000'"));
    check_format(&times[1], "'%8:::z'", Ok("'+0000000'"));
    check_format(&times[4], "'%8:::z'", Ok("'+00:09:21'"));
    check_format(&times[5], "'%8:::z'", Ok("'+0000001'"));

    check_format(&times[0], "'%-10:::z'", Ok("'-000000000'"));
    check_format(&times[1], "'%-10:::z'", Ok("'+000000000'"));
    check_format(&times[4], "'%-10:::z'", Ok("'+000:09:21'"));
    check_format(&times[5], "'%-10:::z'", Ok("'+000000001'"));

    check_format(&times[0], "'%-_10:::z'", Ok("'        -0'"));
    check_format(&times[1], "'%-_10:::z'", Ok("'        +0'"));
    check_format(&times[4], "'%-_10:::z'", Ok("'  +0:09:21'"));
    check_format(&times[5], "'%-_10:::z'", Ok("'        +1'"));

    check_format(&times[4], "'%-_10::::z'", Ok("'%-_10::::z'"));

    check_format(&times[0], "'%10Z'", Ok("'       UTC'"));
    check_format(&times[1], "'%10Z'", Ok("'       UTC'"));
    check_format(&times[4], "'%10Z'", Ok("'       LMT'"));
    check_format(&times[5], "'%-^#10Z'", Ok("'cet'"));
    check_format(&times[2], "'%010Z'", Ok("'00000+0000'"));
    check_format(&times[3], "'%010Z'", Ok("''"));

    check_format(&times[4], "'%^#26c'", Ok("' TUE JAN  2 13:18:19 -0094'"));

    // TODO: other tests

    // Time.new(2022).strftime("%A\u{30a}") == "Saturday\u{30a}"
    // Time.new(2022).strftime("%\u{c5}") == "%\u{c5}"

    // Time.new(4).strftime("%9%y") == "   %y"
    // Time.new(4).strftime("%9%%y") == "   %04"
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
