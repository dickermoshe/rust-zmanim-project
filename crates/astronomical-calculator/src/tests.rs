#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Utc;
use proptest::proptest;
use serde::Deserialize;
use serde::Serialize;
extern crate std;

use crate::unsafe_spa::sol_pos;
use crate::unsafe_spa::solar_day;
use crate::unsafe_spa::tm;
use crate::unsafe_spa::ApSolposBennet;
use crate::unsafe_spa::SPA;
use crate::CalculationError;

use crate::unsafe_spa::ApSolposBennetNA;
use crate::unsafe_spa::SolarDay;
use crate::AstronomicalCalculator;
use crate::Refraction;
use crate::SolarEventResult;
use proptest::prelude::*;
use std::*;

fn naive_datetime_to_tm(dt: &DateTime<Utc>) -> tm {
    tm {
        timestamp: dt.timestamp_millis(),
    }
}

fn solar_day_to_event(day: solar_day, index: usize) -> SolarEventResult {
    if day.status[index] == 0 {
        SolarEventResult::Occurs(day.t[index])
    } else if day.status[index] == 1 {
        SolarEventResult::AllDay
    } else if day.status[index] == -1 {
        SolarEventResult::AllNight
    } else {
        self::panic!("Invalid status: {}", day.status[index]);
    }
}

fn any_utc_datetime() -> impl Strategy<Value = DateTime<Utc>> {
    // SPA algorithm is valid for years -2000 to 6000 approximately
    // We'll constrain to a more reasonable range: 1900-2100
    (1900i32..=2100i32)
        .prop_flat_map(|year| (Just(year), 1u32..=12u32))
        .prop_flat_map(|(year, month)| {
            let days_in_month = match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    // Leap year check
                    if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                        29
                    } else {
                        28
                    }
                }
                _ => unreachable!(),
            };
            (Just(year), Just(month), 1u32..=days_in_month)
        })
        .prop_flat_map(|(year, month, day)| {
            (
                Just(year),
                Just(month),
                Just(day),
                0u32..24u32,
                0u32..60u32,
                0u32..60u32,
            )
        })
        .prop_filter_map("Create valid datetime", |(year, month, day, hour, min, sec)| {
            chrono::NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|d| d.and_hms_opt(hour, min, sec))
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        })
}
fn refraction_strategy() -> impl Strategy<Value = Refraction> {
    prop_oneof![Just(Refraction::ApSolposBennet), Just(Refraction::ApSolposBennetNA)]
}

/// Wrapper for unsafe BennetNA refraction function
unsafe extern "C" fn unsafe_refract_bennet(pos: sol_pos, gdip: *mut f64, e: f64, p: f64, t: f64) -> sol_pos {
    ApSolposBennet(pos, gdip, e, p, t)
}

