use hebrew_holiday_calendar::{HebrewHolidayCalendar, HebrewMonth};

use crate::{
    cycle::Cycle,
    date::{from_hebrew_date, DateExt, HebrewDate},
    interval::Interval,
    limud_calculator::{CycleFinder, LimudCalculator},
};

/// Represents a Pirkei Avos reading unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PirkeiAvosUnit {
    /// A single perek (chapter)
    Single(u8),
    /// Combined perakim (when weeks are compressed)
    Combined(u8, u8),
}

pub struct PirkeiAvos {
    pub in_israel: bool,
}

impl LimudCalculator<PirkeiAvosUnit> for PirkeiAvos {
    fn cycle_finder(&self) -> CycleFinder {
        if self.in_israel {
            CycleFinder::Perpetual(Self::find_yearly_cycle_israel)
        } else {
            CycleFinder::Perpetual(Self::find_yearly_cycle_diaspora)
        }
    }

    fn unit_for_interval(&self, interval: &Interval, _limud_date: &HebrewDate) -> Option<PirkeiAvosUnit> {
        let iteration = interval.iteration;

        // First 18 weeks: standard 1-6 cycle repeated 3 times
        if iteration < 19 {
            let chapter = ((iteration - 1) % 6) + 1;
            return Some(PirkeiAvosUnit::Single(chapter as u8));
        }

        // Fourth round: use weeks remaining logic (like hebcal)
        // Calculate weeks remaining from this interval's Shabbat to the end of the cycle
        let days_until_end = days_between(interval.end_date, interval.cycle.end_date);
        let weeks_remain = (days_until_end + 6) / 7; // ceiling division

        match weeks_remain {
            0 => Some(PirkeiAvosUnit::Combined(5, 6)),
            1 => Some(PirkeiAvosUnit::Combined(3, 4)),
            2 => {
                // If iteration % 6 == 1, return [2], else [1,2]
                if (iteration - 1) % 6 == 0 {
                    Some(PirkeiAvosUnit::Combined(1, 2))
                } else {
                    Some(PirkeiAvosUnit::Single(((iteration - 1) % 6 + 1) as u8))
                }
            }
            3 => Some(PirkeiAvosUnit::Single(1)),
            _ => {
                // Continue normal cycle for weeks > 3 remaining
                let chapter = ((iteration - 1) % 6) + 1;
                Some(PirkeiAvosUnit::Single(chapter as u8))
            }
        }
    }
    fn interval_end_calculation(_cycle: Cycle, hebrew_date: HebrewDate) -> Option<HebrewDate> {
        // Each interval is a week, ending on Shabbos
        let day_number = hebrew_date.day_of_week_number();
        hebrew_date.add_days(7 - day_number)
    }

    fn is_skip_interval(&self, interval: &Interval) -> bool {
        let end_month = interval.end_date.hebrew_month();
        let end_day = interval.end_date.day_of_month().0;

        // Skip erev Tisha B'Av (8th of Av) - applies to both Israel and diaspora
        if end_month == HebrewMonth::Av && end_day == 8 {
            return true;
        }

        // Skip Tisha B'Av (9th of Av) - applies to both Israel and diaspora
        if end_month == HebrewMonth::Av && end_day == 9 {
            return true;
        }

        // Skip 7th of Sivan (2nd day Shavuot) - only outside Israel
        if !self.in_israel && end_month == HebrewMonth::Sivan && end_day == 7 {
            return true;
        }

        false
    }
}

impl PirkeiAvos {
    pub fn new(in_israel: bool) -> Self {
        Self { in_israel }
    }

    fn find_yearly_cycle_israel(date: HebrewDate) -> (HebrewDate, HebrewDate) {
        Self::find_yearly_cycle(true, date)
    }

    fn find_yearly_cycle_diaspora(date: HebrewDate) -> (HebrewDate, HebrewDate) {
        Self::find_yearly_cycle(false, date)
    }

    /// Find the Pirkei Avos cycle for a given date.
    /// Cycle starts the day after Pesach (Nissan 22 in Israel, Nissan 23 outside)
    /// and ends on the last Shabbos before Rosh Hashanah.
    fn find_yearly_cycle(in_israel: bool, date: HebrewDate) -> (HebrewDate, HebrewDate) {
        let year = date.year().extended_year();

        // Day after Pesach: Nissan 22 in Israel, Nissan 23 outside
        let anchor_day = if in_israel { 22 } else { 23 };
        let cycle_start_this_year = from_hebrew_date(year, HebrewMonth::Nissan, anchor_day);

        // Determine which year's cycle we're in
        let (start_date, cycle_year) = if date >= cycle_start_this_year {
            (cycle_start_this_year, year)
        } else {
            // We're before this year's cycle starts, use previous year's cycle
            let prev_year_start = from_hebrew_date(year - 1, HebrewMonth::Nissan, anchor_day);
            (prev_year_start, year - 1)
        };

        // End date: last Shabbos before Rosh Hashanah of the following year
        let rosh_hashana = from_hebrew_date(cycle_year + 1, HebrewMonth::Tishrei, 1);
        let day_number = rosh_hashana.day_of_week_number();
        // Subtract days to get to the previous Shabbos
        let end_date = rosh_hashana.add_days(-day_number).unwrap_or(rosh_hashana);

        (start_date, end_date)
    }
}

