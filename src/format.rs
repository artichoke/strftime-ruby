use core::fmt;
use core::num::IntErrorKind;
use core::str;

use bitflags::bitflags;
use spinoso_time::tzrs::Time;

use crate::assert::{assert_is_ascii_uppercase, assert_sorted, assert_sorted_elem_0};
use crate::utils::{Cursor, Lower, SizeLimiter, Upper};
use crate::week::{iso_8601_year_and_week_number, week_number, WeekStart};
use crate::write::Write;
use crate::Error;

const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

const DAYS_UPPER: [&str; 7] = [
    "SUNDAY",
    "MONDAY",
    "TUESDAY",
    "WEDNESDAY",
    "THURSDAY",
    "FRIDAY",
    "SATURDAY",
];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const MONTHS_UPPER: [&str; 12] = [
    "JANUARY",
    "FEBRUARY",
    "MARCH",
    "APRIL",
    "MAY",
    "JUNE",
    "JULY",
    "AUGUST",
    "SEPTEMBER",
    "OCTOBER",
    "NOVEMBER",
    "DECEMBER",
];

// Check day and month tables
const _: () = {
    assert_is_ascii_uppercase(&DAYS, &DAYS_UPPER);
    assert_is_ascii_uppercase(&MONTHS, &MONTHS_UPPER);
};

bitflags! {
    struct Flags: u32 {
        const LEFT_PADDING = 1 << 0;
        const CHANGE_CASE  = 1 << 1;
        const UPPER_CASE   = 1 << 2;
    }
}

