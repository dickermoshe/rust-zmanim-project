use core::ops::RangeInclusive;

use hebrew_holiday_calendar::{HebrewHolidayCalendar, Holiday};
use icu_calendar::options::DateDifferenceOptions;

use crate::{
    constants::YERUSHALMI_DAF_COUNT,
    date::{from_gregorian_date, DateExt, HebrewDate},
    interval::Interval,
    limud_calculator::{CycleFinder, LimudCalculator},
    units::{Daf, Tractate, YERUSHALMI_TRACTATES},
};

const fn start_daf(_tractate: Tractate, _iteration: i32) -> u16 {
    1
}

/// Last daf number
const fn end_daf(tractate: Tractate, _iteration: i32) -> Option<u16> {
    let daf = match tractate {
        Tractate::Berachos => 68,
        Tractate::Peah => 37,
        Tractate::Demai => 34,
        Tractate::Kilayim => 44,
        Tractate::Sheviis => 31,
        Tractate::Terumos => 59,
        Tractate::Maasros => 26,
        Tractate::MaaserSheni => 33,
        Tractate::Chalah => 28,
        Tractate::Orlah => 20,
        Tractate::Bikurim => 13,
        Tractate::Shabbos => 92,
        Tractate::Eruvin => 65,
        Tractate::Pesachim => 71,
        Tractate::Beitzah => 22,
        Tractate::RoshHashanah => 22,
        Tractate::Yoma => 42,
        Tractate::Sukkah => 26,
        Tractate::Taanis => 26,
        Tractate::Shekalim => 33,
        Tractate::Megillah => 34,
        Tractate::Chagigah => 22,
        Tractate::MoedKatan => 19,
        Tractate::Yevamos => 85,
        Tractate::Kesubos => 72,
        Tractate::Sotah => 47,
        Tractate::Nedarim => 40,
        Tractate::Nazir => 47,
        Tractate::Gitin => 54,
        Tractate::Kiddushin => 48,
        Tractate::BavaKamma => 44,
        Tractate::BavaMetzia => 37,
        Tractate::BavaBasra => 34,
        Tractate::Sanhedrin => 57,
        Tractate::Makkos => 9,
        Tractate::Shevuos => 44,
        Tractate::AvodahZarah => 37,
        Tractate::Horiyos => 19,
        Tractate::Niddah => 13,
        _ => 0,
    };
    if daf == 0 {
        None
    } else {
        Some(daf)
    }
}

const fn iter(tractate: Tractate, iteration: i32) -> RangeInclusive<u16> {
    let end = end_daf(tractate, iteration);
    if let Some(end) = end {
        start_daf(tractate, iteration)..=end
    } else {
        RangeInclusive::new(0, 0)
    }
}
fn iter_daf(iteration: i32) -> impl Iterator<Item = Daf> {
    YERUSHALMI_TRACTATES
        .iter()
        .flat_map(move |i| iter(*i, iteration).map(move |j| Daf { tractate: *i, page: j }))
}

#[derive(Default)]
pub struct DafYomiYerushalmi {}
impl LimudCalculator<Daf> for DafYomiYerushalmi {
    fn cycle_finder(&self) -> CycleFinder {
        CycleFinder::Initial(from_gregorian_date(1980, 2, 2))
    }

    fn cycle_end_calculation(hebrew_date: HebrewDate, _iteration: Option<i32>) -> Option<HebrewDate> {
        let mut end_date = hebrew_date.add_days(YERUSHALMI_DAF_COUNT - 1)?;

        let mut found_days = found_skips_between(hebrew_date, end_date);
        while found_days > 0 {
            let new_start_date = end_date.add_days(1)?;
            end_date = end_date.add_days(found_days)?;
            found_days = found_skips_between(new_start_date, end_date);
        }
        Some(end_date)
    }

    fn unit_for_interval(&self, interval: &Interval, limud_date: &HebrewDate) -> Option<Daf> {
        // If this is a skip interval, return None (no daf today)
        if self.is_skip_interval(interval) {
            return None;
        }
        let cycle_iteration = interval.cycle.iteration?;
        let mut iter = iter_daf(cycle_iteration);
        // Calculate offset accounting for skip days
        let days_from_start = interval
            .cycle
            .start_date
            .try_until_with_options(limud_date, DateDifferenceOptions::default())
            .ok()?
            .days;
        // Subtract the number of skip days between cycle start and limud_date
        let skip_days_count = found_skips_between(interval.cycle.start_date, *limud_date);
        let offset = (days_from_start as i64) - (skip_days_count as i64);
        if offset < 0 {
            return None;
        }
        iter.nth(offset as usize)
    }
    fn is_skip_interval(&self, interval: &Interval) -> bool {
        is_skip_day(&interval.start_date)
    }
}

struct HebDateIter {
    current: HebrewDate,
    end: HebrewDate,
}
impl Iterator for HebDateIter {
    type Item = HebrewDate;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            None
        } else {
            let next = self.current.add_days(1)?;
            self.current = next;
            Some(self.current)
        }
    }
}
fn found_skips_between(a: HebrewDate, b: HebrewDate) -> i32 {
    let iter = HebDateIter { current: a, end: b };
    let mut skips = 0;
    for date in iter {
        if is_skip_day(&date) {
            skips += 1;
        }
    }
    skips
}

