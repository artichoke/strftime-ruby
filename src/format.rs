use core::fmt;
use core::num::IntErrorKind;
use core::str;

use bitflags::bitflags;

use crate::assert::{assert_sorted, assert_sorted_elem_0, assert_to_ascii_uppercase};
use crate::utils::{Cursor, Lower, SizeLimiter, Upper};
use crate::week::{iso_8601_year_and_week_number, week_number, WeekStart};
use crate::write::Write;
use crate::{Error, Time};

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
    assert_to_ascii_uppercase(&DAYS, &DAYS_UPPER);
    assert_to_ascii_uppercase(&MONTHS, &MONTHS_UPPER);
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

    fn compute_offset_parts(&self, time: &impl Time) -> UtcOffset {
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
    fn fmt(&self, f: &mut SizeLimiter<'_>, time: &impl Time) -> Result<(), Error> {
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

pub(crate) struct TimeFormatter<'t, 'f, T> {
    time: &'t T,
    format: &'f [u8],
}

impl<'t, 'f, T: Time> TimeFormatter<'t, 'f, T> {
    pub(crate) fn new<F: AsRef<[u8]> + ?Sized>(time: &'t T, format: &'f F) -> Self {
        Self {
            time,
            format: format.as_ref(),
        }
    }

    pub(crate) fn fmt(&self, buf: &mut dyn Write) -> Result<(), Error> {
        // Use a size limiter to limit the maximum size of the resulting formatted string
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
                None => return Err(Error::InvalidFormatString),
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
mod tests {
    use super::*;

    macro_rules! create_time {
        ($($field_name:ident: $field_type:ty),*,) => {
            struct Time<'a> {
                $($field_name: $field_type),*,
            }

            impl<'a> Time<'a> {
                #[allow(clippy::too_many_arguments)]
                fn new($($field_name: $field_type),*) -> Self {
                    Self { $($field_name),* }
                }
            }

            impl<'a> crate::Time for Time<'a> {
                $(fn $field_name(&self) -> $field_type { self.$field_name })*
            }
        };
    }

    create_time!(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanoseconds: u32,
        day_of_week: u8,
        day_of_year: u16,
        to_int: i64,
        is_utc: bool,
        utc_offset: i32,
        time_zone: &'a str,
    );

    fn check_format(time: &Time<'_>, format: &str, expected: Result<&str, Error>) {
        const SIZE: usize = 100;
        let mut buf = [0u8; SIZE];
        let mut cursor = &mut buf[..];

        let result = TimeFormatter::new(time, format).fmt(&mut cursor);
        let written = SIZE - cursor.len();
        let data = str::from_utf8(&buf[..written]).unwrap();

        assert_eq!(result.map(|_| data), expected);
    }

    #[test]
    fn test_format() {
        #[rustfmt::skip]
        let times = [
            Time::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, true, 0, "UTC"),
            Time::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, false, 0, "UTC"),
            Time::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, false, 0, "+0000"),
            Time::new(1, 1, 1, 1, 1, 1, 1, 1, 1, -62_135_593_139, false, 0, ""),
            Time::new(-94, 1, 2, 13, 18, 19, 9876, 2, 2, -65_133_456_662, false, 561, "LMT"),
            Time::new(2094, 1, 2, 13, 18, 19, 9876, 6, 2, 3_913_273_099, false, 3600, "CET"),
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
        let time = Time::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

        check_format(&time, "%-100000000m", Ok("1"));

        check_format(
            &time,
            "%100000000000000000000m",
            Ok("%100000000000000000000m"),
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_format_formatted_string_too_large() {
        let time = Time::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

        let mut buf = Vec::new();
        let result = TimeFormatter::new(&time, "%4718593m").fmt(&mut buf);

        assert_eq!(buf.len(), 4_718_592);
        assert_eq!(result, Err(Error::FormattedStringTooLarge));
    }

    #[test]
    fn test_format_small_buffer() {
        let time = Time::new(1970, 1, 1, 0, 0, 0, 0, 4, 1, 0, false, 0, "");

        let mut buf = [0u8; 3];
        let result = TimeFormatter::new(&time, "%Y").fmt(&mut &mut buf[..]);
        assert_eq!(result, Err(Error::WriteZero));
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