/// Calculate the number of days between two dates (end - start)
fn days_between(start: HebrewDate, end: HebrewDate) -> i32 {
    use icu_calendar::options::DateDifferenceOptions;
    start
        .try_until_with_options(&end, DateDifferenceOptions::default())
        .map(|d| d.days as i32)
        .unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    // Test cases based on Python test_pirkei_avos_calculator.py

    #[test]
    fn test_simple_date() {
        // JewishDate(5778, 3, 1) - 1st of Sivan 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Sivan, 1);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date).expect("limud exists");
        // Python test expects description '6'
        assert_eq!(limud, PirkeiAvosUnit::Single(6));
    }

    #[test]
    fn test_near_end_of_cycle() {
        // JewishDate(5778, 6, 20) - 20th of Elul 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Elul, 20);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date).expect("limud exists");
        // Python test expects description '3 - 4'
        assert_eq!(limud, PirkeiAvosUnit::Combined(3, 4));
    }

    #[test]
    fn test_after_cycle_completes() {
        // JewishDate(5778, 6, 29) - 29th of Elul 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Elul, 29);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn test_before_cycle_starts() {
        // JewishDate(5778, 1, 20) - 20th of Nissan 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Nissan, 20);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn test_8th_day_pesach_outside_israel() {
        // JewishDate(5778, 1, 22) - 22nd of Nissan 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Nissan, 22);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn test_day_after_pesach_outside_israel() {
        // JewishDate(5778, 1, 23) - 23rd of Nissan 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Nissan, 23);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date).expect("limud exists");
        // Python test expects description '1'
        assert_eq!(limud, PirkeiAvosUnit::Single(1));
    }

    #[test]
    fn test_compounding_before_cycle_end_outside_israel() {
        // JewishDate(5778, 6, 14) - 14th of Elul 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Elul, 14);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date).expect("limud exists");
        assert_eq!(limud, PirkeiAvosUnit::Combined(1, 2));

        // JewishDate(5778, 6, 15) - 15th of Elul 5778
        let test_date2 = from_hebrew_date(5778, HebrewMonth::Elul, 15);
        let limud2 = calculator.limud(test_date2).expect("limud exists");
        assert_eq!(limud2, PirkeiAvosUnit::Combined(3, 4));
    }

    #[test]
    fn test_8th_day_pesach_in_israel() {
        // JewishDate(5778, 1, 22) - 22nd of Nissan 5778 (Shabbos)
        let test_date = from_hebrew_date(5778, HebrewMonth::Nissan, 22);
        let calculator = PirkeiAvos::new(true);
        let limud = calculator.limud(test_date).expect("limud exists");
        // In Israel, cycle starts on 22nd, and if it's Shabbos, that's the first interval
        assert_eq!(limud, PirkeiAvosUnit::Single(1));
    }

    #[test]
    fn test_day_after_pesach_in_israel() {
        // JewishDate(5778, 1, 23) - 23rd of Nissan 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Nissan, 23);
        let calculator = PirkeiAvos::new(true);
        let limud = calculator.limud(test_date).expect("limud exists");
        // Python test expects description '2'
        assert_eq!(limud, PirkeiAvosUnit::Single(2));
    }

    #[test]
    fn test_compounding_before_cycle_end_in_israel() {
        // JewishDate(5778, 6, 21) - 21st of Elul 5778
        let test_date = from_hebrew_date(5778, HebrewMonth::Elul, 21);
        let calculator = PirkeiAvos::new(true);
        let limud = calculator.limud(test_date).expect("limud exists");
        assert_eq!(limud, PirkeiAvosUnit::Combined(3, 4));
    }

    #[test]
    fn test_7_sivan_on_shabbos_outside_israel() {
        // 5769 - Sivan 7 falls on Shabbos outside Israel
        // JewishDate(5769, 3, 3) - 3rd of Sivan 5769
        let test_date = from_hebrew_date(5769, HebrewMonth::Sivan, 3);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date);
        // This interval should be skipped, returning None for the unit
        // The Python test expects limud to exist but with None unit
        // In our implementation, we just don't return a unit for skipped intervals
        assert!(limud.is_none());
    }

    #[test]
    fn test_iteration_following_7_sivan_on_shabbos_outside_israel() {
        // JewishDate(5769, 3, 8) - 8th of Sivan 5769
        let test_date = from_hebrew_date(5769, HebrewMonth::Sivan, 8);
        let calculator = PirkeiAvos::new(false);
        let limud = calculator.limud(test_date).expect("limud exists");
        // Python test expects description '1' (starts new sub-cycle)
        assert_eq!(limud, PirkeiAvosUnit::Single(1));
    }

    #[test]
    fn test_7_sivan_on_shabbos_in_israel() {
        // JewishDate(5769, 3, 3) - 3rd of Sivan 5769
        let test_date = from_hebrew_date(5769, HebrewMonth::Sivan, 3);
        let calculator = PirkeiAvos::new(true);
        let limud = calculator.limud(test_date).expect("limud exists");
        // In Israel, no skip - Python test expects description '1'
        assert_eq!(limud, PirkeiAvosUnit::Single(1));
    }

    #[test]
    fn test_iteration_following_7_sivan_on_shabbos_in_israel() {
        // JewishDate(5769, 3, 8) - 8th of Sivan 5769
        let test_date = from_hebrew_date(5769, HebrewMonth::Sivan, 8);
        let calculator = PirkeiAvos::new(true);
        let limud = calculator.limud(test_date).expect("limud exists");
        // Python test expects description '2'
        assert_eq!(limud, PirkeiAvosUnit::Single(2));
    }
}
