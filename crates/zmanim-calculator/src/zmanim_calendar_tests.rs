#![allow(clippy::unwrap_used)]

use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use std::str::FromStr;

use crate::calculator::ZmanimCalculator;
use crate::types::zman::ZmanLike;
use crate::{
    AlosZman, CandleLightingZman, ChatzosZman, Location, MinchaGedolaZman, MinchaKetanaZman,
    PlagHaminchaZman, SeaLevelNeitzZman, SeaLevelShkiaZman, SofZmanShmaZman, SofZmanTfilaZman,
    TzaisZman,
};

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

fn fmt_local(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&lakewood_tz())
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

fn assert_zman_str<Z: ZmanLike>(calc: &mut ZmanimCalculator<Tz>, zman: Z, expected: &str) {
    let dt = calc.calculate(zman).unwrap();
    assert_time_str(dt, expected);
}

fn assert_time_str(dt: DateTime<Utc>, expected: &str) {
    let expected_dt = DateTime::parse_from_rfc3339(expected)
        .unwrap()
        .with_timezone(&Utc);
    let diff = (dt - expected_dt).num_seconds().abs();
    assert!(
        diff <= MAX_TIME_DIFF_SECONDS,
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
            calc.sunrise_offset_by_degrees(degrees).unwrap(),
            calc.sunset_offset_by_degrees(degrees).unwrap(),
        )
    } else {
        (calc.sunrise().unwrap(), calc.sunset().unwrap())
    };
    let start = start - Duration::minutes(offset_minutes);
    let end = end + Duration::minutes(offset_minutes);
    calc.get_temporal_hour_from_times(&start, &end).unwrap()
}

#[test]
fn test_default_zmanim_times() {
    let mut calc = new_calc(0.0);

    assert_eq!(calc.calculate(SeaLevelNeitzZman), calc.sea_level_sunrise());
    assert_eq!(calc.calculate(SeaLevelShkiaZman), calc.sea_level_sunset());

    assert_zman_str(
        &mut new_calc(0.0),
        TzaisZman::Degrees8Point5,
        "2017-10-17T18:54:29-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        TzaisZman::Degrees19Point8,
        "2017-10-17T19:53:34-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        TzaisZman::Minutes60,
        "2017-10-17T19:13:58-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        TzaisZman::Minutes90Zmanis,
        "2017-10-17T19:36:59-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        TzaisZman::Minutes72,
        "2017-10-17T19:25:58-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        AlosZman::Degrees16Point1,
        "2017-10-17T05:49:30-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        AlosZman::Degrees19Point8,
        "2017-10-17T05:30:07-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        AlosZman::Minutes60,
        "2017-10-17T06:09:51-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        AlosZman::Minutes90Zmanis,
        "2017-10-17T05:46:50-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        AlosZman::Minutes72,
        "2017-10-17T05:57:51-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        ChatzosZman::Astronomical,
        "2017-10-17T12:41:55-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        MinchaGedolaZman::SunriseSunset,
        "2017-10-17T13:09:35-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        MinchaKetanaZman::SunriseSunset,
        "2017-10-17T15:55:37-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        PlagHaminchaZman::SunriseSunset,
        "2017-10-17T17:04:48-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        CandleLightingZman,
        "2017-10-17T17:55:58-04:00",
    );
}

#[test]
fn test_default_zmanim_calculations() {
    let mut calc = new_calc(0.0);

    let day_start = calc.sunrise_offset_by_degrees(6.0).unwrap();
    let day_end = calc.sunset_offset_by_degrees(6.0).unwrap();
    let sof_zman_shma = calc
        .get_sof_zman_shma_from_times(&day_start, Some(&day_end), true)
        .unwrap();
    assert_time_str(sof_zman_shma, "2017-10-17T09:42:10-04:00");

    assert_zman_str(
        &mut new_calc(0.0),
        SofZmanShmaZman::GRA,
        "2017-10-17T09:55:53-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        SofZmanShmaZman::MGA,
        "2017-10-17T09:19:53-04:00",
    );

    let mut calc = new_calc(0.0);
    let day_start = calc.sunrise_offset_by_degrees(6.0).unwrap();
    let day_end = calc.sunset_offset_by_degrees(6.0).unwrap();
    let sof_zman_tfila = calc
        .get_sof_zman_tfila_from_times(&day_start, Some(&day_end), true)
        .unwrap();
    assert_time_str(sof_zman_tfila, "2017-10-17T10:42:05-04:00");

    assert_zman_str(
        &mut new_calc(0.0),
        SofZmanTfilaZman::GRA,
        "2017-10-17T10:51:14-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        SofZmanTfilaZman::MGA,
        "2017-10-17T10:27:14-04:00",
    );
}

