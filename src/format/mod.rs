//! Module containing the formatting logic.

mod assert;
mod utils;
mod week;
mod write;

use core::fmt;
use core::num::IntErrorKind;
use core::str;

use bitflags::bitflags;

use crate::{Error, Time};
use assert::{assert_sorted, assert_sorted_elem_0, assert_to_ascii_uppercase};
use utils::{Cursor, SizeLimiter};
use week::{iso_8601_year_and_week_number, week_number, WeekStart};
use write::Write;

/// Alias to a `c_int`.
#[cfg(feature = "std")]
type Int = std::os::raw::c_int;
/// Fallback alias to a `c_int`.
#[cfg(not(feature = "std"))]
type Int = i32;

/// List of weekday names.
const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// List of uppercase weekday names.
const DAYS_UPPER: [&str; 7] = [
    "SUNDAY",
    "MONDAY",
    "TUESDAY",
    "WEDNESDAY",
    "THURSDAY",
    "FRIDAY",
    "SATURDAY",
];

/// List of month names.
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

/// List of uppercase month names.
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
    /// Formatting flags.
    struct Flags: u32 {
        /// Use left padding, removing all other padding options in most cases.
        const LEFT_PADDING = 1 << 0;
        /// Change case for a string value.
        const CHANGE_CASE  = 1 << 1;
        /// Convert a string value to uppercase.
        const UPPER_CASE   = 1 << 2;
    }
}

impl Flags {
    /// Check if one of the case flags is set.
    fn has_change_or_upper_case(self) -> bool {
        let flag = Flags::CHANGE_CASE | Flags::UPPER_CASE;
        !self.intersection(flag).is_empty()
    }
}

/// Padding method.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Padding {
    /// Left padding.
    Left,
    /// Padding with spaces.
    Spaces,
    /// Padding with zeros.
    Zeros,
}

