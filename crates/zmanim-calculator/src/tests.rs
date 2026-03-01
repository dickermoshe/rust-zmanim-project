#![allow(clippy::unwrap_used)]

use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
extern crate std;
use std::{str::FromStr, string::String, string::ToString};

use crate::calculator::{ZmanLike, ZmanimCalculator};

use crate::presets::*;
use crate::primitive_zman::ZmanPrimitive;
use crate::{duration_helper::multiply_duration, Location};

const LAKEWOOD_LAT: f64 = 40.0721087;
const LAKEWOOD_LON: f64 = -74.2400243;
const LAKEWOOD_ELEVATION_M: f64 = 15.0;
const MAX_TIME_DIFF_SECONDS: i64 = 30;
const MAX_SHAAH_ZMANIS_DIFF_MS: i64 = 1000;

fn lakewood_tz() -> Tz {
    Tz::from_str("America/New_York").unwrap()
}

fn lakewood_location(elevation_m: f64) -> Location<Tz> {
    Location::new(LAKEWOOD_LAT, LAKEWOOD_LON, elevation_m, Some(lakewood_tz())).unwrap()
}

fn lakewood_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2017, 10, 17).unwrap()
}

fn new_calc(elevation_m: f64) -> ZmanimCalculator<Tz> {
    ZmanimCalculator::new(
        lakewood_location(elevation_m),
        lakewood_date(),
        Default::default(),
    )
    .unwrap()
}

fn calc_for(lat: f64, lon: f64, elevation: f64, tz: Tz, date: NaiveDate) -> ZmanimCalculator<Tz> {
    let location = Location::new(lat, lon, elevation, Some(tz)).unwrap();
    ZmanimCalculator::new(location, date, Default::default()).unwrap()
}

fn fmt_local(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&lakewood_tz())
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

fn assert_zman_str(calc: &mut ZmanimCalculator<Tz>, zman: &dyn ZmanLike<Tz>, expected: &str) {
    let dt = zman.calculate(calc).unwrap();
    assert_time_str(dt, expected, None);
}
fn assert_zman_str_with_max_time_diff(
    calc: &mut ZmanimCalculator<Tz>,
    zman: &dyn ZmanLike<Tz>,
    expected: &str,
    max_time_diff_seconds: Option<i64>,
) {
    let dt = zman.calculate(calc).unwrap();
    assert_time_str(dt, expected, max_time_diff_seconds);
}

fn assert_time_str(dt: DateTime<Utc>, expected: &str, max_time_diff_seconds: Option<i64>) {
    let expected_dt = DateTime::parse_from_rfc3339(expected)
        .unwrap()
        .with_timezone(&Utc);
    let diff = (dt - expected_dt).num_seconds().abs();
    assert!(
        diff <= max_time_diff_seconds.unwrap_or(MAX_TIME_DIFF_SECONDS),
        "time mismatch: expected {}, got {} (diff {}s)",
        expected,
        fmt_local(dt),
        diff
    );
}

fn assert_duration_ms_close(actual: Duration, expected_ms: i64) {
    let actual_ms = actual.num_milliseconds();
    let diff = (actual_ms - expected_ms).abs();
    assert!(
        diff <= MAX_SHAAH_ZMANIS_DIFF_MS,
        "duration mismatch: expected {}ms, got {}ms (diff {}ms)",
        expected_ms,
        actual_ms,
        diff
    );
}

fn shaah_zmanis_by_degrees_and_offset(
    calc: &mut ZmanimCalculator<Tz>,
    degrees: f64,
    offset_minutes: i64,
) -> Duration {
    let (start, end) = if degrees.abs() > f64::EPSILON {
        (
            ZmanPrimitive::SunriseOffsetByDegrees(degrees)
                .calculate(calc)
                .unwrap(),
            ZmanPrimitive::SunsetOffsetByDegrees(degrees)
                .calculate(calc)
                .unwrap(),
        )
    } else {
        (
            ZmanPrimitive::Sunrise.calculate(calc).unwrap(),
            ZmanPrimitive::Sunset.calculate(calc).unwrap(),
        )
    };
    let start = start - Duration::minutes(offset_minutes);
    let end = end + Duration::minutes(offset_minutes);
    (end - start) / 12
}

#[test]
fn test_new_without_timezone_uses_longitude_offset() {
    let location = Location::new(0.0, 30.0, 0.0, Option::<Utc>::None).unwrap();
    let calc = ZmanimCalculator::new(
        location,
        NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        Default::default(),
    );
    assert!(calc.is_ok());
}

#[test]
fn test_shaah_zmanis_from_zmanim() {
    let mut calc = new_calc(0.0);
    let alos = ALOS_16_POINT_1_DEGREES.calculate(&mut calc).unwrap();
    let tzais = TZAIS_16_POINT_1_DEGREES.calculate(&mut calc).unwrap();
    let shaah = (tzais - alos) / 12;
    assert!(shaah.num_seconds() > 0);
}

