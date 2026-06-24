#![allow(clippy::unwrap_used)]

use jiff::{SignedDuration as Duration, Timestamp, civil::Date, tz::TimeZone};
extern crate std;
use std::{string::String, string::ToString};

use crate::zmanim::prelude::presets::*;
use crate::zmanim::prelude::*;

const LAKEWOOD_LAT: f64 = 40.0721087;
const LAKEWOOD_LON: f64 = -74.2400243;
const LAKEWOOD_ELEVATION_M: f64 = 15.0;
const MAX_TIME_DIFF_SECONDS: i64 = 30;
const MAX_SHAAH_ZMANIS_DIFF_MS: i64 = 1000;

fn lakewood_tz() -> TimeZone {
    TimeZone::get("America/New_York").unwrap()
}

fn lakewood_location(elevation_m: f64) -> Location {
    Location::new(LAKEWOOD_LAT, LAKEWOOD_LON, elevation_m, Some(lakewood_tz())).unwrap()
}

fn lakewood_date() -> Date {
    Date::new(2017, 10, 17).unwrap()
}

fn new_calc(elevation_m: f64) -> ZmanimCalculator {
    ZmanimCalculator::new(
        lakewood_location(elevation_m),
        lakewood_date(),
        CalculatorConfig {
            candle_lighting_offset: Duration::from_mins(18),
            use_astronomical_chatzos_for_other_zmanim: false,
            use_elevation: true,
            ateret_torah_sunset_offset: Duration::from_mins(40),
            use_astronomical_chatzos: true,
        },
    )
}

fn calc_for(lat: f64, lon: f64, elevation: f64, tz: TimeZone, date: Date) -> ZmanimCalculator {
    let location = Location::new(lat, lon, elevation, Some(tz)).unwrap();
    ZmanimCalculator::new(location, date, Default::default())
}

