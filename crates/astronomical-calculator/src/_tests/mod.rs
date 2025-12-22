extern crate std;
mod continuity_tests;
mod spa_tests;
use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;
use proptest::proptest;
use serde::Deserialize;

use crate::AstronomicalCalculator;

#[derive(Debug, Deserialize, Clone)]
struct SunTimesRecord {
    latitude: f64,
    longitude: f64,
    altitude: f64,
    input_timestamp: String,
    sunrise_rfc: Option<String>,
    transit_rfc: Option<String>,
    sunset_rfc: Option<String>,
}

/// Test sunset, sunrise and transit times against a dataset of known values
/// See the gen.py script for how the dataset was generated.
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
fn _test_sun_times_dataset(csv_path: &str) {
    let mut rdr = csv::Reader::from_path(csv_path).expect("Failed to open CSV file");

    for (idx, result) in rdr.deserialize().enumerate() {
        let record: SunTimesRecord = result.expect("Failed to deserialize CSV record");
        // The tolerances are based how far north or south the location is from the equator
        let second_tolerance = 5 + record.latitude.abs() as i64;

        // Create datetime from input timestamp using the CSV timezone
        let dt = DateTime::parse_from_rfc3339(&record.input_timestamp).unwrap().to_utc();

        // Skip if year is out of reasonable range
        if dt.year() < -2000 || dt.year() > 6000 {
            continue;
        }

        // Calculate solar position
        let calc_result = AstronomicalCalculator::new(
            dt,
            0.0,
            67.0,
            record.longitude,
            record.latitude,
            record.altitude,
            1013.25,
            12.0,
            0.0,
            0.0,
            0.5667,
        );
        println!("Date: {:?}", dt);
        println!("Latitude: {:?}", record.latitude);
        println!("Longitude: {:?}", record.longitude);
        println!("Altitude: {:?}", record.altitude);
        if calc_result.is_err() {
            panic!(
                "Row {}: Failed to create calculator at ({:.6}, {:.6}), altitude {:.2}m - Error: {:?}",
                idx + 2,
                record.latitude,
                record.longitude,
                record.altitude,
                calc_result.err()
            );
        }
        let calc_result = calc_result.unwrap();

        let cloned = record.clone();
        let result = calc_result.calculate();

        if let Some(sunrise_rfc) = record.sunrise_rfc {
            if let Some(sunrise_time) = result.sunrise_time {
                let sunrise_record = DateTime::parse_from_rfc3339(&sunrise_rfc).unwrap().to_utc();
                let diff = (sunrise_time - sunrise_record).abs();
                assert!(
                    diff.num_seconds() <= second_tolerance,
                    "Row {}: Sunrise time mismatch at ({}, {})\n  Expected: {} \n  Got: {} \n  Diff: {} seconds.",
                    idx + 2,
                    record.latitude,
                    record.longitude,
                    sunrise_record,
                    sunrise_time,
                    diff.num_seconds()
                );
            }
        }
        if let Some(sunset_rfc) = record.sunset_rfc {
            if let Some(sunset_time) = result.sunset_time {
                let sunset_record = DateTime::parse_from_rfc3339(&sunset_rfc).unwrap().to_utc();
                let diff = (sunset_time - sunset_record).abs();
                assert!(
                    diff.num_seconds() <= second_tolerance,
                    "Row {}: Sunset time mismatch at ({}, {})\n  Expected: {} \n  Got: {} \n  Diff: {} seconds",
                    idx + 2,
                    record.latitude,
                    record.longitude,
                    sunset_record,
                    sunset_time,
                    diff.num_seconds()
                );
            }
        }
        if let Some(transit_rfc) = record.transit_rfc {
            let transit_record = DateTime::parse_from_rfc3339(&transit_rfc).unwrap().to_utc();
            let diff = (result.solar_transit_time - transit_record).abs();

            // date happens on same date as dt
            assert_eq!(
                result.solar_transit_time.date_naive(),
                dt.date_naive(),
                "Transit time should occur on the current date. Got {:?}, expected {:?}, Record: {:?}, Result {:?}",
                result.solar_transit_time,
                dt.date_naive(),
                cloned,
                result,
            );
            assert!(
                diff.num_seconds() <= second_tolerance,
                "Row {}: Transit time mismatch at ({}, {})\n  Expected: {} \n  Got: {} \n  Diff: {} seconds",
                idx + 2,
                record.latitude,
                record.longitude,
                transit_record,
                result.solar_transit_time,
                diff.num_seconds()
            );
        }
    }
}

/// Test sunset, sunrise and transit times against a dataset of known values
/// See the gen.py script for how the dataset was generated.
#[test]
fn test_sun_times_dataset_unreasonable() {
    let csv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tests/unreasonable.csv");
    _test_sun_times_dataset(csv_path);
}
#[test]
fn test_sun_times_dataset_reasonable() {
    let csv_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tests/reasonable.csv");
    _test_sun_times_dataset(csv_path);
}
proptest! {
    #[allow(clippy::unwrap_used)]
    #[test]
    fn transit_occurs_on_the_correct_date(
        timestamp in -15768000000i64..=15768000000i64,
        delta_ut1 in -1.0..=1.0,
        delta_t in -8000.0..=8000.0,
        longitude in -90.0..=90.0,
        latitude in -89.0_f64..=89.0_f64,
        elevation in -1000.0..=5000.0,
        pressure in 800.0..=1200.0,
        temperature in 0.0..=40.0,
        slope in -30.0..=30.0,
        azm_rotation in -180.0..=180.0,
        atmos_refract in 0.5..=0.6
    ) {
        let dt = Utc.timestamp_opt(timestamp, 0).single().unwrap();

        let calculator = AstronomicalCalculator::new(
            dt,
            delta_ut1,
            delta_t,
            longitude,
            latitude,
            elevation,
            pressure,
            temperature,
            slope,
            azm_rotation,
            atmos_refract,
        ).unwrap();

        // Ensure that the transit always occurs on the current date
        let result = calculator.calculate();
        assert_eq!(
            result.solar_transit_time.date_naive(),
            dt.date_naive(),
            "Transit time should occur on the current date. Got {:?}, expected {:?}",
            result.solar_transit_time.date_naive(),
            dt.date_naive(),
        );


    }
}