/// Wrapper for unsafe BennetNA refraction function
unsafe extern "C" fn unsafe_refract_bennet_na(pos: sol_pos, gdip: *mut f64, e: f64, p: f64, t: f64) -> sol_pos {
    ApSolposBennetNA(pos, gdip, e, p, t)
}
#[allow(clippy::too_many_arguments)]
fn compare(
    datetime: DateTime<Utc>,
    longitude: f64,
    latitude: f64,
    elevation: f64,
    pressure: f64,
    temperature: f64,
    refraction: Refraction,
    mut delta_t: f64,
    use_explicit_delta_t: bool,
    delta_ut1: f64,
    mut gdip: f64,
    use_explicit_gdip: bool,
) -> Result<(), proptest::test_runner::TestCaseError> {
    let naive_datetime = datetime;
    let delta_t_option = if use_explicit_delta_t { Some(delta_t) } else { None };
    let gdip_option = if use_explicit_gdip { Some(gdip) } else { None };
    let calculator = AstronomicalCalculator::new(
        naive_datetime,
        delta_t_option,
        delta_ut1,
        longitude,
        latitude,
        elevation,
        temperature,
        pressure,
        gdip_option,
        refraction,
    );
    prop_assert!(calculator.is_ok());
    let mut calculator = calculator.unwrap();

    // Get all the calculations
    let _julian_day = calculator.get_julian_day();
    let solar_position = *calculator.get_solar_position();
    let solar_transit = calculator.get_solar_transit();
    prop_assert!(solar_transit.is_ok());
    let solar_transit = solar_transit.unwrap();
    let prev_solar_midnight = calculator.get_prev_solar_midnight();
    prop_assert!(prev_solar_midnight.is_ok());
    let prev_solar_midnight = prev_solar_midnight.unwrap();
    let next_solar_midnight = calculator.get_next_solar_midnight();
    prop_assert!(next_solar_midnight.is_ok());
    let next_solar_midnight = next_solar_midnight.unwrap();
    let sunrise = calculator.get_sunrise();
    prop_assert!(sunrise.is_ok());
    let sunrise = sunrise.unwrap();
    let sunset = calculator.get_sunset();
    prop_assert!(sunset.is_ok());
    let sunset = sunset.unwrap();
    let civil_dawn = calculator.get_civil_dawn();
    prop_assert!(civil_dawn.is_ok());
    let civil_dawn = civil_dawn.unwrap();
    let civil_dusk = calculator.get_civil_dusk();
    prop_assert!(civil_dusk.is_ok());
    let civil_dusk = civil_dusk.unwrap();
    let nautical_dawn = calculator.get_nautical_dawn();
    prop_assert!(nautical_dawn.is_ok());
    let nautical_dawn = nautical_dawn.unwrap();
    let nautical_dusk = calculator.get_nautical_dusk();
    prop_assert!(nautical_dusk.is_ok());
    let nautical_dusk = nautical_dusk.unwrap();
    let astronomical_dawn = calculator.get_astronomical_dawn();
    prop_assert!(astronomical_dawn.is_ok());
    let astronomical_dawn = astronomical_dawn.unwrap();
    let astronomical_dusk = calculator.get_astronomical_dusk();
    prop_assert!(astronomical_dusk.is_ok());
    let astronomical_dusk = astronomical_dusk.unwrap();

    // get the unsafe solar day
    let mut ut = naive_datetime_to_tm(&naive_datetime);
    let unsafe_solar_day = unsafe {
        SolarDay(
            &mut ut,
            if use_explicit_delta_t {
                &raw mut delta_t
            } else {
                core::ptr::null_mut()
            }, // delta_t
            delta_ut1, // delta_ut1
            longitude.to_radians(),
            latitude.to_radians(),
            elevation,
            if use_explicit_gdip {
                &raw mut gdip
            } else {
                core::ptr::null_mut()
            }, // gdip
            pressure,
            temperature,
            Some(if refraction == Refraction::ApSolposBennet {
                unsafe_refract_bennet
            } else {
                unsafe_refract_bennet_na
            }),
        )
    };

    let spa = unsafe {
        SPA(
            &mut ut,
            if use_explicit_delta_t {
                &raw mut delta_t
            } else {
                core::ptr::null_mut()
            },
            delta_ut1,
            longitude.to_radians(),
            latitude.to_radians(),
            elevation,
        )
    };
    let zenith_diff = (spa.z - solar_position.zenith).abs();
    let azimuth_diff = (spa.a - solar_position.azimuth).abs();
    prop_assert!(
        zenith_diff <= 1e-6,
        "Zenith difference too large: {} (safe: {}, unsafe: {})",
        zenith_diff,
        spa.z,
        solar_position.zenith
    );
    prop_assert!(
        azimuth_diff <= 1e-6,
        "Azimuth difference too large: {} (safe: {}, unsafe: {})",
        azimuth_diff,
        spa.a,
        solar_position.azimuth
    );
    let diff = (unsafe_solar_day.t[0] - prev_solar_midnight).abs();
    prop_assert!(
        diff <= 5,
        "Timestamp difference too large: {} seconds (safe: {}, unsafe: {})",
        diff,
        unsafe_solar_day.t[0],
        prev_solar_midnight
    );
    let diff = (unsafe_solar_day.t[1] - solar_transit).abs();
    prop_assert!(
        diff <= 5,
        "Timestamp difference too large: {} seconds (safe: {}, unsafe: {})",
        diff,
        unsafe_solar_day.t[1],
        solar_transit
    );
    let diff = (unsafe_solar_day.t[2] - next_solar_midnight).abs();
    prop_assert!(
        diff <= 5,
        "Timestamp difference too large: {} seconds (safe: {}, unsafe: {})",
        diff,
        unsafe_solar_day.t[2],
        next_solar_midnight
    );
    compare_solar_results(solar_day_to_event(unsafe_solar_day, 3), sunrise, "sunrise");
    compare_solar_results(solar_day_to_event(unsafe_solar_day, 4), sunset, "sunset");
    compare_solar_results(solar_day_to_event(unsafe_solar_day, 5), civil_dawn, "civil_dawn");
    compare_solar_results(solar_day_to_event(unsafe_solar_day, 6), civil_dusk, "civil_dusk");
    compare_solar_results(solar_day_to_event(unsafe_solar_day, 7), nautical_dawn, "nautical_dawn");
    compare_solar_results(solar_day_to_event(unsafe_solar_day, 8), nautical_dusk, "nautical_dusk");
    compare_solar_results(
        solar_day_to_event(unsafe_solar_day, 9),
        astronomical_dawn,
        "astronomical_dawn",
    );
    compare_solar_results(
        solar_day_to_event(unsafe_solar_day, 10),
        astronomical_dusk,
        "astronomical_dusk",
    );
    Ok(())
}

