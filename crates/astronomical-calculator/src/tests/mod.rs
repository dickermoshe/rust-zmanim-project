#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
use crate::*;
use chrono::prelude::*;

/// Validates solar event calculations using sampled records from the GeoNames CSV dataset.
mod geonames_tests;
/// Property-based tests (`proptest`) that compare the safe calculator to the SPA reference in `unsafe_spa`.
mod spa_tests;
/// Property-based tests that compare the safe calculator to the SunCalc test data.
mod suncalc_tests;
/// This contains The original implementation of SPA converted to Rust using automated tooling.
/// We proptest our implementation against the original C implementation.
mod unsafe_spa;

extern crate std;

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

#[test]
fn test_alos_hashachar_regression_2051_07_10() {
    use chrono_tz::Europe::Paris;

    let lat = 49.8468936628077;
    let lon = -1.4570257298796037;
    let elevation = 178.05645489643047;

    let date = Paris.with_ymd_and_hms(2051, 7, 10, 12, 0, 0).unwrap();

    use crate::get_delta_t;
    let delta_t = get_delta_t(&date.to_utc());

    let mut calc = AstronomicalCalculator::new(
        date.to_utc(),
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

    let alos_result = calc.get_sunrise_offset_by_degrees(16.1, true).unwrap();
    let alos_ts = alos_result.timestamp().expect("Alos Hashachar should occur");
    let alos_dt = Paris.timestamp_millis_opt((alos_ts * 1000) as i64).unwrap();

    let expected_dt = Paris
        .with_ymd_and_hms(2051, 7, 10, 3, 24, 29)
        .unwrap()
        .checked_add_signed(chrono::Duration::milliseconds(164))
        .unwrap();

    let diff_seconds = (alos_dt.timestamp() - expected_dt.timestamp()).abs();

    assert!(
        diff_seconds <= 10,
        "Alos Hashachar calculation differs by {} seconds (expected ≤10s). Calculated: {}, Expected: {}",
        diff_seconds,
        alos_dt.format("%Y-%m-%dT%H:%M:%S%.3f%z"),
        expected_dt.format("%Y-%m-%dT%H:%M:%S%.3f%z")
    );
}
