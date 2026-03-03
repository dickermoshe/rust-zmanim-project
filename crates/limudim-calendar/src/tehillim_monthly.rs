use hebrew_holiday_calendar::HebrewHolidayCalendar;

use crate::{
    date::{from_hebrew_date, HebrewDate},
    interval::Interval,
    limud_calculator::{CycleFinder, InternalLimudCalculator},
    LimudCalculator,
};

/// Cumulative ending psalm for each day of the month (0-indexed by day-1)
/// Day 1: psalms 1-9, Day 2: psalms 10-17, etc.
const DEFAULT_UNITS: [u8; 30] = [
    9, 17, 22, 28, 34, 38, 43, 48, 54, 59, 65, 68, 71, 76, 78, 82, 87, 89, 96, 103, 105, 107, 112, 118, 119, 119, 134,
    139, 144, 150,
];

/// Represents a Tehillim (Psalms) reading unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub enum TehillimUnit {
    /// A range of complete psalms (e.g., psalms 1-9)
    Psalms { start: u8, end: u8 },
    /// A range of verses within a single psalm (for Psalm 119)
    PsalmVerses {
        psalm: u8,
        start_verse: u16,
        end_verse: u16,
    },
}

#[derive(Default)]
/// Calculates the Tehillim (Psalms) monthly schedule.
pub struct TehillimMonthly;

/// Find the 1st of the current Hebrew month and the last day of the month
fn find_monthly_cycle(date: HebrewDate) -> (HebrewDate, HebrewDate) {
    let year = date.year().extended_year();
    let month = date.hebrew_month();

    // Start of cycle: 1st of current month
    let start = from_hebrew_date(year, month, 1);

    // End of cycle: last day of current month
    let days_in_month = date.days_in_month();
    let end = from_hebrew_date(year, month, days_in_month);

    (start, end)
}

impl InternalLimudCalculator<TehillimUnit> for TehillimMonthly {
    fn cycle_finder(&self) -> CycleFinder {
        CycleFinder::Perpetual(find_monthly_cycle)
    }

    fn unit_for_interval(&self, interval: &Interval, _limud_date: &HebrewDate) -> Option<TehillimUnit> {
        let iteration = interval.iteration;

        // Special cases for Psalm 119 on days 25 and 26
        if iteration == 25 {
            return Some(TehillimUnit::PsalmVerses {
                psalm: 119,
                start_verse: 1,
                end_verse: 96,
            });
        }

        if iteration == 26 {
            return Some(TehillimUnit::PsalmVerses {
                psalm: 119,
                start_verse: 97,
                end_verse: 176,
            });
        }

        // Normal psalm range calculation
        let (start, mut stop) = if iteration == 1 {
            (1, DEFAULT_UNITS[0])
        } else {
            let prev_end = DEFAULT_UNITS[(iteration - 2) as usize];
            let curr_end = DEFAULT_UNITS[(iteration - 1) as usize];
            (prev_end + 1, curr_end)
        };

        // On the 29th day of a 29-day month, include the next day's reading too
        let day = interval.end_date.day_of_month().0;
        let days_in_month = interval.end_date.days_in_month();
        if day == 29 && days_in_month == 29 && iteration < 30 {
            stop = DEFAULT_UNITS[iteration as usize];
        }

        Some(TehillimUnit::Psalms { start, end: stop })
    }
}
impl LimudCalculator<TehillimUnit> for TehillimMonthly {}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use hebrew_holiday_calendar::HebrewMonth;

    use super::*;

    #[test]
    fn tehillim_monthly_simple_date() {
        // JewishDate(5778, 10, 8) - 8th of Teves
        let test_date = from_hebrew_date(5778, HebrewMonth::Teves, 8);
        let limud = TehillimMonthly.limud(test_date).expect("limud exists");
        // Day 8 should be psalms 44-48
        assert_eq!(limud, TehillimUnit::Psalms { start: 44, end: 48 });
    }

    #[test]
    fn tehillim_monthly_beginning_of_month() {
        // JewishDate(5778, 10, 1) - 1st of Teves
        let test_date = from_hebrew_date(5778, HebrewMonth::Teves, 1);
        let limud = TehillimMonthly.limud(test_date).expect("limud exists");
        // Day 1 should be psalms 1-9
        assert_eq!(limud, TehillimUnit::Psalms { start: 1, end: 9 });
    }

    #[test]
    fn tehillim_monthly_end_of_short_month() {
        // JewishDate(5778, 10, 29) - 29th of Teves (29-day month)
        let test_date = from_hebrew_date(5778, HebrewMonth::Teves, 29);
        let limud = TehillimMonthly.limud(test_date).expect("limud exists");
        // Day 29 of a 29-day month should include day 30's reading: psalms 140-150
        assert_eq!(limud, TehillimUnit::Psalms { start: 140, end: 150 });
    }

    #[test]
    fn tehillim_monthly_end_of_long_month() {
        // JewishDate(5778, 11, 30) - 30th of Shevat (30-day month)
        let test_date = from_hebrew_date(5778, HebrewMonth::Shevat, 30);
        let limud = TehillimMonthly.limud(test_date).expect("limud exists");
        // Day 30 should be psalms 145-150
        assert_eq!(limud, TehillimUnit::Psalms { start: 145, end: 150 });
    }

    #[test]
    fn tehillim_monthly_29th_day_of_long_month() {
        // JewishDate(5778, 11, 29) - 29th of Shevat (30-day month)
        let test_date = from_hebrew_date(5778, HebrewMonth::Shevat, 29);
        let limud = TehillimMonthly.limud(test_date).expect("limud exists");
        // Day 29 of a 30-day month should be psalms 140-144 only
        assert_eq!(limud, TehillimUnit::Psalms { start: 140, end: 144 });
    }

    #[test]
    fn tehillim_monthly_day_25_special_case() {
        // JewishDate(5778, 11, 25) - 25th of Shevat
        let test_date = from_hebrew_date(5778, HebrewMonth::Shevat, 25);
        let limud = TehillimMonthly.limud(test_date).expect("limud exists");
        // Day 25 is Psalm 119 verses 1-96
        assert_eq!(
            limud,
            TehillimUnit::PsalmVerses {
                psalm: 119,
                start_verse: 1,
                end_verse: 96
            }
        );
    }

    #[test]
    fn tehillim_monthly_day_26_special_case() {
        // JewishDate(5778, 11, 26) - 26th of Shevat
        let test_date = from_hebrew_date(5778, HebrewMonth::Shevat, 26);
        let limud = TehillimMonthly.limud(test_date).expect("limud exists");
        // Day 26 is Psalm 119 verses 97-176
        assert_eq!(
            limud,
            TehillimUnit::PsalmVerses {
                psalm: 119,
                start_verse: 97,
                end_verse: 176
            }
        );
    }
}