#[test]
fn test_local_mean_time_invalid_hours() {
    let mut calc = new_calc(0.0);
    let date = lakewood_date();
    #[allow(clippy::clone_on_copy)]
    let location = calc.location.clone();
    assert!(calc.local_mean_time(date, &location, -1.0).is_err());
}

#[test]
fn test_half_day_based_zman_negative_hours() {
    let mut calc = new_calc(0.0);
    let sunrise = ZmanPrimitive::Sunrise.calculate(&mut calc).unwrap();
    let sunset = ZmanPrimitive::Sunset.calculate(&mut calc).unwrap();
    let shaah = (sunset - sunrise) / 6;
    let expected = sunset + multiply_duration(shaah, -1.0).unwrap();
    let actual =
        ZmanPrimitive::HalfDayBasedOffset(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, -1.0)
            .calculate(&mut calc)
            .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_high_latitude_sunrise_sunset_ordering() {
    let date = NaiveDate::from_ymd_opt(2017, 3, 21).unwrap();
    let mut calc = calc_for(64.1466, -21.9426, 0.0, chrono_tz::Atlantic::Reykjavik, date);
    #[allow(clippy::expect_used)]
    let sunrise = ZmanPrimitive::Sunrise.calculate(&mut calc).unwrap();
    #[allow(clippy::expect_used)]
    let sunset = ZmanPrimitive::Sunset.calculate(&mut calc).unwrap();
    assert!(sunrise < sunset);

    let dawn = ZmanPrimitive::SunriseOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .unwrap();
    let dusk = ZmanPrimitive::SunsetOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .unwrap();
    assert!(dawn < sunrise);
    assert!(dusk > sunset);
}

#[test]
fn test_extreme_elevation_shifts_sunrise_sunset() {
    let date = NaiveDate::from_ymd_opt(2017, 10, 17).unwrap();
    let mut high = calc_for(27.9881, 86.9250, 8848.0, chrono_tz::Asia::Kathmandu, date);
    let mut sea = calc_for(27.9881, 86.9250, 0.0, chrono_tz::Asia::Kathmandu, date);

    let sunrise_high = ZmanPrimitive::Sunrise.calculate(&mut high).unwrap();
    let sunrise_sea = ZmanPrimitive::Sunrise.calculate(&mut sea).unwrap();
    assert!(sunrise_high < sunrise_sea);

    let sunset_high = ZmanPrimitive::Sunset.calculate(&mut high).unwrap();
    let sunset_sea = ZmanPrimitive::Sunset.calculate(&mut sea).unwrap();
    assert!(sunset_high > sunset_sea);
}

#[test]
fn test_polar_day_returns_none_for_sun_times() {
    let date = NaiveDate::from_ymd_opt(2017, 6, 21).unwrap();
    let mut calc = calc_for(69.6492, 18.9553, 0.0, chrono_tz::Europe::Oslo, date);

    assert!(ZmanPrimitive::Sunrise.calculate(&mut calc).is_err());
    assert!(ZmanPrimitive::Sunset.calculate(&mut calc).is_err());
    assert!(ZmanPrimitive::SeaLevelSunrise.calculate(&mut calc).is_err());
    assert!(ZmanPrimitive::SeaLevelSunset.calculate(&mut calc).is_err());
    assert!(ZmanPrimitive::SunriseOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .is_err());
    assert!(ZmanPrimitive::SunsetOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .is_err());
}

#[test]
fn test_new_returns_none_for_invalid_location() {
    let location = Location {
        latitude: f64::NAN,
        longitude: 0.0,
        elevation: 0.0,
        timezone: Some(Utc),
    };
    let calc = ZmanimCalculator::new(
        location,
        NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        Default::default(),
    );
    assert!(calc.is_err());
}

#[test]
fn test_reykjavik_equinox_java_expected_times() {
    let date = NaiveDate::from_ymd_opt(2017, 3, 21).unwrap();
    let mut calc = calc_for(64.1466, -21.9426, 0.0, chrono_tz::Atlantic::Reykjavik, date);

    assert_zman_str(&mut calc, &SUNRISE, "2017-03-21T07:24:24Z");
    assert_zman_str(&mut calc, &SUNSET, "2017-03-21T19:46:56Z");
    assert_zman_str(&mut calc, &SEA_LEVEL_SUNRISE, "2017-03-21T07:24:24Z");
    assert_zman_str(&mut calc, &SEA_LEVEL_SUNSET, "2017-03-21T19:46:56Z");
    assert_zman_str(&mut calc, &CHATZOS_ASTRONOMICAL, "2017-03-21T13:34:59Z");
}

#[test]
fn test_everest_java_expected_times() {
    let date = NaiveDate::from_ymd_opt(2017, 10, 17).unwrap();
    let mut calc = calc_for(27.9881, 86.9250, 8826.0, chrono_tz::Asia::Kathmandu, date);
    // At very high elevation our refraction model and javas refraction model start to differ slightly, so we allow for a larger time difference.
    assert_zman_str_with_max_time_diff(&mut calc, &SUNRISE, "2017-10-17T05:44:49+05:45", Some(120));
    assert_zman_str_with_max_time_diff(&mut calc, &SUNSET, "2017-10-17T17:40:04+05:45", Some(120));
    assert_zman_str_with_max_time_diff(
        &mut calc,
        &SEA_LEVEL_SUNRISE,
        "2017-10-17T05:58:42+05:45",
        Some(120),
    );
    assert_zman_str_with_max_time_diff(
        &mut calc,
        &SEA_LEVEL_SUNSET,
        "2017-10-17T17:26:12+05:45",
        Some(120),
    );
    assert_zman_str_with_max_time_diff(
        &mut calc,
        &CHATZOS_ASTRONOMICAL,
        "2017-10-17T11:42:44+05:45",
        Some(120),
    );
}

#[test]
fn test_default_zmanim_times() {
    let mut calc = new_calc(0.0);

    assert_eq!(
        SEA_LEVEL_SUNRISE.calculate(&mut calc),
        ZmanPrimitive::SeaLevelSunrise.calculate(&mut calc)
    );
    assert_eq!(
        SEA_LEVEL_SUNSET.calculate(&mut calc),
        ZmanPrimitive::SeaLevelSunset.calculate(&mut calc)
    );

    assert_zman_str(
        &mut new_calc(0.0),
        &TZAIS_DEGREES_8_POINT_5,
        "2017-10-17T18:54:29-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &TZAIS_19_POINT_8_DEGREES,
        "2017-10-17T19:53:34-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &TZAIS_MINUTES_60,
        "2017-10-17T19:13:58-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &TZAIS_90_ZMANIS,
        "2017-10-17T19:36:59-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &TZAIS_MINUTES_72,
        "2017-10-17T19:25:58-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        &ALOS_16_POINT_1_DEGREES,
        "2017-10-17T05:49:30-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &ALOS_19_POINT_8_DEGREES,
        "2017-10-17T05:30:07-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &ALOS_60_MINUTES,
        "2017-10-17T06:09:51-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &ALOS_90_ZMANIS,
        "2017-10-17T05:46:50-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &ALOS_72_MINUTES,
        "2017-10-17T05:57:51-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        &CHATZOS_ASTRONOMICAL,
        "2017-10-17T12:41:55-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &MINCHA_GEDOLA_SUNRISE_SUNSET,
        "2017-10-17T13:09:35-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &MINCHA_KETANA_SUNRISE_SUNSET,
        "2017-10-17T15:55:37-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &PLAG_HAMINCHA_SUNRISE_SUNSET,
        "2017-10-17T17:04:48-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &CANDLE_LIGHTING,
        "2017-10-17T17:55:58-04:00",
    );
}

#[test]
fn test_default_zmanim_calculations() {
    let mut calc = new_calc(0.0);

    let sof_zman_shma = ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&mut calc)
    .unwrap();
    assert_time_str(sof_zman_shma, "2017-10-17T09:42:10-04:00", None);

    assert_zman_str(
        &mut new_calc(0.0),
        &SOF_ZMAN_SHMA_GRA,
        "2017-10-17T09:55:53-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &SOF_ZMAN_SHMA_MGA,
        "2017-10-17T09:19:53-04:00",
    );

    let mut calc = new_calc(0.0);

    let sof_zman_tfila = ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&mut calc)
    .unwrap();
    assert_time_str(sof_zman_tfila, "2017-10-17T10:42:05-04:00", None);

    assert_zman_str(
        &mut new_calc(0.0),
        &SOF_ZMAN_TFILA_GRA,
        "2017-10-17T10:51:14-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &SOF_ZMAN_TFILA_MGA,
        "2017-10-17T10:27:14-04:00",
    );
}