fn fmt_local(dt: Timestamp) -> String {
    dt.to_zoned(lakewood_tz()).strftime("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

fn assert_zman_str(calc: &ZmanimCalculator, zman: &dyn ZmanLike, expected: &str) {
    let dt = zman.calculate(calc).unwrap();
    assert_time_str(dt, expected, None);
}
fn assert_zman_str_with_max_time_diff(
    calc: &ZmanimCalculator,
    zman: &dyn ZmanLike,
    expected: &str,
    max_time_diff_seconds: Option<i64>,
) {
    let dt = zman.calculate(calc).unwrap();
    assert_time_str(dt, expected, max_time_diff_seconds);
}

fn assert_time_str(dt: Timestamp, expected: &str, max_time_diff_seconds: Option<i64>) {
    let expected_dt: Timestamp = expected.parse().unwrap();
    let diff = dt.duration_since(expected_dt).as_secs().abs();
    assert!(
        diff <= max_time_diff_seconds.unwrap_or(MAX_TIME_DIFF_SECONDS),
        "time mismatch: expected {}, got {} (diff {}s)",
        expected,
        fmt_local(dt),
        diff
    );
}

fn assert_duration_ms_close(actual: Duration, expected_ms: i64) {
    let actual_ms = actual.as_millis();
    let diff = (actual_ms - i128::from(expected_ms)).abs();
    assert!(
        diff <= i128::from(MAX_SHAAH_ZMANIS_DIFF_MS),
        "duration mismatch: expected {}ms, got {}ms (diff {}ms)",
        expected_ms,
        actual_ms,
        diff
    );
}

fn shaah_zmanis_by_degrees_and_offset(calc: &ZmanimCalculator, degrees: f64, offset_minutes: i64) -> Duration {
    let (start, end) = if degrees.abs() > f64::EPSILON {
        (
            ZmanPrimitive::SunriseOffsetByDegrees(degrees).calculate(calc).unwrap(),
            ZmanPrimitive::SunsetOffsetByDegrees(degrees).calculate(calc).unwrap(),
        )
    } else {
        (
            ZmanPrimitive::ElevationAdjustedSunrise.calculate(calc).unwrap(),
            ZmanPrimitive::ElevationAdjustedSunset.calculate(calc).unwrap(),
        )
    };
    let start = start - Duration::from_mins(offset_minutes);
    let end = end + Duration::from_mins(offset_minutes);
    end.duration_since(start) / 12
}

#[test]
fn test_shaah_zmanis_from_zmanim() {
    let calc = new_calc(0.0);
    let alos = ALOS_16_POINT_1_DEGREES.calculate(&calc).unwrap();
    let tzais = TZAIS_16_POINT_1_DEGREES.calculate(&calc).unwrap();
    let shaah = tzais.duration_since(alos) / 12;
    assert!(shaah.as_secs() > 0);
}

#[test]
fn test_local_mean_time_invalid_hours() {
    let calc = new_calc(0.0);

    assert!(ZmanPrimitive::LocalMeanTime(-1.0).calculate(&calc).is_err());
}

#[test]
fn test_half_day_based_zman_negative_hours() {
    let calc = new_calc(0.0);
    let sunrise = ZmanPrimitive::ElevationAdjustedSunrise.calculate(&calc).unwrap();
    let sunset = ZmanPrimitive::ElevationAdjustedSunset.calculate(&calc).unwrap();
    let shaah = sunset.duration_since(sunrise) / 6;
    let expected = sunset + shaah.mul_f64(-1.0);
    let actual = ZmanPrimitive::HalfDayBasedOffset(
        &ZmanPrimitive::ElevationAdjustedSunrise,
        &ZmanPrimitive::ElevationAdjustedSunset,
        -1.0,
    )
    .calculate(&calc)
    .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_high_latitude_sunrise_sunset_ordering() {
    let date = Date::new(2017, 3, 21).unwrap();
    let calc = calc_for(
        64.1466,
        -21.9426,
        0.0,
        TimeZone::get("Atlantic/Reykjavik").unwrap(),
        date,
    );
    #[allow(clippy::expect_used)]
    let sunrise = ZmanPrimitive::ElevationAdjustedSunrise.calculate(&calc).unwrap();
    #[allow(clippy::expect_used)]
    let sunset = ZmanPrimitive::ElevationAdjustedSunset.calculate(&calc).unwrap();
    assert!(sunrise < sunset);

    let dawn = ZmanPrimitive::SunriseOffsetByDegrees(6.0).calculate(&calc).unwrap();
    let dusk = ZmanPrimitive::SunsetOffsetByDegrees(6.0).calculate(&calc).unwrap();
    assert!(dawn < sunrise);
    assert!(dusk > sunset);
}

#[test]
fn test_extreme_elevation_shifts_sunrise_sunset() {
    let date = Date::new(2017, 10, 17).unwrap();
    let high = calc_for(27.9881, 86.9250, 8848.0, TimeZone::get("Asia/Kathmandu").unwrap(), date);
    let sea = calc_for(27.9881, 86.9250, 0.0, TimeZone::get("Asia/Kathmandu").unwrap(), date);

    let sunrise_high = ZmanPrimitive::ElevationAdjustedSunrise.calculate(&high).unwrap();
    let sunrise_sea = ZmanPrimitive::ElevationAdjustedSunrise.calculate(&sea).unwrap();
    assert!(sunrise_high < sunrise_sea);

    let sunset_high = ZmanPrimitive::ElevationAdjustedSunset.calculate(&high).unwrap();
    let sunset_sea = ZmanPrimitive::ElevationAdjustedSunset.calculate(&sea).unwrap();
    assert!(sunset_high > sunset_sea);
}

#[test]
fn test_polar_ben_ish_chai_unavailable_on_normal_days() {
    let calc = new_calc(0.0);
    assert!(ZmanPrimitive::PolarSunriseBenIshChai.calculate(&calc).is_err());
    assert!(ZmanPrimitive::PolarSunsetBenIshChai.calculate(&calc).is_err());
    assert!(POLAR_SUNRISE_BEN_ISH_CHAI.calculate(&calc).is_err());
    assert!(POLAR_SUNSET_BEN_ISH_CHAI.calculate(&calc).is_err());
}

#[test]
fn test_polar_ben_ish_chai_azimuth_fallback_on_polar_day() {
    let date = Date::new(2017, 6, 21).unwrap();
    let calc = calc_for(69.6492, 18.9553, 0.0, TimeZone::get("Europe/Oslo").unwrap(), date);

    assert!(ZmanPrimitive::ElevationAdjustedSunrise.calculate(&calc).is_err());
    assert!(ZmanPrimitive::PolarSunriseBenIshChai.calculate(&calc).is_ok());
    assert!(ZmanPrimitive::PolarSunsetBenIshChai.calculate(&calc).is_ok());
}

#[test]
fn test_polar_day_returns_none_for_sun_times() {
    let date = Date::new(2017, 6, 21).unwrap();
    let calc = calc_for(69.6492, 18.9553, 0.0, TimeZone::get("Europe/Oslo").unwrap(), date);

    assert!(ZmanPrimitive::ElevationAdjustedSunrise.calculate(&calc).is_err());
    assert!(ZmanPrimitive::ElevationAdjustedSunset.calculate(&calc).is_err());
    assert!(ZmanPrimitive::SeaLevelSunrise.calculate(&calc).is_err());
    assert!(ZmanPrimitive::SeaLevelSunset.calculate(&calc).is_err());
    assert!(ZmanPrimitive::SunriseOffsetByDegrees(6.0).calculate(&calc).is_err());
    assert!(ZmanPrimitive::SunsetOffsetByDegrees(6.0).calculate(&calc).is_err());
}

#[test]
fn test_reykjavik_equinox_java_expected_times() {
    let date = Date::new(2017, 3, 21).unwrap();
    let calc = calc_for(
        64.1466,
        -21.9426,
        0.0,
        TimeZone::get("Atlantic/Reykjavik").unwrap(),
        date,
    );

    assert_zman_str(&calc, &ELEVATION_ADJUSTED_SUNRISE, "2017-03-21T07:24:24Z");
    assert_zman_str(&calc, &ELEVATION_ADJUSTED_SUNSET, "2017-03-21T19:46:56Z");
    assert_zman_str(&calc, &SEA_LEVEL_SUNRISE, "2017-03-21T07:24:24Z");
    assert_zman_str(&calc, &SEA_LEVEL_SUNSET, "2017-03-21T19:46:56Z");
    assert_zman_str(&calc, &CHATZOS_HAYOM, "2017-03-21T13:34:59Z");
}

#[test]
fn test_everest_java_expected_times() {
    let date = Date::new(2017, 10, 17).unwrap();
    let calc = calc_for(27.9881, 86.9250, 8826.0, TimeZone::get("Asia/Kathmandu").unwrap(), date);
    assert_zman_str_with_max_time_diff(&calc, &ELEVATION_ADJUSTED_SUNRISE, "2017-10-17T05:44:51+05:45", Some(1));
    assert_zman_str_with_max_time_diff(&calc, &ELEVATION_ADJUSTED_SUNSET, "2017-10-17T17:40:04+05:45", Some(1));
    assert_zman_str_with_max_time_diff(&calc, &SEA_LEVEL_SUNRISE, "2017-10-17T05:58:42+05:45", Some(1));
    assert_zman_str_with_max_time_diff(&calc, &SEA_LEVEL_SUNSET, "2017-10-17T17:26:12+05:45", Some(1));
    assert_zman_str_with_max_time_diff(&calc, &CHATZOS_HAYOM, "2017-10-17T11:42:38+05:45", Some(1));
}

#[test]
fn test_lakewood_noaa_baseline_events() {
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &ELEVATION_ADJUSTED_SUNRISE,
        "2017-10-17T07:09:11-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &ELEVATION_ADJUSTED_SUNSET,
        "2017-10-17T18:14:38-04:00",
    );
    assert_zman_str(&new_calc(0.0), &CHATZOS_HAYOM, "2017-10-17T12:41:55-04:00");
    assert_zman_str(&new_calc(0.0), &ALOS_16_POINT_1_DEGREES, "2017-10-17T05:49:30-04:00");
    assert_zman_str(&new_calc(0.0), &TZAIS_16_POINT_1_DEGREES, "2017-10-17T19:34:14-04:00");
}

#[test]
fn test_polar_night_returns_none_for_sun_times() {
    let date = Date::new(2017, 12, 21).unwrap();
    let calc = calc_for(69.6492, 18.9553, 0.0, TimeZone::get("Europe/Oslo").unwrap(), date);

    assert!(matches!(
        ZmanPrimitive::ElevationAdjustedSunrise.calculate(&calc),
        Err(ZmanimError::AllNight)
    ));
    assert!(matches!(
        ZmanPrimitive::ElevationAdjustedSunset.calculate(&calc),
        Err(ZmanimError::AllNight)
    ));
}

#[test]
fn test_anti_meridian_timezone_date_adjustment() {
    let date = Date::new(2018, 2, 3).unwrap();
    let timezone = TimeZone::get("Pacific/Apia").unwrap();
    let calc = calc_for(-13.8599, -171.7513, 0.0, timezone.clone(), date);

    let sunrise = ELEVATION_ADJUSTED_SUNRISE.calculate(&calc).unwrap();
    let sunset = ELEVATION_ADJUSTED_SUNSET.calculate(&calc).unwrap();
    let sunrise_local = sunrise.to_zoned(timezone.clone());
    let sunset_local = sunset.to_zoned(timezone);

    assert_eq!(sunrise_local.date(), date);
    assert_eq!(sunset_local.date(), date);
    assert!(sunrise_local.hour() < 12);
    assert!(sunset_local.hour() >= 12);
}

#[test]
fn test_solar_midnight_rolls_to_next_local_date() {
    let date = lakewood_date();
    let calc = new_calc(0.0);
    let midnight = CHATZOS_HALAYLA.calculate(&calc).unwrap();
    let local = midnight.to_zoned(lakewood_tz());

    assert_eq!(local.date(), Date::new(2017, 10, 18).unwrap());
    assert!(local.hour() < 2);
    assert!(local.date() > date);
}

#[test]
fn test_default_zmanim_times() {
    let calc = new_calc(0.0);

    assert_eq!(
        SEA_LEVEL_SUNRISE.calculate(&calc),
        ZmanPrimitive::SeaLevelSunrise.calculate(&calc)
    );
    assert_eq!(
        SEA_LEVEL_SUNSET.calculate(&calc),
        ZmanPrimitive::SeaLevelSunset.calculate(&calc)
    );

    assert_zman_str(
        &new_calc(0.0),
        &TZAIS_GEONIM_8_POINT_5_DEGREES,
        "2017-10-17T18:54:29-04:00",
    );
    assert_zman_str(&new_calc(0.0), &TZAIS_19_POINT_8_DEGREES, "2017-10-17T19:53:34-04:00");
    assert_zman_str(&new_calc(0.0), &TZAIS_60_MINUTES, "2017-10-17T19:13:58-04:00");
    assert_zman_str(&new_calc(0.0), &TZAIS_90_ZMANIS, "2017-10-17T19:36:59-04:00");
    assert_zman_str(&new_calc(0.0), &TZAIS_72_MINUTES, "2017-10-17T19:25:58-04:00");

    assert_zman_str(&new_calc(0.0), &ALOS_16_POINT_1_DEGREES, "2017-10-17T05:49:30-04:00");
    assert_zman_str(&new_calc(0.0), &ALOS_19_POINT_8_DEGREES, "2017-10-17T05:30:07-04:00");
    assert_zman_str(&new_calc(0.0), &ALOS_60_MINUTES, "2017-10-17T06:09:51-04:00");
    assert_zman_str(&new_calc(0.0), &ALOS_90_ZMANIS, "2017-10-17T05:46:50-04:00");
    assert_zman_str(&new_calc(0.0), &ALOS_72_MINUTES, "2017-10-17T05:57:51-04:00");

    assert_zman_str(&new_calc(0.0), &CHATZOS_HAYOM, "2017-10-17T12:41:55-04:00");
    assert_zman_str(&new_calc(0.0), &MINCHA_GEDOLA_GRA, "2017-10-17T13:09:35-04:00");
    assert_zman_str(&new_calc(0.0), &MINCHA_KETANA_GRA, "2017-10-17T15:55:37-04:00");
    assert_zman_str(&new_calc(0.0), &PLAG_HAMINCHA_GRA, "2017-10-17T17:04:48-04:00");
    assert_zman_str(&new_calc(0.0), &CANDLE_LIGHTING, "2017-10-17T17:55:58-04:00");
}

#[test]
fn test_default_zmanim_calculations() {
    let calc = new_calc(0.0);

    let sof_zman_shma = ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&calc)
    .unwrap();
    assert_time_str(sof_zman_shma, "2017-10-17T09:42:10-04:00", None);

    assert_zman_str(&new_calc(0.0), &SOF_ZMAN_SHMA_GRA, "2017-10-17T09:55:53-04:00");
    assert_zman_str(
        &new_calc(0.0),
        &SOF_ZMAN_SHMA_MGA_72_MINUTES,
        "2017-10-17T09:19:53-04:00",
    );

    let calc = new_calc(0.0);

    let sof_zman_tfila = ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&calc)
    .unwrap();
    assert_time_str(sof_zman_tfila, "2017-10-17T10:42:05-04:00", None);

    assert_zman_str(&new_calc(0.0), &SOF_ZMAN_TFILA_GRA, "2017-10-17T10:51:14-04:00");
    assert_zman_str(
        &new_calc(0.0),
        &SOF_ZMAN_TFILA_MGA_72_MINUTES,
        "2017-10-17T10:27:14-04:00",
    );
}

