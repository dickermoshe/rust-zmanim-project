use icu_calendar::options::DateDifferenceOptions;

use crate::{
    constants::BAVLI_TOTAL_AMUDIM,
    date::{from_gregorian_date, DateExt, HebrewDate},
    interval::Interval,
    limud_calculator::{CycleFinder, InternalLimudCalculator},
    units::*,
    LimudCalculator,
};

pub const fn start_daf(tractate: Tractate, _iteration: i32) -> Amud {
    match tractate {
        Tractate::Kinnim => Amud::new(Tractate::Kinnim, 22, Side::Bet),
        Tractate::Tamid => Amud::new(Tractate::Tamid, 25, Side::Bet),
        Tractate::Midos => Amud::new(Tractate::Midos, 34, Side::Aleph),
        _ => Amud::new(tractate, 2, Side::Aleph),
    }
}

pub const fn end_daf(tractate: Tractate, _iteration: i32) -> Option<Amud> {
    let amud = match tractate {
        Tractate::Berachos => Amud::new(Tractate::Berachos, 64, Side::Aleph),
        Tractate::Shabbos => Amud::new(Tractate::Shabbos, 157, Side::Bet),
        Tractate::Eruvin => Amud::new(Tractate::Eruvin, 105, Side::Aleph),
        Tractate::Pesachim => Amud::new(Tractate::Pesachim, 121, Side::Bet),
        Tractate::Shekalim => Amud::new(Tractate::Shekalim, 22, Side::Bet),
        Tractate::Yoma => Amud::new(Tractate::Yoma, 88, Side::Aleph),
        Tractate::Sukkah => Amud::new(Tractate::Sukkah, 56, Side::Bet),
        Tractate::Beitzah => Amud::new(Tractate::Beitzah, 40, Side::Bet),
        Tractate::RoshHashanah => Amud::new(Tractate::RoshHashanah, 35, Side::Aleph),
        Tractate::Taanis => Amud::new(Tractate::Taanis, 31, Side::Aleph),
        Tractate::Megillah => Amud::new(Tractate::Megillah, 32, Side::Aleph),
        Tractate::MoedKatan => Amud::new(Tractate::MoedKatan, 29, Side::Aleph),
        Tractate::Chagigah => Amud::new(Tractate::Chagigah, 27, Side::Aleph),
        Tractate::Yevamos => Amud::new(Tractate::Yevamos, 122, Side::Bet),
        Tractate::Kesubos => Amud::new(Tractate::Kesubos, 112, Side::Bet),
        Tractate::Nedarim => Amud::new(Tractate::Nedarim, 91, Side::Bet),
        Tractate::Nazir => Amud::new(Tractate::Nazir, 66, Side::Bet),
        Tractate::Sotah => Amud::new(Tractate::Sotah, 49, Side::Bet),
        Tractate::Gitin => Amud::new(Tractate::Gitin, 90, Side::Bet),
        Tractate::Kiddushin => Amud::new(Tractate::Kiddushin, 82, Side::Bet),
        Tractate::BavaKamma => Amud::new(Tractate::BavaKamma, 119, Side::Bet),
        Tractate::BavaMetzia => Amud::new(Tractate::BavaMetzia, 119, Side::Aleph),
        Tractate::BavaBasra => Amud::new(Tractate::BavaBasra, 176, Side::Bet),
        Tractate::Sanhedrin => Amud::new(Tractate::Sanhedrin, 113, Side::Bet),
        Tractate::Makkos => Amud::new(Tractate::Makkos, 24, Side::Bet),
        Tractate::Shevuos => Amud::new(Tractate::Shevuos, 49, Side::Bet),
        Tractate::AvodahZarah => Amud::new(Tractate::AvodahZarah, 76, Side::Bet),
        Tractate::Horiyos => Amud::new(Tractate::Horiyos, 14, Side::Aleph),
        Tractate::Zevachim => Amud::new(Tractate::Zevachim, 120, Side::Bet),
        Tractate::Menachos => Amud::new(Tractate::Menachos, 110, Side::Aleph),
        Tractate::Chullin => Amud::new(Tractate::Chullin, 142, Side::Aleph),
        Tractate::Bechoros => Amud::new(Tractate::Bechoros, 61, Side::Aleph),
        Tractate::Arachin => Amud::new(Tractate::Arachin, 34, Side::Aleph),
        Tractate::Temurah => Amud::new(Tractate::Temurah, 34, Side::Aleph),
        Tractate::Kerisos => Amud::new(Tractate::Kerisos, 28, Side::Bet),
        Tractate::Meilah => Amud::new(Tractate::Meilah, 22, Side::Aleph),
        Tractate::Kinnim => Amud::new(Tractate::Kinnim, 25, Side::Aleph),
        Tractate::Tamid => Amud::new(Tractate::Tamid, 33, Side::Bet),
        Tractate::Midos => Amud::new(Tractate::Midos, 37, Side::Bet),
        Tractate::Niddah => Amud::new(Tractate::Niddah, 73, Side::Aleph),
        _ => Amud::new(Tractate::Bechoros, 0, Side::Aleph),
    };
    if amud.page == 0 {
        None
    } else {
        Some(amud)
    }
}