impl Flags {
    fn has_change_or_upper_case(self) -> bool {
        let flag = Flags::CHANGE_CASE | Flags::UPPER_CASE;
        !self.intersection(flag).is_empty()
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
enum Padding {
    #[default]
    Left,
    Spaces,
    Zeros,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Spec {
    /// %Y
    Year4Digits,
    /// %C
    YearDiv100,
    /// %y
    YearRem100,
    /// %m
    Month,
    /// %B
    MonthName,
    /// %b, %h
    MonthNameAbbr,
    /// %d
    MonthDayZero,
    /// %e
    MonthDaySpace,
    /// %j
    YearDay,
    /// %H
    Hour24hZero,
    /// %k
    Hour24hSpace,
    /// %I
    Hour12hZero,
    /// %l
    Hour12hSpace,
    /// %P
    MeridianLower,
    /// %p
    MeridianUpper,
    /// %M
    Minute,
    /// %S
    Second,
    /// %L
    MilliSecond,
    /// %N
    FractionalSecond,
    /// %z
    TimeZoneOffsetHourMinute,
    /// %:z
    TimeZoneOffsetHourMinuteColon,
    /// %::z
    TimeZoneOffsetHourMinuteSecondColon,
    /// %:::z
    TimeZoneOffsetColonMinimal,
    /// %Z
    TimeZoneName,
    /// %A
    WeekDayName,
    /// %a
    WeekDayNameAbbr,
    /// %u
    WeekDayFrom1,
    /// %w
    WeekDayFrom0,
    /// %G
    YearIso8601,
    /// %g
    YearIso8601Rem100,
    /// %V
    WeekNumberIso8601,
    /// %U
    WeekNumberFromSunday,
    /// %W
    WeekNumberFromMonday,
    /// %s
    SecondsSinceEpoch,
    /// %n
    Newline,
    /// %t
    Tabulation,
    /// %%
    Percent,
    /// %c --> "%a %b %e %H:%M:%S %Y"
    CombinationDateTime,
    /// %D, %x --> "%m/%d/%y"
    CombinationDate,
    /// %F --> "%Y-%m-%d"
    CombinationIso8601,
    /// %v --> "%e-%^b-%4Y"
    CombinationVmsDate,
    /// %r --> "%I:%M:%S %p"
    CombinationTime12h,
    /// %R --> "%H:%M"
    CombinationHourMinute24h,
    /// %T, %X --> "%H:%M:%S"
    CombinationTime24h,
}

#[derive(Debug)]
struct UtcOffset {
    hour: f64,
    minute: u32,
    second: u32,
}

impl UtcOffset {
    fn new(hour: f64, minute: u32, second: u32) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }
}

#[derive(Debug)]
struct Piece {
    width: Option<usize>,
    padding: Padding,
    flags: Flags,
    spec: Spec,
}

impl Piece {
    fn new(width: Option<usize>, padding: Padding, flags: Flags, spec: Spec) -> Self {
        Self {
            width,
            padding,
            flags,
            spec,
        }
    }

    fn format_num_zeros(
        &self,
        f: &mut SizeLimiter<'_>,
        value: impl fmt::Display,
        default_width: usize,
    ) -> Result<(), Error> {
        if self.flags.contains(Flags::LEFT_PADDING) {
            write!(f, "{}", value)
        } else if self.padding == Padding::Spaces {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{: >width$}", value)
        } else {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{:0width$}", value)
        }
    }

    fn format_num_spaces(
        &self,
        f: &mut SizeLimiter<'_>,
        value: impl fmt::Display,
        default_width: usize,
    ) -> Result<(), Error> {
        if self.flags.contains(Flags::LEFT_PADDING) {
            write!(f, "{}", value)
        } else if self.padding == Padding::Zeros {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{:0width$}", value)
        } else {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{: >width$}", value)
        }
    }

    fn format_nanoseconds(
        &self,
        f: &mut SizeLimiter<'_>,
        nanoseconds: u32,
        default_width: usize,
    ) -> Result<(), Error> {
        let width = self.width.unwrap_or(default_width);

        if width <= 9 {
            let value = nanoseconds / 10u32.pow(9 - width as u32);
            write!(f, "{:0n$}", value, n = width)
        } else {
            write!(f, "{:09}{:0n$}", nanoseconds, 0, n = width - 9)
        }
    }

    fn format_string(&self, f: &mut SizeLimiter<'_>, s: impl fmt::Display) -> Result<(), Error> {
        match self.width {
            None => write!(f, "{}", s),
            Some(width) => {
                if self.flags.contains(Flags::LEFT_PADDING) {
                    write!(f, "{}", s)
                } else if self.padding == Padding::Zeros {
                    write!(f, "{:0>width$}", s)
                } else {
                    write!(f, "{: >width$}", s)
                }
            }
        }
    }

    fn compute_offset_parts(&self, time: &Time) -> UtcOffset {
        let utc_offset = time.utc_offset();
        let utc_offset_abs = utc_offset.unsigned_abs();

        // UTC is represented as "-00:00" if the '-' flag is set
        let sign = if utc_offset < 0 || time.is_utc() && self.flags.contains(Flags::LEFT_PADDING) {
            -1.0
        } else {
            1.0
        };

        // Convert to f64 to have signed zero
        let hour = sign * f64::from(utc_offset_abs / 3600);
        let minute = (utc_offset_abs / 60) % 60;
        let second = utc_offset_abs % 60;

        UtcOffset::new(hour, minute, second)
    }

    fn hour_padding(&self, min_width: usize) -> usize {
        const MIN_PADDING: usize = "+hh".len();

        match self.width {
            Some(width) => width.saturating_sub(min_width) + MIN_PADDING,
            None => MIN_PADDING,
        }
    }

    fn write_offset_hh(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let n = self.hour_padding("+hh".len());

        match self.padding {
            Padding::Spaces => write!(f, "{: >+n$.0}", utc_offset.hour),
            _ => write!(f, "{:+0n$.0}", utc_offset.hour),
        }
    }

    fn write_offset_hhmm(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let n = self.hour_padding("+hhmm".len());

        match self.padding {
            Padding::Spaces => write!(f, "{: >+n$.0}{:02}", utc_offset.hour, utc_offset.minute),
            _ => write!(f, "{:+0n$.0}{:02}", utc_offset.hour, utc_offset.minute),
        }
    }

    fn write_offset_hh_mm(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let n = self.hour_padding("+hh:mm".len());

        match self.padding {
            Padding::Spaces => write!(f, "{: >+n$.0}:{:02}", utc_offset.hour, utc_offset.minute),
            _ => write!(f, "{:+0n$.0}:{:02}", utc_offset.hour, utc_offset.minute),
        }
    }

    fn write_offset_hh_mm_ss(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let n = self.hour_padding("+hh:mm:ss".len());

        match self.padding {
            Padding::Spaces => write!(
                f,
                "{: >+n$.0}:{:02}:{:02}",
                utc_offset.hour, utc_offset.minute, utc_offset.second
            ),
            _ => write!(
                f,
                "{:+0n$.0}:{:02}:{:02}",
                utc_offset.hour, utc_offset.minute, utc_offset.second
            ),
        }
    }

    fn write_padding(&self, f: &mut SizeLimiter<'_>, min_width: usize) -> Result<(), Error> {
        if let Some(width) = self.width {
            let n = width.saturating_sub(min_width);

            match self.padding {
                Padding::Zeros => write!(f, "{:0>n$}", "")?,
                _ => write!(f, "{: >n$}", "")?,
            };
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut SizeLimiter<'_>, time: &Time) -> Result<(), Error> {
        match self.spec {
            Spec::Year4Digits => {
                let year = time.year();
                let default_width = if year < 0 { 5 } else { 4 };
                self.format_num_zeros(f, year, default_width)
            }
            Spec::YearDiv100 => self.format_num_zeros(f, time.year().div_euclid(100), 2),
            Spec::YearRem100 => self.format_num_zeros(f, time.year().rem_euclid(100), 2),
            Spec::Month => self.format_num_zeros(f, time.month(), 2),
            Spec::MonthName => {
                let index = (time.month() - 1) as usize;
                if self.flags.has_change_or_upper_case() {
                    self.format_string(f, MONTHS_UPPER[index])
                } else {
                    self.format_string(f, MONTHS[index])
                }
            }
            Spec::MonthNameAbbr => {
                let index = (time.month() - 1) as usize;
                if self.flags.has_change_or_upper_case() {
                    self.format_string(f, &MONTHS_UPPER[index][..3])
                } else {
                    self.format_string(f, &MONTHS[index][..3])
                }
            }
            Spec::MonthDayZero => self.format_num_zeros(f, time.day(), 2),
            Spec::MonthDaySpace => self.format_num_spaces(f, time.day(), 2),
            Spec::YearDay => self.format_num_zeros(f, time.day_of_year(), 3),
            Spec::Hour24hZero => self.format_num_zeros(f, time.hour(), 2),
            Spec::Hour24hSpace => self.format_num_spaces(f, time.hour(), 2),
            Spec::Hour12hZero => {
                let hour = time.hour() % 12;
                let hour = if hour == 0 { 12 } else { hour };
                self.format_num_zeros(f, hour, 2)
            }
            Spec::Hour12hSpace => {
                let hour = time.hour() % 12;
                let hour = if hour == 0 { 12 } else { hour };
                self.format_num_spaces(f, hour, 2)
            }
            Spec::MeridianLower => {
                let (am, pm) = if self.flags.has_change_or_upper_case() {
                    ("AM", "PM")
                } else {
                    ("am", "pm")
                };
                let meridian = if time.hour() < 12 { am } else { pm };
                self.format_string(f, meridian)
            }
            Spec::MeridianUpper => {
                let (am, pm) = if self.flags.contains(Flags::CHANGE_CASE) {
                    ("am", "pm")
                } else {
                    ("AM", "PM")
                };
                let meridian = if time.hour() < 12 { am } else { pm };
                self.format_string(f, meridian)
            }
            Spec::Minute => self.format_num_zeros(f, time.minute(), 2),
            Spec::Second => self.format_num_zeros(f, time.second(), 2),
            Spec::MilliSecond => self.format_nanoseconds(f, time.nanoseconds(), 3),
            Spec::FractionalSecond => self.format_nanoseconds(f, time.nanoseconds(), 9),
            Spec::TimeZoneOffsetHourMinute => {
                self.write_offset_hhmm(f, &self.compute_offset_parts(time))
            }
            Spec::TimeZoneOffsetHourMinuteColon => {
                self.write_offset_hh_mm(f, &self.compute_offset_parts(time))
            }
            Spec::TimeZoneOffsetHourMinuteSecondColon => {
                self.write_offset_hh_mm_ss(f, &self.compute_offset_parts(time))
            }
            Spec::TimeZoneOffsetColonMinimal => {
                let utc_offset = self.compute_offset_parts(time);

                if utc_offset.second == 0 && utc_offset.minute == 0 {
                    self.write_offset_hh(f, &utc_offset)
                } else if utc_offset.minute == 0 {
                    self.write_offset_hh_mm(f, &utc_offset)
                } else {
                    self.write_offset_hh_mm_ss(f, &utc_offset)
                }
            }
            Spec::TimeZoneName => {
                let tz_name = time.time_zone();
                if !tz_name.is_empty() {
                    if self.flags.contains(Flags::CHANGE_CASE) {
                        self.format_string(f, Lower::new(tz_name.as_bytes()))?;
                    } else if self.flags.contains(Flags::UPPER_CASE) {
                        self.format_string(f, Upper::new(tz_name.as_bytes()))?;
                    } else {
                        self.format_string(f, tz_name)?;
                    }
                }
                Ok(())
            }
            Spec::WeekDayName => {
                let index = time.day_of_week() as usize;
                if self.flags.has_change_or_upper_case() {
                    self.format_string(f, DAYS_UPPER[index])
                } else {
                    self.format_string(f, DAYS[index])
                }
            }
            Spec::WeekDayNameAbbr => {
                let index = time.day_of_week() as usize;
                if self.flags.has_change_or_upper_case() {
                    self.format_string(f, &DAYS_UPPER[index][..3])
                } else {
                    self.format_string(f, &DAYS[index][..3])
                }
            }
            Spec::WeekDayFrom1 => {
                let day_of_week = time.day_of_week();
                let day_of_week = if day_of_week == 0 { 7 } else { day_of_week };
                self.format_num_zeros(f, day_of_week, 1)
            }
            Spec::WeekDayFrom0 => self.format_num_zeros(f, time.day_of_week(), 1),
            Spec::YearIso8601 => {
                let (iso_year, _) = iso_8601_year_and_week_number(
                    time.year().into(),
                    time.day_of_week().into(),
                    time.day_of_year().into(),
                );
                let default_width = if iso_year < 0 { 5 } else { 4 };
                self.format_num_zeros(f, iso_year, default_width)
            }
            Spec::YearIso8601Rem100 => {
                let (iso_year, _) = iso_8601_year_and_week_number(
                    time.year().into(),
                    time.day_of_week().into(),
                    time.day_of_year().into(),
                );
                self.format_num_zeros(f, iso_year.rem_euclid(100), 2)
            }
            Spec::WeekNumberIso8601 => {
                let (_, iso_week_number) = iso_8601_year_and_week_number(
                    time.year().into(),
                    time.day_of_week().into(),
                    time.day_of_year().into(),
                );
                self.format_num_zeros(f, iso_week_number, 2)
            }
            Spec::WeekNumberFromSunday => {
                let week_number = week_number(
                    time.day_of_week().into(),
                    time.day_of_year().into(),
                    WeekStart::Sunday,
                );
                self.format_num_zeros(f, week_number, 2)
            }
            Spec::WeekNumberFromMonday => {
                let week_number = week_number(
                    time.day_of_week().into(),
                    time.day_of_year().into(),
                    WeekStart::Monday,
                );
                self.format_num_zeros(f, week_number, 2)
            }
            Spec::SecondsSinceEpoch => self.format_num_zeros(f, time.to_int(), 1),
            Spec::Newline => self.format_string(f, "\n"),
            Spec::Tabulation => self.format_string(f, "\t"),
            Spec::Percent => self.format_string(f, "%"),
            Spec::CombinationDateTime => {
                const MIN_WIDTH_NO_YEAR: usize = "www mmm dd HH:MM:SS ".len();

                let year = time.year();
                let default_year_width = if year < 0 { 5 } else { 4 };
                let min_width = MIN_WIDTH_NO_YEAR + year_width(year).max(default_year_width);
                self.write_padding(f, min_width)?;

                let (day_names, month_names) = if self.flags.contains(Flags::UPPER_CASE) {
                    (&DAYS_UPPER, &MONTHS_UPPER)
                } else {
                    (&DAYS, &MONTHS)
                };

                let week_day_name = &day_names[time.day_of_week() as usize][..3];
                let month_name = &month_names[(time.month() - 1) as usize][..3];
                let day = time.day();
                let (hour, minute, second) = (time.hour(), time.minute(), time.second());

                write!(
                    f,
                    "{} {} {: >2} {:02}:{:02}:{:02} {:0default_year_width$}",
                    week_day_name, month_name, day, hour, minute, second, year
                )
            }
            Spec::CombinationDate => {
                self.write_padding(f, "mm/dd/yy".len())?;

                let year = time.year().rem_euclid(100);
                let month = time.month();
                let day = time.day();

                write!(f, "{:02}:{:02}:{:02}", month, day, year)
            }
            Spec::CombinationIso8601 => {
                const MIN_WIDTH_NO_YEAR: usize = "-mm-dd".len();

                let year = time.year();
                let default_year_width = if year < 0 { 5 } else { 4 };
                let min_width = MIN_WIDTH_NO_YEAR + year_width(year).max(default_year_width);
                self.write_padding(f, min_width)?;

                let month = time.month();
                let day = time.day();

                write!(f, "{:0default_year_width$}-{:02}-{:02}", year, month, day)
            }
            Spec::CombinationVmsDate => {
                let year = time.year();
                self.write_padding(f, "dd-mmm-".len() + year_width(year).max(4))?;

                let month_name = &MONTHS_UPPER[(time.month() - 1) as usize][..3];
                let day = time.day();

                write!(f, "{: >2}-{}-{:04}", day, month_name, year)
            }
            Spec::CombinationTime12h => {
                self.write_padding(f, "HH:MM:SS PM".len())?;

                let hour = time.hour() % 12;
                let hour = if hour == 0 { 12 } else { hour };

                let (minute, second) = (time.minute(), time.second());
                let meridian = if time.hour() < 12 { "AM" } else { "PM" };

                write!(f, "{:02}:{:02}:{:02} {}", hour, minute, second, meridian)
            }
            Spec::CombinationHourMinute24h => {
                self.write_padding(f, "HH:MM".len())?;
                write!(f, "{:02}:{:02}", time.hour(), time.minute())
            }
            Spec::CombinationTime24h => {
                self.write_padding(f, "HH:MM:SS".len())?;
                let (hour, minute, second) = (time.hour(), time.minute(), time.second());
                write!(f, "{:02}:{:02}:{:02}", hour, minute, second)
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct TimeFormatter<'t, 'f> {
    time: &'t Time,
    format: &'f [u8],
}

impl<'t, 'f> TimeFormatter<'t, 'f> {
    pub(crate) fn new<T: AsRef<[u8]> + ?Sized>(time: &'t Time, format: &'f T) -> Self {
        Self {
            time,
            format: format.as_ref(),
        }
    }

    pub(crate) fn fmt(&self, buf: &mut dyn Write) -> Result<(), Error> {
        // Use a size limiter to avoid large writes
        let size_limit = self.format.len().saturating_mul(512 * 1024).max(1024);
        let mut f = SizeLimiter::new(buf, size_limit);

        let mut cursor = Cursor::new(self.format);

        loop {
            f.write_all(cursor.read_until(|&x| x == b'%'))?;

            let remaining_before = cursor.remaining();

            // Read the '%' character
            if cursor.next().is_none() {
                break;
            }

            match Self::parse_spec(&mut cursor)? {
                Some(piece) => piece.fmt(&mut f, self.time)?,
                None => {
                    // No valid format specifier was found
                    let remaining_after = cursor.remaining();
                    let text = &remaining_before[..remaining_before.len() - remaining_after.len()];
                    f.write_all(text)?;
                }
            }
        }

        Ok(())
    }

    fn parse_spec(cursor: &mut Cursor<'_>) -> Result<Option<Piece>, Error> {
        // Parse flags
        let mut padding = Padding::default();
        let mut flags = Flags::empty();

        loop {
            // The left padding overrides the other padding options for most cases.
            // It is also used for the hour sign in the %z specifier.
            //
            // Similary, the change case flag overrides the upper case flag, except
            // when using combination specifiers (%c, %D, %x, %F, %v, %r, %R, %T, %X).
            match cursor.remaining().first() {
                Some(&b'-') => {
                    padding = Padding::Left;
                    flags.insert(Flags::LEFT_PADDING);
                }
                Some(&b'_') => padding = Padding::Spaces,
                Some(&b'0') => padding = Padding::Zeros,
                Some(&b'^') => flags.insert(Flags::UPPER_CASE),
                Some(&b'#') => flags.insert(Flags::CHANGE_CASE),
                _ => break,
            }
            cursor.next();
        }

        // Parse width
        let width_digits = str::from_utf8(cursor.read_while(u8::is_ascii_digit))
            .expect("reading ASCII digits should yield a valid UTF-8 slice");

        let width = match width_digits.parse::<usize>() {
            Ok(width) => Some(width),
            Err(err) if *err.kind() == IntErrorKind::Empty => None,
            Err(_) => return Ok(None),
        };

        // Ignore POSIX locale extensions (https://github.com/ruby/ruby/blob/4491bb740a9506d76391ac44bb2fe6e483fec952/strftime.c#L713-L722)
        if let Some(&[ext, spec]) = cursor.remaining().get(..2) {
            const EXT_E_SPECS: &[u8] = assert_sorted(b"CXYcxy");
            const EXT_O_SPECS: &[u8] = assert_sorted(b"HIMSUVWdeklmuwy");

            match ext {
                b'E' if EXT_E_SPECS.binary_search(&spec).is_ok() => cursor.next(),
                b'O' if EXT_O_SPECS.binary_search(&spec).is_ok() => cursor.next(),
                _ => None,
            };
        }

        // Parse spec
        let colons = cursor.read_while(|&x| x == b':');

        let spec = if colons.is_empty() {
            const POSSIBLE_SPECS: &[(u8, Spec)] = assert_sorted_elem_0(&[
                (b'%', Spec::Percent),
                (b'A', Spec::WeekDayName),
                (b'B', Spec::MonthName),
                (b'C', Spec::YearDiv100),
                (b'D', Spec::CombinationDate),
                (b'F', Spec::CombinationIso8601),
                (b'G', Spec::YearIso8601),
                (b'H', Spec::Hour24hZero),
                (b'I', Spec::Hour12hZero),
                (b'L', Spec::MilliSecond),
                (b'M', Spec::Minute),
                (b'N', Spec::FractionalSecond),
                (b'P', Spec::MeridianLower),
                (b'R', Spec::CombinationHourMinute24h),
                (b'S', Spec::Second),
                (b'T', Spec::CombinationTime24h),
                (b'U', Spec::WeekNumberFromSunday),
                (b'V', Spec::WeekNumberIso8601),
                (b'W', Spec::WeekNumberFromMonday),
                (b'X', Spec::CombinationTime24h),
                (b'Y', Spec::Year4Digits),
                (b'Z', Spec::TimeZoneName),
                (b'a', Spec::WeekDayNameAbbr),
                (b'b', Spec::MonthNameAbbr),
                (b'c', Spec::CombinationDateTime),
                (b'd', Spec::MonthDayZero),
                (b'e', Spec::MonthDaySpace),
                (b'g', Spec::YearIso8601Rem100),
                (b'h', Spec::MonthNameAbbr),
                (b'j', Spec::YearDay),
                (b'k', Spec::Hour24hSpace),
                (b'l', Spec::Hour12hSpace),
                (b'm', Spec::Month),
                (b'n', Spec::Newline),
                (b'p', Spec::MeridianUpper),
                (b'r', Spec::CombinationTime12h),
                (b's', Spec::SecondsSinceEpoch),
                (b't', Spec::Tabulation),
                (b'u', Spec::WeekDayFrom1),
                (b'v', Spec::CombinationVmsDate),
                (b'w', Spec::WeekDayFrom0),
                (b'x', Spec::CombinationDate),
                (b'y', Spec::YearRem100),
                (b'z', Spec::TimeZoneOffsetHourMinute),
            ]);

            match cursor.next() {
                Some(x) => match POSSIBLE_SPECS.binary_search_by_key(&x, |&(c, _)| c) {
                    Ok(index) => Some(POSSIBLE_SPECS[index].1),
                    Err(_) => None,
                },
                None => return Err(Error::InvalidFormat),
            }
        } else if cursor.read_optional_tag(b"z") {
            match colons.len() {
                1 => Some(Spec::TimeZoneOffsetHourMinuteColon),
                2 => Some(Spec::TimeZoneOffsetHourMinuteSecondColon),
                3 => Some(Spec::TimeZoneOffsetColonMinimal),
                _ => None,
            }
        } else {
            None
        };

        Ok(spec.map(|spec| Piece::new(width, padding, flags, spec)))
    }
}

fn year_width(year: i32) -> usize {
    let mut n = if year <= 0 { 1 } else { 0 };
    let mut val = year;
    while val != 0 {
        val /= 10;
        n += 1;
    }
    n
}

#[cfg(test)]
#[cfg(feature = "alloc")]
mod tests {
    use super::*;

    use spinoso_time::tzrs::Offset;
    use tz::TimeZoneRef;

    fn format(time: &Time, format: &str) -> Result<String, Error> {
        let mut buf = Vec::new();
        TimeFormatter::new(time, format).fmt(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    #[test]
    fn test_format() {
        let utc = tzdb::time_zone::UTC;
        let time_zone_ref = tzdb::tz_by_name("Europe/Paris").unwrap();

        let time = Time::new(-94, 1, 2, 13, 18, 19, 9876, time_zone_ref.into()).unwrap();

        assert!(format(&time, "%").is_err());
        assert!(format(&time, "%-4").is_err());
        assert!(format(&time, "%-").is_err());
        assert!(format(&time, "%-_").is_err());

        assert_eq!(format(&time, "% ").unwrap(), "% ");
        assert_eq!(format(&time, "%-4 ").unwrap(), "%-4 ");
        assert_eq!(format(&time, "%- ").unwrap(), "%- ");
        assert_eq!(format(&time, "%-_ ").unwrap(), "%-_ ");

        assert_eq!(format(&time, "'%4Y'").unwrap(), "'-094'");
        assert_eq!(format(&time, "'%_Y'").unwrap(), "'  -94'");
        assert_eq!(format(&time, "'%y'").unwrap(), "'06'");
        assert_eq!(format(&time, "'%C'").unwrap(), "'-1'");

        let time0 = Time::new(1, 1, 1, 1, 1, 1, 1, Offset::utc()).unwrap();
        let time1 = Time::new(1, 1, 1, 1, 1, 1, 1, utc.into()).unwrap();
        let time2 = Time::new(-94, 1, 2, 13, 18, 19, 9876, time_zone_ref.into()).unwrap();
        let time3 = Time::new(2094, 1, 2, 13, 18, 19, 9876, time_zone_ref.into()).unwrap();

        assert_eq!(format(&time0, "'%z'").unwrap(), "'+0000'");
        assert_eq!(format(&time1, "'%z'").unwrap(), "'+0000'");
        assert_eq!(format(&time2, "'%z'").unwrap(), "'+0009'");
        assert_eq!(format(&time3, "'%z'").unwrap(), "'+0100'");

        assert_eq!(format(&time0, "'%1z'").unwrap(), "'+0000'");
        assert_eq!(format(&time1, "'%1z'").unwrap(), "'+0000'");
        assert_eq!(format(&time2, "'%1z'").unwrap(), "'+0009'");
        assert_eq!(format(&time3, "'%1z'").unwrap(), "'+0100'");

        assert_eq!(format(&time0, "'%-6z'").unwrap(), "'-00000'");
        assert_eq!(format(&time1, "'%-6z'").unwrap(), "'+00000'");
        assert_eq!(format(&time2, "'%-6z'").unwrap(), "'+00009'");
        assert_eq!(format(&time3, "'%-6z'").unwrap(), "'+00100'");

        assert_eq!(format(&time0, "'%_6z'").unwrap(), "'  +000'");
        assert_eq!(format(&time1, "'%_6z'").unwrap(), "'  +000'");
        assert_eq!(format(&time2, "'%_6z'").unwrap(), "'  +009'");
        assert_eq!(format(&time3, "'%_6z'").unwrap(), "'  +100'");

        assert_eq!(format(&time0, "'%:z'").unwrap(), "'+00:00'");
        assert_eq!(format(&time1, "'%:z'").unwrap(), "'+00:00'");
        assert_eq!(format(&time2, "'%:z'").unwrap(), "'+00:09'");
        assert_eq!(format(&time3, "'%:z'").unwrap(), "'+01:00'");

        assert_eq!(format(&time0, "'%1:z'").unwrap(), "'+00:00'");
        assert_eq!(format(&time1, "'%1:z'").unwrap(), "'+00:00'");
        assert_eq!(format(&time2, "'%1:z'").unwrap(), "'+00:09'");
        assert_eq!(format(&time3, "'%1:z'").unwrap(), "'+01:00'");

        assert_eq!(format(&time0, "'%-7:z'").unwrap(), "'-000:00'");
        assert_eq!(format(&time1, "'%-7:z'").unwrap(), "'+000:00'");
        assert_eq!(format(&time2, "'%-7:z'").unwrap(), "'+000:09'");
        assert_eq!(format(&time3, "'%-7:z'").unwrap(), "'+001:00'");

        assert_eq!(format(&time0, "'%_7:z'").unwrap(), "'  +0:00'");
        assert_eq!(format(&time1, "'%_7:z'").unwrap(), "'  +0:00'");
        assert_eq!(format(&time2, "'%_7:z'").unwrap(), "'  +0:09'");
        assert_eq!(format(&time3, "'%_7:z'").unwrap(), "'  +1:00'");

        assert_eq!(format(&time0, "'%::z'").unwrap(), "'+00:00:00'");
        assert_eq!(format(&time1, "'%::z'").unwrap(), "'+00:00:00'");
        assert_eq!(format(&time2, "'%::z'").unwrap(), "'+00:09:21'");
        assert_eq!(format(&time3, "'%::z'").unwrap(), "'+01:00:00'");

        assert_eq!(format(&time0, "'%1::z'").unwrap(), "'+00:00:00'");
        assert_eq!(format(&time1, "'%1::z'").unwrap(), "'+00:00:00'");
        assert_eq!(format(&time2, "'%1::z'").unwrap(), "'+00:09:21'");
        assert_eq!(format(&time3, "'%1::z'").unwrap(), "'+01:00:00'");

        assert_eq!(format(&time0, "'%-10::z'").unwrap(), "'-000:00:00'");
        assert_eq!(format(&time1, "'%-10::z'").unwrap(), "'+000:00:00'");
        assert_eq!(format(&time2, "'%-10::z'").unwrap(), "'+000:09:21'");
        assert_eq!(format(&time3, "'%-10::z'").unwrap(), "'+001:00:00'");

        assert_eq!(format(&time0, "'%_10::z'").unwrap(), "'  +0:00:00'");
        assert_eq!(format(&time1, "'%_10::z'").unwrap(), "'  +0:00:00'");
        assert_eq!(format(&time2, "'%_10::z'").unwrap(), "'  +0:09:21'");
        assert_eq!(format(&time3, "'%_10::z'").unwrap(), "'  +1:00:00'");

        assert_eq!(format(&time0, "'%1:::z'").unwrap(), "'+00'");
        assert_eq!(format(&time1, "'%1:::z'").unwrap(), "'+00'");
        assert_eq!(format(&time2, "'%1:::z'").unwrap(), "'+00:09:21'");
        assert_eq!(format(&time3, "'%1:::z'").unwrap(), "'+01'");

        assert_eq!(format(&time0, "'%8:::z'").unwrap(), "'+0000000'");
        assert_eq!(format(&time1, "'%8:::z'").unwrap(), "'+0000000'");
        assert_eq!(format(&time2, "'%8:::z'").unwrap(), "'+00:09:21'");
        assert_eq!(format(&time3, "'%8:::z'").unwrap(), "'+0000001'");

        assert_eq!(format(&time0, "'%-10:::z'").unwrap(), "'-000000000'");
        assert_eq!(format(&time1, "'%-10:::z'").unwrap(), "'+000000000'");
        assert_eq!(format(&time2, "'%-10:::z'").unwrap(), "'+000:09:21'");
        assert_eq!(format(&time3, "'%-10:::z'").unwrap(), "'+000000001'");

        assert_eq!(format(&time0, "'%-_10:::z'").unwrap(), "'        -0'");
        assert_eq!(format(&time1, "'%-_10:::z'").unwrap(), "'        +0'");
        assert_eq!(format(&time2, "'%-_10:::z'").unwrap(), "'  +0:09:21'");
        assert_eq!(format(&time3, "'%-_10:::z'").unwrap(), "'        +1'");

        assert_eq!(format(&time, "'%-_10::::z'").unwrap(), "'%-_10::::z'");

        let time4 = Time::new(1, 1, 1, 1, 1, 1, 1, Offset::fixed(0).unwrap()).unwrap();
        let time5 = Time::new(1, 1, 1, 1, 1, 1, 1, TimeZoneRef::utc().into()).unwrap();

        assert_eq!(format(&time0, "'%10Z'").unwrap(), "'       UTC'");
        assert_eq!(format(&time1, "'%10Z'").unwrap(), "'       UTC'");
        assert_eq!(format(&time2, "'%10Z'").unwrap(), "'       LMT'");
        assert_eq!(format(&time3, "'%-^#10Z'").unwrap(), "'cet'");
        assert_eq!(format(&time4, "'%010Z'").unwrap(), "'00000+0000'");
        assert_eq!(format(&time5, "'%010Z'").unwrap(), "''");

        assert_eq!(
            format(&time, "'%^#26c'").unwrap(),
            "' TUE JAN  2 13:18:19 -0094'"
        );

        // TODO: other tests

        // Time.new(2022).strftime("%A\u{30a}") == "Saturday\u{30a}"
        // Time.new(2022).strftime("%\u{c5}") == "%\u{c5}"

        // Time.new(4).strftime("%9%y") == "   %y"
        // Time.new(4).strftime("%9%%y") == "   %04"
    }

    #[test]
    fn test_format_large_width() {
        let time_zone_ref = tzdb::tz_by_name("Europe/Paris").unwrap();
        let time = Time::new(1, 1, 1, 1, 1, 1, 1, time_zone_ref.into()).unwrap();

        assert_eq!(format(&time, "%-100000000Y").unwrap(), "1");

        assert_eq!(
            format(&time, "%100000000000000000000Y").unwrap(),
            "%100000000000000000000Y"
        );

        let mut buf = Vec::new();
        let result = TimeFormatter::new(&time, "%4718593Y").fmt(&mut buf);

        assert_eq!(buf.len(), 4_718_592);
        assert!(matches!(result.unwrap_err(), Error::WriteZero));
    }

    #[test]
    fn test_year_width() {
        assert_eq!(year_width(-100), 4);
        assert_eq!(year_width(-99), 3);
        assert_eq!(year_width(-10), 3);
        assert_eq!(year_width(-9), 2);
        assert_eq!(year_width(-1), 2);
        assert_eq!(year_width(0), 1);
        assert_eq!(year_width(1), 1);
        assert_eq!(year_width(9), 1);
        assert_eq!(year_width(10), 2);
        assert_eq!(year_width(99), 2);
        assert_eq!(year_width(100), 3);
    }
}
