use core::ops::RangeInclusive;

use crate::{
    constants::BAVLI_DAF_COUNT_MODERN,
    date::{from_gregorian_date, DateExt, HebrewDate},
    interval::Interval,
    limud_calculator::{CycleFinder, InternalLimudCalculator},
    units::{Daf, Tractate, BAVLI_TRACTATES},
    LimudCalculator,
};

const fn start_daf(tractate: Tractate, _iteration: i32) -> u16 {
    match tractate {
        Tractate::Kinnim => 23,
        Tractate::Tamid => 26,
        Tractate::Midos => 34,
        _ => 2,
    }
}

const fn end_daf(tractate: Tractate, _iteration: i32) -> Option<u16> {
    let daf = match tractate {
        Tractate::Berachos => 64,
        Tractate::Shabbos => 157,
        Tractate::Eruvin => 105,
        Tractate::Pesachim => 121,
        Tractate::Shekalim => 22,
        Tractate::Yoma => 88,
        Tractate::Sukkah => 56,
        Tractate::Beitzah => 40,
        Tractate::RoshHashanah => 35,
        Tractate::Taanis => 31,
        Tractate::Megillah => 32,
        Tractate::MoedKatan => 29,
        Tractate::Chagigah => 27,
        Tractate::Yevamos => 122,
        Tractate::Kesubos => 112,
        Tractate::Nedarim => 91,
        Tractate::Nazir => 66,
        Tractate::Sotah => 49,
        Tractate::Gitin => 90,
        Tractate::Kiddushin => 82,
        Tractate::BavaKamma => 119,
        Tractate::BavaMetzia => 119,
        Tractate::BavaBasra => 176,
        Tractate::Sanhedrin => 113,
        Tractate::Makkos => 24,
        Tractate::Shevuos => 49,
        Tractate::AvodahZarah => 76,
        Tractate::Horiyos => 14,
        Tractate::Zevachim => 120,
        Tractate::Menachos => 110,
        Tractate::Chullin => 142,
        Tractate::Bechoros => 61,
        Tractate::Arachin => 34,
        Tractate::Temurah => 34,
        Tractate::Kerisos => 28,
        Tractate::Meilah => 22,
        Tractate::Kinnim => 25,
        Tractate::Tamid => 33,
        Tractate::Midos => 37,
        Tractate::Niddah => 73,
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
        // Empty range: start > end yields no items
        RangeInclusive::new(1, 0)
    }
}
fn iter_daf(iteration: i32) -> impl Iterator<Item = Daf> {
    BAVLI_TRACTATES
        .iter()
        .flat_map(move |i| iter(*i, iteration).map(move |j| Daf { tractate: *i, page: j }))
}

#[derive(Default)]
/// Calculates the Daf Hashavua Bavli schedule.
pub struct DafHashavuaBavli {}

impl InternalLimudCalculator<Daf> for DafHashavuaBavli {
    fn interval_end_calculation(_cycle: crate::cycle::Cycle, hebrew_date: HebrewDate) -> Option<HebrewDate> {
        let day_number = hebrew_date.day_of_week_number();
        hebrew_date.add_days(7 - day_number)
    }
    fn cycle_finder(&self) -> CycleFinder {
        CycleFinder::Initial(from_gregorian_date(2005, 3, 6))
    }
    fn cycle_end_calculation(hebrew_date: HebrewDate, _iteration: Option<i32>) -> Option<HebrewDate> {
        let day_number = hebrew_date.day_of_week_number();
        hebrew_date.add_days((BAVLI_DAF_COUNT_MODERN * 7) - day_number)
    }

    fn unit_for_interval(&self, interval: &Interval, _limud_date: &HebrewDate) -> Option<Daf> {
        let cycle_iteration = interval.cycle.iteration?;
        let mut iter = iter_daf(cycle_iteration);
        iter.nth((interval.iteration - 1) as usize)
    }
}

impl LimudCalculator<Daf> for DafHashavuaBavli {}
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::date::from_gregorian_date;

    use super::*;

    #[test]
    fn daf_hashavua_bavli_simple_date() {
        let test_date = from_gregorian_date(2018, 10, 10);
        let limud = DafHashavuaBavli::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.tractate, Tractate::Megillah);
        assert_eq!(limud.page, 2);
    }

    #[test]
    fn daf_hashavua_bavli_before_cycle_began() {
        let test_date = from_gregorian_date(2005, 3, 5);
        let limud = DafHashavuaBavli::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn daf_hashavua_bavli_first_day_of_cycle() {
        let test_date = from_gregorian_date(2057, 2, 18);
        let limud = DafHashavuaBavli::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 2);
        assert_eq!(limud.tractate, Tractate::Berachos);
    }

    #[test]
    fn daf_hashavua_bavli_last_day_of_cycle() {
        let test_date = from_gregorian_date(2057, 2, 17);
        let limud = DafHashavuaBavli::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 73);
        assert_eq!(limud.tractate, Tractate::Niddah);
    }
}