#[test]
fn test_default_shaah_zmanis() {
    let calc = new_calc(0.0);
    let day_start = ZmanPrimitive::SunriseOffsetByDegrees(6.0).calculate(&calc).unwrap();
    let day_end = ZmanPrimitive::SunsetOffsetByDegrees(6.0).calculate(&calc).unwrap();
    let shaah = day_end.duration_since(day_start) / 12;
    assert_duration_ms_close(shaah, 3_594_499);

    let calc = new_calc(0.0);
    let shaah_degrees = shaah_zmanis_by_degrees_and_offset(&calc, 6.0, 0);
    assert_duration_ms_close(shaah_degrees, 3_594_499);

    let calc = new_calc(0.0);
    let shaah_offset = shaah_zmanis_by_degrees_and_offset(&calc, 0.0, 72);
    assert_duration_ms_close(shaah_offset, 4_040_608);

    let calc = new_calc(0.0);
    let shaah_both = shaah_zmanis_by_degrees_and_offset(&calc, 6.0, 72);
    assert_duration_ms_close(shaah_both, 4_314_499);
}

#[test]
fn test_use_elevation_zmanim_times() {
    assert_zman_str(
        &new_calc(0.0),
        &TZAIS_GEONIM_8_POINT_5_DEGREES,
        "2017-10-17T18:54:29-04:00",
    );
    assert_zman_str(&new_calc(0.0), &TZAIS_19_POINT_8_DEGREES, "2017-10-17T19:53:34-04:00");
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &TZAIS_60_MINUTES,
        "2017-10-17T19:14:38-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &TZAIS_90_ZMANIS,
        "2017-10-17T19:37:49-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &TZAIS_72_MINUTES,
        "2017-10-17T19:26:38-04:00",
    );

    assert_zman_str(&new_calc(0.0), &ALOS_16_POINT_1_DEGREES, "2017-10-17T05:49:30-04:00");
    assert_zman_str(&new_calc(0.0), &ALOS_19_POINT_8_DEGREES, "2017-10-17T05:30:07-04:00");
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &ALOS_60_MINUTES,
        "2017-10-17T06:09:11-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &ALOS_90_ZMANIS,
        "2017-10-17T05:46:00-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &ALOS_72_MINUTES,
        "2017-10-17T05:57:11-04:00",
    );

    assert_zman_str(&new_calc(0.0), &CHATZOS_HAYOM, "2017-10-17T12:41:55-04:00");
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &MINCHA_GEDOLA_GRA,
        "2017-10-17T13:09:38-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &MINCHA_KETANA_GRA,
        "2017-10-17T15:56:00-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &PLAG_HAMINCHA_GRA,
        "2017-10-17T17:05:19-04:00",
    );
    assert_zman_str(&new_calc(0.0), &CANDLE_LIGHTING, "2017-10-17T17:55:58-04:00");
}

