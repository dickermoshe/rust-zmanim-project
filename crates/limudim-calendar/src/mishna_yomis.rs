use crate::{
    constants::MISHNA_YOMIS_CYCLE_DAYS,
    date::{from_gregorian_date, DateExt},
    limud_calculator::{CycleFinder, LimudCalculator},
    units::{Mishna, Tractate, ALL_TRACTATES},
};

const fn chapters(tractate: Tractate) -> usize {
    match tractate {
        Tractate::Berachos => 9,
        Tractate::Peah => 8,
        Tractate::Demai => 7,
        Tractate::Kilayim => 9,
        Tractate::Sheviis => 10,
        Tractate::Terumos => 11,
        Tractate::Maasros => 5,
        Tractate::MaaserSheni => 5,
        Tractate::Chalah => 4,
        Tractate::Orlah => 3,
        Tractate::Bikurim => 4,
        Tractate::Shabbos => 24,
        Tractate::Eruvin => 10,
        Tractate::Pesachim => 10,
        Tractate::Shekalim => 8,
        Tractate::Yoma => 8,
        Tractate::Sukkah => 5,
        Tractate::Beitzah => 5,
        Tractate::RoshHashanah => 4,
        Tractate::Taanis => 4,
        Tractate::Megillah => 4,
        Tractate::MoedKatan => 3,
        Tractate::Chagigah => 3,
        Tractate::Yevamos => 16,
        Tractate::Kesubos => 13,
        Tractate::Nedarim => 11,
        Tractate::Nazir => 9,
        Tractate::Sotah => 9,
        Tractate::Gitin => 9,
        Tractate::Kiddushin => 4,
        Tractate::BavaKamma => 10,
        Tractate::BavaMetzia => 10,
        Tractate::BavaBasra => 10,
        Tractate::Sanhedrin => 11,
        Tractate::Makkos => 3,
        Tractate::Shevuos => 8,
        Tractate::Eduyos => 8,
        Tractate::AvodahZarah => 5,
        Tractate::Avos => 6,
        Tractate::Horiyos => 3,
        Tractate::Zevachim => 14,
        Tractate::Menachos => 13,
        Tractate::Chullin => 12,
        Tractate::Bechoros => 9,
        Tractate::Arachin => 9,
        Tractate::Temurah => 7,
        Tractate::Kerisos => 6,
        Tractate::Meilah => 6,
        Tractate::Tamid => 7,
        Tractate::Midos => 5,
        Tractate::Kinnim => 3,
        Tractate::Keilim => 30,
        Tractate::Ohalos => 18,
        Tractate::Negaim => 14,
        Tractate::Parah => 12,
        Tractate::Taharos => 10,
        Tractate::Mikvaos => 10,
        Tractate::Niddah => 10,
        Tractate::Machshirin => 6,
        Tractate::Zavim => 5,
        Tractate::TevulYom => 4,
        Tractate::Yadayim => 4,
        Tractate::Uktzin => 3,
    }
}
const fn chapter_length(tractate: Tractate, chapter: usize) -> u16 {
    let chapter_index = chapter - 1;
    match tractate {
        Tractate::Berachos => [5, 8, 6, 7, 5, 8, 5, 8, 5][chapter_index],
        Tractate::Peah => [6, 8, 8, 11, 8, 11, 8, 9][chapter_index],
        Tractate::Demai => [4, 5, 6, 7, 11, 12, 8][chapter_index],
        Tractate::Kilayim => [9, 11, 7, 9, 8, 9, 8, 6, 10][chapter_index],
        Tractate::Sheviis => [8, 10, 10, 10, 9, 6, 7, 11, 9, 9][chapter_index],
        Tractate::Terumos => [10, 6, 9, 13, 9, 6, 7, 12, 7, 12, 10][chapter_index],
        Tractate::Maasros => [8, 8, 10, 6, 8][chapter_index],
        Tractate::MaaserSheni => [7, 10, 13, 12, 15][chapter_index],
        Tractate::Chalah => [9, 8, 10, 11][chapter_index],
        Tractate::Orlah => [9, 17, 9][chapter_index],
        Tractate::Bikurim => [11, 11, 12, 5][chapter_index],
        Tractate::Shabbos => [11, 7, 6, 2, 4, 10, 4, 7, 7, 6, 6, 6, 7, 4, 3, 8, 8, 3, 6, 5, 3, 6, 5, 5][chapter_index],
        Tractate::Eruvin => [10, 6, 9, 11, 9, 10, 11, 11, 4, 15][chapter_index],
        Tractate::Pesachim => [7, 8, 8, 9, 10, 6, 13, 8, 11, 9][chapter_index],
        Tractate::Shekalim => [7, 5, 4, 9, 6, 6, 7, 8][chapter_index],
        Tractate::Yoma => [8, 7, 11, 6, 7, 8, 5, 9][chapter_index],
        Tractate::Sukkah => [11, 9, 15, 10, 8][chapter_index],
        Tractate::Beitzah => [10, 10, 8, 7, 7][chapter_index],
        Tractate::RoshHashanah => [9, 9, 8, 9][chapter_index],
        Tractate::Taanis => [7, 10, 9, 8][chapter_index],
        Tractate::Megillah => [11, 6, 6, 10][chapter_index],
        Tractate::MoedKatan => [10, 5, 9][chapter_index],
        Tractate::Chagigah => [8, 7, 8][chapter_index],
        Tractate::Yevamos => [4, 10, 10, 13, 6, 6, 6, 6, 6, 9, 7, 6, 13, 9, 10, 7][chapter_index],
        Tractate::Kesubos => [10, 10, 9, 12, 9, 7, 10, 8, 9, 6, 6, 4, 11][chapter_index],
        Tractate::Nedarim => [4, 5, 11, 8, 6, 10, 9, 7, 10, 8, 12][chapter_index],
        Tractate::Nazir => [7, 10, 7, 7, 7, 11, 4, 2, 5][chapter_index],
        Tractate::Sotah => [9, 6, 8, 5, 5, 4, 8, 7, 15][chapter_index],
        Tractate::Gitin => [6, 7, 8, 9, 9, 7, 9, 10, 10][chapter_index],
        Tractate::Kiddushin => [10, 10, 13, 14][chapter_index],
        Tractate::BavaKamma => [4, 6, 11, 9, 7, 6, 7, 7, 12, 10][chapter_index],
        Tractate::BavaMetzia => [8, 11, 12, 12, 11, 8, 11, 9, 13, 6][chapter_index],
        Tractate::BavaBasra => [6, 14, 8, 9, 11, 8, 4, 8, 10, 8][chapter_index],
        Tractate::Sanhedrin => [6, 5, 8, 5, 5, 6, 11, 7, 6, 6, 6][chapter_index],
        Tractate::Makkos => [10, 8, 16][chapter_index],
        Tractate::Shevuos => [7, 5, 11, 13, 5, 7, 8, 6][chapter_index],
        Tractate::Eduyos => [14, 10, 12, 12, 7, 3, 9, 7][chapter_index],
        Tractate::AvodahZarah => [9, 7, 10, 12, 12][chapter_index],
        Tractate::Avos => [18, 16, 18, 22, 23, 11][chapter_index],
        Tractate::Horiyos => [5, 7, 8][chapter_index],
        Tractate::Zevachim => [4, 5, 6, 6, 8, 7, 6, 12, 7, 8, 8, 6, 8, 10][chapter_index],
        Tractate::Menachos => [4, 5, 7, 5, 9, 7, 6, 7, 9, 9, 9, 5, 11][chapter_index],
        Tractate::Chullin => [7, 10, 7, 7, 5, 7, 6, 6, 8, 4, 2, 5][chapter_index],
        Tractate::Bechoros => [7, 9, 4, 10, 6, 12, 7, 10, 8][chapter_index],
        Tractate::Arachin => [4, 6, 5, 4, 6, 5, 5, 7, 8][chapter_index],
        Tractate::Temurah => [6, 3, 5, 4, 6, 5, 6][chapter_index],
        Tractate::Kerisos => [7, 6, 10, 3, 8, 9][chapter_index],
        Tractate::Meilah => [4, 9, 8, 6, 5, 6][chapter_index],
        Tractate::Tamid => [4, 5, 9, 3, 6, 3, 4][chapter_index],
        Tractate::Midos => [9, 6, 8, 7, 4][chapter_index],
        Tractate::Kinnim => [4, 5, 6][chapter_index],
        Tractate::Keilim => [
            9, 8, 8, 4, 11, 4, 6, 11, 8, 8, 9, 8, 8, 8, 6, 8, 17, 9, 10, 7, 3, 10, 5, 17, 9, 9, 12, 10, 8, 4,
        ][chapter_index],
        Tractate::Ohalos => [8, 7, 7, 3, 7, 7, 6, 6, 16, 7, 9, 8, 6, 7, 10, 5, 5, 10][chapter_index],
        Tractate::Negaim => [6, 5, 8, 11, 5, 8, 5, 10, 3, 10, 12, 7, 12, 13][chapter_index],
        Tractate::Parah => [4, 5, 11, 4, 9, 5, 12, 11, 9, 6, 9, 11][chapter_index],
        Tractate::Taharos => [9, 8, 8, 13, 9, 10, 9, 9, 9, 8][chapter_index],
        Tractate::Mikvaos => [8, 10, 4, 5, 6, 11, 7, 5, 7, 8][chapter_index],
        Tractate::Niddah => [7, 7, 7, 7, 9, 14, 5, 4, 11, 8][chapter_index],
        Tractate::Machshirin => [6, 11, 8, 10, 11, 8][chapter_index],
        Tractate::Zavim => [6, 4, 3, 7, 12][chapter_index],
        Tractate::TevulYom => [5, 8, 6, 7][chapter_index],
        Tractate::Yadayim => [5, 4, 5, 8][chapter_index],
        Tractate::Uktzin => [6, 10, 12][chapter_index],
    }
}

