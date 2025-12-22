//! SPA (Solar Position Algorithm) comparison tests.
//!
//! This module contains tests that compare the results of our solar position
//! calculations against the original NREL SPA library. These tests are only
//! compiled when the `__spa-sys` feature is enabled.
//!
//! The SPA library provides the authoritative reference implementation for
//! solar position calculations.

#[cfg(feature = "__spa-sys")]
mod spa_sys_tests {
    #[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
    mod spa_sys {
        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    }
    use chrono::{DateTime, TimeZone, Utc};
    use chrono::{Duration, NaiveTime};

    use crate::delta_t;
    use crate::AstronomicalCalculator;

    extern crate std;
    use chrono::Datelike;
    use chrono::Timelike;
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;
    use std::*;

    fn any_utc_datetime() -> impl Strategy<Value = DateTime<Utc>> {
        arb::<DateTime<Utc>>().prop_filter("SPA library doesn't accept leap seconds (second >= 60)", |dt| {
            (dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0) < 60.0
        })
    }

    #[test]
    fn compare_validation_to_original_spa() {
        proptest!(|(
                dt in any_utc_datetime(),
                delta_ut1 in -1.0..=1.0,
                delta_t in -8000.0..=8000.0,
                longitude in -180.0..=180.0,
                latitude in -90.0..=90.0,
                elevation in -6500000.0..=6500000.0,
                pressure in 0.0..=5000.0,
                temperature in -273.0..=6000.0,
                slope in -360.0..=360.0,
                azm_rotation in -360.0..=360.0,
                atmos_refract in -5.0..=5.0
                )| {
            let our_result = AstronomicalCalculator::new(
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
                atmos_refract
            ).is_ok();
            let mut their_inputs = spa_sys::spa_data {
                year: dt.year(),
                month: dt.month() as i32,
                day: dt.day() as i32,
                hour: dt.hour() as i32,
                minute: dt.minute() as i32,
                second: dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0,
                delta_ut1,
                delta_t,
                timezone: 0.0,
                longitude,
                latitude,
                elevation,
                pressure,
                temperature,
                slope,
                azm_rotation,
                atmos_refract,
                function: 3 ,
                jd: 0.0,
                jc: 0.0,
                jde: 0.0,
                jce: 0.0,
                jme: 0.0,
                l: 0.0,
                b: 0.0,
                r: 0.0,
                theta: 0.0,
                beta: 0.0,
                x0: 0.0,
                x1: 0.0,
                x2: 0.0,
                x3: 0.0,
                x4: 0.0,
                del_psi: 0.0,
                del_epsilon: 0.0,
                epsilon0: 0.0,
                epsilon: 0.0,
                del_tau: 0.0,
                lamda: 0.0,
                nu0: 0.0,
                nu: 0.0,
                alpha: 0.0,
                delta: 0.0,
                h: 0.0,
                xi: 0.0,
                del_alpha: 0.0,
                delta_prime: 0.0,
                alpha_prime: 0.0,
                h_prime: 0.0,
                e0: 0.0,
                del_e: 0.0,
                e: 0.0,
                eot: 0.0,
                srha: 0.0,
                ssha: 0.0,
                sta: 0.0,
                zenith: 0.0,
                azimuth_astro: 0.0,
                azimuth: 0.0,
                incidence: 0.0,
                suntransit: 0.0,
                sunrise: 0.0,
                sunset: 0.0,
            };
            let their_result = unsafe { spa_sys::spa_calculate(&mut their_inputs) };
            prop_assert_eq!(our_result, their_result == 0);
        })
    }