/// Formatting specifier.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Spec {
    /// `"%Y"`: Year with century if provided, zero-padded to at least 4 digits
    /// plus the possible negative sign.
    Year4Digits,
    /// `"%C"`: `Year / 100` using Euclidean division, zero-padded to at least 2
    /// digits.
    YearDiv100,
    /// `"%y"`: `Year % 100` in `00..=99`, using Euclidean remainder, zero-padded
    /// to 2 digits.
    YearRem100,
    /// `"%m"`: Month of the year in `01..=12`, zero-padded to 2 digits.
    Month,
    /// `"%B"`: Locale independent full month name.
    MonthName,
    /// `"%b"` and `"%h"`: Locale independent abbreviated month name, using the
    /// first 3 letters.
    MonthNameAbbr,
    /// `"%d"`: Day of the month in `01..=31`, zero-padded to 2 digits.
    MonthDayZero,
    /// `"%e"`: Day of the month in ` 1..=31`, blank-padded to 2 digits.
    MonthDaySpace,
    /// `"%j"`: Day of the year in `001..=366`, zero-padded to 3 digits.
    YearDay,
    /// `"%H"`: Hour of the day (24-hour clock) in `00..=23`, zero-padded to 2
    /// digits.
    Hour24hZero,
    /// `"%k"`: Hour of the day (24-hour clock) in ` 0..=23`, blank-padded to 2
    /// digits.
    Hour24hSpace,
    /// `"%I"`: Hour of the day (12-hour clock) in `01..=12`, zero-padded to 2
    /// digits.
    Hour12hZero,
    /// `"%l"`: Hour of the day (12-hour clock) in ` 1..=12`, blank-padded to 2
    /// digits.
    Hour12hSpace,
    /// `"%P"`: Lowercase meridian indicator (`"am"` or `"pm"`).
    MeridianLower,
    /// `"%p"`: Uppercase meridian indicator (`"AM"` or `"PM"`).
    MeridianUpper,
    /// `"%M"`: Minute of the hour in `00..=59`, zero-padded to 2 digits.
    Minute,
    /// `"%S"`: Second of the minute in `00..=60`, zero-padded to 2 digits.
    Second,
    /// `"%L"`: Truncated fractional seconds digits, with 3 digits by default.
    /// Number of digits is specified by the width field.
    MilliSecond,
    /// `"%N"`: Truncated fractional seconds digits, with 9 digits by default.
    /// Number of digits is specified by the width field.
    FractionalSecond,
    /// `"%z"`: Zero-padded signed time zone UTC hour and minute offsets
    /// (`+hhmm`).
    TimeZoneOffsetHourMinute,
    /// `"%:z"`: Zero-padded signed time zone UTC hour and minute offsets with
    /// colons (`+hh:mm`).
    TimeZoneOffsetHourMinuteColon,
    /// `"%::z"`: Zero-padded signed time zone UTC hour, minute and second
    /// offsets with colons (`+hh:mm:ss`).
    TimeZoneOffsetHourMinuteSecondColon,
    /// `"%:::z"`: Zero-padded signed time zone UTC hour offset, with optional
    /// minute and second offsets with colons (`+hh[:mm[:ss]]`).
    TimeZoneOffsetColonMinimal,
    /// `"%Z"`: Platform-dependent abbreviated time zone name.
    TimeZoneName,
    /// `"%A"`: Locale independent full weekday name.
    WeekDayName,
    /// `"%a"`: Locale independent abbreviated weekday name, using the first 3
    /// letters.
    WeekDayNameAbbr,
    /// `"%u"`: Day of the week from Monday in `1..=7`, zero-padded to 1 digit.
    WeekDayFrom1,
    /// `"%w"`: Day of the week from Sunday in `0..=6`, zero-padded to 1 digit.
    WeekDayFrom0,
    /// `"%G"`: Same as `%Y`, but using the ISO 8601 week-based year.
    YearIso8601,
    /// `"%g"`: Same as `%y`, but using the ISO 8601 week-based year.
    YearIso8601Rem100,
    /// `"%V"`: ISO 8601 week number in `01..=53`, zero-padded to 2 digits.
    WeekNumberIso8601,
    /// `"%U"`: Week number from Sunday in `00..=53`, zero-padded to 2 digits.
    /// The week `1` starts with the first Sunday of the year.
    WeekNumberFromSunday,
    /// `"%W"`: Week number from Monday in `00..=53`, zero-padded to 2 digits.
    /// The week `1` starts with the first Monday of the year.
    WeekNumberFromMonday,
    /// `"%s"`: Number of seconds since `1970-01-01 00:00:00 UTC`, zero-padded
    /// to at least 1 digit.
    SecondsSinceEpoch,
    /// `"%n"`: Newline character `'\n'`.
    Newline,
    /// `"%t"`: Tab character `'\t'`.
    Tabulation,
    /// `"%%"`: Literal `'%'` character.
    Percent,
    /// `"%c"`: Date and time, equivalent to `"%a %b %e %H:%M:%S %Y"`.
    CombinationDateTime,
    /// `"%D"` and `"%x"`: Date, equivalent to `"%m/%d/%y"`.
    CombinationDate,
    /// `"%F"`: ISO 8601 date, equivalent to `"%Y-%m-%d"`.
    CombinationIso8601,
    /// `"%v"`: VMS date, equivalent to `"%e-%^b-%4Y"`.
    CombinationVmsDate,
    /// `"%r"`: 12-hour time, equivalent to `"%I:%M:%S %p"`.
    CombinationTime12h,
    /// `"%R"`: 24-hour time without seconds, equivalent to `"%H:%M"`.
    CombinationHourMinute24h,
    /// `"%T"` and `"%X"`: 24-hour time, equivalent to `"%H:%M:%S"`.
    CombinationTime24h,
}

/// UTC offset parts.
#[derive(Debug)]
struct UtcOffset {
    /// Signed hour.
    hour: f64,
    /// Minute.
    minute: u32,
    /// Second.
    second: u32,
}

impl UtcOffset {
    /// Construct a new `UtcOffset`.
    fn new(hour: f64, minute: u32, second: u32) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }
}

/// Formatting directive.
#[derive(Debug)]
struct Piece {
    /// Optional width.
    width: Option<usize>,
    /// Padding method.
    padding: Padding,
    /// Formatting flags.
    flags: Flags,
    /// Formatting specifier.
    spec: Spec,
}

impl Piece {
    /// Construct a new `Piece`.
    fn new(width: Option<usize>, padding: Padding, flags: Flags, spec: Spec) -> Self {
        Self {
            width,
            padding,
            flags,
            spec,
        }
    }

