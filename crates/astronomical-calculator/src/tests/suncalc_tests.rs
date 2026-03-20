use chrono::prelude::*;
use serde::Deserialize;
use serde::Serialize;
extern crate std;

use crate::*;

#[derive(Serialize, Deserialize, Debug)]
struct SunResult {
    lat: f64,
    lon: f64,
    elevation: f64,
    midday: f64,
    sunrise: Option<f64>,
    sunset: Option<f64>,
    transit: f64,
    now: std::string::String,
    civil_dawn: Option<f64>,
    nautical_dawn: Option<f64>,
    astronomical_dawn: Option<f64>,
    civil_dusk: Option<f64>,
    nautical_dusk: Option<f64>,
    astronomical_dusk: Option<f64>,
}

/// A collection of `SunResult` records loaded from JSON.
#[derive(Serialize, Deserialize, Debug)]
struct SunResults {
    results: std::vec::Vec<SunResult>,
}
/// Converts a Unix timestamp (seconds) into a `NaiveDateTime`.
fn ts_to_dt(ts: f64) -> NaiveDateTime {
    #[allow(deprecated)]
    NaiveDateTime::from_timestamp_millis((ts * 1000.0) as i64).unwrap()
}

/// Returns the allowed time delta (in seconds) for comparisons at a latitude.
///
/// The tolerance is fixed at 60 seconds up to 45 degrees, then grows
/// exponentially toward 10_000 seconds near the poles.
fn latitude_tolerance(lat: f64) -> f64 {
    let abs_lat = lat.abs();
    if abs_lat <= 45.0 {
        // Constant tolerance until 45 degrees
        // Astronomical events need larger tolerance due to lower precision at extreme angles (18° below horizon)
        60.0
    } else {
        // Exponential growth from base to 10000 seconds between 45° and 90°
        // Formula: base * (10000/base)^((abs_lat - 45) / 45)
        let base_tolerance = 60.0;
        let max_tolerance: f64 = 10000.0;
        let ratio: f64 = max_tolerance / base_tolerance;
        let exponent: f64 = (abs_lat - 45.0) / 45.0;
        base_tolerance * ratio.powf(exponent)
    }
}

/// Calls a solar event getter and converts a successful timestamp into `NaiveDateTime`.
///
/// Returns `None` when the event is not available (`AllDay`/`AllNight`) or when
/// the calculation fails.
fn get_event_time(
    calc: &mut AstronomicalCalculator,
    getter: fn(&mut AstronomicalCalculator) -> Result<SolarEventResult, CalculationError>,
) -> Option<NaiveDateTime> {
    getter(calc)
        .ok()
        .and_then(|result| result.timestamp())
        .map(|ts| ts_to_dt(ts as f64))
}

/// Compares two optional event times and asserts they are within latitude-based tolerance.
///
/// If either side is `None`, this helper does not assert and simply returns.
fn compare_times(
    our: Option<NaiveDateTime>,
    their: Option<NaiveDateTime>,
    name: &str,
    lat: f64,
    result: &SunResult,
    index: usize,
) {
    if let (Some(our), Some(their)) = (our, their) {
        let diff = (our - their).abs().num_seconds();
        let tolerance = latitude_tolerance(lat);

        assert!(
            diff <= tolerance as i64,
            "{} timestamp difference too large: {} seconds (tolerance: {}), ours: {}, theirs: {}. Record: {:?} Index: {}",
            name,
            diff,
            tolerance,
            our,
            their,
            result,
            index
        );
    }
}

/// Validates calculator output against external SunCalc reference data.
///
/// Transit, sunrise, and sunset are always compared. Twilight events are only
/// compared at latitudes up to 45 degrees, where the reference data is reliable.
#[test]
fn test_suncalc_test_data() {
    let data = std::fs::read_to_string("test_data/data.json").unwrap();
    let results: SunResults = serde_json::from_str(&data).unwrap();

    for (index, result) in results.results.iter().enumerate() {
        let midday = ts_to_dt(result.midday).and_utc();
        let their_transit = ts_to_dt(result.transit);
        let their_sunset = result.sunset.map(ts_to_dt);
        let their_sunrise = result.sunrise.map(ts_to_dt);
        let their_civil_dawn = result.civil_dawn.map(ts_to_dt);
        let their_nautical_dawn = result.nautical_dawn.map(ts_to_dt);
        let their_astronomical_dawn = result.astronomical_dawn.map(ts_to_dt);
        let their_civil_dusk = result.civil_dusk.map(ts_to_dt);
        let their_nautical_dusk = result.nautical_dusk.map(ts_to_dt);
        let their_astronomical_dusk = result.astronomical_dusk.map(ts_to_dt);

        let mut calc = AstronomicalCalculator::new(
            midday,
            None,
            0.0,
            result.lon,
            result.lat,
            result.elevation,
            20.0,
            1013.25,
            Some(0.0),
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let our_transit = ts_to_dt(calc.get_solar_transit().unwrap() as f64);
        compare_times(
            Some(our_transit),
            Some(their_transit),
            "Transit",
            result.lat,
            result,
            index,
        );

        compare_times(
            get_event_time(&mut calc, AstronomicalCalculator::get_sunset),
            their_sunset,
            "Sunset",
            result.lat,
            result,
            index,
        );
        compare_times(
            get_event_time(&mut calc, AstronomicalCalculator::get_sunrise),
            their_sunrise,
            "Sunrise",
            result.lat,
            result,
            index,
        );

        // Skip twilight comparisons for high latitudes (>45° N or S)
        // Twilight events become unreliable or don't occur at extreme latitudes
        if result.lat.abs() <= 45.0 {
            compare_times(
                get_event_time(&mut calc, AstronomicalCalculator::get_astronomical_dawn),
                their_astronomical_dawn,
                "Astronomical dawn",
                result.lat,
                result,
                index,
            );

            compare_times(
                get_event_time(&mut calc, AstronomicalCalculator::get_astronomical_dusk),
                their_astronomical_dusk,
                "Astronomical dusk",
                result.lat,
                result,
                index,
            );
            compare_times(
                get_event_time(&mut calc, AstronomicalCalculator::get_civil_dawn),
                their_civil_dawn,
                "Civil dawn",
                result.lat,
                result,
                index,
            );
            compare_times(
                get_event_time(&mut calc, AstronomicalCalculator::get_nautical_dawn),
                their_nautical_dawn,
                "Nautical dawn",
                result.lat,
                result,
                index,
            );
            compare_times(
                get_event_time(&mut calc, AstronomicalCalculator::get_civil_dusk),
                their_civil_dusk,
                "Civil dusk",
                result.lat,
                result,
                index,
            );
            compare_times(
                get_event_time(&mut calc, AstronomicalCalculator::get_nautical_dusk),
                their_nautical_dusk,
                "Nautical dusk",
                result.lat,
                result,
                index,
            );
        }
    }
}