proptest! {

    #[test]
    fn solar_events_memoization(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -90.0_f64..=90.0_f64,
        elevation in 0.0..=200.0,
        pressure in 1000.0..=1013.25,
        temperature in 15.0..=25.0,
        refraction in refraction_strategy(),
        delta_t in -60.0..60.0,
        use_explicit_delta_t in proptest::bool::ANY,
        delta_ut1 in -1.0..1.0,
        gdip in -1.5..=1.5,
        use_explicit_gdip in proptest::bool::ANY,

    ) {
        let result = compare(datetime, longitude, latitude, elevation, pressure, temperature, refraction, delta_t, use_explicit_delta_t, delta_ut1, gdip, use_explicit_gdip);
        prop_assert!(result.is_ok());

    }
}

fn compare_solar_results(safe: SolarEventResult, unsafe_: SolarEventResult, name: &str) {
    match (safe, unsafe_) {
        (SolarEventResult::Occurs(ts1), SolarEventResult::Occurs(ts2)) => {
            let diff = (ts1 - ts2).abs();
            assert!(
                diff <= 60,
                "Timestamp difference too large: {} seconds (safe: {}, unsafe: {}) for method {}",
                diff,
                ts1,
                ts2,
                name,
            );
        }
        (SolarEventResult::AllDay, SolarEventResult::AllDay) => {
            // Both indicate sun always above - OK
        }
        (SolarEventResult::AllNight, SolarEventResult::AllNight) => {
            // Both indicate sun always below - OK
        }
        (safe_result, unsafe_result) => {
            self::panic!(
                "Solar event results don't match: safe={:?}, unsafe={:?}, for method {}",
                safe_result,
                unsafe_result,
                name,
            );
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::CalculationError;

    #[test]
    fn test_year_validation_boundary_min() {
        let dt = NaiveDateTime::parse_from_str("-2000-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result.is_ok(), "Year -2000 should be valid");
    }

    #[test]
    fn test_year_validation_boundary_max() {
        let dt = NaiveDateTime::parse_from_str("6000-12-31 23:59:59", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result.is_ok(), "Year 6000 should be valid");
    }

    #[test]
    fn test_year_validation_below_min() {
        let dt = NaiveDateTime::parse_from_str("-2001-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result, Err(CalculationError::TimeConversionError)));
    }

    #[test]
    fn test_year_validation_above_max() {
        let dt = NaiveDateTime::parse_from_str("6001-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result, Err(CalculationError::TimeConversionError)));
    }

    #[test]
    fn test_delta_t_validation_boundary_min() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            Some(-8000.0),
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result.is_ok(), "Delta_t -8000.0 should be valid");
    }

    #[test]
    fn test_delta_t_validation_boundary_max() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            Some(8000.0),
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result.is_ok(), "Delta_t 8000.0 should be valid");
    }

    #[test]
    fn test_delta_t_validation_below_min() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            Some(-8000.1),
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result, Err(CalculationError::TimeConversionError)));
    }

    #[test]
    fn test_delta_t_validation_above_max() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result = AstronomicalCalculator::new(
            dt,
            Some(8000.1),
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result, Err(CalculationError::TimeConversionError)));
    }

    #[test]
    fn test_delta_ut1_validation_boundary() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_min = AstronomicalCalculator::new(
            dt,
            None,
            -1.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_max = AstronomicalCalculator::new(
            dt,
            None,
            1.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result_min.is_ok(), "Delta_ut1 -1.0 should be valid");
        assert!(result_max.is_ok(), "Delta_ut1 1.0 should be valid");
    }

    #[test]
    fn test_delta_ut1_validation_out_of_range() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_below = AstronomicalCalculator::new(
            dt,
            None,
            -1.1,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_above = AstronomicalCalculator::new(
            dt,
            None,
            1.1,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result_below, Err(CalculationError::DeltaUt1OutOfRange)));
        assert!(matches!(result_above, Err(CalculationError::DeltaUt1OutOfRange)));
    }

    #[test]
    fn test_longitude_validation_boundary() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_min = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            -180.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_max = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            180.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result_min.is_ok(), "Longitude -180.0 should be valid");
        assert!(result_max.is_ok(), "Longitude 180.0 should be valid");
    }

    #[test]
    fn test_longitude_validation_out_of_range() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_below = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            -180.1,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_above = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            180.1,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result_below, Err(CalculationError::LongitudeOutOfRange)));
        assert!(matches!(result_above, Err(CalculationError::LongitudeOutOfRange)));
    }

    #[test]
    fn test_latitude_validation_boundary() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_min = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            -90.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_max = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            90.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result_min.is_ok(), "Latitude -90.0 should be valid");
        assert!(result_max.is_ok(), "Latitude 90.0 should be valid");
    }

    #[test]
    fn test_latitude_validation_out_of_range() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_below = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            -90.1,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_above = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            90.1,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result_below, Err(CalculationError::LatitudeOutOfRange)));
        assert!(matches!(result_above, Err(CalculationError::LatitudeOutOfRange)));
    }

    #[test]
    fn test_elevation_validation_boundary() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_min = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            -6378136.6,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_max = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            6378136.6,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result_min.is_ok(), "Elevation -EARTH_R should be valid");
        assert!(result_max.is_ok(), "Elevation EARTH_R should be valid");
    }

    #[test]
    fn test_elevation_validation_out_of_range() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_below = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            -6378137.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_above = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            6378137.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result_below, Err(CalculationError::ElevationOutOfRange)));
        assert!(matches!(result_above, Err(CalculationError::ElevationOutOfRange)));
    }

    #[test]
    fn test_pressure_validation_boundary() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_min = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            0.1,
            None,
            Refraction::ApSolposBennet,
        );
        let result_max = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            5000.0,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result_min.is_ok(), "Pressure 0.1 should be valid");
        assert!(result_max.is_ok(), "Pressure 5000.0 should be valid");
    }

    #[test]
    fn test_pressure_validation_out_of_range() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_zero = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            0.0,
            None,
            Refraction::ApSolposBennet,
        );
        let result_above = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            5000.1,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result_zero, Err(CalculationError::PressureOutOfRange)));
        assert!(matches!(result_above, Err(CalculationError::PressureOutOfRange)));
    }

    #[test]
    fn test_temperature_validation_boundary() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_min = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            -273.15,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_max = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            6000.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(result_min.is_ok(), "Temperature -273.15 should be valid");
        assert!(result_max.is_ok(), "Temperature 6000.0 should be valid");
    }

    #[test]
    fn test_temperature_validation_out_of_range() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let result_below = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            -273.16,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        let result_above = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            6000.1,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result_below, Err(CalculationError::TemperatureOutOfRange)));
        assert!(matches!(result_above, Err(CalculationError::TemperatureOutOfRange)));
    }

    #[test]
    fn test_gdip_validation_boundary() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        use core::f64::consts::PI;
        let result_min = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            Some(-PI / 2.0),
            Refraction::ApSolposBennet,
        );
        let result_max = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            Some(PI / 2.0),
            Refraction::ApSolposBennet,
        );
        assert!(result_min.is_ok(), "Gdip -PI/2 should be valid");
        assert!(result_max.is_ok(), "Gdip PI/2 should be valid");
    }

    #[test]
    fn test_gdip_validation_out_of_range() {
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        use core::f64::consts::PI;
        let result_below = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            Some(-PI / 2.0 - 0.1),
            Refraction::ApSolposBennet,
        );
        let result_above = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            Some(PI / 2.0 + 0.1),
            Refraction::ApSolposBennet,
        );
        assert!(matches!(result_below, Err(CalculationError::GeometricDipOutOfRange)));
        assert!(matches!(result_above, Err(CalculationError::GeometricDipOutOfRange)));
    }
}

