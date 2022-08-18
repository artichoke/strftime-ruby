use arbitrary::Arbitrary;
use strftime::Time;

macro_rules! create_mock_time {
    ($($field_name:ident: $field_type:ty),*,) => {
        #[derive(Debug, Arbitrary)]
        pub(super) struct MockTime<'a> {
            $($field_name: $field_type),*,
        }

        impl<'a> MockTime<'a> {
            #[allow(clippy::too_many_arguments)]
            fn new($($field_name: $field_type),*) -> Self {
                Self { $($field_name),* }
            }
        }

        impl<'a> Time for MockTime<'a> {
            $(fn $field_name(&self) -> $field_type { self.$field_name })*
        }
    };
}

create_mock_time!(
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