fn iter_mishna() -> impl Iterator<Item = Mishna> {
    ALL_TRACTATES.iter().flat_map(move |t| {
        (1..=chapters(*t)).flat_map(move |c| {
            (1..=chapter_length(*t, c)).map(move |m| Mishna {
                tractate: *t,
                chapter: c,
                mishna: m,
            })
        })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mishnas(pub Mishna, pub Mishna);

#[derive(Default)]
pub struct MishnaYomis;
impl LimudCalculator<Mishnas> for MishnaYomis {
    fn cycle_finder(&self) -> crate::limud_calculator::CycleFinder {
        CycleFinder::Initial(from_gregorian_date(1947, 5, 20))
    }

    fn unit_for_interval(
        &self,
        interval: &crate::interval::Interval,
        _limud_date: &crate::date::HebrewDate,
    ) -> Option<Mishnas> {
        let mut iter = iter_mishna();
        // Each day covers 2 mishnas (unit_step = 2)
        // Offset calculation: ((iteration - 1) * unit_step) + base_unit
        // For iteration 1: ((1-1)*2)+1 = 1 (mishnas 1-2)
        // For iteration 2: ((2-1)*2)+1 = 3 (mishnas 3-4)
        // etc.
        let unit_step = 2i32;
        let base_unit = 1i32;
        let offset = ((interval.iteration - 1) * unit_step) + base_unit;
        // Convert to 0-indexed for nth
        let mishna1 = iter.nth((offset - 1) as usize)?;
        let mishna2 = iter.next()?;
        Some(Mishnas(mishna1, mishna2))
    }
    fn cycle_end_calculation(
        hebrew_date: crate::date::HebrewDate,
        _iteration: Option<i32>,
    ) -> Option<crate::date::HebrewDate> {
        hebrew_date.add_days(MISHNA_YOMIS_CYCLE_DAYS)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::date::from_gregorian_date;

    use super::*;

    #[test]
    fn mishna_yomis_simple_date() {
        let test_date = from_gregorian_date(2017, 12, 28);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        // Python test expects: 'megillah 3:4-5'
        assert_eq!(limud.0.tractate, Tractate::Megillah);
        assert_eq!(limud.0.chapter, 3);
        assert_eq!(limud.0.mishna, 4);
        assert_eq!(limud.1.tractate, Tractate::Megillah);
        assert_eq!(limud.1.chapter, 3);
        assert_eq!(limud.1.mishna, 5);
    }

    #[test]
    fn mishna_yomis_before_cycle_began() {
        let test_date = from_gregorian_date(1947, 1, 1);
        let limud = MishnaYomis.limud(test_date);
        assert!(limud.is_none());
    }

    #[test]
    fn mishna_yomis_first_day_of_cycle() {
        let test_date = from_gregorian_date(2016, 3, 30);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        // Python test expects: 'berachos 1:1-2'
        assert_eq!(limud.0.tractate, Tractate::Berachos);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 1);
        assert_eq!(limud.1.tractate, Tractate::Berachos);
        assert_eq!(limud.1.chapter, 1);
        assert_eq!(limud.1.mishna, 2);
    }

    #[test]
    fn mishna_yomis_last_day_of_cycle() {
        let test_date = from_gregorian_date(2016, 3, 29);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        // Python test expects: 'uktzin 3:11-12'
        assert_eq!(limud.0.tractate, Tractate::Uktzin);
        assert_eq!(limud.0.chapter, 3);
        assert_eq!(limud.0.mishna, 11);
        assert_eq!(limud.1.tractate, Tractate::Uktzin);
        assert_eq!(limud.1.chapter, 3);
        assert_eq!(limud.1.mishna, 12);
    }

    #[test]
    fn mishna_yomis_span_two_masechtos() {
        let test_date = from_gregorian_date(2016, 4, 27);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        // Python test expects: 'berachos 9:5 - peah 1:1'
        assert_eq!(limud.0.tractate, Tractate::Berachos);
        assert_eq!(limud.0.chapter, 9);
        assert_eq!(limud.0.mishna, 5);
        assert_eq!(limud.1.tractate, Tractate::Peah);
        assert_eq!(limud.1.chapter, 1);
        assert_eq!(limud.1.mishna, 1);
    }

    #[test]
    fn mishna_yomis_span_two_perakim() {
        let test_date = from_gregorian_date(2017, 12, 23);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        // Python test expects: 'megillah 1:11-2:1'
        assert_eq!(limud.0.tractate, Tractate::Megillah);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 11);
        assert_eq!(limud.1.tractate, Tractate::Megillah);
        assert_eq!(limud.1.chapter, 2);
        assert_eq!(limud.1.mishna, 1);
    }

    #[test]
    fn mishna_yomis_1947_05_20() {
        let test_date = from_gregorian_date(1947, 5, 20);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Berachos);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 1);
        assert_eq!(limud.1.tractate, Tractate::Berachos);
        assert_eq!(limud.1.chapter, 1);
        assert_eq!(limud.1.mishna, 2);
    }

    #[test]
    fn mishna_yomis_1950_01_01() {
        let test_date = from_gregorian_date(1950, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::BavaKamma);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 1);
        assert_eq!(limud.1.tractate, Tractate::BavaKamma);
        assert_eq!(limud.1.chapter, 1);
        assert_eq!(limud.1.mishna, 2);
    }

    #[test]
    fn mishna_yomis_1960_01_01() {
        let test_date = from_gregorian_date(1960, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Eruvin);
        assert_eq!(limud.0.chapter, 5);
        assert_eq!(limud.0.mishna, 5);
        assert_eq!(limud.1.tractate, Tractate::Eruvin);
        assert_eq!(limud.1.chapter, 5);
        assert_eq!(limud.1.mishna, 6);
    }

    #[test]
    fn mishna_yomis_1970_01_01() {
        let test_date = from_gregorian_date(1970, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Mikvaos);
        assert_eq!(limud.0.chapter, 10);
        assert_eq!(limud.0.mishna, 6);
        assert_eq!(limud.1.tractate, Tractate::Mikvaos);
        assert_eq!(limud.1.chapter, 10);
        assert_eq!(limud.1.mishna, 7);
    }

    #[test]
    fn mishna_yomis_1980_01_01() {
        let test_date = from_gregorian_date(1980, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Bechoros);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 2);
        assert_eq!(limud.1.tractate, Tractate::Bechoros);
        assert_eq!(limud.1.chapter, 1);
        assert_eq!(limud.1.mishna, 3);
    }

    #[test]
    fn mishna_yomis_1990_01_01() {
        let test_date = from_gregorian_date(1990, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Sotah);
        assert_eq!(limud.0.chapter, 9);
        assert_eq!(limud.0.mishna, 14);
        assert_eq!(limud.1.tractate, Tractate::Sotah);
        assert_eq!(limud.1.chapter, 9);
        assert_eq!(limud.1.mishna, 15);
    }

    #[test]
    fn mishna_yomis_2000_01_01() {
        let test_date = from_gregorian_date(2000, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Shabbos);
        assert_eq!(limud.0.chapter, 9);
        assert_eq!(limud.0.mishna, 5);
        assert_eq!(limud.1.tractate, Tractate::Shabbos);
        assert_eq!(limud.1.chapter, 9);
        assert_eq!(limud.1.mishna, 6);
    }

    #[test]
    fn mishna_yomis_2010_01_01() {
        let test_date = from_gregorian_date(2010, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Taharos);
        assert_eq!(limud.0.chapter, 4);
        assert_eq!(limud.0.mishna, 12);
        assert_eq!(limud.1.tractate, Tractate::Taharos);
        assert_eq!(limud.1.chapter, 4);
        assert_eq!(limud.1.mishna, 13);
    }

    #[test]
    fn mishna_yomis_2016_04_01() {
        let test_date = from_gregorian_date(2016, 4, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        // Python: 'berachos 1:5-2:1' - spans two chapters
        assert_eq!(limud.0.tractate, Tractate::Berachos);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 5);
        assert_eq!(limud.1.tractate, Tractate::Berachos);
        assert_eq!(limud.1.chapter, 2);
        assert_eq!(limud.1.mishna, 1);
    }

    #[test]
    fn mishna_yomis_2016_05_01() {
        let test_date = from_gregorian_date(2016, 5, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Peah);
        assert_eq!(limud.0.chapter, 2);
        assert_eq!(limud.0.mishna, 2);
        assert_eq!(limud.1.tractate, Tractate::Peah);
        assert_eq!(limud.1.chapter, 2);
        assert_eq!(limud.1.mishna, 3);
    }

    #[test]
    fn mishna_yomis_2017_01_01() {
        let test_date = from_gregorian_date(2017, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Chalah);
        assert_eq!(limud.0.chapter, 2);
        assert_eq!(limud.0.mishna, 3);
        assert_eq!(limud.1.tractate, Tractate::Chalah);
        assert_eq!(limud.1.chapter, 2);
        assert_eq!(limud.1.mishna, 4);
    }

    #[test]
    fn mishna_yomis_2017_06_01() {
        let test_date = from_gregorian_date(2017, 6, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Eruvin);
        assert_eq!(limud.0.chapter, 7);
        assert_eq!(limud.0.mishna, 8);
        assert_eq!(limud.1.tractate, Tractate::Eruvin);
        assert_eq!(limud.1.chapter, 7);
        assert_eq!(limud.1.mishna, 9);
    }

    #[test]
    fn mishna_yomis_2017_12_01() {
        let test_date = from_gregorian_date(2017, 12, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Taanis);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 1);
        assert_eq!(limud.1.tractate, Tractate::Taanis);
        assert_eq!(limud.1.chapter, 1);
        assert_eq!(limud.1.mishna, 2);
    }

    #[test]
    fn mishna_yomis_2018_01_01() {
        let test_date = from_gregorian_date(2018, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Megillah);
        assert_eq!(limud.0.chapter, 4);
        assert_eq!(limud.0.mishna, 6);
        assert_eq!(limud.1.tractate, Tractate::Megillah);
        assert_eq!(limud.1.chapter, 4);
        assert_eq!(limud.1.mishna, 7);
    }

    #[test]
    fn mishna_yomis_2020_01_01() {
        let test_date = from_gregorian_date(2020, 1, 1);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Menachos);
        assert_eq!(limud.0.chapter, 8);
        assert_eq!(limud.0.mishna, 2);
        assert_eq!(limud.1.tractate, Tractate::Menachos);
        assert_eq!(limud.1.chapter, 8);
        assert_eq!(limud.1.mishna, 3);
    }

    #[test]
    fn mishna_yomis_2016_03_31() {
        let test_date = from_gregorian_date(2016, 3, 31);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Berachos);
        assert_eq!(limud.0.chapter, 1);
        assert_eq!(limud.0.mishna, 3);
        assert_eq!(limud.1.tractate, Tractate::Berachos);
        assert_eq!(limud.1.chapter, 1);
        assert_eq!(limud.1.mishna, 4);
    }

    #[test]
    fn mishna_yomis_2016_04_02() {
        let test_date = from_gregorian_date(2016, 4, 2);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Berachos);
        assert_eq!(limud.0.chapter, 2);
        assert_eq!(limud.0.mishna, 2);
        assert_eq!(limud.1.tractate, Tractate::Berachos);
        assert_eq!(limud.1.chapter, 2);
        assert_eq!(limud.1.mishna, 3);
    }

    #[test]
    fn mishna_yomis_2017_12_24() {
        let test_date = from_gregorian_date(2017, 12, 24);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Megillah);
        assert_eq!(limud.0.chapter, 2);
        assert_eq!(limud.0.mishna, 2);
        assert_eq!(limud.1.tractate, Tractate::Megillah);
        assert_eq!(limud.1.chapter, 2);
        assert_eq!(limud.1.mishna, 3);
    }

    #[test]
    fn mishna_yomis_2017_12_25() {
        let test_date = from_gregorian_date(2017, 12, 25);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Megillah);
        assert_eq!(limud.0.chapter, 2);
        assert_eq!(limud.0.mishna, 4);
        assert_eq!(limud.1.tractate, Tractate::Megillah);
        assert_eq!(limud.1.chapter, 2);
        assert_eq!(limud.1.mishna, 5);
    }

    #[test]
    fn mishna_yomis_2017_12_26() {
        let test_date = from_gregorian_date(2017, 12, 26);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        // Python: 'megillah 2:6-3:1' - spans two chapters
        assert_eq!(limud.0.tractate, Tractate::Megillah);
        assert_eq!(limud.0.chapter, 2);
        assert_eq!(limud.0.mishna, 6);
        assert_eq!(limud.1.tractate, Tractate::Megillah);
        assert_eq!(limud.1.chapter, 3);
        assert_eq!(limud.1.mishna, 1);
    }

    #[test]
    fn mishna_yomis_2017_12_27() {
        let test_date = from_gregorian_date(2017, 12, 27);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::Megillah);
        assert_eq!(limud.0.chapter, 3);
        assert_eq!(limud.0.mishna, 2);
        assert_eq!(limud.1.tractate, Tractate::Megillah);
        assert_eq!(limud.1.chapter, 3);
        assert_eq!(limud.1.mishna, 3);
    }

    #[test]
    fn mishna_yomis_2012_02_26_rosh_hashanah_chapter_boundary() {
        // Regression test: Rosh Hashanah chapter 2 has 9 mishnayot, not 8
        // This date crosses from chapter 2 to chapter 3
        let test_date = from_gregorian_date(2012, 2, 26);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::RoshHashanah);
        assert_eq!(limud.0.chapter, 2);
        assert_eq!(limud.0.mishna, 9);
        assert_eq!(limud.1.tractate, Tractate::RoshHashanah);
        assert_eq!(limud.1.chapter, 3);
        assert_eq!(limud.1.mishna, 1);
    }

    #[test]
    fn mishna_yomis_2012_02_27_rosh_hashanah_chapter_3() {
        // Regression test: Day after crossing chapter boundary
        let test_date = from_gregorian_date(2012, 2, 27);
        let limud = MishnaYomis.limud(test_date).expect("limud exists");
        assert_eq!(limud.0.tractate, Tractate::RoshHashanah);
        assert_eq!(limud.0.chapter, 3);
        assert_eq!(limud.0.mishna, 2);
        assert_eq!(limud.1.tractate, Tractate::RoshHashanah);
        assert_eq!(limud.1.chapter, 3);
        assert_eq!(limud.1.mishna, 3);
    }
}
