#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use crate::{zman::*, Location, ZmanimCalculator};
    use chrono::NaiveDate;
    use chrono_tz::Tz;

    fn polar_day_calc() -> ZmanimCalculator<Tz> {
        let date = NaiveDate::from_ymd_opt(2017, 6, 21).unwrap();
        let location = Location::new(69.6492, 18.9553, 0.0, Some(chrono_tz::Europe::Oslo)).unwrap();
        ZmanimCalculator::new(location, date, Default::default()).unwrap()
    }

    #[test]
    fn test_bain_hashmashos_uses_elevation() {
        assert!(!ZmanLike::<chrono_tz::Tz>::uses_elevation(
            &BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES
        ));
    }

    #[test]
    fn test_polar_day_zmanim_return_none() {
        let alos_variants: [&dyn ZmanLike<Tz>; 9] = [
            &ALOS_60_MINUTES,
            &ALOS_72_MINUTES,
            &ALOS_72_ZMANIS,
            &ALOS_90_MINUTES,
            &ALOS_90_ZMANIS,
            &ALOS_96_MINUTES,
            &ALOS_96_ZMANIS,
            &ALOS_120_MINUTES,
            &ALOS_120_ZMANIS,
        ];
        for zman in alos_variants {
            let mut calc = polar_day_calc();
            assert!(zman.calculate(&mut calc).is_none());
        }

        let bain_variants: [&dyn ZmanLike<Tz>; 6] = [
            &BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES,
            &BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES,
            &BAIN_HASHMASHOS_RT_2_STARS,
            &BAIN_HASHMASHOS_YEREIM_18_MINUTES,
            &BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES,
            &BAIN_HASHMASHOS_YEREIM_13_POINT_5_MINUTES,
        ];
        for zman in bain_variants {
            let mut calc = polar_day_calc();
            assert!(zman.calculate(&mut calc).is_none());
        }

        let mut calc = polar_day_calc();
        assert!(CANDLE_LIGHTING.calculate(&mut calc).is_none());

        let mut calc = polar_day_calc();
        assert!(CHATZOS_HALF_DAY.calculate(&mut calc).is_none());

        let mincha_variants: [&dyn ZmanLike<Tz>; 7] = [
            &MINCHA_GEDOLA_16_POINT_1_DEGREES,
            &MINCHA_GEDOLA_MINUTES_72,
            &MINCHA_GEDOLA_AHAVAT_SHALOM,
            &MINCHA_GEDOLA_ATERET_TORAH,
            &MINCHA_GEDOLA_BAAL_HATANYA,
            &MINCHA_GEDOLA_BAAL_HATANYA_GREATER_THAN_30,
            &MINCHA_GEDOLA_GREATER_THAN_30,
        ];
        for zman in mincha_variants {
            let mut calc = polar_day_calc();
            assert!(zman.calculate(&mut calc).is_none());
        }
    }
}