#[test]
fn test_default_shaah_zmanis() {
    let mut calc = new_calc(0.0);
    let day_start = ZmanPrimitive::SunriseOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .unwrap();
    let day_end = ZmanPrimitive::SunsetOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .unwrap();
    let shaah = (day_end - day_start) / 12;
    assert_duration_ms_close(shaah, 3_594_499);

    let mut calc = new_calc(0.0);
    let shaah_degrees = shaah_zmanis_by_degrees_and_offset(&mut calc, 6.0, 0);
    assert_duration_ms_close(shaah_degrees, 3_594_499);

    let mut calc = new_calc(0.0);
    let shaah_offset = shaah_zmanis_by_degrees_and_offset(&mut calc, 0.0, 72);
    assert_duration_ms_close(shaah_offset, 4_040_608);

    let mut calc = new_calc(0.0);
    let shaah_both = shaah_zmanis_by_degrees_and_offset(&mut calc, 6.0, 72);
    assert_duration_ms_close(shaah_both, 4_314_499);
}

#[test]
fn test_use_elevation_zmanim_times() {
    assert_zman_str(
        &mut new_calc(0.0),
        &TZAIS_DEGREES_8_POINT_5,
        "2017-10-17T18:54:29-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &TZAIS_19_POINT_8_DEGREES,
        "2017-10-17T19:53:34-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &TZAIS_MINUTES_60,
        "2017-10-17T19:14:38-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &TZAIS_90_ZMANIS,
        "2017-10-17T19:37:49-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &TZAIS_MINUTES_72,
        "2017-10-17T19:26:38-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        &ALOS_16_POINT_1_DEGREES,
        "2017-10-17T05:49:30-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &ALOS_19_POINT_8_DEGREES,
        "2017-10-17T05:30:07-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &ALOS_60_MINUTES,
        "2017-10-17T06:09:11-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &ALOS_90_ZMANIS,
        "2017-10-17T05:46:00-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &ALOS_72_MINUTES,
        "2017-10-17T05:57:11-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        &CHATZOS_ASTRONOMICAL,
        "2017-10-17T12:41:55-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &MINCHA_GEDOLA_SUNRISE_SUNSET,
        "2017-10-17T13:09:38-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &MINCHA_KETANA_SUNRISE_SUNSET,
        "2017-10-17T15:56:00-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &PLAG_HAMINCHA_SUNRISE_SUNSET,
        "2017-10-17T17:05:19-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        &CANDLE_LIGHTING,
        "2017-10-17T17:55:58-04:00",
    );
}