#[test]
fn test_use_elevation_zmanim_calculations() {
    let calc = new_calc(0.0);

    let sof_zman_shma = ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&calc)
    .unwrap();
    assert_time_str(sof_zman_shma, "2017-10-17T09:42:10-04:00", None);

    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_SHMA_GRA,
        "2017-10-17T09:55:33-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_SHMA_MGA_72_MINUTES,
        "2017-10-17T09:19:33-04:00",
    );

    let calc = new_calc(0.0);

    let sof_zman_tfila = ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(6.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(6.0),
        true,
    )
    .calculate(&calc)
    .unwrap();
    assert_time_str(sof_zman_tfila, "2017-10-17T10:42:05-04:00", None);

    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_TFILA_GRA,
        "2017-10-17T10:51:00-04:00",
    );
    assert_zman_str(
        &new_calc(LAKEWOOD_ELEVATION_M),
        &SOF_ZMAN_TFILA_MGA_72_MINUTES,
        "2017-10-17T10:27:00-04:00",
    );
}

#[test]
fn test_use_elevation_shaah_zmanis() {
    let calc = new_calc(0.0);
    let day_start = ZmanPrimitive::SunriseOffsetByDegrees(6.0).calculate(&calc).unwrap();
    let day_end = ZmanPrimitive::SunsetOffsetByDegrees(6.0).calculate(&calc).unwrap();
    let shaah = day_end.duration_since(day_start) / 12;
    assert_duration_ms_close(shaah, 3_594_499);

    let calc = new_calc(0.0);
    let shaah_degrees = shaah_zmanis_by_degrees_and_offset(&calc, 6.0, 0);
    assert_duration_ms_close(shaah_degrees, 3_594_499);

    let calc = new_calc(LAKEWOOD_ELEVATION_M);
    let shaah_offset = shaah_zmanis_by_degrees_and_offset(&calc, 0.0, 72);
    assert_duration_ms_close(shaah_offset, 4_047_251);

    let calc = new_calc(0.0);
    let shaah_both = shaah_zmanis_by_degrees_and_offset(&calc, 6.0, 72);
    assert_duration_ms_close(shaah_both, 4_314_499);
}

