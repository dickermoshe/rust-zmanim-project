#![allow(clippy::unwrap_used, clippy::panic)]
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::Utc;
use proptest::proptest;
extern crate std;
use chrono::Datelike;
use chrono::Timelike;
use proptest::prelude::*;
use std::*;

use crate::spa2::SpaCalculator;
use crate::unsafe_spa::tm;
use crate::unsafe_spa::ApSolposBennet;
use crate::unsafe_spa::ApSolposBennetNA;
use crate::unsafe_spa::TrueSolarTime;
use crate::unsafe_spa::SPA;

fn naive_datetime_to_tm(dt: &NaiveDateTime) -> tm {
    tm {
        tm_sec: dt.second() as i32,
        tm_min: dt.minute() as i32,
        tm_hour: dt.hour() as i32,
        tm_mday: dt.day() as i32,
        tm_mon: dt.month0() as i32,
        tm_year: dt.year() - 1900,
        tm_wday: dt.weekday().num_days_from_sunday() as i32,
        tm_yday: dt.ordinal0() as i32,
        tm_isdst: -1,
        tm_gmtoff: 0,
        tm_zone: core::ptr::null(),
    }
}

fn tm_to_naive_datetime(tm: &tm) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(tm.tm_year + 1900, tm.tm_mon as u32 + 1, tm.tm_mday as u32)
        .and_then(|d| d.and_hms_opt(tm.tm_hour as u32, tm.tm_min as u32, tm.tm_sec as u32))
        .unwrap()
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
proptest! {
    #[test]
    fn solar_position_tests(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -90.0_f64..=90.0_f64,
        elevation in -1000.0..=5000.0,
        delta_ut1 in -1.0..1.0,
    ) {
        let dt = datetime.naive_utc();


        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Call safe spa2 function
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation).unwrap();
        let safe_result = calculator.get_solar_position();

        // Prepare parameters for unsafe SPA function
        let mut ut = naive_datetime_to_tm(&dt);


        // Call unsafe SPA function
        let unsafe_result = unsafe {
            SPA(
                &mut ut as *mut tm,
                core::ptr::null_mut(), // Pass null pointer for delta_t
                delta_ut1,
                lon_rad,
                lat_rad,
                elevation,
            )
        };

        // Compare results
        // Note: Allow for floating-point rounding differences between implementations
        // The SPA algorithm involves many trigonometric calculations that accumulate small errors.
        // Observed differences range from 1e-6 to 5e-4 radians across random test cases.
        // 0.0005 radians ≈ 0.029 degrees ≈ 1.7 arcminutes (acceptable for most applications)
        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Zenith mismatch: unsafe={}, safe={}, diff={}",
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
        prop_assert!(
            (unsafe_result.a - safe_result.azimuth).abs() < epsilon,
            "Azimuth mismatch: unsafe={}, safe={}, diff={}",
            unsafe_result.a,
            safe_result.azimuth,
            (unsafe_result.a - safe_result.azimuth).abs()
        );
        prop_assert_eq!(
            unsafe_result.E, 0,
            "Unsafe SPA returned error code: {}",
            unsafe_result.E
        );
    }

    #[test]
    fn solar_time_tests(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -90.0_f64..=90.0_f64,
    ) {
        let dt = datetime.naive_utc();

        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Call safe spa2 function
        let mut calculator = SpaCalculator::new(dt, None, 0.0, lon_rad, lat_rad, 0.0).unwrap();
        let safe_result = calculator.get_solar_time();

        // Prepare parameters for unsafe TrueSolarTime function
        let mut ut = naive_datetime_to_tm(&dt);

        // Call unsafe TrueSolarTime function
        let unsafe_result_tm = unsafe {
            TrueSolarTime(
                &mut ut as *mut tm,
                core::ptr::null_mut(), // Pass null pointer for delta_t
                0.0,
                lon_rad,
                lat_rad,
            )
        };

        // Compare results
        if let Ok(safe_time) = safe_result {
            let unsafe_time = tm_to_naive_datetime(&unsafe_result_tm);

            // Allow up to 1 second difference due to rounding
            let time_diff = (safe_time.and_utc().timestamp() - unsafe_time.and_utc().timestamp()).abs();
            prop_assert!(
                time_diff <= 1,
                "Solar time mismatch: unsafe={}, safe={}, diff={} seconds",
                unsafe_time,
                safe_time,
                time_diff
            );
        }else{
            prop_assert!(false, "Failed to calculate solar time");
        }
    }

    #[test]
    fn atmospheric_refraction_bennet_tests(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -90.0_f64..=90.0_f64,
        elevation in -1000.0..=5000.0,
        pressure in 0.1..=2000.0,
        temperature in -273.15..=50.0,
        delta_ut1 in -1.0..1.0,
        use_explicit_gdip in proptest::bool::ANY,
        gdip in -1.5..=1.5,
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Randomly decide whether to use explicit gdip or None
        let gdip_option = if use_explicit_gdip { Some(gdip) } else { None };

        // Get solar position from safe implementation
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation).unwrap();
        let safe_pos = calculator.get_solar_position();

        // Apply Bennet refraction correction (safe)
        let safe_corrected = safe_pos.ApSolposBennet(gdip_option, elevation, pressure, temperature);

        // Get solar position from unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_pos = unsafe {
            SPA(
                &mut ut as *mut tm,
                core::ptr::null_mut(),
                delta_ut1,
                lon_rad,
                lat_rad,
                elevation,
            )
        };

        // Apply Bennet refraction correction (unsafe)
        let unsafe_corrected = unsafe {
            if let Some(gdip_val) = gdip_option {
                let mut gdip_mut = gdip_val;
                ApSolposBennet(
                    unsafe_pos,
                    &mut gdip_mut as *mut f64,
                    elevation,
                    pressure,
                    temperature,
                )
            } else {
                ApSolposBennet(
                    unsafe_pos,
                    core::ptr::null_mut(),
                    elevation,
                    pressure,
                    temperature,
                )
            }
        };

        // Compare results - both should succeed or both should fail
        match safe_corrected {
            Ok(safe_result) => {
                prop_assert_eq!(
                    unsafe_corrected.E, 0,
                    "Safe version succeeded but unsafe returned error code: {} (gdip={:?})",
                    unsafe_corrected.E,
                    gdip_option
                );

                let epsilon = 5e-4; // Same tolerance as solar position tests
                prop_assert!(
                    (unsafe_corrected.z - safe_result.zenith).abs() < epsilon,
                    "Bennet zenith mismatch (gdip={:?}): unsafe={}, safe={}, diff={}",
                    gdip_option,
                    unsafe_corrected.z,
                    safe_result.zenith,
                    (unsafe_corrected.z - safe_result.zenith).abs()
                );
                prop_assert!(
                    (unsafe_corrected.a - safe_result.azimuth).abs() < epsilon,
                    "Bennet azimuth mismatch (gdip={:?}): unsafe={}, safe={}, diff={}",
                    gdip_option,
                    unsafe_corrected.a,
                    safe_result.azimuth,
                    (unsafe_corrected.a - safe_result.azimuth).abs()
                );
            }
            Err(_) => {
                // Safe version failed, unsafe should also indicate error
                prop_assert!(
                    unsafe_corrected.E != 0,
                    "Safe version failed but unsafe succeeded (error code: {}, gdip={:?})",
                    unsafe_corrected.E,
                    gdip_option
                );
            }
        }
    }

    #[test]
    fn atmospheric_refraction_bennet_na_tests(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -90.0_f64..=90.0_f64,
        elevation in -1000.0..=5000.0,
        pressure in 0.1..=2000.0,
        temperature in -273.15..=50.0,
        delta_ut1 in -1.0..1.0,
        use_explicit_gdip in proptest::bool::ANY,
        gdip in -1.5..=1.5,
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Randomly decide whether to use explicit gdip or None
        let gdip_option = if use_explicit_gdip { Some(gdip) } else { None };

        // Get solar position from safe implementation
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation).unwrap();
        let safe_pos = calculator.get_solar_position();

        // Apply BennetNA refraction correction (safe)
        let safe_corrected = safe_pos.ApSolposBennetNA(gdip_option, elevation, pressure, temperature);

        // Get solar position from unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_pos = unsafe {
            SPA(
                &mut ut as *mut tm,
                core::ptr::null_mut(),
                delta_ut1,
                lon_rad,
                lat_rad,
                elevation,
            )
        };

        // Apply BennetNA refraction correction (unsafe)
        let unsafe_corrected = unsafe {
            if let Some(gdip_val) = gdip_option {
                let mut gdip_mut = gdip_val;
                ApSolposBennetNA(
                    unsafe_pos,
                    &mut gdip_mut as *mut f64,
                    elevation,
                    pressure,
                    temperature,
                )
            } else {
                ApSolposBennetNA(
                    unsafe_pos,
                    core::ptr::null_mut(),
                    elevation,
                    pressure,
                    temperature,
                )
            }
        };

        // Compare results - both should succeed or both should fail
        match safe_corrected {
            Ok(safe_result) => {
                prop_assert_eq!(
                    unsafe_corrected.E, 0,
                    "Safe version succeeded but unsafe returned error code: {} (gdip={:?})",
                    unsafe_corrected.E,
                    gdip_option
                );

                let epsilon = 5e-4; // Same tolerance as solar position tests
                prop_assert!(
                    (unsafe_corrected.z - safe_result.zenith).abs() < epsilon,
                    "BennetNA zenith mismatch (gdip={:?}): unsafe={}, safe={}, diff={}",
                    gdip_option,
                    unsafe_corrected.z,
                    safe_result.zenith,
                    (unsafe_corrected.z - safe_result.zenith).abs()
                );
                prop_assert!(
                    (unsafe_corrected.a - safe_result.azimuth).abs() < epsilon,
                    "BennetNA azimuth mismatch (gdip={:?}): unsafe={}, safe={}, diff={}",
                    gdip_option,
                    unsafe_corrected.a,
                    safe_result.azimuth,
                    (unsafe_corrected.a - safe_result.azimuth).abs()
                );
            }
            Err(_) => {
                // Safe version failed, unsafe should also indicate error
                prop_assert!(
                    unsafe_corrected.E != 0,
                    "Safe version failed but unsafe succeeded (error code: {}, gdip={:?})",
                    unsafe_corrected.E,
                    gdip_option
                );
            }
        }
    }
}
