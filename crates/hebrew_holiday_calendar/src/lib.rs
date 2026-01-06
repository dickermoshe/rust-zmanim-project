//! # Hebrew Holiday Calendar
//!
//! A comprehensive library for working with the Hebrew calendar, including holidays,
//! Torah readings (parshiyot), and calendar calculations.
//!
//! This library provides no_std support and can be used in embedded environments.
//!
//! ## Features
//!
//! - Determine Jewish holidays
//! - Find weekly Torah readings (parshiyot) for any date
//! - Identify special Shabbatot (Shekalim, Zachor, etc.)
//! - Support for both Israel and Diaspora customs
//! - Fast day and candle lighting determinations
//!
//! ## Example
//!
//! ```no_run
//! use hebrew_holiday_calendar::*;
//! use icu_calendar::{Date, cal::Hebrew};
//!
//! // Create a Hebrew date
//! let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 1).unwrap();
//!
//! // Get holidays for that date
//! for holiday in date.holidays(false, false) {
//!     println!("Holiday: {:?}", holiday);
//! }
//!
//! // Get the weekly parsha
//! if let Some(parsha) = date.todays_parsha(false) {
//!     println!("Parsha: {}", parsha.he());
//! }
//! ```

use core::ops::RangeInclusive;
use core::slice::Iter;
#[cfg(test)]
mod java_tests;
mod parshas;
use crate::parshas::*;
use chrono::Weekday;
use icu_calendar::options::DateAddOptions;
use icu_calendar::types::{DateDuration, MonthCode, Weekday as IcuWeekday};
use icu_calendar::{cal::Hebrew, Date, Gregorian};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Number of chalakim (parts) from the molad tohu (theoretical first new moon)
pub(crate) const CHALAKIM_MOLAD_TOHU: i64 = 31524;
/// Number of chalakim in a lunar month
pub(crate) const CHALAKIM_PER_MONTH: i64 = 765433;
/// Number of chalakim in a day
pub(crate) const CHALAKIM_PER_DAY: i64 = 25920;

/// Iterator that yields holidays occurring on a specific Hebrew date.
///
/// This iterator filters through all possible holidays and returns only those
/// that occur on the given date, respecting both location (Israel vs. Diaspora)
/// and whether to include modern holidays.
pub struct HolidayIterator {
    iter: Iter<'static, Holiday>,
    date: Date<Hebrew>,
    in_israel: bool,
    use_modern_holidays: bool,
}

impl Iterator for HolidayIterator {
    type Item = &'static Holiday;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let holiday = self.iter.next()?;
            if holiday.rule().is_today(&self.date, self.in_israel)
                && (self.use_modern_holidays || !holiday.is_modern_holiday())
            {
                return Some(holiday);
            }
        }
    }
}

/// Trait providing Hebrew calendar functionality for dates.
///
/// This trait extends the capabilities of Hebrew dates with Jewish calendar-specific
/// operations including holiday determination, Torah readings, and calendar calculations.
pub trait HebrewHolidayCalendar {
    type HolidayIter: Iterator<Item = &'static Holiday>;

    /// Returns a copy of this date converted to the Gregorian calendar.
    fn gregorian_date(&self) -> Date<Gregorian>;

    /// Returns the current month as a `HebrewMonth` enum.
    fn hebrew_month(&self) -> HebrewMonth;

    /// Returns the day of week as a `chrono::Weekday`.
    fn chrono_day_of_week(&self) -> chrono::Weekday;

    /// Returns the number of days in the given Hebrew year.
    fn days_in_hebrew_year(year: i32) -> i32;

    /// Returns the number of days in the given Hebrew month for a specific year.
    fn days_in_hebrew_month(year: i32, month: HebrewMonth) -> u8;

    /// Returns whether Cheshvan has 30 days (long) in the given year.
    fn is_cheshvan_long(year: i32) -> bool;

    /// Returns whether Kislev has 29 days (short) in the given year.
    fn is_kislev_short(year: i32) -> bool;

    /// Returns whether the given year is a Hebrew leap year.
    fn is_hebrew_leap_year(year: i32) -> bool;

    /// Returns the year type based on the lengths of Cheshvan and Kislev.
    fn cheshvan_kislev_kviah(year: i32) -> YearLengthType;

    /// Returns an iterator over holidays occurring on this date.
    ///
    /// # Arguments
    ///
    /// * `in_israel` - Whether to use Israeli customs (affects second day observances)
    /// * `use_modern_holidays` - Whether to include modern Israeli holidays
    fn holidays(&self, in_israel: bool, use_modern_holidays: bool) -> Self::HolidayIter;

    /// Returns whether work is forbidden (assur bemelacha) on this date.
    fn is_assur_bemelacha(&self, in_israel: bool) -> bool;

    /// Returns whether candle lighting should occur on this date (day before Yom Tov/Shabbat).
    fn has_candle_lighting(&self, in_israel: bool) -> bool;

    /// Returns whether this date falls during the Ten Days of Repentance.
    fn is_aseres_yemei_teshuva(&self) -> bool;

    /// Returns the weekly Torah reading (parsha) if this is Shabbat and not a holiday.
    fn todays_parsha(&self, in_israel: bool) -> Option<Parsha>;

    /// Returns the special Shabbat designation if applicable (e.g., Shekalim, Zachor).
    fn special_parsha(&self, in_israel: bool) -> Option<Parsha>;

    /// Returns the upcoming Shabbat's Torah reading, skipping over holidays.
    fn upcoming_parsha(&self, in_israel: bool) -> Parsha;

    /// Returns the day of Chanukah. None if not Chanukah.
    fn day_of_chanukah(&self) -> Option<u8>;

    /// Returns the day of the Omer. None if not counting the Omer.
    fn day_of_the_omer(&self) -> Option<u8>;

    /// Creates a new Hebrew date from a year, month, and day.
    fn from_hebrew_date(year: i32, month: HebrewMonth, day: u8) -> Option<Date<Hebrew>> {
        let is_leap_year =
            Date::try_new_from_codes(Some("am"), year, MonthCode("M01".parse().ok()?), 1, Hebrew)
                .ok()?
                .is_in_leap_year();

        let month_code: MonthCode = match is_leap_year {
            true => {
                let month_code_str = match month {
                    HebrewMonth::Tishrei => "M01",
                    HebrewMonth::Cheshvan => "M02",
                    HebrewMonth::Kislev => "M03",
                    HebrewMonth::Teves => "M04",
                    HebrewMonth::Shevat => "M05",
                    HebrewMonth::Adar => "M05L",
                    HebrewMonth::AdarII => "M06",
                    HebrewMonth::Nissan => "M07",
                    HebrewMonth::Iyar => "M08",
                    HebrewMonth::Sivan => "M09",
                    HebrewMonth::Tammuz => "M10",
                    HebrewMonth::Av => "M11",
                    HebrewMonth::Elul => "M12",
                };

                MonthCode(month_code_str.parse().ok()?)
            }
            false => {
                let month_code_str = match month {
                    HebrewMonth::Tishrei => "M01",
                    HebrewMonth::Cheshvan => "M02",
                    HebrewMonth::Kislev => "M03",
                    HebrewMonth::Teves => "M04",
                    HebrewMonth::Shevat => "M05",
                    HebrewMonth::Adar => "M06",
                    HebrewMonth::Nissan => "M07",
                    HebrewMonth::Iyar => "M08",
                    HebrewMonth::Sivan => "M09",
                    HebrewMonth::Tammuz => "M10",
                    HebrewMonth::Av => "M11",
                    HebrewMonth::Elul => "M12",
                    _ => return None,
                };
                MonthCode(month_code_str.parse().ok()?)
            }
        };

        let hebrew_date = Date::try_new_from_codes(Some("am"), year, month_code, day, Hebrew);

        let hebrew_date = hebrew_date.ok()?;
        Some(hebrew_date)
    }
}

