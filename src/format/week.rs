//! Module containing week-related items.

/// Start day of the week.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum WeekStart {
    /// Sunday.
    Sunday = 0,
    /// Monday.
    Monday = 1,
}

/// Compute the week number, beginning at the provided start day of the week.
///
/// ## Inputs
///
/// * `week_day`: Day of the week from Sunday in `0..=6`.
/// * `year_day_1`: Day of the year in `1..=366`.
/// * `week_start`: Start day of the week.
///
pub(crate) fn week_number(week_day: i64, year_day_1: i64, week_start: WeekStart) -> i64 {
    let year_day = year_day_1 - 1;
    let start_of_first_week = (year_day - week_day + week_start as i64).rem_euclid(7);
    (year_day + 7 - start_of_first_week) / 7
}

/// Compute the ISO 8601 week-based year and week number.
///
/// The first week of `YYYY` starts with a Monday and includes `YYYY-01-04`.
/// The days in the year before the first week are in the last week of the
/// previous year.
///
/// ## Inputs
///
/// * `year`: Year.
/// * `week_day`: Day of the week from Sunday in `0..=6`.
/// * `year_day_1`: Day of the year in `1..=366`.
///
pub(crate) fn iso_8601_year_and_week_number(
    year: i64,
    week_day: i64,
    year_day_1: i64,
) -> (i64, i64) {
    let year_day = year_day_1 - 1;

    let mut start_of_first_week = (year_day - week_day + 1).rem_euclid(7);

    if start_of_first_week > 3 {
        start_of_first_week -= 7;
    }

    if year_day < start_of_first_week {
        // Use previous year
        let previous_year = year - 1;

        let previous_year_day = if is_leap_year(previous_year) {
            366 + year_day
        } else {
            365 + year_day
        };

        return iso_8601_year_and_week_number(previous_year, week_day, previous_year_day + 1);
    }

    let week_number = (year_day + 7 - start_of_first_week) / 7;

    if week_number >= 52 {
        let last_year_day = if is_leap_year(year) { 365 } else { 364 };

        let week_day_of_last_year_day = (week_day + last_year_day - year_day) % 7;

        if (1..=3).contains(&week_day_of_last_year_day) {
            let last_monday = last_year_day - (week_day_of_last_year_day - 1);
            if year_day >= last_monday {
                // Use next year
                return (year + 1, 1);
            }
        }
    }

    // Use current year
    (year, week_number)
}

/// Check if a year is a leap year.
fn is_leap_year(year: i64) -> bool {
    year % 400 == 0 || (year % 4 == 0 && year % 100 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_week_number() {
        assert_eq!(week_number(1, 0, WeekStart::Sunday), 0);
        assert_eq!(week_number(2, 1, WeekStart::Sunday), 0);
        assert_eq!(week_number(3, 2, WeekStart::Sunday), 0);
        assert_eq!(week_number(4, 3, WeekStart::Sunday), 0);
        assert_eq!(week_number(5, 4, WeekStart::Sunday), 0);
        assert_eq!(week_number(6, 5, WeekStart::Sunday), 0);
        assert_eq!(week_number(0, 6, WeekStart::Sunday), 1);
        assert_eq!(week_number(1, 7, WeekStart::Sunday), 1);
        assert_eq!(week_number(2, 8, WeekStart::Sunday), 1);

        assert_eq!(week_number(0, 0, WeekStart::Monday), 0);
        assert_eq!(week_number(1, 1, WeekStart::Monday), 1);
        assert_eq!(week_number(2, 2, WeekStart::Monday), 1);
        assert_eq!(week_number(3, 3, WeekStart::Monday), 1);
        assert_eq!(week_number(4, 4, WeekStart::Monday), 1);
        assert_eq!(week_number(5, 5, WeekStart::Monday), 1);
        assert_eq!(week_number(6, 6, WeekStart::Monday), 1);
        assert_eq!(week_number(7, 7, WeekStart::Monday), 1);
        assert_eq!(week_number(8, 8, WeekStart::Monday), 2);

        assert_eq!(week_number(0, 365, WeekStart::Sunday), 53);
    }

    #[test]
    fn test_iso_8601_year_and_week() {
        assert_eq!(iso_8601_year_and_week_number(2025, 0, 362), (2025, 52));
        assert_eq!(iso_8601_year_and_week_number(2025, 1, 363), (2026, 1));
        assert_eq!(iso_8601_year_and_week_number(2025, 2, 364), (2026, 1));
        assert_eq!(iso_8601_year_and_week_number(2025, 3, 365), (2026, 1));
        assert_eq!(iso_8601_year_and_week_number(2026, 4, 1), (2026, 1));
        assert_eq!(iso_8601_year_and_week_number(2026, 5, 2), (2026, 1));
        assert_eq!(iso_8601_year_and_week_number(2026, 6, 3), (2026, 1));
        assert_eq!(iso_8601_year_and_week_number(2026, 0, 4), (2026, 1));
        assert_eq!(iso_8601_year_and_week_number(2026, 1, 5), (2026, 2));

        assert_eq!(iso_8601_year_and_week_number(2026, 0, 361), (2026, 52));
        assert_eq!(iso_8601_year_and_week_number(2026, 1, 362), (2026, 53));
        assert_eq!(iso_8601_year_and_week_number(2026, 2, 363), (2026, 53));
        assert_eq!(iso_8601_year_and_week_number(2026, 3, 364), (2026, 53));
        assert_eq!(iso_8601_year_and_week_number(2026, 4, 365), (2026, 53));
        assert_eq!(iso_8601_year_and_week_number(2027, 5, 1), (2026, 53));
        assert_eq!(iso_8601_year_and_week_number(2027, 6, 2), (2026, 53));
        assert_eq!(iso_8601_year_and_week_number(2027, 0, 3), (2026, 53));
        assert_eq!(iso_8601_year_and_week_number(2027, 1, 4), (2027, 1));

        assert_eq!(iso_8601_year_and_week_number(2020, 0, 362), (2020, 52));
        assert_eq!(iso_8601_year_and_week_number(2020, 1, 363), (2020, 53));
        assert_eq!(iso_8601_year_and_week_number(2020, 2, 364), (2020, 53));
        assert_eq!(iso_8601_year_and_week_number(2020, 3, 365), (2020, 53));
        assert_eq!(iso_8601_year_and_week_number(2020, 4, 366), (2020, 53));
        assert_eq!(iso_8601_year_and_week_number(2021, 5, 1), (2020, 53));
        assert_eq!(iso_8601_year_and_week_number(2021, 6, 2), (2020, 53));
        assert_eq!(iso_8601_year_and_week_number(2021, 0, 3), (2020, 53));
        assert_eq!(iso_8601_year_and_week_number(2021, 1, 4), (2021, 1));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(2001));
        assert!(is_leap_year(2004));
        assert!(!is_leap_year(2100));
        assert!(!is_leap_year(2200));
        assert!(!is_leap_year(2300));
        assert!(is_leap_year(2400));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_week_start_debug_is_non_empty() {
        use alloc::format;

        assert!(!format!("{:?}", WeekStart::Sunday).is_empty());
        assert!(!format!("{:?}", WeekStart::Monday).is_empty());
    }
}