#[test]
fn test_use_elevation_zmanim_calculations() {
    let mut calc = new_calc(0.0);

    let sof_zman_shma = ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&mut calc)
    .unwrap();
    assert_time_str(sof_zman_shma, "2017-10-17T09:42:10-04:00", None);

    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_SHMA_GRA,
        "2017-10-17T09:55:33-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_SHMA_MGA,
        "2017-10-17T09:19:33-04:00",
    );

    let mut calc = new_calc(0.0);

    let sof_zman_tfila = ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&mut calc)
    .unwrap();
    assert_time_str(sof_zman_tfila, "2017-10-17T10:42:05-04:00", None);

    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_TFILA_GRA,
        "2017-10-17T10:51:00-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_TFILA_MGA,
        "2017-10-17T10:27:00-04:00",
    );
}

#[test]
fn test_use_elevation_shaah_zmanis() {
    let mut calc = new_calc(0.0);
    let day_start = ZmanPrimitive::SunriseOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .unwrap();
    let day_end = ZmanPrimitive::SunsetOffsetByDegrees(6.0)
        .calculate(&mut calc)
        .unwrap();
    let shaah = (day_end - day_start) / 12;
    assert_duration_ms_close(shaah, 3_594_499);

    let mut calc = new_calc(0.0);
    let shaah_degrees = shaah_zmanis_by_degrees_and_offset(&mut calc, 6.0, 0);
    assert_duration_ms_close(shaah_degrees, 3_594_499);

    let mut calc = new_calc(LAKEWOOD_ELEVATION_M);
    let shaah_offset = shaah_zmanis_by_degrees_and_offset(&mut calc, 0.0, 72);
    assert_duration_ms_close(shaah_offset, 4_047_251);

    let mut calc = new_calc(0.0);
    let shaah_both = shaah_zmanis_by_degrees_and_offset(&mut calc, 6.0, 72);
    assert_duration_ms_close(shaah_both, 4_314_499);
}

fn polar_day_calc() -> ZmanimCalculator<Tz> {
    let date = NaiveDate::from_ymd_opt(2017, 6, 21).unwrap();
    let location = Location::new(69.6492, 18.9553, 0.0, Some(chrono_tz::Europe::Oslo)).unwrap();
    ZmanimCalculator::new(location, date, Default::default()).unwrap()
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
        assert!(zman.calculate(&mut calc).is_err());
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
        assert!(zman.calculate(&mut calc).is_err());
    }

    let mut calc = polar_day_calc();
    assert!(CANDLE_LIGHTING.calculate(&mut calc).is_err());

    let mut calc = polar_day_calc();
    assert!(CHATZOS_HALF_DAY.calculate(&mut calc).is_err());

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
        assert!(zman.calculate(&mut calc).is_err());
    }
}
