//! Continuity tests for solar position calculations.
//!
//! These tests verify that solar position calculations change smoothly over time
//! and don't exhibit sudden jumps or discontinuities. This helps ensure the
//! mathematical accuracy and numerical stability of the algorithms.
//!

extern crate std;
use crate::AstronomicalCalculator;
use chrono::{DateTime, Days, TimeZone, Utc};
use proptest::prelude::*;

proptest! {
#[allow(clippy::unwrap_used)]

    #[test]
    fn solar_position_changes_smoothly_over_consecutive_days(
        timestamp in -15768000000i64..=15768000000i64,
        longitude in -90.0..=90.0,
        latitude in -89.0_f64..=89.0_f64,  // Avoid poles to prevent edge cases
        elevation in -1000.0..=5000.0,
        pressure in 800.0..=1200.0,
        temperature in 0.0..=40.0,
        slope in -30.0..=30.0,
        azm_rotation in -180.0..=180.0,
        atmos_refract in 0.5..=0.6
    ) {
        // Start from a random date
        let start_date = Utc.timestamp_opt(timestamp, 0).single().unwrap();

        // Calculate solar positions for 10 consecutive days
        let mut prev_transit: Option<DateTime<Utc>> = None;
        let mut prev_sunrise: Option<DateTime<Utc>> = None;
        let mut prev_sunset: Option<DateTime<Utc>> = None;

        // At very high latitudes we can have a much larger jump in sunrise and sunset times, so we need to account for that.
        let max_jump_seconds = if latitude.abs() > 60.0 {
            3*60*60 // 3 hours
        } else {
            60*60 // 1 hour
        };


        for day_offset in 0..10 {
            let current_date = start_date.checked_add_days( Days::new(day_offset));

            if current_date.is_none() {
                return Ok(());
            }
            let current_date = current_date.unwrap();


            let calculator = AstronomicalCalculator::new(
                current_date,
                0.0, // delta_ut1
                69.0, // approximate delta_t
                longitude,
                latitude,
                elevation,
                pressure,
                temperature,
                slope,
                azm_rotation,
                atmos_refract,
            ).unwrap();

            let result = calculator.calculate();

            let solar_transit_time = result.solar_transit_time;
            let sunrise_time = result.sunrise_time;
            let sunset_time = result.sunset_time;



            if let Some(prev) = prev_transit {
                let adjusted_prev_transit = prev.checked_add_days( Days::new(1)).unwrap();
                let transit_diff_seconds = (solar_transit_time - adjusted_prev_transit).as_seconds_f64().abs();
                prop_assert!(transit_diff_seconds < max_jump_seconds as f64,  // Max ~10 minutes change per day
                    "Transit time jumped {} seconds between days {} -> {}. Max allowed: {} seconds. Today's transit: {}. Yesterday's transit: {}",
                    transit_diff_seconds,  prev, solar_transit_time, max_jump_seconds, solar_transit_time, adjusted_prev_transit);
            }

                prev_transit = Some(solar_transit_time);


            if let (Some(prev), Some(curr)) = (prev_sunrise, sunrise_time) {
                let adjusted_prev_sunrise = prev.checked_add_days( Days::new(1)).unwrap();
                let sunrise_diff_seconds = (curr - adjusted_prev_sunrise).as_seconds_f64().abs();
                prop_assert!(sunrise_diff_seconds < max_jump_seconds as f64,  // Max ~10 minutes change per day
                    "Sunrise time jumped {} seconds between days {} -> {}. Max allowed: {} seconds. Today's sunrise: {}. Yesterday's sunrise: {}",
                    sunrise_diff_seconds,  prev, curr, max_jump_seconds, curr, adjusted_prev_sunrise);
            }
            if let Some(curr) = sunrise_time {
                prev_sunrise = Some(curr);
            } else {
                prev_sunrise = None;
            }

            if let (Some(prev), Some(curr)) = (prev_sunset, sunset_time) {
                let adjusted_prev_sunset = prev.checked_add_days( Days::new(1)).unwrap();
                let sunset_diff_seconds = (curr - adjusted_prev_sunset).as_seconds_f64().abs();
                prop_assert!(sunset_diff_seconds < max_jump_seconds as f64,  // Max ~10 minutes change per day
                    "Sunset time jumped {} seconds between days {} -> {}. Max allowed: {} seconds. Today's sunset: {}. Yesterday's sunset: {}",
                    sunset_diff_seconds,  prev, curr, max_jump_seconds, curr, adjusted_prev_sunset);
            }
            if let Some(curr) = sunset_time {
                prev_sunset = Some(curr);
            } else {
                prev_sunset = None;
            }
        }
    }
}