const fn iter(tractate: Tractate, iteration: i32) -> AmudIter {
    let end = end_daf(tractate, iteration);
    if let Some(end) = end {
        AmudIter::new(start_daf(tractate, iteration), end)
    } else {
        AmudIter::empty()
    }
}
fn iter_amud(iteration: i32) -> impl Iterator<Item = Amud> {
    BAVLI_TRACTATES.iter().flat_map(move |i| iter(*i, iteration))
}
#[derive(Default)]
/// Calculates the Amud Yomi Bavli Dirshu schedule.
pub struct AmudYomiBavliDirshu {}

impl InternalLimudCalculator<Amud> for AmudYomiBavliDirshu {
    fn cycle_finder(&self) -> CycleFinder {
        CycleFinder::Initial(from_gregorian_date(2023, 10, 16))
    }
    fn cycle_end_calculation(hebrew_date: HebrewDate, _iteration: Option<i32>) -> Option<HebrewDate> {
        hebrew_date.add_days(BAVLI_TOTAL_AMUDIM - 1)
    }

    fn unit_for_interval(&self, interval: &Interval, limud_date: &HebrewDate) -> Option<Amud> {
        let cycle_iteration = interval.cycle.iteration?;
        let mut iter = iter_amud(cycle_iteration);
        // Offset from cycle start_date to limud_date
        let offset = interval
            .cycle
            .start_date
            .try_until_with_options(limud_date, DateDifferenceOptions::default())
            .ok()?
            .days;
        iter.nth(offset as usize)
    }
}
impl LimudCalculator<Amud> for AmudYomiBavliDirshu {}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::date::from_gregorian_date;

    use super::*;

    #[test]
    fn amud_yomi_bavli_dirshu_simple_date() {
        let test_date = from_gregorian_date(2024, 5, 30);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 53);
        assert_eq!(limud.side, Side::Aleph);
        assert_eq!(limud.tractate, Tractate::Shabbos);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_before_cycle_began() {
        let test_date = from_gregorian_date(2023, 1, 1);
        let limud = AmudYomiBavliDirshu::default().limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn amud_yomi_bavli_dirshu_first_day_of_cycle() {
        let test_date = from_gregorian_date(2038, 8, 4);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 2);
        assert_eq!(limud.side, Side::Aleph);
        assert_eq!(limud.tractate, Tractate::Berachos);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_last_day_of_cycle() {
        let test_date = from_gregorian_date(2038, 8, 3);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 73);
        assert_eq!(limud.side, Side::Aleph);
        assert_eq!(limud.tractate, Tractate::Niddah);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_end_of_meilah() {
        let test_date = from_gregorian_date(2038, 2, 10);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 22);
        assert_eq!(limud.side, Side::Aleph);
        assert_eq!(limud.tractate, Tractate::Meilah);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_beginning_of_kinnim() {
        let test_date = from_gregorian_date(2038, 2, 11);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 22);
        assert_eq!(limud.side, Side::Bet);
        assert_eq!(limud.tractate, Tractate::Kinnim);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_end_of_kinnim() {
        let test_date = from_gregorian_date(2038, 2, 16);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 25);
        assert_eq!(limud.side, Side::Aleph);
        assert_eq!(limud.tractate, Tractate::Kinnim);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_beginning_of_tamid() {
        let test_date = from_gregorian_date(2038, 2, 17);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 25);
        assert_eq!(limud.side, Side::Bet);
        assert_eq!(limud.tractate, Tractate::Tamid);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_end_of_tamid() {
        let test_date = from_gregorian_date(2038, 3, 5);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 33);
        assert_eq!(limud.side, Side::Bet);
        assert_eq!(limud.tractate, Tractate::Tamid);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_beginning_of_midos() {
        let test_date = from_gregorian_date(2038, 3, 6);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 34);
        assert_eq!(limud.side, Side::Aleph);
        assert_eq!(limud.tractate, Tractate::Midos);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_end_of_midos() {
        let test_date = from_gregorian_date(2038, 3, 13);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 37);
        assert_eq!(limud.side, Side::Bet);
        assert_eq!(limud.tractate, Tractate::Midos);
    }

    #[test]
    fn amud_yomi_bavli_dirshu_after_midos() {
        let test_date = from_gregorian_date(2038, 3, 14);
        let limud = AmudYomiBavliDirshu::default().limud(test_date).expect("limud exists");
        assert_eq!(limud.page, 2);
        assert_eq!(limud.side, Side::Aleph);
        assert_eq!(limud.tractate, Tractate::Niddah);
    }
}