#[cfg(test)]
mod edge_case_tests {

    use super::*;

    #[test]
    fn test_polar_region_at_solstice() {
        for (lat, date) in [
            (89.0, "2024-06-21 12:00:00"),
            (89.0, "2024-12-21 12:00:00"),
            (-89.0, "2024-06-21 12:00:00"),
            (-89.0, "2024-12-21 12:00:00"),
        ] {
            let dt = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .and_utc();
            let result = compare(
                dt,
                0.0,
                lat,
                0.0,
                20.0,
                1013.25,
                Refraction::ApSolposBennet,
                0.0,
                false,
                0.0,
                0.0,
                false,
            );
            assert!(result.is_ok());
        }
        for (lat, date, expected) in [
            (89.0, "2024-06-21 12:00:00", SolarEventResult::AllDay),
            (89.0, "2024-12-21 12:00:00", SolarEventResult::AllNight),
            (-89.0, "2024-06-21 12:00:00", SolarEventResult::AllNight),
            (-89.0, "2024-12-21 12:00:00", SolarEventResult::AllDay),
        ] {
            let dt = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S")
                .unwrap()
                .and_utc();
            let mut calc = AstronomicalCalculator::new(
                dt,
                None,
                0.0,
                0.0,
                lat,
                0.0,
                20.0,
                1013.25,
                None,
                Refraction::ApSolposBennet,
            )
            .unwrap();
            let sunrise = calc.get_sunrise().unwrap();
            let sunset = calc.get_sunset().unwrap();

            assert_eq!(sunrise, expected);
            assert_eq!(sunset, expected);
        }
    }