fn get_parsha_list(&date: &Date<Hebrew>, in_israel: bool) -> Option<ParshaList> {
    let rosh_hashana_day_of_week = (get_hebrew_elapsed_days(date.extended_year()) + 1) % 7;
    let rosh_hashana_day_of_week = match rosh_hashana_day_of_week {
        0 => Some(Weekday::Sat),
        1 => Some(Weekday::Sun),
        2 => Some(Weekday::Mon),
        3 => Some(Weekday::Tue),
        4 => Some(Weekday::Wed),
        5 => Some(Weekday::Thu),
        6 => Some(Weekday::Fri),
        _ => None,
    }?;
    let is_kislev_short = Date::<Hebrew>::is_kislev_short(date.extended_year());
    let is_cheshvan_long = Date::<Hebrew>::is_cheshvan_long(date.extended_year());

    if date.is_in_leap_year() {
        match rosh_hashana_day_of_week {
            Weekday::Mon => {
                if is_kislev_short {
                    if in_israel {
                        Some(PARSHA_LIST_14)
                    } else {
                        Some(PARSHA_LIST_6)
                    }
                } else if is_cheshvan_long {
                    if in_israel {
                        Some(PARSHA_LIST_15)
                    } else {
                        Some(PARSHA_LIST_7)
                    }
                } else {
                    None
                }
            }
            Weekday::Tue => {
                if in_israel {
                    Some(PARSHA_LIST_15)
                } else {
                    Some(PARSHA_LIST_7)
                }
            }
            Weekday::Thu => {
                if is_kislev_short {
                    Some(PARSHA_LIST_8)
                } else if is_cheshvan_long {
                    Some(PARSHA_LIST_9)
                } else {
                    None
                }
            }
            Weekday::Sat => {
                if is_kislev_short {
                    Some(PARSHA_LIST_10)
                } else if is_cheshvan_long {
                    if in_israel {
                        Some(PARSHA_LIST_16)
                    } else {
                        Some(PARSHA_LIST_11)
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        match rosh_hashana_day_of_week {
            Weekday::Mon => {
                if is_kislev_short {
                    Some(PARSHA_LIST_0)
                } else if is_cheshvan_long {
                    if in_israel {
                        Some(PARSHA_LIST_12)
                    } else {
                        Some(PARSHA_LIST_1)
                    }
                } else {
                    None
                }
            }
            Weekday::Tue => {
                if in_israel {
                    Some(PARSHA_LIST_12)
                } else {
                    Some(PARSHA_LIST_1)
                }
            }
            Weekday::Thu => {
                if is_cheshvan_long {
                    Some(PARSHA_LIST_3)
                } else if !is_kislev_short {
                    if in_israel {
                        Some(PARSHA_LIST_13)
                    } else {
                        Some(PARSHA_LIST_2)
                    }
                } else {
                    None
                }
            }
            Weekday::Sat => {
                if is_kislev_short {
                    Some(PARSHA_LIST_4)
                } else if is_cheshvan_long {
                    Some(PARSHA_LIST_5)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Returns the `HebrewMonth` as a `u8` which is indexed starting from Tishrei
/// instead of Nissan.
fn hebrew_month_of_year(year: i32, month: HebrewMonth) -> u8 {
    let is_leap_year = Date::<Hebrew>::is_hebrew_leap_year(year);
    (month as u8 + if is_leap_year { 6 } else { 5 }) % if is_leap_year { 13 } else { 12 } + 1
}

/// Advances to the next Hebrew month, handling year boundaries and leap years.
///
/// Returns a tuple of (new_year, new_month).
fn next_hebrew_month(year: i32, month: HebrewMonth) -> (i32, HebrewMonth) {
    match month {
        HebrewMonth::Elul => (year + 1, HebrewMonth::Tishrei),
        HebrewMonth::Adar if !Date::<Hebrew>::is_hebrew_leap_year(year) => {
            (year, HebrewMonth::Nissan)
        }
        HebrewMonth::AdarII => (year, HebrewMonth::Nissan),
        _ => {
            let month_num: u8 = month.into();
            let next_month = (month_num + 1).try_into().unwrap_or(HebrewMonth::Nissan);
            (year, next_month)
        }
    }
}

/// Adds days to a Hebrew date, handling month and year overflow.
///
/// Returns a tuple of (year, month, day) or None if the operation fails.
fn add_days_to_hebrew_date(
    year: i32,
    month: HebrewMonth,
    day: u8,
    days_to_add: u8,
) -> Option<(i32, HebrewMonth, u8)> {
    let mut current_year = year;
    let mut current_month = month;
    let mut current_day = day + days_to_add;

    loop {
        let days_in_month = Date::<Hebrew>::days_in_hebrew_month(current_year, current_month);
        if current_day <= days_in_month {
            return Some((current_year, current_month, current_day));
        }
        current_day -= days_in_month;
        (current_year, current_month) = next_hebrew_month(current_year, current_month);
    }
}
/// Returns the number of chalakim from the original hypothetical Molad Tohu
fn chalakim_since_molad_tohu(year: i32, month: HebrewMonth) -> i64 {
    let month_of_year = hebrew_month_of_year(year, month);
    let months_elapsed = (235 * ((year - 1) / 19))
        + (12 * ((year - 1) % 19))
        + ((7 * ((year - 1) % 19) + 1) / 19)
        + (month_of_year as i32 - 1);

    CHALAKIM_MOLAD_TOHU + (CHALAKIM_PER_MONTH * months_elapsed as i64)
}

/// Returns the number of days elapsed from the Sunday prior to the start of the Jewish calendar to the mean conjunction of Tishri of the Jewish year.
fn get_hebrew_elapsed_days(year: i32) -> i32 {
    let chalakim_since = chalakim_since_molad_tohu(year, HebrewMonth::Tishrei);
    let molad_day = chalakim_since / CHALAKIM_PER_DAY;
    let molad_parts = chalakim_since - molad_day * CHALAKIM_PER_DAY;
    let mut rosh_hashana_day = molad_day;

    if (molad_parts >= 19440)
        || (((molad_day % 7) == 2)
            && (molad_parts >= 9924)
            && !Date::<Hebrew>::is_hebrew_leap_year(year))
        || (((molad_day % 7) == 1)
            && (molad_parts >= 16789)
            && (Date::<Hebrew>::is_hebrew_leap_year(year - 1)))
    {
        rosh_hashana_day += 1;
    }

    if ((rosh_hashana_day % 7) == 0)
        || ((rosh_hashana_day % 7) == 3)
        || ((rosh_hashana_day % 7) == 5)
    {
        rosh_hashana_day += 1;
    }

    rosh_hashana_day as i32
}

impl HebrewHolidayCalendar for Date<Hebrew> {
    type HolidayIter = HolidayIterator;

    #[inline]
    fn is_assur_bemelacha(&self, in_israel: bool) -> bool {
        self.holidays(in_israel, false)
            .any(|i| i.is_assur_bemelacha())
            || self.chrono_day_of_week() == Weekday::Sat
    }
    #[inline]
    fn has_candle_lighting(&self, in_israel: bool) -> bool {
        self.try_added_with_options(DateDuration::for_days(1), DateAddOptions::default())
            .map(|next_day| next_day.is_assur_bemelacha(in_israel))
            .unwrap_or(false)
    }

    #[inline]
    fn is_cheshvan_long(year: i32) -> bool {
        Self::days_in_hebrew_year(year) % 10 == 5
    }
    #[inline]
    fn is_kislev_short(year: i32) -> bool {
        Self::days_in_hebrew_year(year) % 10 == 3
    }
    #[inline]
    fn hebrew_month(&self) -> HebrewMonth {
        let month_code = self.month().formatting_code.0;
        match month_code.as_str() {
            "M01" => HebrewMonth::Tishrei,
            "M02" => HebrewMonth::Cheshvan,
            "M03" => HebrewMonth::Kislev,
            "M04" => HebrewMonth::Teves,
            "M05" => HebrewMonth::Shevat,
            "M05L" => HebrewMonth::Adar,
            "M06" => HebrewMonth::Adar,
            "M06L" => HebrewMonth::AdarII,
            "M07" => HebrewMonth::Nissan,
            "M08" => HebrewMonth::Iyar,
            "M09" => HebrewMonth::Sivan,
            "M10" => HebrewMonth::Tammuz,
            "M11" => HebrewMonth::Av,
            "M12" => HebrewMonth::Elul,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn gregorian_date(&self) -> Date<Gregorian> {
        self.to_calendar(Gregorian)
    }
    #[inline]
    fn chrono_day_of_week(&self) -> chrono::Weekday {
        let weekday = self.day_of_week();
        match weekday {
            IcuWeekday::Sunday => Weekday::Sun,
            IcuWeekday::Monday => Weekday::Mon,
            IcuWeekday::Tuesday => Weekday::Tue,
            IcuWeekday::Wednesday => Weekday::Wed,
            IcuWeekday::Thursday => Weekday::Thu,
            IcuWeekday::Friday => Weekday::Fri,
            IcuWeekday::Saturday => Weekday::Sat,
        }
    }
    #[inline]
    fn days_in_hebrew_year(year: i32) -> i32 {
        get_hebrew_elapsed_days(year + 1) - get_hebrew_elapsed_days(year)
    }
    #[inline]
    fn days_in_hebrew_month(year: i32, month: HebrewMonth) -> u8 {
        match month {
            HebrewMonth::Iyar | HebrewMonth::Tammuz | HebrewMonth::Elul | HebrewMonth::Teves => 29,
            HebrewMonth::Cheshvan => {
                if Self::is_cheshvan_long(year) {
                    30
                } else {
                    29
                }
            }
            HebrewMonth::Kislev => {
                if Self::is_kislev_short(year) {
                    29
                } else {
                    30
                }
            }
            HebrewMonth::Adar => {
                if Self::is_hebrew_leap_year(year) {
                    30
                } else {
                    29
                }
            }
            HebrewMonth::AdarII => 29,
            _ => 30,
        }
    }
    #[inline]
    fn is_hebrew_leap_year(year: i32) -> bool {
        let year_in_cycle = ((year - 1) % 19) + 1;
        matches!(year_in_cycle, 3 | 6 | 8 | 11 | 14 | 17 | 19)
    }
    #[inline]
    fn cheshvan_kislev_kviah(year: i32) -> YearLengthType {
        if Self::is_cheshvan_long(year) && !Self::is_kislev_short(year) {
            YearLengthType::Shelaimim
        } else if !Self::is_cheshvan_long(year) && Self::is_kislev_short(year) {
            YearLengthType::Chaserim
        } else {
            YearLengthType::Kesidran
        }
    }
    #[inline]
    fn holidays(&self, in_israel: bool, use_modern_holidays: bool) -> Self::HolidayIter {
        HolidayIterator {
            iter: Holiday::all().iter(),
            date: *self,
            in_israel,
            use_modern_holidays,
        }
    }

    fn is_aseres_yemei_teshuva(&self) -> bool {
        self.hebrew_month() == HebrewMonth::Tishrei && self.day_of_month().0 <= 10
    }

    fn todays_parsha(&self, in_israel: bool) -> Option<Parsha> {
        if self.chrono_day_of_week() != Weekday::Sat {
            return None;
        }

        let parsha_list = get_parsha_list(self, in_israel)?;

        let rosh_hashana_day_of_week = get_hebrew_elapsed_days(self.extended_year()) % 7;
        let day = rosh_hashana_day_of_week + self.day_of_year().0 as i32;
        parsha_list[(day / 7) as usize]
    }

    fn special_parsha(&self, in_israel: bool) -> Option<Parsha> {
        if self.chrono_day_of_week() != Weekday::Sat {
            return None;
        }

        let month = self.hebrew_month();
        let day = self.day_of_month().0;
        let is_leap = Date::<Hebrew>::is_hebrew_leap_year(self.extended_year());

        // Shkalim
        if ((month == HebrewMonth::Shevat && !is_leap) || (month == HebrewMonth::Adar && is_leap))
            && (day == 25 || day == 27 || day == 29)
        {
            return Some(Parsha::Shekalim);
        }

        if (month == HebrewMonth::Adar && !is_leap) || month == HebrewMonth::AdarII {
            if day == 1 {
                return Some(Parsha::Shekalim);
            }
            // Zachor
            if day == 8 || day == 9 || day == 11 || day == 13 {
                return Some(Parsha::Zachor);
            }
            // Para
            if day == 18 || day == 20 || day == 22 || day == 23 {
                return Some(Parsha::Parah);
            }
            // Hachodesh
            if day == 25 || day == 27 || day == 29 {
                return Some(Parsha::Hachodesh);
            }
        }

        if month == HebrewMonth::Nissan {
            if day == 1 {
                return Some(Parsha::Hachodesh);
            }
            // Hagadol
            if (8..=14).contains(&day) {
                return Some(Parsha::Hagadol);
            }
        }

        if month == HebrewMonth::Av {
            // Chazon
            if (4..=9).contains(&day) {
                return Some(Parsha::Chazon);
            }
            // Nachamu
            if (10..=16).contains(&day) {
                return Some(Parsha::Nachamu);
            }
        }

        if month == HebrewMonth::Tishrei {
            // Shuva
            if (3..=8).contains(&day) {
                return Some(Parsha::Shuva);
            }
        }

        // Shira
        if self.todays_parsha(in_israel) == Some(Parsha::Beshalach) {
            return Some(Parsha::Shira);
        }

        None
    }

    #[allow(clippy::expect_used)] // This is an internal algorithm that should never fail with valid Hebrew dates
    fn upcoming_parsha(&self, in_israel: bool) -> Parsha {
        // Calculate days to next Shabbos
        let day_of_week = self.chrono_day_of_week();
        let days_to_shabbos = match day_of_week {
            Weekday::Mon => 5,
            Weekday::Tue => 4,
            Weekday::Wed => 3,
            Weekday::Thu => 2,
            Weekday::Fri => 1,
            Weekday::Sat => 7,
            Weekday::Sun => 6,
        };

        // Create a new calendar for the upcoming Shabbos
        let (mut current_year, mut current_month, mut current_day) = add_days_to_hebrew_date(
            self.extended_year(),
            self.hebrew_month(),
            self.day_of_month().0,
            days_to_shabbos,
        )
        .expect("Failed to calculate upcoming Shabbos");

        // Get parshah for that date, advancing by weeks if it's a Yom Tov
        loop {
            let date = Date::<Hebrew>::from_hebrew_date(current_year, current_month, current_day)
                .expect("Failed to create Hebrew date");

            if let Some(parshah) = date.todays_parsha(in_israel) {
                return parshah;
            }

            // Advance by 7 days
            (current_year, current_month, current_day) =
                add_days_to_hebrew_date(current_year, current_month, current_day, 7)
                    .expect("Failed to advance to next Shabbos");
        }
    }

    fn day_of_chanukah(&self) -> Option<u8> {
        if !self.holidays(false, false).any(|i| i == &Holiday::Chanukah) {
            return None;
        }

        let month = self.hebrew_month();
        let day = self.day_of_month().0;

        if month == HebrewMonth::Kislev {
            Some(day - 24)
        } else if Self::is_kislev_short(self.extended_year()) {
            Some(day + 5)
        } else {
            Some(day + 6)
        }
    }

    fn day_of_the_omer(&self) -> Option<u8> {
        let month = self.hebrew_month();
        let day = self.day_of_month().0;

        if month == HebrewMonth::Nissan && day >= 16 {
            Some(day - 15)
        } else if month == HebrewMonth::Iyar {
            Some(day + 15)
        } else if month == HebrewMonth::Sivan && day < 6 {
            Some(day + 44)
        } else {
            None
        }
    }
}

/// Represents the weekly Torah readings (parshiyot) and special Shabbatot.
///
/// This enum includes all 54 Torah portions from the annual cycle, combined readings
/// for non-leap years, and special Shabbatot like Shekalim, Zachor, etc.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Parsha {
    Bereshis = 0,
    Noach = 1,
    LechLecha = 2,
    Vayera = 3,
    ChayeiSara = 4,
    Toldos = 5,
    Vayetzei = 6,
    Vayishlach = 7,
    Vayeshev = 8,
    Miketz = 9,
    Vayigash = 10,
    Vayechi = 11,
    Shemos = 12,
    Vaera = 13,
    Bo = 14,
    Beshalach = 15,
    Yisro = 16,
    Mishpatim = 17,
    Terumah = 18,
    Tetzaveh = 19,
    KiSisa = 20,
    Vayakhel = 21,
    Pekudei = 22,
    Vayikra = 23,
    Tzav = 24,
    Shmini = 25,
    Tazria = 26,
    Metzora = 27,
    AchreiMos = 28,
    Kedoshim = 29,
    Emor = 30,
    Behar = 31,
    Bechukosai = 32,
    Bamidbar = 33,
    Nasso = 34,
    Behaaloscha = 35,
    Shlach = 36,
    Korach = 37,
    Chukas = 38,
    Balak = 39,
    Pinchas = 40,
    Matos = 41,
    Masei = 42,
    Devarim = 43,
    Vaeschanan = 44,
    Eikev = 45,
    Reeh = 46,
    Shoftim = 47,
    KiSeitzei = 48,
    KiSavo = 49,
    Nitzavim = 50,
    Vayeilech = 51,
    HaAzinu = 52,
    VezosHabracha = 53,
    VayakhelPekudei = 54,
    TazriaMetzora = 55,
    AchreiMosKedoshim = 56,
    BeharBechukosai = 57,
    ChukasBalak = 58,
    MatosMasei = 59,
    NitzavimVayeilech = 60,
    Shekalim = 61,
    Zachor = 62,
    Parah = 63,
    Hachodesh = 64,
    Shuva = 65,
    Shira = 66,
    Hagadol = 67,
    Chazon = 68,
    Nachamu = 69,
}
impl Parsha {
    /// Returns the Hebrew name of the parsha.
    pub fn he(&self) -> &str {
        match self {
            Parsha::Bereshis => "בראשית",
            Parsha::Noach => "נח",
            Parsha::LechLecha => "לך לך",
            Parsha::Vayera => "וירא",
            Parsha::ChayeiSara => "חיי שרה",
            Parsha::Toldos => "תולדות",
            Parsha::Vayetzei => "ויצא",
            Parsha::Vayishlach => "וישלח",
            Parsha::Vayeshev => "וישב",
            Parsha::Miketz => "מקץ",
            Parsha::Vayigash => "ויגש",
            Parsha::Vayechi => "ויחי",
            Parsha::Shemos => "שמות",
            Parsha::Vaera => "וארא",
            Parsha::Bo => "בא",
            Parsha::Beshalach => "בשלח",
            Parsha::Yisro => "יתרו",
            Parsha::Mishpatim => "משפטים",
            Parsha::Terumah => "תרומה",
            Parsha::Tetzaveh => "תצוה",
            Parsha::KiSisa => "כי תשא",
            Parsha::Vayakhel => "ויקהל",
            Parsha::Pekudei => "פקודי",
            Parsha::Vayikra => "ויקרא",
            Parsha::Tzav => "צו",
            Parsha::Shmini => "שמיני",
            Parsha::Tazria => "תזריע",
            Parsha::Metzora => "מצרע",
            Parsha::AchreiMos => "אחרי מות",
            Parsha::Kedoshim => "קדושים",
            Parsha::Emor => "אמור",
            Parsha::Behar => "בהר",
            Parsha::Bechukosai => "בחקתי",
            Parsha::Bamidbar => "במדבר",
            Parsha::Nasso => "נשא",
            Parsha::Behaaloscha => "בהעלתך",
            Parsha::Shlach => "שלח לך",
            Parsha::Korach => "קרח",
            Parsha::Chukas => "חוקת",
            Parsha::Balak => "בלק",
            Parsha::Pinchas => "פינחס",
            Parsha::Matos => "מטות",
            Parsha::Masei => "מסעי",
            Parsha::Devarim => "דברים",
            Parsha::Vaeschanan => "ואתחנן",
            Parsha::Eikev => "עקב",
            Parsha::Reeh => "ראה",
            Parsha::Shoftim => "שופטים",
            Parsha::KiSeitzei => "כי תצא",
            Parsha::KiSavo => "כי תבוא",
            Parsha::Nitzavim => "נצבים",
            Parsha::Vayeilech => "וילך",
            Parsha::HaAzinu => "האזינו",
            Parsha::VezosHabracha => "וזאת הברכה ",
            Parsha::VayakhelPekudei => "ויקהל פקודי",
            Parsha::TazriaMetzora => "תזריע מצרע",
            Parsha::AchreiMosKedoshim => "אחרי מות קדושים",
            Parsha::BeharBechukosai => "בהר בחקתי",
            Parsha::ChukasBalak => "חוקת בלק",
            Parsha::MatosMasei => "מטות מסעי",
            Parsha::NitzavimVayeilech => "נצבים וילך",
            Parsha::Shekalim => "שקלים",
            Parsha::Zachor => "זכור",
            Parsha::Parah => "פרה",
            Parsha::Hachodesh => "החדש",
            Parsha::Shuva => "שובה",
            Parsha::Shira => "שירה",
            Parsha::Hagadol => "הגדול",
            Parsha::Chazon => "חזון",
            Parsha::Nachamu => "נחמו",
        }
    }
}

impl core::fmt::Display for Parsha {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.he())
    }
}

/// Internal type representing different rules for when holidays occur.
enum HolidayRule<'a> {
    ExactDate(u8, HebrewMonth),
    ExactDates(RangeInclusive<u8>, HebrewMonth),
    ExactDateChutz(u8, HebrewMonth),
    ExactDateIsrael(u8, HebrewMonth),
    ExactDates2([&'a HolidayRule<'a>; 2]),
    ExactDates4([&'a HolidayRule<'a>; 4]),
    ExactDates6([&'a HolidayRule<'a>; 6]),
    Custom(fn(&Date<Hebrew>, bool) -> bool),
}
impl HolidayRule<'_> {
    fn is_today(&self, date: &Date<Hebrew>, in_israel: bool) -> bool {
        match self {
            HolidayRule::ExactDate(day, month) => {
                date.day_of_month().0 == *day && date.hebrew_month() == *month
            }
            HolidayRule::ExactDates(range, month) => {
                range.contains(&date.day_of_month().0) && date.hebrew_month() == *month
            }
            HolidayRule::ExactDateChutz(day, month) => {
                date.day_of_month().0 == *day && date.hebrew_month() == *month && !in_israel
            }
            HolidayRule::ExactDateIsrael(day, month) => {
                date.day_of_month().0 == *day && date.hebrew_month() == *month && in_israel
            }
            HolidayRule::ExactDates2(rules) => {
                rules.iter().any(|rule| rule.is_today(date, in_israel))
            }

            HolidayRule::ExactDates4(rules) => {
                rules.iter().any(|rule| rule.is_today(date, in_israel))
            }

            HolidayRule::ExactDates6(rules) => {
                rules.iter().any(|rule| rule.is_today(date, in_israel))
            }
            HolidayRule::Custom(func) => func(date, in_israel),
        }
    }
}

/// Represents Jewish holidays and special days in the Hebrew calendar.
///
/// This enum covers traditional holidays, fast days, modern Israeli holidays,
/// and other significant dates in the Jewish calendar.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Holiday {
    ErevPesach,
    Pesach,
    CholHamoed,
    PesachSheni,
    ErevShavuos,
    Shavuos,
    SeventeenthOfTammuz,
    TishahBav,
    TuBav,
    ErevRoshHashana,
    RoshHashana,
    FastOfGedalyah,
    ErevYomKippur,
    YomKippur,
    ErevSuccos,
    Succos,
    HoshanaRabbah,
    SheminiAtzeres,
    SimchasTorah,
    Chanukah,
    TenthOfTeves,
    TuBshvat,
    FastOfEsther,
    Purim,
    ShushanPurim,
    PurimKatan,
    RoshChodesh,
    YomHaShoah,
    YomHazikaron,
    YomHaatzmaut,
    YomYerushalayim,
    LagBomer,
    ShushanPurimKatan,
    IsruChag,
    YomKippurKatan,
    Behab,
    FastOfTheFirstborn,
    CountOfTheOmer,
    BirchasHachamah,
    MacharHachodesh,
    ShabbosMevarchim,
}
impl Holiday {
    /// Returns the internal rule that determines when this holiday occurs.
    fn rule(&self) -> HolidayRule<'_> {
        match self {
            Holiday::ErevPesach => HolidayRule::ExactDate(14, HebrewMonth::Nissan),
            Holiday::Pesach => HolidayRule::ExactDates4([
                &HolidayRule::ExactDate(15, HebrewMonth::Nissan),
                &HolidayRule::ExactDate(21, HebrewMonth::Nissan),
                &HolidayRule::ExactDateChutz(16, HebrewMonth::Nissan),
                &HolidayRule::ExactDateChutz(22, HebrewMonth::Nissan),
            ]),
            Holiday::CholHamoed => HolidayRule::ExactDates4([
                &HolidayRule::ExactDateIsrael(16, HebrewMonth::Nissan),
                &HolidayRule::ExactDates(17..=20, HebrewMonth::Nissan),
                &HolidayRule::ExactDateIsrael(16, HebrewMonth::Tishrei),
                &HolidayRule::ExactDates(17..=20, HebrewMonth::Tishrei),
            ]),
            Holiday::PesachSheni => HolidayRule::ExactDate(14, HebrewMonth::Iyar),
            Holiday::ErevShavuos => HolidayRule::ExactDate(5, HebrewMonth::Sivan),
            Holiday::Shavuos => HolidayRule::ExactDates2([
                &HolidayRule::ExactDate(6, HebrewMonth::Sivan),
                &HolidayRule::ExactDateChutz(7, HebrewMonth::Sivan),
            ]),
            Holiday::SeventeenthOfTammuz => HolidayRule::Custom(|date, _in_israel| {
                if date.hebrew_month() != HebrewMonth::Tammuz {
                    return false;
                }
                let day = date.day_of_month().0;
                let day_of_week = date.chrono_day_of_week();
                (day == 17 && day_of_week != Weekday::Sat)
                    || (day == 18 && day_of_week == Weekday::Sun)
            }),
            Holiday::TishahBav => HolidayRule::Custom(|date, _in_israel| {
                if date.hebrew_month() != HebrewMonth::Av {
                    return false;
                }
                let day = date.day_of_month().0;
                let day_of_week = date.chrono_day_of_week();
                (day_of_week == Weekday::Sun && day == 10)
                    || (day_of_week != Weekday::Sat && day == 9)
            }),
            Holiday::TuBav => HolidayRule::ExactDate(15, HebrewMonth::Av),
            Holiday::ErevRoshHashana => HolidayRule::ExactDate(29, HebrewMonth::Elul),
            Holiday::RoshHashana => HolidayRule::ExactDates(1..=2, HebrewMonth::Tishrei),
            Holiday::FastOfGedalyah => HolidayRule::Custom(|date, _in_israel| {
                if date.hebrew_month() != HebrewMonth::Tishrei {
                    return false;
                }
                let day = date.day_of_month().0;
                let day_of_week = date.chrono_day_of_week();
                (day == 3 && day_of_week != Weekday::Sat)
                    || (day == 4 && day_of_week == Weekday::Sun)
            }),
            Holiday::ErevYomKippur => HolidayRule::ExactDate(9, HebrewMonth::Tishrei),
            Holiday::YomKippur => HolidayRule::ExactDate(10, HebrewMonth::Tishrei),
            Holiday::ErevSuccos => HolidayRule::ExactDate(14, HebrewMonth::Tishrei),
            Holiday::Succos => HolidayRule::ExactDates2([
                &HolidayRule::ExactDate(15, HebrewMonth::Tishrei),
                &HolidayRule::ExactDateChutz(16, HebrewMonth::Tishrei),
            ]),

            Holiday::HoshanaRabbah => HolidayRule::ExactDate(21, HebrewMonth::Tishrei),
            Holiday::SheminiAtzeres => HolidayRule::ExactDate(22, HebrewMonth::Tishrei),
            Holiday::SimchasTorah => HolidayRule::ExactDateChutz(23, HebrewMonth::Tishrei),
            Holiday::Chanukah => HolidayRule::Custom(|date, _in_israel| {
                let month = date.hebrew_month();
                let day = date.day_of_month().0;
                if month == HebrewMonth::Kislev && day >= 25 {
                    return true;
                } else if month == HebrewMonth::Teves {
                    let is_kislev_short = Date::<Hebrew>::is_kislev_short(date.extended_year());
                    let max_teves_day = if is_kislev_short { 3 } else { 2 };
                    return day <= max_teves_day;
                }
                false
            }),

            Holiday::TenthOfTeves => HolidayRule::ExactDate(10, HebrewMonth::Teves),
            Holiday::TuBshvat => HolidayRule::ExactDate(15, HebrewMonth::Shevat),
            Holiday::FastOfEsther => HolidayRule::Custom(|date, _in_israel| {
                let month = date.hebrew_month();
                if month == HebrewMonth::AdarII
                    || (!date.is_in_leap_year() && month == HebrewMonth::Adar)
                {
                    let day = date.day_of_month().0;
                    let day_of_week = date.chrono_day_of_week();
                    ((day == 11 || day == 12) && day_of_week == Weekday::Thu)
                        || (day == 13
                            && !(day_of_week == Weekday::Fri || day_of_week == Weekday::Sat))
                } else {
                    false
                }
            }),
            Holiday::Purim => HolidayRule::Custom(|date, _in_israel| {
                let month = date.hebrew_month();
                let day = date.day_of_month().0;
                ((month == HebrewMonth::Adar && !date.is_in_leap_year())
                    || month == HebrewMonth::AdarII)
                    && day == 14
            }),
            Holiday::ShushanPurim => HolidayRule::Custom(|date, _in_israel| {
                let month = date.hebrew_month();
                let day = date.day_of_month().0;
                ((month == HebrewMonth::Adar && !date.is_in_leap_year())
                    || month == HebrewMonth::AdarII)
                    && day == 15
            }),
            Holiday::PurimKatan => HolidayRule::Custom(|date, _in_israel| {
                date.hebrew_month() == HebrewMonth::Adar
                    && date.is_in_leap_year()
                    && date.day_of_month().0 == 14
            }),
            Holiday::RoshChodesh => HolidayRule::Custom(|date, _in_israel| {
                (date.day_of_month().0 == 1 && date.hebrew_month() != HebrewMonth::Tishrei)
                    || date.day_of_month().0 == 30
            }),
            Holiday::YomHaShoah => HolidayRule::Custom(|date, _in_israel| {
                if date.hebrew_month() != HebrewMonth::Nissan {
                    return false;
                }
                let day = date.day_of_month().0;
                let day_of_week = date.chrono_day_of_week();
                (day == 26 && day_of_week == Weekday::Thu)
                    || (day == 28 && day_of_week == Weekday::Mon)
                    || (day == 27 && day_of_week != Weekday::Sun && day_of_week != Weekday::Fri)
            }),
            Holiday::YomHazikaron => HolidayRule::Custom(|date, _in_israel| {
                if date.hebrew_month() != HebrewMonth::Iyar {
                    return false;
                }
                let day = date.day_of_month().0;
                let day_of_week = date.chrono_day_of_week();
                (day == 4 && day_of_week == Weekday::Tue)
                    || ((day == 3 || day == 2) && day_of_week == Weekday::Wed)
                    || (day == 5 && day_of_week == Weekday::Mon)
            }),
            Holiday::YomHaatzmaut => HolidayRule::Custom(|date, _in_israel| {
                if date.hebrew_month() != HebrewMonth::Iyar {
                    return false;
                }
                let day = date.day_of_month().0;
                let day_of_week = date.chrono_day_of_week();
                (day == 5 && day_of_week == Weekday::Wed)
                    || ((day == 4 || day == 3) && day_of_week == Weekday::Thu)
                    || (day == 6 && day_of_week == Weekday::Tue)
            }),
            Holiday::YomYerushalayim => HolidayRule::ExactDate(28, HebrewMonth::Iyar),
            Holiday::LagBomer => HolidayRule::ExactDate(18, HebrewMonth::Iyar),
            Holiday::ShushanPurimKatan => HolidayRule::Custom(|date, _in_israel| {
                date.hebrew_month() == HebrewMonth::Adar
                    && date.is_in_leap_year()
                    && date.day_of_month().0 == 15
            }),
            Holiday::IsruChag => HolidayRule::ExactDates6([
                &HolidayRule::ExactDateIsrael(22, HebrewMonth::Nissan),
                &HolidayRule::ExactDateChutz(23, HebrewMonth::Nissan),
                &HolidayRule::ExactDateIsrael(7, HebrewMonth::Sivan),
                &HolidayRule::ExactDateChutz(8, HebrewMonth::Sivan),
                &HolidayRule::ExactDateIsrael(23, HebrewMonth::Tishrei),
                &HolidayRule::ExactDateChutz(24, HebrewMonth::Tishrei),
            ]),
            Holiday::YomKippurKatan => HolidayRule::Custom(|date, _in_israel| {
                let day_of_week = date.chrono_day_of_week();
                let month = date.hebrew_month();
                let day = date.day_of_month().0;

                // Not observed in Elul, Tishrei, Kislev, or Nissan
                if matches!(
                    month,
                    HebrewMonth::Elul
                        | HebrewMonth::Tishrei
                        | HebrewMonth::Kislev
                        | HebrewMonth::Nissan
                ) {
                    return false;
                }

                // On 29th if not Friday or Shabbos
                if day == 29 && day_of_week != Weekday::Fri && day_of_week != Weekday::Sat {
                    return true;
                }

                // On 27th or 28th if Thursday (moved back from Friday/Shabbos)
                (day == 27 || day == 28) && day_of_week == Weekday::Thu
            }),
            Holiday::Behab => HolidayRule::Custom(|date, _in_israel| {
                let day_of_week = date.chrono_day_of_week();
                let month = date.hebrew_month();
                let day = date.day_of_month().0;

                // BeHaB is only in Cheshvan and Iyar
                if month == HebrewMonth::Cheshvan || month == HebrewMonth::Iyar {
                    // Monday between 5-17 or Thursday between 8-13
                    return (day_of_week == Weekday::Mon && day > 4 && day < 18)
                        || (day_of_week == Weekday::Thu && day > 7 && day < 14);
                }
                false
            }),
            Holiday::FastOfTheFirstborn => HolidayRule::Custom(|date, _in_israel| {
                let month = date.hebrew_month();
                let day = date.day_of_month().0;
                let day_of_week = date.chrono_day_of_week();
                month == HebrewMonth::Nissan
                    && ((day == 14 && day_of_week != Weekday::Sat)
                        || (day == 12 && day_of_week == Weekday::Thu))
            }),
            Holiday::CountOfTheOmer => HolidayRule::Custom(|date, _in_israel| {
                let month = date.hebrew_month();
                let day = date.day_of_month().0;

                (month == HebrewMonth::Nissan && day >= 16)
                    || month == HebrewMonth::Iyar
                    || (month == HebrewMonth::Sivan && day < 6)
            }),
            Holiday::BirchasHachamah => HolidayRule::Custom(|date, _in_israel| {
                let elapsed_days = get_hebrew_elapsed_days(date.extended_year());
                let elapsed_days = elapsed_days + date.day_of_year().0 as i32;
                let cycle_length = 10227i32;
                (elapsed_days % cycle_length) == 172
            }),
            Holiday::MacharHachodesh => HolidayRule::Custom(|date, _in_israel| {
                date.chrono_day_of_week() == Weekday::Sat
                    && (date.day_of_month().0 == 30 || date.day_of_month().0 == 29)
            }),
            Holiday::ShabbosMevarchim => HolidayRule::Custom(|date, _in_israel| {
                date.chrono_day_of_week() == Weekday::Sat
                    && date.day_of_month().0 >= 23
                    && date.day_of_month().0 <= 29
                    && date.hebrew_month() != HebrewMonth::Elul
            }),
        }
    }

    /// Returns whether work is forbidden (assur bemelacha) on this holiday.
    pub fn is_assur_bemelacha(&self) -> bool {
        matches!(
            self,
            Holiday::Pesach
                | Holiday::Shavuos
                | Holiday::Succos
                | Holiday::SheminiAtzeres
                | Holiday::SimchasTorah
                | Holiday::RoshHashana
                | Holiday::YomKippur
        )
    }

    /// Returns whether this is a modern Israeli holiday.
    pub fn is_modern_holiday(&self) -> bool {
        matches!(
            self,
            Holiday::YomHaShoah
                | Holiday::YomHazikaron
                | Holiday::YomHaatzmaut
                | Holiday::YomYerushalayim
        )
    }

    /// Returns whether this holiday is a fast day.
    pub fn is_fast_day(&self) -> bool {
        matches!(
            self,
            Holiday::FastOfGedalyah
                | Holiday::FastOfEsther
                | Holiday::TishahBav
                | Holiday::TenthOfTeves
                | Holiday::SeventeenthOfTammuz
                | Holiday::YomKippur
        )
    }

    /// Returns a slice of all possible holidays.
    pub const fn all() -> &'static [Holiday; 41] {
        &ALL_HOLIDAYS
    }
    pub fn he(&self) -> &str {
        match self {
            Holiday::ErevPesach => "ערב פסח",
            Holiday::Pesach => "פסח",
            Holiday::CholHamoed => "חול המועד",
            Holiday::PesachSheni => "פסח שני",
            Holiday::ErevShavuos => "ערב שבועות",
            Holiday::Shavuos => "שבועות",
            Holiday::SeventeenthOfTammuz => "שבעה עשר בתמוז",
            Holiday::TishahBav => "תשעה באב",
            Holiday::TuBav => "ט״ו באב",
            Holiday::ErevRoshHashana => "ערב ראש השנה",
            Holiday::RoshHashana => "ראש השנה",
            Holiday::FastOfGedalyah => "צום גדליה",
            Holiday::ErevYomKippur => "ערב יום כיפור",
            Holiday::YomKippur => "יום כיפור",
            Holiday::ErevSuccos => "ערב סוכות",
            Holiday::Succos => "סוכות",
            Holiday::HoshanaRabbah => "הושענא רבה",
            Holiday::SheminiAtzeres => "שמיני עצרת",
            Holiday::SimchasTorah => "שמחת תורה",
            Holiday::Chanukah => "חנוכה",
            Holiday::TenthOfTeves => "עשרה בטבת",
            Holiday::TuBshvat => "ט״ו בשבט",
            Holiday::FastOfEsther => "תענית אסתר",
            Holiday::Purim => "פורים",
            Holiday::ShushanPurim => "שושן פורים",
            Holiday::PurimKatan => "פורים קטן",
            Holiday::RoshChodesh => "ראש חודש",
            Holiday::YomHaShoah => "יום השואה",
            Holiday::YomHazikaron => "יום הזיכרון",
            Holiday::YomHaatzmaut => "יום העצמאות",
            Holiday::YomYerushalayim => "יום ירושלים",
            Holiday::LagBomer => "ל״ג בעומר",
            Holiday::ShushanPurimKatan => "שושן פורים קטן",
            Holiday::IsruChag => "אסרו חג",
            Holiday::YomKippurKatan => "יום העצמאות",
            Holiday::Behab => "יום כיפור קטן",
            Holiday::FastOfTheFirstborn => "תענית בכורות",
            Holiday::CountOfTheOmer => "ספירת העומר",
            Holiday::BirchasHachamah => "ברכת החמה",
            Holiday::MacharHachodesh => "מחר החדש",
            Holiday::ShabbosMevarchim => "שבת מברכים",
        }
    }
}

impl core::fmt::Display for Holiday {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.he())
    }
}

const ALL_HOLIDAYS: [Holiday; 41] = [
    Holiday::ErevPesach,
    Holiday::Pesach,
    Holiday::PesachSheni,
    Holiday::ErevShavuos,
    Holiday::Shavuos,
    Holiday::SeventeenthOfTammuz,
    Holiday::TishahBav,
    Holiday::TuBav,
    Holiday::ErevRoshHashana,
    Holiday::RoshHashana,
    Holiday::FastOfGedalyah,
    Holiday::ErevYomKippur,
    Holiday::YomKippur,
    Holiday::ErevSuccos,
    Holiday::Succos,
    Holiday::CholHamoed,
    Holiday::HoshanaRabbah,
    Holiday::SheminiAtzeres,
    Holiday::SimchasTorah,
    Holiday::Chanukah,
    Holiday::TenthOfTeves,
    Holiday::TuBshvat,
    Holiday::FastOfEsther,
    Holiday::Purim,
    Holiday::ShushanPurim,
    Holiday::PurimKatan,
    Holiday::RoshChodesh,
    Holiday::YomHaShoah,
    Holiday::YomHazikaron,
    Holiday::YomHaatzmaut,
    Holiday::YomYerushalayim,
    Holiday::LagBomer,
    Holiday::ShushanPurimKatan,
    Holiday::IsruChag,
    Holiday::YomKippurKatan,
    Holiday::Behab,
    Holiday::FastOfTheFirstborn,
    Holiday::CountOfTheOmer,
    Holiday::BirchasHachamah,
    Holiday::MacharHachodesh,
    Holiday::ShabbosMevarchim,
];

/// Represents the months of the Hebrew calendar.
///
/// Months are numbered starting from Nissan (1) as per traditional Jewish counting.
/// In leap years, there are two Adar months: Adar (I) and AdarII.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialOrd, Ord)]
#[repr(u8)]
pub enum HebrewMonth {
    Nissan = 1,
    Iyar = 2,
    Sivan = 3,
    Tammuz = 4,
    Av = 5,
    Elul = 6,
    Tishrei = 7,
    Cheshvan = 8,
    Kislev = 9,
    Teves = 10,
    Shevat = 11,
    Adar = 12,
    AdarII = 13,
}

impl HebrewMonth {
    /// Returns the Hebrew name of the month.
    pub fn he(&self) -> &str {
        match self {
            HebrewMonth::Nissan => "ניסן",
            HebrewMonth::Iyar => "אייר",
            HebrewMonth::Sivan => "סיון",
            HebrewMonth::Tammuz => "תמוז",
            HebrewMonth::Av => "אב",
            HebrewMonth::Elul => "אלול",
            HebrewMonth::Tishrei => "תשרי",
            HebrewMonth::Cheshvan => "חשון",
            HebrewMonth::Kislev => "כסלו",
            HebrewMonth::Teves => "טבת",
            HebrewMonth::Shevat => "שבט",
            HebrewMonth::Adar => "אדר",
            HebrewMonth::AdarII => "אדר ב",
        }
    }
}

impl core::fmt::Display for HebrewMonth {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.he())
    }
}

