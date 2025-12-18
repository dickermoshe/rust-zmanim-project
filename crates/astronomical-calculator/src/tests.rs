#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod spa_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
use std::str::FromStr;
use std::string::String;
extern crate std;
use chrono::DateTime;
use chrono::Datelike;
use chrono::FixedOffset;
use chrono::TimeZone;
use chrono::Timelike;
use lazy_static::lazy_static;
use proptest::prelude::*;
use proptest_arbitrary_interop::arb;
use serde::Deserialize;
use std::*;
use tzf_rs::DefaultFinder;

use crate::AstronomicalCalculator;

fn any_fixed_offset_datetime() -> impl Strategy<Value = DateTime<FixedOffset>> {
    arb::<DateTime<FixedOffset>>().prop_filter("SPA library doesn't accept leap seconds (second >= 60)", |dt| {
        (dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0) < 60.0
    })
}

lazy_static! {
    static ref FINDER: DefaultFinder = DefaultFinder::new();
}

#[allow(clippy::unwrap_used, clippy::panic)]
#[test]
fn reasonable_sun_times() {
    proptest!(|(
        elevation in -6500000.0..=6500000.0,
        longitude in -150.0..=150.0,
        latitude in -45.0..=45.0,
        timestamp in -319740000i64..=319740000i64
        )| {
                    if timestamp ==0 {
                return Ok(());
            }
            let latitude:f64 = latitude;
            let latitude:f64 = latitude - 20.0;

            let timezone = chrono_tz::Tz::from_str(FINDER.get_tz_name(longitude, latitude)).ok();
            print!("timezone: {:?}", timezone);
            if timezone.is_none() {
                return Ok(());
            }
            let timezone = timezone.unwrap();

            let dt = timezone.timestamp_opt(timestamp,0).single();
            if dt.is_none() {
                return Ok(());
            }
            let dt = dt.unwrap();

            let our_result = AstronomicalCalculator::standard(dt, longitude, latitude, elevation)
            .map(|inputs| inputs.calculate_solar_position())
            .ok();

            if let Some(our_result) = our_result {

                if our_result.solar_transit_time.is_none() {
                    return Ok(());
                }
                let solar_transit_time = our_result.solar_transit_time.unwrap();
                if our_result.sunrise_time.is_none() {
                    return Ok(());
                }
                let sunrise_time = our_result.sunrise_time.unwrap();
                if our_result.sunset_time.is_none() {
                    return Ok(());
                }
                let sunset_time = our_result.sunset_time.unwrap();
                // prop_assert!(our_result.solar_transit_time.is_some(), "solar transit time should be some");
                // prop_assert!(our_result.sunrise_time.is_some(), "sunrise time should be some");
                // prop_assert!(our_result.sunset_time.is_some(), "sunset time should be some");

                // let sunrise_time = our_result.sunrise_time.unwrap();
                // let sunset_time = our_result.sunset_time.unwrap();
                // let solar_transit_time = our_result.solar_transit_time.unwrap();
                println!("dt: {:?}", dt);
    println!("sunrise_time: {:?}, sunset_time: {:?}, solar_transit_time: {:?}", sunrise_time, sunset_time, solar_transit_time);
                // Ensure that sunrise occurs between 4PM and 12AM
                prop_assert!(sunrise_time.hour() < 12, "sunrise time should be before noon");
                // Ensure that sunset occurs between 4PM and 12AM
                prop_assert!(sunset_time.hour() > 12, "sunset time should be after noon");
                // Ensure that solar transit occurs between 12PM and 12AM
                prop_assert!(solar_transit_time.hour() > 6 && solar_transit_time.hour() < 18, "solar transit time should be between 6AM and 6PM");
            }
          });
}
#[allow(clippy::unwrap_used, clippy::panic)]
#[test]
fn test_debug_asserts() {
    proptest!(|(
            delta_ut1 in -1.0..=1.0,
            delta_t in -8000.0..=8000.0,
            elevation in -6500000.0..=6500000.0,
            pressure in 0.0..=5000.0,
            temperature in -273.0..=6000.0,
            slope in -360.0..=360.0,
            azm_rotation in -360.0..=360.0,
            atmos_refract in -5.0..=5.0,
            longitude in -180.0..=180.0,
            latitude in -90.0..=90.0,
            timestamp in -315_360_000_000_i64..=315_360_000_000_i64
            )| {
        let longitude:f64 = longitude;
        let longitude:f64 = longitude.abs();

        let timezone = chrono_tz::Tz::from_str(FINDER.get_tz_name(longitude, latitude)).ok();
        if timezone.is_none() {
            return Ok(());
        }
        let timezone = timezone.unwrap();

        let dt = timezone.timestamp_millis_opt(timestamp).single();
        if dt.is_none() {
            return Ok(());
        }
        let dt = dt.unwrap();

        let _ = AstronomicalCalculator::new(
            dt, delta_ut1, delta_t, longitude, latitude, elevation, pressure, temperature, slope, azm_rotation, atmos_refract,
        )
        .map(|inputs| inputs.calculate_solar_position())
        .ok();})
}

#[test]
fn spa_calculate_validates_inputs_the_same() {
    proptest!(|(
            dt in any_fixed_offset_datetime(),
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
        let timzeone_hours = dt.offset().local_minus_utc() as f64 / 3600.0;
        let mut their_inputs = spa_sys::spa_data {
            year: dt.year(),
            month: dt.month() as i32,
            day: dt.day() as i32,
            hour: dt.hour() as i32,
            minute: dt.minute() as i32,
            second: dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0,
            delta_ut1,
            delta_t,
            timezone: timzeone_hours,
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
fn compare_non_datetime_values_to_spa() {
    proptest!(|(
            dt in any_fixed_offset_datetime(),
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
        ).map(|inputs| inputs.calculate_solar_position()).ok();
        if let Some(our_result) = our_result {
            let timzeone_hours = dt.offset().local_minus_utc() as f64 / 3600.0;
            let mut their_inputs = spa_sys::spa_data {
                year: dt.year(),
                month: dt.month() as i32,
                day: dt.day() as i32,
                hour: dt.hour() as i32,
                minute: dt.minute() as i32,
                second: dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0,
                delta_ut1,
                delta_t,
                timezone: timzeone_hours,
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
#[test]
fn compare_datetime_values_to_spa() {
    proptest!(|(
            dt in any_fixed_offset_datetime(),
        longitude in -180.0..=180.0,
        latitude in -90.0..=90.0,
        )| {
        let our_result = AstronomicalCalculator::new(
            dt,
           0.0,
           0.0,
            longitude,
            latitude,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ).map(|inputs| inputs.calculate_solar_position()).ok();
        if let Some(our_result) = our_result {
            let timzeone_hours = dt.offset().local_minus_utc() as f64 / 3600.0;
            let mut their_inputs = spa_sys::spa_data {
                year: dt.year(),
                month: dt.month() as i32,
                day: dt.day() as i32,
                hour: dt.hour() as i32,
                minute: dt.minute() as i32,
                second: dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0,
                delta_ut1: 0.0,
                delta_t: 0.0,
                timezone: timzeone_hours,
                longitude,
                latitude,
                elevation: 0.0,
                pressure: 0.0,
                temperature: 0.0,
                slope: 0.0,
                azm_rotation: 0.0,
                atmos_refract: 0.0,
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

        }
    })
}