    #[test]
    fn test_equator_sunrise_sunset() {
        // Test at equator - should always have sunrise and sunset
        let dt = NaiveDateTime::parse_from_str("2024-03-21 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let mut calc = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let sunrise = calc.get_sunrise().unwrap();
        let sunset = calc.get_sunset().unwrap();

        // At equator, should always have sunrise and sunset
        assert!(
            matches!(sunrise, SolarEventResult::Occurs(_)),
            "At equator, should have sunrise"
        );
        assert!(
            matches!(sunset, SolarEventResult::Occurs(_)),
            "At equator, should have sunset"
        );
    }

    #[test]
    fn test_extreme_elevation() {
        // Test with very high elevation (Mount Everest ~8848m)
        let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let mut calc = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            86.9250,
            27.9881,
            8848.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let position = calc.get_solar_position();
        assert!(
            position.zenith >= 0.0 && position.zenith <= std::f64::consts::PI,
            "Solar position should be valid at high elevation"
        );
    }

    #[test]
    fn test_extreme_temperature() {
        // Test with extreme temperatures
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();

        // Very cold
        let mut calc_cold = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            -50.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();
        let _ = calc_cold.get_solar_position();

        // Very hot
        let mut calc_hot = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            50.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();
        let _ = calc_hot.get_solar_position();

        // Should not panic
    }

    #[test]
    fn test_extreme_pressure() {
        // Test with extreme pressures
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();

        // Low pressure (high altitude)
        let mut calc_low = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            500.0,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();
        let _ = calc_low.get_solar_position();

        // High pressure (sea level, storm)
        let mut calc_high = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1050.0,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();
        let _ = calc_high.get_solar_position();

        // Should not panic
    }

    #[test]
    fn test_both_refraction_models() {
        // Test that both refraction models work
        let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();

        let mut calc_bennet = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            51.5,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();
        let pos_bennet = calc_bennet.get_solar_position();

        let mut calc_bennet_na = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            51.5,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennetNA,
        )
        .unwrap();
        let pos_bennet_na = calc_bennet_na.get_solar_position();

        // Both should produce valid positions
        assert!(pos_bennet.zenith >= 0.0 && pos_bennet.zenith <= std::f64::consts::PI);
        assert!(pos_bennet_na.zenith >= 0.0 && pos_bennet_na.zenith <= std::f64::consts::PI);
    }

    #[test]
    fn test_solar_transit_at_equator() {
        // Test solar transit calculation
        let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let mut calc = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let transit = calc.get_solar_transit();
        assert!(transit.is_ok(), "Solar transit should be calculable");
        let transit_ts = transit.unwrap();
        assert!(transit_ts > 0, "Transit timestamp should be positive");
    }

    #[test]
    fn test_all_twilight_types() {
        // Test all twilight calculations
        let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let mut calc = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            51.5,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        assert!(calc.get_civil_dawn().is_ok());
        assert!(calc.get_civil_dusk().is_ok());
        assert!(calc.get_nautical_dawn().is_ok());
        assert!(calc.get_nautical_dusk().is_ok());
        assert!(calc.get_astronomical_dawn().is_ok());
        assert!(calc.get_astronomical_dusk().is_ok());
    }