/// Represents the length type of a Hebrew year based on Cheshvan and Kislev.
///
/// - `Chaserim`: Both months are short (Cheshvan 29 days, Kislev 29 days)
/// - `Kesidran`: Normal length (Cheshvan 29 days, Kislev 30 days)
/// - `Shelaimim`: Both months are long (Cheshvan 30 days, Kislev 30 days)
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum YearLengthType {
    Chaserim = 0,
    Kesidran = 1,
    Shelaimim = 2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_calendar::{cal::Hebrew, Date};
    extern crate std;
    use std::format;
    use std::vec::Vec;

    #[test]
    fn test_hebrew_leap_year() {
        // Test leap years in 19-year cycle
        assert!(Date::<Hebrew>::is_hebrew_leap_year(5784)); // Year 3 in cycle
        assert!(!Date::<Hebrew>::is_hebrew_leap_year(5785));
        assert!(Date::<Hebrew>::is_hebrew_leap_year(5787)); // Year 6 in cycle
        assert!(Date::<Hebrew>::is_hebrew_leap_year(5790)); // Year 8 in cycle
    }

    #[test]
    fn test_days_in_year() {
        let days = Date::<Hebrew>::days_in_hebrew_year(5784);
        assert!(
            days >= 353 && days <= 385,
            "Year length out of range: {}",
            days
        );
    }

    #[test]
    fn test_days_in_month() {
        // Regular month - always 30 days
        assert_eq!(
            Date::<Hebrew>::days_in_hebrew_month(5784, HebrewMonth::Nissan),
            30
        );

        // Regular month - always 29 days
        assert_eq!(
            Date::<Hebrew>::days_in_hebrew_month(5784, HebrewMonth::Iyar),
            29
        );

        // Adar in non-leap year is 29 days
        assert_eq!(
            Date::<Hebrew>::days_in_hebrew_month(5785, HebrewMonth::Adar),
            29
        );

        // Adar I in leap year is 30 days
        assert_eq!(
            Date::<Hebrew>::days_in_hebrew_month(5784, HebrewMonth::Adar),
            30
        );
    }

    #[test]
    fn test_hebrew_month_enum() {
        let month = HebrewMonth::Tishrei;
        assert_eq!(month.he(), "תשרי");
        assert_eq!(format!("{}", month), "תשרי");
    }

    #[test]
    fn test_from_hebrew_date() {
        // Create Rosh Hashana 5784
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 1).unwrap();
        assert_eq!(date.day_of_month().0, 1);
        assert_eq!(date.hebrew_month(), HebrewMonth::Tishrei);
        assert_eq!(date.extended_year(), 5784);
    }

    #[test]
    fn test_rosh_hashana_holiday() {
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 1).unwrap();
        let holidays: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays.contains(&&Holiday::RoshHashana));
        assert!(date.is_assur_bemelacha(false));
    }

    #[test]
    fn test_yom_kippur() {
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 10).unwrap();
        let holidays: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays.contains(&&Holiday::YomKippur));
        assert!(date.is_assur_bemelacha(false));
    }

    #[test]
    fn test_chanukah() {
        // First day of Chanukah
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Kislev, 25).unwrap();
        let holidays: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays.contains(&&Holiday::Chanukah));
        assert!(!date.is_assur_bemelacha(false)); // Work is permitted
    }

    #[test]
    fn test_purim() {
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::AdarII, 14).unwrap();
        let holidays: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays.contains(&&Holiday::Purim));
    }

    #[test]
    fn test_pesach_israel_vs_diaspora() {
        // Second day of Pesach - Yom Tov in diaspora, Chol Hamoed in Israel
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Nissan, 16).unwrap();

        // In diaspora
        let holidays_diaspora: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays_diaspora.contains(&&Holiday::Pesach));
        assert!(date.is_assur_bemelacha(false));

        // In Israel
        let holidays_israel: Vec<_> = date.holidays(true, false).collect();
        assert!(holidays_israel.contains(&&Holiday::CholHamoed));
        assert!(!date.is_assur_bemelacha(true));
    }

    #[test]
    fn test_candle_lighting() {
        // Erev Shabbat (Friday)
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 6).unwrap();
        if date.chrono_day_of_week() == chrono::Weekday::Fri {
            assert!(date.has_candle_lighting(false));
        }
    }

    #[test]
    fn test_aseres_yemei_teshuva() {
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 5).unwrap();
        assert!(date.is_aseres_yemei_teshuva());

        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 11).unwrap();
        assert!(!date.is_aseres_yemei_teshuva());
    }

    #[test]
    fn test_modern_holidays() {
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Iyar, 28).unwrap();

        // Without modern holidays
        let holidays_traditional: Vec<_> = date.holidays(true, false).collect();
        assert!(!holidays_traditional.iter().any(|h| h.is_modern_holiday()));

        // With modern holidays
        let holidays_modern: Vec<_> = date.holidays(true, true).collect();
        assert!(holidays_modern.contains(&&Holiday::YomYerushalayim));
    }

    #[test]
    fn test_fast_days() {
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Teves, 10).unwrap();
        let holidays: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays.iter().any(|h| h.is_fast_day()));
    }

    #[test]
    fn test_rosh_chodesh() {
        // First day of month
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Cheshvan, 1).unwrap();
        let holidays: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays.contains(&&Holiday::RoshChodesh));

        // 30th of month (second day of Rosh Chodesh)
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 30).unwrap();
        let holidays: Vec<_> = date.holidays(false, false).collect();
        assert!(holidays.contains(&&Holiday::RoshChodesh));
    }

    #[test]
    fn test_kviah_types() {
        let year = 5784;
        let kviah = Date::<Hebrew>::cheshvan_kislev_kviah(year);

        // Should be one of the three types
        assert!(matches!(
            kviah,
            YearLengthType::Chaserim | YearLengthType::Kesidran | YearLengthType::Shelaimim
        ));

        // Test consistency
        let is_cheshvan_long = Date::<Hebrew>::is_cheshvan_long(year);
        let is_kislev_short = Date::<Hebrew>::is_kislev_short(year);

        match kviah {
            YearLengthType::Shelaimim => {
                assert!(is_cheshvan_long && !is_kislev_short);
            }
            YearLengthType::Chaserim => {
                assert!(!is_cheshvan_long && is_kislev_short);
            }
            YearLengthType::Kesidran => {
                assert!(!is_cheshvan_long && !is_kislev_short);
            }
        }
    }

    #[test]
    fn test_parsha_on_shabbat() {
        // Create multiple dates and check parsha
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 21).unwrap();

        // Only returns parsha on Shabbat
        if date.chrono_day_of_week() == chrono::Weekday::Sat {
            let parsha = date.todays_parsha(false);
            assert!(parsha.is_some() || date.holidays(false, false).count() > 0);
        } else {
            assert_eq!(date.todays_parsha(false), None);
        }
    }

    #[test]
    fn test_upcoming_parsha() {
        let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 15).unwrap();
        let upcoming = date.upcoming_parsha(false);

        // Should return a valid parsha
        assert!(upcoming.he().len() > 0);
    }

    #[test]
    fn test_next_hebrew_month() {
        // Regular month
        let (year, month) = next_hebrew_month(5784, HebrewMonth::Nissan);
        assert_eq!(year, 5784);
        assert_eq!(month, HebrewMonth::Iyar);

        // End of year
        let (year, month) = next_hebrew_month(5784, HebrewMonth::Elul);
        assert_eq!(year, 5785);
        assert_eq!(month, HebrewMonth::Tishrei);

        // Non-leap year Adar
        let (year, month) = next_hebrew_month(5785, HebrewMonth::Adar);
        assert_eq!(year, 5785);
        assert_eq!(month, HebrewMonth::Nissan);
    }

    #[test]
    fn test_add_days_to_hebrew_date() {
        // Simple addition within month
        let result = add_days_to_hebrew_date(5784, HebrewMonth::Tishrei, 1, 5);
        assert_eq!(result, Some((5784, HebrewMonth::Tishrei, 6)));

        // Addition spanning months
        let result = add_days_to_hebrew_date(5784, HebrewMonth::Tishrei, 28, 5);
        assert!(result.is_some());
        let (_year, month, _day) = result.unwrap();
        assert!(month == HebrewMonth::Tishrei || month == HebrewMonth::Cheshvan);
    }

    #[test]
    fn test_holiday_display() {
        let holiday = Holiday::RoshHashana;
        assert_eq!(format!("{}", holiday), "ראש השנה");
        assert_eq!(holiday.he(), "ראש השנה");
    }

    #[test]
    fn test_parsha_display() {
        let parsha = Parsha::Bereshis;
        assert_eq!(format!("{}", parsha), "בראשית");
        assert_eq!(parsha.he(), "בראשית");
    }

    #[test]
    fn test_hebrew_elapsed_days() {
        // Basic sanity check - should be positive
        let elapsed = get_hebrew_elapsed_days(5784);
        assert!(elapsed > 0);

        // Next year should have more elapsed days
        let next_elapsed = get_hebrew_elapsed_days(5785);
        assert!(next_elapsed > elapsed);

        // Difference should be the number of days in the year
        let year_length = Date::<Hebrew>::days_in_hebrew_year(5784);
        assert_eq!(next_elapsed - elapsed, year_length);
    }

    #[test]
    fn test_chalakim_calculations() {
        // Test that chalakim are calculated consistently
        let chalakim1 = chalakim_since_molad_tohu(5784, HebrewMonth::Tishrei);
        let chalakim2 = chalakim_since_molad_tohu(5784, HebrewMonth::Cheshvan);

        // Next month should have more chalakim
        assert!(chalakim2 > chalakim1);

        // Difference should be approximately one month
        let diff = chalakim2 - chalakim1;
        assert!((diff - CHALAKIM_PER_MONTH).abs() < 100);
    }
}