#[test]
fn test_default_shaah_zmanis() {
    let mut calc = new_calc(0.0);
    let day_start = calc.sunrise_offset_by_degrees(6.0).unwrap();
    let day_end = calc.sunset_offset_by_degrees(6.0).unwrap();
    let shaah = calc
        .get_temporal_hour_from_times(&day_start, &day_end)
        .unwrap();
    assert_duration_ms_close(shaah, 3_594_499);

    let mut calc = new_calc(0.0);
    let shaah_gra = calc.get_shaah_zmanis_gra().unwrap();
    assert_duration_ms_close(shaah_gra, 3_320_608);

    let mut calc = new_calc(0.0);
    let shaah_mga = calc.get_shaah_zmanis_mga().unwrap();
    assert_duration_ms_close(shaah_mga, 4_040_608);

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
    let mut calc = new_calc(LAKEWOOD_ELEVATION_M);

    assert_eq!(calc.calculate(crate::NeitzZman), calc.sunrise());
    assert_eq!(calc.calculate(crate::ShkiaZman), calc.sunset());

    assert_zman_str(
        &mut new_calc(0.0),
        TzaisZman::Degrees8Point5,
        "2017-10-17T18:54:29-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        TzaisZman::Degrees19Point8,
        "2017-10-17T19:53:34-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        TzaisZman::Minutes60,
        "2017-10-17T19:14:38-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        TzaisZman::Minutes90Zmanis,
        "2017-10-17T19:37:49-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        TzaisZman::Minutes72,
        "2017-10-17T19:26:38-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        AlosZman::Degrees16Point1,
        "2017-10-17T05:49:30-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        AlosZman::Degrees19Point8,
        "2017-10-17T05:30:07-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        AlosZman::Minutes60,
        "2017-10-17T06:09:11-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        AlosZman::Minutes90Zmanis,
        "2017-10-17T05:46:00-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        AlosZman::Minutes72,
        "2017-10-17T05:57:11-04:00",
    );

    assert_zman_str(
        &mut new_calc(0.0),
        ChatzosZman::Astronomical,
        "2017-10-17T12:41:55-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        MinchaGedolaZman::SunriseSunset,
        "2017-10-17T13:09:38-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        MinchaKetanaZman::SunriseSunset,
        "2017-10-17T15:56:00-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        PlagHaminchaZman::SunriseSunset,
        "2017-10-17T17:05:19-04:00",
    );
    assert_zman_str(
        &mut new_calc(0.0),
        CandleLightingZman,
        "2017-10-17T17:55:58-04:00",
    );
}

#[test]
fn test_use_elevation_zmanim_calculations() {
    let mut calc = new_calc(0.0);
    let day_start = calc.sunrise_offset_by_degrees(6.0).unwrap();
    let day_end = calc.sunset_offset_by_degrees(6.0).unwrap();
    let sof_zman_shma = calc
        .get_sof_zman_shma_from_times(&day_start, Some(&day_end), true)
        .unwrap();
    assert_time_str(sof_zman_shma, "2017-10-17T09:42:10-04:00");

    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        SofZmanShmaZman::GRA,
        "2017-10-17T09:55:33-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        SofZmanShmaZman::MGA,
        "2017-10-17T09:19:33-04:00",
    );

    let mut calc = new_calc(0.0);
    let day_start = calc.sunrise_offset_by_degrees(6.0).unwrap();
    let day_end = calc.sunset_offset_by_degrees(6.0).unwrap();
    let sof_zman_tfila = calc
        .get_sof_zman_tfila_from_times(&day_start, Some(&day_end), true)
        .unwrap();
    assert_time_str(sof_zman_tfila, "2017-10-17T10:42:05-04:00");

    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        SofZmanTfilaZman::GRA,
        "2017-10-17T10:51:00-04:00",
    );
    assert_zman_str(
        &mut new_calc(LAKEWOOD_ELEVATION_M),
        SofZmanTfilaZman::MGA,
        "2017-10-17T10:27:00-04:00",
    );
}

#[test]
fn test_use_elevation_shaah_zmanis() {
    let mut calc = new_calc(0.0);
    let day_start = calc.sunrise_offset_by_degrees(6.0).unwrap();
    let day_end = calc.sunset_offset_by_degrees(6.0).unwrap();
    let shaah = calc
        .get_temporal_hour_from_times(&day_start, &day_end)
        .unwrap();
    assert_duration_ms_close(shaah, 3_594_499);

    let mut calc = new_calc(LAKEWOOD_ELEVATION_M);
    let shaah_gra = calc.get_shaah_zmanis_gra().unwrap();
    assert_duration_ms_close(shaah_gra, 3_327_251);

    let mut calc = new_calc(LAKEWOOD_ELEVATION_M);
    let shaah_mga = calc.get_shaah_zmanis_mga().unwrap();
    assert_duration_ms_close(shaah_mga, 4_047_251);

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