fn polar_day_calc() -> ZmanimCalculator {
    let date = Date::new(2017, 6, 21).unwrap();
    let location = Location::new(69.6492, 18.9553, 0.0, Some(TimeZone::get("Europe/Oslo").unwrap())).unwrap();
    ZmanimCalculator::new(location, date, Default::default())
}

#[test]
fn test_polar_day_zmanim_return_none() {
    let alos_variants: [&dyn ZmanLike; 9] = [
        &ALOS_60_MINUTES,
        &ALOS_72_MINUTES,
        &ALOS_72_ZMANIS,
        &ALOS_90_MINUTES,
        &ALOS_90_ZMANIS,
        &ALOS_96_MINUTES,
        &ALOS_96_ZMANIS,
        #[allow(deprecated)]
        &ALOS_120_MINUTES,
        #[allow(deprecated)]
        &ALOS_120_ZMANIS,
    ];
    for zman in alos_variants {
        let calc = polar_day_calc();
        assert!(zman.calculate(&calc).is_err());
    }

    let bain_variants: [&dyn ZmanLike; 6] = [
        &BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES,
        &BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES,
        &BAIN_HASHMASHOS_RT_2_STARS,
        &BAIN_HASHMASHOS_YEREIM_18_MINUTES,
        &BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES,
        &BAIN_HASHMASHOS_YEREIM_13_POINT_5_MINUTES,
    ];
    for zman in bain_variants {
        let calc = polar_day_calc();
        assert!(zman.calculate(&calc).is_err());
    }

    let calc = polar_day_calc();
    assert!(CANDLE_LIGHTING.calculate(&calc).is_err());

    let calc = polar_day_calc();
    assert!(CHATZOS_HAYOM_AS_HALF_DAY.calculate(&calc).is_err());

    let mincha_variants: [&dyn ZmanLike; 5] = [
        &MINCHA_GEDOLA_16_POINT_1_DEGREES,
        &MINCHA_GEDOLA_72_MINUTES,
        &MINCHA_GEDOLA_AHAVAT_SHALOM,
        &MINCHA_GEDOLA_ATERET_TORAH,
        &MINCHA_GEDOLA_BAAL_HATANYA,
    ];
    for zman in mincha_variants {
        let calc = polar_day_calc();
        assert!(zman.calculate(&calc).is_err());
    }
}

