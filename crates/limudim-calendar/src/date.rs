use hebrew_holiday_calendar::{HebrewHolidayCalendar, HebrewMonth};
use icu_calendar::{
    cal::Hebrew,
    options::DateAddOptions,
    types::{DateDuration, Weekday},
    Date, Gregorian,
};

pub type HebrewDate = Date<Hebrew>;

#[allow(clippy::expect_used)]
pub(crate) fn from_gregorian_date(year: i32, month: u8, day: u8) -> HebrewDate {
    let gregorian = Date::<Gregorian>::try_new_gregorian(year, month, day).expect("valid gregorian date");
    gregorian.to_calendar(Hebrew)
}
#[allow(clippy::expect_used)]
pub(crate) fn from_hebrew_date(year: i32, month: HebrewMonth, day: u8) -> HebrewDate {
    Date::<Hebrew>::from_hebrew_date(year, month, day).expect("valid hebrew date")
}

pub(crate) trait DateExt {
    fn add_days(&self, days: i32) -> Option<HebrewDate>;
    /// Returns the day of week as a number (Sunday = 1, Saturday = 7)
    fn day_of_week_number(&self) -> i32;
}
impl DateExt for HebrewDate {
    fn add_days(&self, days: i32) -> Option<HebrewDate> {
        let duration = DateDuration::for_days(days as i64);
        self.try_added_with_options(duration, DateAddOptions::default()).ok()
    }

    fn day_of_week_number(&self) -> i32 {
        match self.day_of_week() {
            Weekday::Sunday => 1,
            Weekday::Monday => 2,
            Weekday::Tuesday => 3,
            Weekday::Wednesday => 4,
            Weekday::Thursday => 5,
            Weekday::Friday => 6,
            Weekday::Saturday => 7,
        }
    }
}