fn is_skip_day(date: &HebrewDate) -> bool {
    date.holidays(false, false)
        .any(|h| h == &Holiday::TishahBav || h == &Holiday::YomKippur)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use icu_calendar::{cal::Hebrew, Date};

    use crate::date::from_gregorian_date;

    use super::*;

    #[test]
    fn daf_yomi_yerushalmi_simple_date() {
        let test_date = from_gregorian_date(2017, 12, 28);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 33);
        assert_eq!(limud.tractate, Tractate::BavaMetzia);
    }

    #[test]
    fn daf_yomi_yerushalmi_before_cycle_began() {
        let test_date = from_gregorian_date(1980, 1, 1);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_first_day_of_cycle() {
        let test_date = from_gregorian_date(2005, 10, 3);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 1);
        assert_eq!(limud.tractate, Tractate::Berachos);
    }

    #[test]
    fn daf_yomi_yerushalmi_last_day_of_cycle() {
        let test_date = from_gregorian_date(2010, 1, 12);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 13);
        assert_eq!(limud.tractate, Tractate::Niddah);
    }

    #[test]
    fn daf_yomi_yerushalmi_last_skip_day() {
        // JewishDate(5778, 7, 10) is Tishrei 10 (Yom Kippur) - a skip day
        let test_date = Date::<Hebrew>::from_hebrew_date(5778, hebrew_holiday_calendar::HebrewMonth::Tishrei, 10)
            .expect("valid hebrew date");
        let limud = DafYomiYerushalmi::default().limud(test_date);

        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_1980_02_02() {
        let test_date = from_gregorian_date(1980, 2, 2);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 1);
        assert_eq!(limud.tractate, Tractate::Berachos);
    }

    #[test]
    fn daf_yomi_yerushalmi_1982_05_15() {
        let test_date = from_gregorian_date(1982, 5, 15);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 4);
        assert_eq!(limud.tractate, Tractate::Chagigah);
    }

    #[test]
    fn daf_yomi_yerushalmi_1984_05_12() {
        let test_date = from_gregorian_date(1984, 5, 12);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 13);
        assert_eq!(limud.tractate, Tractate::Niddah);
    }

    #[test]
    fn daf_yomi_yerushalmi_1984_05_13() {
        let test_date = from_gregorian_date(1984, 5, 13);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 1);
        assert_eq!(limud.tractate, Tractate::Berachos);
    }

    #[test]
    fn daf_yomi_yerushalmi_1990_08_01() {
        let test_date = from_gregorian_date(1990, 8, 1);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 40);
        assert_eq!(limud.tractate, Tractate::Yoma);
    }

    #[test]
    fn daf_yomi_yerushalmi_2000_01_01() {
        let test_date = from_gregorian_date(2000, 1, 1);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 66);
        assert_eq!(limud.tractate, Tractate::Kesubos);
    }

    #[test]
    fn daf_yomi_yerushalmi_2005_10_02() {
        let test_date = from_gregorian_date(2005, 10, 2);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 13);
        assert_eq!(limud.tractate, Tractate::Niddah);
    }

    #[test]
    fn daf_yomi_yerushalmi_2007_06_15() {
        let test_date = from_gregorian_date(2007, 6, 15);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 68);
        assert_eq!(limud.tractate, Tractate::Pesachim);
    }

    #[test]
    fn daf_yomi_yerushalmi_2010_01_11() {
        let test_date = from_gregorian_date(2010, 1, 11);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 12);
        assert_eq!(limud.tractate, Tractate::Niddah);
    }

    #[test]
    fn daf_yomi_yerushalmi_2015_04_23() {
        let test_date = from_gregorian_date(2015, 4, 23);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 3);
        assert_eq!(limud.tractate, Tractate::Orlah);
    }

    #[test]
    fn daf_yomi_yerushalmi_2020_01_01() {
        let test_date = from_gregorian_date(2020, 1, 1);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 28);
        assert_eq!(limud.tractate, Tractate::Eruvin);
    }

    #[test]
    fn daf_yomi_yerushalmi_2025_10_02() {
        let test_date = from_gregorian_date(2025, 10, 2);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_1980_09_20() {
        let test_date = from_gregorian_date(1980, 9, 20);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_1990_09_29() {
        let test_date = from_gregorian_date(1990, 9, 29);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_2000_10_09() {
        let test_date = from_gregorian_date(2000, 10, 9);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_2010_09_18() {
        let test_date = from_gregorian_date(2010, 9, 18);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_1980_07_21() {
        let test_date = from_gregorian_date(1980, 7, 21);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 32);
        assert_eq!(limud.tractate, Tractate::Kilayim);
    }

    #[test]
    fn daf_yomi_yerushalmi_1990_07_31() {
        let test_date = from_gregorian_date(1990, 7, 31);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_2000_08_10() {
        let test_date = from_gregorian_date(2000, 8, 10);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_2010_07_20() {
        let test_date = from_gregorian_date(2010, 7, 20);
        let limud = DafYomiYerushalmi::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_yomi_yerushalmi_1980_03_01() {
        let test_date = from_gregorian_date(1980, 3, 1);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 29);
        assert_eq!(limud.tractate, Tractate::Berachos);
    }

    #[test]
    fn daf_yomi_yerushalmi_1982_01_01() {
        let test_date = from_gregorian_date(1982, 1, 1);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 31);
        assert_eq!(limud.tractate, Tractate::Yoma);
    }

    #[test]
    fn daf_yomi_yerushalmi_1984_04_01() {
        let test_date = from_gregorian_date(1984, 4, 1);
        let limud = DafYomiYerushalmi::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 28);
        assert_eq!(limud.tractate, Tractate::AvodahZarah);
    }
}