    /// Format a numerical value, padding with zeros by default.
    fn format_num_zeros(
        &self,
        f: &mut SizeLimiter<'_>,
        value: impl fmt::Display,
        default_width: usize,
    ) -> Result<(), Error> {
        if self.flags.contains(Flags::LEFT_PADDING) {
            write!(f, "{value}")
        } else if self.padding == Padding::Spaces {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{value: >width$}")
        } else {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{value:0width$}")
        }
    }

    /// Format a numerical value, padding with spaces by default.
    fn format_num_spaces(
        &self,
        f: &mut SizeLimiter<'_>,
        value: impl fmt::Display,
        default_width: usize,
    ) -> Result<(), Error> {
        if self.flags.contains(Flags::LEFT_PADDING) {
            write!(f, "{value}")
        } else if self.padding == Padding::Zeros {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{value:0width$}")
        } else {
            let width = self.width.unwrap_or(default_width);
            write!(f, "{value: >width$}")
        }
    }

    /// Format nanoseconds with the specified precision.
    fn format_nanoseconds(
        &self,
        f: &mut SizeLimiter<'_>,
        nanoseconds: u32,
        default_width: usize,
    ) -> Result<(), Error> {
        let width = self.width.unwrap_or(default_width);

        if width <= 9 {
            let value = nanoseconds / 10u32.pow(9 - width as u32);
            write!(f, "{value:0n$}", n = width)
        } else {
            write!(f, "{nanoseconds:09}{:0n$}", 0, n = width - 9)
        }
    }

    /// Format a string value.
    fn format_string(&self, f: &mut SizeLimiter<'_>, s: &str) -> Result<(), Error> {
        match self.width {
            None => write!(f, "{s}"),
            Some(width) => {
                if self.flags.contains(Flags::LEFT_PADDING) {
                    write!(f, "{s}")
                } else if self.padding == Padding::Zeros {
                    write!(f, "{s:0>width$}")
                } else {
                    write!(f, "{s: >width$}")
                }
            }
        }
    }

    /// Write padding separately.
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

    /// Compute UTC offset parts for the `%z` specifier.
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

    /// Compute hour padding for the `%z` specifier.
    fn hour_padding(&self, min_width: usize) -> usize {
        const MIN_PADDING: usize = "+hh".len();

        match self.width {
            Some(width) => width.saturating_sub(min_width) + MIN_PADDING,
            None => MIN_PADDING,
        }
    }

    /// Write the time zone UTC offset as `"+hh"`.
    fn write_offset_hh(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let hour = utc_offset.hour;
        let n = self.hour_padding("+hh".len());

        match self.padding {
            Padding::Spaces => write!(f, "{hour: >+n$.0}"),
            _ => write!(f, "{hour:+0n$.0}"),
        }
    }

    /// Write the time zone UTC offset as `"+hhmm"`.
    fn write_offset_hhmm(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let UtcOffset { hour, minute, .. } = utc_offset;
        let n = self.hour_padding("+hhmm".len());

        match self.padding {
            Padding::Spaces => write!(f, "{hour: >+n$.0}{minute:02}"),
            _ => write!(f, "{hour:+0n$.0}{minute:02}"),
        }
    }

    /// Write the time zone UTC offset as `"+hh:mm"`.
    fn write_offset_hh_mm(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let UtcOffset { hour, minute, .. } = utc_offset;
        let n = self.hour_padding("+hh:mm".len());

        match self.padding {
            Padding::Spaces => write!(f, "{hour: >+n$.0}:{minute:02}"),
            _ => write!(f, "{hour:+0n$.0}:{minute:02}"),
        }
    }

    /// Write the time zone UTC offset as `"+hh:mm:ss"`.
    fn write_offset_hh_mm_ss(
        &self,
        f: &mut SizeLimiter<'_>,
        utc_offset: &UtcOffset,
    ) -> Result<(), Error> {
        let UtcOffset {
            hour,
            minute,
            second,
        } = utc_offset;

        let n = self.hour_padding("+hh:mm:ss".len());

        match self.padding {
            Padding::Spaces => write!(f, "{hour: >+n$.0}:{minute:02}:{second:02}"),
            _ => write!(f, "{hour:+0n$.0}:{minute:02}:{second:02}"),
        }
    }

    /// Format time using the formatting directive.
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