    #[test]
    fn compare_solar_positioning_to_spa() {
        proptest!(|(
            timestamp in -15768000000i64..=15768000000i64,

            delta_ut1 in -1.0..=1.0,
            delta_t in -8000.0..=8000.0,
            longitude in -180.0..=180.0,
            latitude in -90.0..=90.0,
            elevation in -6500000.0..=6500000.0,
            pressure in 0.0..=5000.0,
            temperature in -273.0..=6000.0,
            slope in -360.0..=360.0,
            azm_rotation in -360.0..=360.0,
            atmos_refract in -5.0..=5.0
            )| {
                #[allow(clippy::unwrap_used)]
                let dt = Utc.timestamp_opt(timestamp,0).single().unwrap();
            let our_result = AstronomicalCalculator::new(
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
                atmos_refract
            ).map(|inputs| inputs.calculate_solar_position()).ok();
            if let Some(our_result) = our_result {
                let mut their_inputs = spa_sys::spa_data {
                    year: dt.year(),
                    month: dt.month() as i32,
                    day: dt.day() as i32,
                    hour: dt.hour() as i32,
                    minute: dt.minute() as i32,
                    second: dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0,
                    delta_ut1,
                    delta_t,
                    timezone: 0.0,
                    longitude,
                    latitude,
                    elevation,
                    pressure,
                    temperature,
                    slope,
                    azm_rotation,
                    atmos_refract,
                    function: 3 ,
                    jd: 0.0,
                    jc: 0.0,
                    jde: 0.0,
                    jce: 0.0,
                    jme: 0.0,
                    l: 0.0,
                    b: 0.0,
                    r: 0.0,
                    theta: 0.0,
                    beta: 0.0,
                    x0: 0.0,
                    x1: 0.0,
                    x2: 0.0,
                    x3: 0.0,
                    x4: 0.0,
                    del_psi: 0.0,
                    del_epsilon: 0.0,
                    epsilon0: 0.0,
                    epsilon: 0.0,
                    del_tau: 0.0,
                    lamda: 0.0,
                    nu0: 0.0,
                    nu: 0.0,
                    alpha: 0.0,
                    delta: 0.0,
                    h: 0.0,
                    xi: 0.0,
                    del_alpha: 0.0,
                    delta_prime: 0.0,
                    alpha_prime: 0.0,
                    h_prime: 0.0,
                    e0: 0.0,
                    del_e: 0.0,
                    e: 0.0,
                    eot: 0.0,
                    srha: 0.0,
                    ssha: 0.0,
                    sta: 0.0,
                    zenith: 0.0,
                    azimuth_astro: 0.0,
                    azimuth: 0.0,
                    incidence: 0.0,
                    suntransit: 0.0,
                    sunrise: 0.0,
                    sunset: 0.0,
                };
                let their_result = unsafe { spa_sys::spa_calculate(&mut their_inputs) };
                prop_assert_eq!(their_result, 0);
                // Small tolerance for zenith due to cumulative floating point errors
                prop_assert!((our_result.zenith - their_inputs.zenith).abs() < 0.0000001,
                    "zenith mismatch: {} vs {}", our_result.zenith, their_inputs.zenith);
                // Small tolerance for azimuth_astronomical due to cumulative floating point errors
                prop_assert!((our_result.azimuth_astronomical - their_inputs.azimuth_astro).abs() < 0.0000001,
                    "azimuth_astronomical mismatch: {} vs {}", our_result.azimuth_astronomical, their_inputs.azimuth_astro);
                // Small tolerance for azimuth due to cumulative floating point errors
                prop_assert!((our_result.azimuth - their_inputs.azimuth).abs() < 0.0000001,
                    "azimuth mismatch: {} vs {}", our_result.azimuth, their_inputs.azimuth);
                // Small tolerance for incidence due to cumulative floating point errors
                prop_assert!((our_result.incidence - their_inputs.incidence).abs() < 0.0000001,
                    "incidence mismatch: {} vs {}", our_result.incidence, their_inputs.incidence);
            }
        })
    }
    #[allow(clippy::unwrap_used)]
    #[test]
    fn compare_sun_times_to_spa() {
        proptest!(|(
            timestamp in -15768000000i64..=15768000000i64,
            longitude in -180.0..=180.0,
            latitude in -90.0..=90.0,
            elevation in -6500000.0..=6500000.0,
            )| {
                 #[allow(clippy::unwrap_used)]
                let dt = Utc.timestamp_opt(timestamp,0).single().unwrap();
            let our_result = AstronomicalCalculator::standard(
                dt,
                longitude,
                latitude,
                elevation,
            ).map(|inputs| inputs.calculate()).ok();

            if let Some(our_result) = our_result {
                #[allow(clippy::unwrap_used)]
                let mut their_inputs = spa_sys::spa_data {
                    year: dt.year(),
                    month: dt.month() as i32,
                    day: dt.day() as i32,
                    hour: dt.hour() as i32,
                    minute: dt.minute() as i32,
                    second: dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0,
                delta_ut1:    0.0,
                    delta_t:delta_t::estimate_from_date_like(&dt).unwrap(),
                    timezone: 0.0,
                    longitude,
                    latitude,
                    elevation,
                    pressure: 1013.25,
                    temperature: 15.0,
                    slope: 0.0,
                    azm_rotation: 0.0,
                    atmos_refract: 0.5667,
                    function: 3 ,
                    jd: 0.0,
                    jc: 0.0,
                    jde: 0.0,
                    jce: 0.0,
                    jme: 0.0,
                    l: 0.0,
                    b: 0.0,
                    r: 0.0,
                    theta: 0.0,
                    beta: 0.0,
                    x0: 0.0,
                    x1: 0.0,
                    x2: 0.0,
                    x3: 0.0,
                    x4: 0.0,
                    del_psi: 0.0,
                    del_epsilon: 0.0,
                    epsilon0: 0.0,
                    epsilon: 0.0,
                    del_tau: 0.0,
                    lamda: 0.0,
                    nu0: 0.0,
                    nu: 0.0,
                    alpha: 0.0,
                    delta: 0.0,
                    h: 0.0,
                    xi: 0.0,
                    del_alpha: 0.0,
                    delta_prime: 0.0,
                    alpha_prime: 0.0,
                    h_prime: 0.0,
                    e0: 0.0,
                    del_e: 0.0,
                    e: 0.0,
                    eot: 0.0,
                    srha: 0.0,
                    ssha: 0.0,
                    sta: 0.0,
                    zenith: 0.0,
                    azimuth_astro: 0.0,
                    azimuth: 0.0,
                    incidence: 0.0,
                    suntransit: 0.0,
                    sunrise: 0.0,
                    sunset: 0.0,
                };
                let their_result = unsafe { spa_sys::spa_calculate(&mut their_inputs) };
                prop_assert_eq!(their_result, 0);
                if their_inputs.sunrise != 0.0 && (-24.0..=24.0).contains(&their_inputs.sunrise) {
                    let midnight = dt.with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()).unwrap();
                    let their_sunrise = midnight + Duration::milliseconds((their_inputs.sunrise  * 3600.0*1000.0) as i64);
                    let their_sunrise_yesterday = their_sunrise - Duration::days(1);
                    if let Some(our_sunrise) = our_result.sunrise_time {
                        let diff_today = (our_sunrise - their_sunrise).num_seconds().abs();
                        let diff_yesterday = (our_sunrise - their_sunrise_yesterday).num_seconds().abs();
                        let min_diff = diff_today.min(diff_yesterday);
                        prop_assert!(min_diff < 10, "sunrise time mismatch: {} vs {} (today) or {} (yesterday), min diff: {}s", our_sunrise, their_sunrise, their_sunrise_yesterday, min_diff);
                    }
                }
                if their_inputs.sunset != 0.0 && (-24.0..=24.0).contains(&their_inputs.sunset) {
                    let midnight = dt.with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()).unwrap();
                    let their_sunset = midnight + Duration::milliseconds((their_inputs.sunset  * 3600.0*1000.0) as i64);
                    let their_sunset_tomorrow = their_sunset + Duration::days(1);
                    if let Some(our_sunset) = our_result.sunset_time {
                        let diff_today = (our_sunset - their_sunset).num_seconds().abs();
                        let diff_tomorrow = (our_sunset - their_sunset_tomorrow).num_seconds().abs();
                        let min_diff = diff_today.min(diff_tomorrow);
                        prop_assert!(min_diff < 10, "sunset time mismatch: {} vs {} (today) or {} (tomorrow), min diff: {}s", our_sunset, their_sunset, their_sunset_tomorrow, min_diff);
                    }
                }
                if their_inputs.suntransit != 0.0 && (-24.0..=24.0).contains(&their_inputs.suntransit) {
                    let midnight = dt.with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()).unwrap();
                    let their_suntransit = midnight + Duration::milliseconds((their_inputs.suntransit  * 3600.0*1000.0) as i64);
                    let their_suntransit_yesterday = their_suntransit - Duration::days(1);
                    let their_suntransit_tomorrow = their_suntransit + Duration::days(1);
                        let diff_today = (our_result.solar_transit_time - their_suntransit).num_seconds().abs();
                        let diff_yesterday = (our_result.solar_transit_time - their_suntransit_yesterday).num_seconds().abs();
                        let diff_tomorrow = (our_result.solar_transit_time - their_suntransit_tomorrow).num_seconds().abs();
                        let min_diff = diff_today.min(diff_yesterday).min(diff_tomorrow);
                        prop_assert!(min_diff < 10, "suntransit time mismatch: {} vs {} (today), {} (yesterday), or {} (tomorrow), min diff: {}s", our_result.solar_transit_time, their_suntransit, their_suntransit_yesterday, their_suntransit_tomorrow, min_diff);
                }
            }
        })
    }
}