#[test]
fn test_mcmurdo_antimeridian_alos_matches_java() {
    let tz = TimeZone::get("Antarctica/McMurdo").unwrap();
    let date = Date::new(1901, 4, 18).unwrap();
    let location = Location::new(
        -77.10300593682803,
        -166.39303270398793,
        61.48216934280892,
        Some(tz.clone()),
    )
    .unwrap();
    let config = CalculatorConfig {
        candle_lighting_offset: Duration::from_mins(12),
        use_astronomical_chatzos_for_other_zmanim: true,
        use_elevation: false,
        ateret_torah_sunset_offset: Duration::from_mins(4),
        use_astronomical_chatzos: false,
    };
    let calc = ZmanimCalculator::new(location, date, config);

    let alos = ALOS_120_MINUTES.calculate(&calc).unwrap();
    let local = alos.to_zoned(tz);

    assert_eq!(local.date(), date);
    assert!(local.hour() < 12, "alos should be in the morning, got {local}");
    assert_time_str(alos, "1901-04-18T05:45:48.837784872+11:30", Some(1));
}

#[cfg(feature = "alloc")]
#[test]
#[allow(clippy::field_reassign_with_default)]
fn test_preset_description_uses_chatzos_hayom_config_flag() {
    let calc = new_calc(0.0);
    let desc = CHATZOS_HAYOM.description(&calc);
    assert!(desc.contains("astronomical chatzos (solar transit)"));

    let mut config = CalculatorConfig::default();
    config.use_astronomical_chatzos = false;
    let calc = ZmanimCalculator::new(lakewood_location(0.0), lakewood_date(), config);
    let desc = CHATZOS_HAYOM.description(&calc);
    assert!(desc.contains("midpoint between sunrise and sunset"));
}

#[cfg(feature = "alloc")]
#[test]
#[allow(clippy::field_reassign_with_default)]
fn test_preset_description_uses_chatzos_for_other_zmanim_config_flag() {
    let mut config = CalculatorConfig::default();
    config.use_astronomical_chatzos_for_other_zmanim = true;
    let calc = ZmanimCalculator::new(lakewood_location(0.0), lakewood_date(), config);
    let desc = SOF_ZMAN_SHMA_GRA.description(&calc);
    assert!(desc.contains("divides the afternoon from astronomical chatzos"));
    assert!(!desc.contains("midpoint between sunrise and sunset"));

    config.use_astronomical_chatzos_for_other_zmanim = false;
    let calc = ZmanimCalculator::new(lakewood_location(0.0), lakewood_date(), config);
    let desc = SOF_ZMAN_SHMA_GRA.description(&calc);
    assert!(desc.contains("fixed fraction of the sunrise-to-sunset interval"));
    assert!(!desc.contains("divides the afternoon from astronomical chatzos"));
}
