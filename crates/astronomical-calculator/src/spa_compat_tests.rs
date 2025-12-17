#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod spa_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
extern crate std;
use chrono::DateTime;
use chrono::Datelike;
use chrono::FixedOffset;
use chrono::Timelike;
use proptest::prelude::*;
use proptest_arbitrary_interop::arb;

use crate::AstronomicalCalculator;

fn any_fixed_offset_datetime() -> impl Strategy<Value = DateTime<FixedOffset>> {
    arb::<DateTime<FixedOffset>>().prop_filter("SPA library doesn't accept leap seconds (second >= 60)", |dt| {
        (dt.second() as f64 + dt.nanosecond() as f64 / 1_000_000_000.0) < 60.0
    })
}

proptest! {


    #[test]
    fn spa_calculate_validates_inputs_the_same(
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
    ) {
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
    }
    #[test]
    fn spa_calculate_same_outputs(
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
    ) {
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
        ).map(|inputs| inputs.calculate_solar_position()).ok().flatten();
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
            // Small tolerance for solar_transit_time due to cumulative floating point errors
            prop_assert!((our_result.solar_transit_time - their_inputs.suntransit).abs() < 0.0000001,
                "solar_transit_time mismatch: {} vs {}", our_result.solar_transit_time, their_inputs.suntransit);
            // Small tolerance for sunrise_time due to cumulative floating point errors
            prop_assert!((our_result.sunrise_time - their_inputs.sunrise).abs() < 0.0000001,
                "sunrise_time mismatch: {} vs {}", our_result.sunrise_time, their_inputs.sunrise);
            // Small tolerance for sunset_time due to cumulative floating point errors
            prop_assert!((our_result.sunset_time - their_inputs.sunset).abs() < 0.0000001,
                "sunset_time mismatch: {} vs {}", our_result.sunset_time, their_inputs.sunset);

        }


    }
}