                if utc_offset.second != 0 {
                    self.write_offset_hh_mm_ss(f, &utc_offset)
                } else if utc_offset.minute != 0 {
                    self.write_offset_hh_mm(f, &utc_offset)
                } else {
                    self.write_offset_hh(f, &utc_offset)
                }
            }
            Spec::TimeZoneName => {
                let tz_name = time.time_zone();
                if !tz_name.is_empty() {
                    assert!(tz_name.is_ascii());

                    if !self.flags.contains(Flags::LEFT_PADDING) {
                        self.write_padding(f, tz_name.len())?;
                    }

                    let convert: fn(&u8) -> u8 = if self.flags.contains(Flags::CHANGE_CASE) {
                        u8::to_ascii_lowercase
                    } else if self.flags.contains(Flags::UPPER_CASE) {
                        u8::to_ascii_uppercase
                    } else {
                        |&x| x
                    };

                    for x in tz_name.as_bytes() {
                        f.write(&[convert(x)])?;
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

                write!(f, "{week_day_name} {month_name} ")?;
                write!(f, "{day: >2} {hour:02}:{minute:02}:{second:02} ")?;
                write!(f, "{year:0default_year_width$}")
            }
            Spec::CombinationDate => {
                self.write_padding(f, "mm/dd/yy".len())?;

                let year = time.year().rem_euclid(100);
                let month = time.month();
                let day = time.day();

                write!(f, "{month:02}/{day:02}/{year:02}")
            }
            Spec::CombinationIso8601 => {
                const MIN_WIDTH_NO_YEAR: usize = "-mm-dd".len();

                let year = time.year();
                let default_year_width = if year < 0 { 5 } else { 4 };
                let min_width = MIN_WIDTH_NO_YEAR + year_width(year).max(default_year_width);
                self.write_padding(f, min_width)?;

                let month = time.month();
                let day = time.day();

                write!(f, "{year:0default_year_width$}-{month:02}-{day:02}")
            }
            Spec::CombinationVmsDate => {
                let year = time.year();
                self.write_padding(f, "dd-mmm-".len() + year_width(year).max(4))?;

                let month_name = &MONTHS_UPPER[(time.month() - 1) as usize][..3];
                let day = time.day();

                write!(f, "{day: >2}-{month_name}-{year:04}")
            }
            Spec::CombinationTime12h => {
                self.write_padding(f, "HH:MM:SS PM".len())?;

                let hour = time.hour() % 12;
                let hour = if hour == 0 { 12 } else { hour };

                let (minute, second) = (time.minute(), time.second());
                let meridian = if time.hour() < 12 { "AM" } else { "PM" };

                write!(f, "{hour:02}:{minute:02}:{second:02} {meridian}")
            }
            Spec::CombinationHourMinute24h => {
                self.write_padding(f, "HH:MM".len())?;
                let (hour, minute) = (time.hour(), time.minute());
                write!(f, "{hour:02}:{minute:02}")
            }
            Spec::CombinationTime24h => {
                self.write_padding(f, "HH:MM:SS".len())?;
                let (hour, minute, second) = (time.hour(), time.minute(), time.second());
                write!(f, "{hour:02}:{minute:02}:{second:02}")
            }
        }
    }
}

/// Wrapper struct for formatting time with the provided format string.
pub(crate) struct TimeFormatter<'t, 'f, T> {
    /// Time implementation
    time: &'t T,
    /// Format string
    format: &'f [u8],
}

impl<'t, 'f, T: Time> TimeFormatter<'t, 'f, T> {
    /// Construct a new `TimeFormatter` wrapper.
    pub(crate) fn new<F: AsRef<[u8]> + ?Sized>(time: &'t T, format: &'f F) -> Self {
        Self {
            time,
            format: format.as_ref(),
        }
    }

    /// Format time using the format string.
    pub(crate) fn fmt(&self, buf: &mut dyn Write) -> Result<(), Error> {
        // Do nothing if the format string is empty
        if self.format.is_empty() {
            return Ok(());
        }

        // Use a size limiter to limit the maximum size of the resulting
        // formatted string
        let size_limit = self.format.len().saturating_mul(512 * 1024);
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

    /// Parse a formatting directive.
    fn parse_spec(cursor: &mut Cursor<'_>) -> Result<Option<Piece>, Error> {
        // Parse flags
        let mut padding = Padding::Left;
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
            Ok(width) if Int::try_from(width).is_ok() => Some(width),
            Err(err) if *err.kind() == IntErrorKind::Empty => None,
            _ => return Ok(None),
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

/// Compute the width of the string representation of a year.
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