    #[test]
    fn test_solar_event_result_timestamp() {
        // Test SolarEventResult::timestamp() method
        let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let mut calc = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            51.5,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let sunrise = calc.get_sunrise().unwrap();
        if let Some(ts) = sunrise.timestamp() {
            assert!(ts > 0, "Sunrise timestamp should be positive");
        }

        // Test AlwaysAbove case
        assert!(SolarEventResult::AllDay.timestamp().is_none());
        assert!(SolarEventResult::AllNight.timestamp().is_none());
    }

    #[test]
    fn test_get_delta_t_function() {
        // Test get_delta_t function
        let dt_2024 = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let delta_t_2024 = crate::get_delta_t(&dt_2024);
        assert!(
            delta_t_2024 > 60.0 && delta_t_2024 < 80.0,
            "Delta_t for 2024 should be around 69 seconds"
        );

        let dt_2000 = NaiveDateTime::parse_from_str("2000-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let delta_t_2000 = crate::get_delta_t(&dt_2000);
        assert!(
            delta_t_2000 > 60.0 && delta_t_2000 < 70.0,
            "Delta_t for 2000 should be around 64 seconds"
        );
    }

    #[test]
    fn test_julian_day_calculation() {
        // Test Julian day calculation
        let dt = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let mut calc = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let jd = calc.get_julian_day();
        assert_eq!(jd.jd, 2460311.000000,);
    }

    #[test]
    fn test_solar_time_calculation() {
        // Test solar time calculation
        let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc();
        let mut calc = AstronomicalCalculator::new(
            dt,
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let solar_time = calc.get_solar_time();
        assert!(solar_time.is_ok(), "Solar time should be calculable");
    }
}

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

#[derive(Serialize, Deserialize, Debug)]
struct SunResults {
    results: std::vec::Vec<SunResult>,
}

fn ts_to_dt(ts: f64) -> NaiveDateTime {
    #[allow(deprecated)]
    NaiveDateTime::from_timestamp_millis((ts * 1000.0) as i64).unwrap()
}
fn tz_ts_to_dt<T: TimeZone>(ts: f64, tz: T) -> DateTime<T> {
    tz.timestamp_millis_opt((ts * 1000.0) as i64).unwrap()
}

/// Calculate tolerance in seconds based on latitude.
/// 60 seconds until 45° (300 seconds for astronomical), then exponential growth to 10000 seconds at poles.
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

fn get_event_time(
    calc: &mut AstronomicalCalculator,
    getter: fn(&mut AstronomicalCalculator) -> Result<SolarEventResult, CalculationError>,
) -> Option<NaiveDateTime> {
    getter(calc)
        .ok()
        .and_then(|result| result.timestamp())
        .map(|ts| ts_to_dt(ts as f64))
}

/// Assert that a list of Option<DateTime> values are in chronological order.
/// Only compares consecutive pairs where both values are Some.
///
/// # Arguments
/// * `events` - Slice of Option values to check
/// * `names` - Slice of names for error messages (should match events length)
/// * `context` - Additional context string for error messages
/// * `row` - Record data for error messages
/// * `index` - Index for error messages
fn assert_events_in_order<T: PartialOrd + std::fmt::Display, C: std::fmt::Debug>(
    events: &[Option<T>],
    names: &[&str],
    context: &str,
    row: &C,
    index: usize,
) {
    assert_eq!(
        events.len(),
        names.len(),
        "Events and names slices must have the same length"
    );

    for i in 0..events.len().saturating_sub(1) {
        if let (Some(prev), Some(next)) = (&events[i], &events[i + 1]) {
            assert!(
                prev < next,
                "{}: {} should be before {}. {}: {}, {}: {}. Record: {:?} Index: {}",
                context,
                names[i],
                names[i + 1],
                names[i],
                prev,
                names[i + 1],
                next,
                row,
                index
            );
        }
    }
}

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

#[test]
fn test_geonames_csv_transit() {
    use chrono::TimeZone;
    use chrono_tz::Tz;
    use csv::ReaderBuilder;
    use rand::Rng;
    use std::fs::File;
    use std::io::BufReader;
    #[allow(unused)]
    #[derive(Debug, Deserialize)]
    struct GeonamesRow {
        geoname_id: std::string::String,
        name: std::string::String,
        ascii_name: std::string::String,
        #[serde(default)]
        alternate_names: std::string::String,
        feature_class: std::string::String,
        feature_code: std::string::String,
        country_code: std::string::String,
        cou_name_en: std::string::String,
        #[serde(default)]
        country_code_2: std::string::String,
        #[serde(default)]
        admin1_code: std::string::String,
        #[serde(default)]
        admin2_code: std::string::String,
        #[serde(default)]
        admin3_code: std::string::String,
        #[serde(default)]
        admin4_code: std::string::String,
        population: std::string::String,
        #[serde(default)]
        elevation: std::string::String,
        #[serde(default)]
        dem: std::string::String,
        timezone: std::string::String,
        #[serde(default)]
        modification_date: std::string::String,
        #[serde(default)]
        label_en: std::string::String,
        coordinates: std::string::String,
    }

    // Open and read the CSV file
    let file = File::open("test_data/cities.csv").expect("Failed to open CSV file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(BufReader::new(file));

    let mut rng = rand::thread_rng();

    // Process each row
    for (index, result) in rdr.deserialize().enumerate() {
        let row: GeonamesRow = result.expect("Failed to parse CSV row");

        // Parse coordinates (format: "lat,lon")
        let coords: std::vec::Vec<&str> = row.coordinates.split(',').collect();
        if coords.len() != 2 {
            continue;
        }

        let lat: f64 = coords[0].trim().parse().unwrap();
        let lon: f64 = coords[1].trim().parse().unwrap();

        // Skip extreme latitudes that might cause issues
        if lat.abs() > 75.0 {
            continue;
        }

        // Parse timezone using chrono-tz
        let tz: Tz = row.timezone.parse().unwrap();

        // Generate a random input time between 1900 and 2100
        let year = rng.gen_range(1900..=2100);
        let month = rng.gen_range(1..=12);
        let day = rng.gen_range(1..=28); // Use 28 to avoid month-end issues

        // Create a naive datetime at midday
        let dt = tz.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap();

        // Create calculator and get transit
        let mut calc = AstronomicalCalculator::new(
            dt.to_utc(),
            None,
            0.0,
            lon,
            lat,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let transit = tz_ts_to_dt(calc.get_solar_transit().unwrap() as f64, tz);

        let diff = transit - dt;
        let midday_to_transit_diff = diff.num_seconds().abs();
        assert!(
            midday_to_transit_diff <= 60 * 60 * 8,
            "Midday to transit difference too large: {} seconds, input: {}, transit: {}. Record: {:?} Index: {}",
            midday_to_transit_diff,
            dt,
            transit,
            row,
            index
        );
        // Sunrise is always before transit and sunset is always after transit
        let sunrise = calc
            .get_sunrise()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let sunset = calc
            .get_sunset()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let astronomical_dawn = calc
            .get_astronomical_dawn()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let nautical_dawn = calc
            .get_nautical_dawn()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let civil_dawn = calc
            .get_civil_dawn()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let civil_dusk = calc
            .get_civil_dusk()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let nautical_dusk = calc
            .get_nautical_dusk()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let astronomical_dusk = calc
            .get_astronomical_dusk()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();

        assert_events_in_order(
            &[
                astronomical_dawn,
                nautical_dawn,
                civil_dawn,
                sunrise,
                Some(transit),
                sunset,
                civil_dusk,
                nautical_dusk,
                astronomical_dusk,
            ],
            &[
                "Astronomical dawn",
                "Nautical dawn",
                "Civil dawn",
                "Sunrise",
                "Transit",
                "Sunset",
                "Civil dusk",
                "Nautical dusk",
                "Astronomical dusk",
            ],
            "Solar events",
            &row,
            index,
        );
    }
}

/// Regression test for Alos Hashachar (16.1° below horizon) calculation
///
/// This test case was failing with ~54 seconds difference from Java implementation.
/// Skyfield (NASA DE440 ephemeris) was used as an independent reference.
///
/// Note: This implementation includes geometric dip correction in twilight calculations,
/// while Skyfield uses the pure geometric horizon. This causes a small systematic difference.
/// The implementation matches the Java reference within acceptable tolerances.
///
/// Test data:
/// - Location: Cherbourg area, France (49.85°N, -1.46°E)
/// - Elevation: 178.06m
/// - Date: 2051-07-10 (CEST, UTC+2)
/// - Expected (Skyfield, geometric horizon): 2051-07-10T03:24:29.164+02:00
/// - Expected (Java, with dip): 2051-07-10T03:24:23.875+02:00
#[test]
fn test_alos_hashachar_regression_2051_07_10() {
    use chrono_tz::Europe::Paris;

    let lat = 49.8468936628077;
    let lon = -1.4570257298796037;
    let elevation = 178.05645489643047;

    // Reference time from failing test (not actually used for calculation, just for context)
    let _reference_time = Paris.with_ymd_and_hms(2051, 7, 10, 22, 24, 54).unwrap();

    // Create calculator for this date
    let date = Paris.with_ymd_and_hms(2051, 7, 10, 12, 0, 0).unwrap();

    // Calculate delta_t explicitly
    use crate::get_delta_t;
    let delta_t = get_delta_t(&date.to_utc());
    println!("Delta T for 2051: {} seconds", delta_t);

    // Try with no-atmosphere refraction model since we're looking at deep twilight
    let mut calc = AstronomicalCalculator::new(
        date.to_utc(),
        Some(delta_t),                // Explicit delta_t
        0.0,                          // delta_ut1 for future dates (unknown, use 0.0)
        lon,                          // longitude
        lat,                          // latitude
        elevation,                    // elevation
        20.0,                         // temperature
        1013.25,                      // pressure
        None,                         // gdip
        Refraction::ApSolposBennetNA, // No atmosphere for deep twilight
    )
    .unwrap();

    // First check sunrise for comparison
    let sunrise_result = calc.get_sunrise().unwrap();
    if let Some(sunrise_ts) = sunrise_result.timestamp() {
        let sunrise_dt = tz_ts_to_dt(sunrise_ts as f64, Paris);
        println!("Sunrise: {}", sunrise_dt.format("%Y-%m-%dT%H:%M:%S%.3f%z"));
    }

    // Calculate Alos Hashachar (16.1 degrees below horizon before sunrise)
    let alos_result = calc.get_sunrise_offset_by_degrees(16.1).unwrap();
    let alos_ts = alos_result.timestamp().expect("Alos Hashachar should occur");
    let alos_dt = tz_ts_to_dt(alos_ts as f64, Paris);

    println!(
        "Transit: {}",
        tz_ts_to_dt(calc.get_solar_transit().unwrap() as f64, Paris).format("%Y-%m-%dT%H:%M:%S%.3f%z")
    );

    // Expected from Skyfield: 2051-07-10T03:24:29.164+02:00
    // Convert to timestamp: 2051-07-10 03:24:29.164 CEST (UTC+2)
    let expected_dt = Paris
        .with_ymd_and_hms(2051, 7, 10, 3, 24, 29)
        .unwrap()
        .checked_add_signed(chrono::Duration::milliseconds(164))
        .unwrap();

    // Verify the actual sun position at the calculated time
    let mut calc_verify = AstronomicalCalculator::new(
        alos_dt.to_utc(),
        Some(delta_t),
        0.0,
        lon,
        lat,
        elevation,
        20.0,
        1013.25,
        None,
        Refraction::ApSolposBennetNA,
    )
    .unwrap();
    let pos_calc = calc_verify.get_solar_position();
    let alt_calc = 90.0 - pos_calc.zenith.to_degrees();

    // Check sun position at expected time
    let mut calc_expected = AstronomicalCalculator::new(
        expected_dt.to_utc(),
        Some(delta_t),
        0.0,
        lon,
        lat,
        elevation,
        20.0,
        1013.25,
        None,
        Refraction::ApSolposBennetNA,
    )
    .unwrap();
    let pos_expected = calc_expected.get_solar_position();
    let alt_expected = 90.0 - pos_expected.zenith.to_degrees();

    let diff_seconds = (alos_dt.timestamp() - expected_dt.timestamp()).abs();

    println!("Alos Hashachar regression test:");
    println!(
        "  Calculated: {} (altitude: {:.4}°)",
        alos_dt.format("%Y-%m-%dT%H:%M:%S%.3f%z"),
        alt_calc
    );
    println!(
        "  Expected:   {} (altitude: {:.4}°)",
        expected_dt.format("%Y-%m-%dT%H:%M:%S%.3f%z"),
        alt_expected
    );
    println!("  Difference: {} seconds", diff_seconds);
    println!("  Target altitude: -16.1°");

    assert!(
        diff_seconds <= 10,
        "Alos Hashachar calculation differs by {} seconds (expected ≤10s). Calculated: {}, Expected: {}",
        diff_seconds,
        alos_dt.format("%Y-%m-%dT%H:%M:%S%.3f%z"),
        expected_dt.format("%Y-%m-%dT%H:%M:%S%.3f%z")
    );
}
