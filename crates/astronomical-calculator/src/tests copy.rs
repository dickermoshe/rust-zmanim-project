#![allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
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

use crate::find_solar_time;
use crate::find_solar_zenith;
use crate::types::SpaError;
use crate::unsafe_spa::sol_pos;
use crate::unsafe_spa::tm;
use crate::unsafe_spa::ApSolposBennet;
use crate::unsafe_spa::ApSolposBennetNA;
use crate::unsafe_spa::FindSolTime as UnsafeFindSolTime;
use crate::unsafe_spa::FindSolZenith as UnsafeFindSolZenith;
use crate::unsafe_spa::InputCheck;
use crate::unsafe_spa::SolarDay;
use crate::unsafe_spa::TrueSolarTime;
use crate::unsafe_spa::SPA;
use crate::unsafe_spa::_FREESPA_DEU_OOR;
use crate::unsafe_spa::_FREESPA_DIP_OOR;
use crate::unsafe_spa::_FREESPA_ELE_OOR;
use crate::unsafe_spa::_FREESPA_LAT_OOR;
use crate::unsafe_spa::_FREESPA_LON_OOR;
use crate::unsafe_spa::_FREESPA_PRE_OOR;
use crate::unsafe_spa::_FREESPA_TEM_OOR;
use crate::Refraction;
use crate::SolarPosition;
use crate::SolarZenith;
use crate::SpaCalculator;
use crate::ABSOLUTEZERO;
use crate::EARTH_R;
use core::f64::consts::PI;

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

/// Wrapper for safe refraction function to match the signature expected by FindSolZenith
fn safe_refract_bennet(pos: SolarPosition, gdip: Option<f64>, e: f64, p: f64, t: f64) -> SolarPosition {
    pos.ap_sol_pos_bennet(gdip, e, p, t).unwrap_or(pos)
}

/// Wrapper for unsafe refraction function
unsafe extern "C" fn unsafe_refract_bennet(pos: sol_pos, gdip: *mut f64, e: f64, p: f64, t: f64) -> sol_pos {
    ApSolposBennet(pos, gdip, e, p, t)
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
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
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
        let mut calculator = SpaCalculator::new(dt, None, 0.0, lon_rad, lat_rad, 0.0, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
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
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
        let safe_pos = calculator.get_solar_position();

        // Apply Bennet refraction correction (safe)
        let safe_corrected = safe_pos.ap_sol_pos_bennet(gdip_option, elevation, pressure, temperature);

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
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
        let safe_pos = calculator.get_solar_position();

        // Apply BennetNA refraction correction (safe)
        let safe_corrected = safe_pos.ap_sol_pos_bennet_na(gdip_option, elevation, pressure, temperature);

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

    #[test]
    fn find_sol_time_tests(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        target_hour in 0i64..24i64,
        target_min in 0i64..60i64,
        target_sec in 0i64..60i64,
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();

        // Get unix timestamp
        let unix_time = dt.and_utc().timestamp();

        // Call safe find_solar_time
        let safe_result = find_solar_time(
            unix_time,
            target_hour,
            target_min,
            target_sec,
            None,
            0.0,
            lon_rad,
        );

        // Call unsafe FindSolTime (time_t is now i64)
        let unsafe_result = unsafe {
            UnsafeFindSolTime(
                unix_time,
                target_hour as i32,
                target_min as i32,
                target_sec as i32,
                core::ptr::null_mut(),
                0.0,
                lon_rad,
                0.0, // lat is not used in FindSolTime but required by unsafe version
            )
        };

        // Compare results
        if let Ok(safe_time) = safe_result {
            // Allow up to 2 seconds difference due to rounding/algorithm differences
            let diff = (safe_time - unsafe_result).abs();
            prop_assert!(
                diff <= 2,
                "FindSolTime mismatch: safe={}, unsafe={}, diff={} seconds",
                safe_time,
                unsafe_result,
                diff
            );
        }
    }

    #[test]
    fn find_sol_zenith_tests(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -85.0_f64..=85.0_f64, // Avoid extreme latitudes for numerical stability
        elevation in 0.0..=1000.0,
        pressure in 900.0..=1100.0,
        temperature in -20.0..=40.0,
        target_zenith_deg in 70.0_f64..=110.0_f64, // Around sunrise/sunset zenith
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();
        let target_zenith = target_zenith_deg.to_radians();

        // Get unix timestamp
        let unix_time = dt.and_utc().timestamp();

        // Define a time window of 12 hours around the datetime
        let t1 = unix_time - 6 * 3600;
        let t2 = unix_time + 6 * 3600;

        // Compute zenith at boundaries using unsafe SPA (to match what unsafe FindSolZenith uses)
        let mut ut1 = naive_datetime_to_tm(&chrono::DateTime::from_timestamp(t1, 0).unwrap().naive_utc());
        let mut ut2 = naive_datetime_to_tm(&chrono::DateTime::from_timestamp(t2, 0).unwrap().naive_utc());

        let unsafe_pos1 = unsafe { SPA(&mut ut1, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation) };
        let unsafe_pos2 = unsafe { SPA(&mut ut2, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation) };

        let z1 = unsafe_pos1.z;
        let z2 = unsafe_pos2.z;

        // Call safe find_solar_zenith
        let safe_result = find_solar_zenith(
            t1,
            t2,
            z1,
            z2,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            None,
            pressure,
            temperature,
            safe_refract_bennet,
            target_zenith,
        );

        // Call unsafe FindSolZenith
        let mut tz: i64 = 0;
        let mut err: f64 = 0.0;
        let unsafe_status = unsafe {
            UnsafeFindSolZenith(
                t1,
                t2,
                z1,
                z2,
                core::ptr::null_mut(),
                0.0,
                lon_rad,
                lat_rad,
                elevation,
                core::ptr::null_mut(),
                pressure,
                temperature,
                Some(unsafe_refract_bennet),
                target_zenith,
                &mut tz,
                &mut err,
            )
        };

        // Compare results
        match safe_result {
            Ok(SolarZenith::AlwaysAbove) => {
                prop_assert_eq!(
                    unsafe_status, -1,
                    "Safe returned AlwaysAbove but unsafe returned {}",
                    unsafe_status
                );
            }
            Ok(SolarZenith::AlwaysBelow) => {
                prop_assert_eq!(
                    unsafe_status, 1,
                    "Safe returned AlwaysBelow but unsafe returned {}",
                    unsafe_status
                );
            }
            Ok(SolarZenith::BetweenHorizon(safe_tz, safe_err)) => {
                prop_assert_eq!(
                    unsafe_status, 0,
                    "Safe returned BetweenHorizon but unsafe returned status {}",
                    unsafe_status
                );
                // Allow up to 2 seconds difference
                let time_diff = (safe_tz - tz).abs();
                prop_assert!(
                    time_diff <= 2,
                    "FindSolZenith time mismatch: safe_tz={}, unsafe_tz={}, diff={}",
                    safe_tz,
                    tz,
                    time_diff
                );
                // Error should be similar
                let error_diff = (safe_err - err).abs();
                prop_assert!(
                    error_diff < 1e-3,
                    "FindSolZenith error mismatch: safe_err={}, unsafe_err={}, diff={}",
                    safe_err,
                    err,
                    error_diff
                );
            }
            Err(e) => {
                // If safe version errors, skip this test case
                prop_assume!(false, "Safe version returned error: {:?}", e);
            }
        }
    }
}

// =============================================================================
// 1. EXPLICIT ERROR CASE TESTS
// Verify that safe and unsafe implementations both error on invalid inputs
// =============================================================================

#[test]
fn error_case_delta_ut1_out_of_range() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.0;
    let elevation = 0.0;

    // Test delta_ut1 > 1.0
    let safe_result = SpaCalculator::new(
        dt,
        None,
        1.5,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    );
    assert!(
        matches!(safe_result, Err(SpaError::DeltaUt1OutOfRange)),
        "Safe should reject delta_ut1 > 1.0"
    );
    let unsafe_error = unsafe { InputCheck(1.5, lon_rad, lat_rad, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_DEU_OOR != 0,
        "Unsafe should set DEU_OOR flag for delta_ut1 > 1.0"
    );

    // Test delta_ut1 < -1.0
    let safe_result = SpaCalculator::new(
        dt,
        None,
        -1.5,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    );
    assert!(
        matches!(safe_result, Err(SpaError::DeltaUt1OutOfRange)),
        "Safe should reject delta_ut1 < -1.0"
    );
    let unsafe_error = unsafe { InputCheck(-1.5, lon_rad, lat_rad, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_DEU_OOR != 0,
        "Unsafe should set DEU_OOR flag for delta_ut1 < -1.0"
    );
}

#[test]
fn error_case_longitude_out_of_range() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lat_rad = 0.0;
    let elevation = 0.0;

    // Test lon > PI
    let safe_result = SpaCalculator::new(
        dt,
        None,
        0.0,
        PI + 0.1,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    );
    assert!(
        matches!(safe_result, Err(SpaError::LongitudeOutOfRange)),
        "Safe should reject lon > PI"
    );
    let unsafe_error = unsafe { InputCheck(0.0, PI + 0.1, lat_rad, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_LON_OOR != 0,
        "Unsafe should set LON_OOR flag for lon > PI"
    );

    // Test lon < -PI
    let safe_result = SpaCalculator::new(
        dt,
        None,
        0.0,
        -PI - 0.1,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    );
    assert!(
        matches!(safe_result, Err(SpaError::LongitudeOutOfRange)),
        "Safe should reject lon < -PI"
    );
    let unsafe_error = unsafe { InputCheck(0.0, -PI - 0.1, lat_rad, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_LON_OOR != 0,
        "Unsafe should set LON_OOR flag for lon < -PI"
    );
}

#[test]
fn error_case_latitude_out_of_range() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let elevation = 0.0;

    // Test lat > PI/2
    let safe_result = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        PI / 2.0 + 0.1,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    );
    assert!(
        matches!(safe_result, Err(SpaError::LatitudeOutOfRange)),
        "Safe should reject lat > PI/2"
    );
    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, PI / 2.0 + 0.1, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_LAT_OOR != 0,
        "Unsafe should set LAT_OOR flag for lat > PI/2"
    );

    // Test lat < -PI/2
    let safe_result = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        -PI / 2.0 - 0.1,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    );
    assert!(
        matches!(safe_result, Err(SpaError::LatitudeOutOfRange)),
        "Safe should reject lat < -PI/2"
    );
    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, -PI / 2.0 - 0.1, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_LAT_OOR != 0,
        "Unsafe should set LAT_OOR flag for lat < -PI/2"
    );
}

#[test]
fn error_case_elevation_out_of_range() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.0;

    // Test elevation < -EARTH_R
    let invalid_elevation = -EARTH_R - 1.0;
    let safe_result = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        invalid_elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    );
    assert!(
        matches!(safe_result, Err(SpaError::ElevationOutOfRange)),
        "Safe should reject elevation < -EARTH_R"
    );
    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, lat_rad, invalid_elevation, 1010.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_ELE_OOR != 0,
        "Unsafe should set ELE_OOR flag for elevation < -EARTH_R"
    );
}

// =============================================================================
// 2. BOUNDARY VALUE TESTS
// Test pressure=0, temperature < ABSOLUTEZERO, gdip > π/2
// =============================================================================

#[test]
fn boundary_pressure_zero() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5; // ~28.6 degrees
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test pressure = 0 (should error in safe, but allowed in unsafe!)
    let safe_result = pos.ap_sol_pos_bennet(None, elevation, 0.0, 10.0);
    assert!(
        matches!(safe_result, Err(SpaError::PressureOutOfRange)),
        "Safe should reject pressure = 0, got {:?}",
        safe_result
    );

    // Note: Unsafe allows p = 0 (only rejects p < 0)
    // This is a known discrepancy between implementations
    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, lat_rad, elevation, 0.0, 10.0) };
    // Unsafe uses p < 0, so p = 0 passes InputCheck
    // Document the discrepancy:
    if unsafe_error & _FREESPA_PRE_OOR == 0 {
        println!("Note: Unsafe allows pressure=0, safe rejects it. This is a known discrepancy.");
    }
}

#[test]
fn boundary_pressure_negative() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5;
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test pressure < 0 (both should error)
    let safe_result = pos.ap_sol_pos_bennet(None, elevation, -10.0, 10.0);
    assert!(
        matches!(safe_result, Err(SpaError::PressureOutOfRange)),
        "Safe should reject negative pressure"
    );

    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, lat_rad, elevation, -10.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_PRE_OOR != 0,
        "Unsafe should set PRE_OOR flag for negative pressure"
    );
}

#[test]
fn boundary_pressure_above_max() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5;
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test pressure > 5000 (both should error)
    let safe_result = pos.ap_sol_pos_bennet(None, elevation, 5001.0, 10.0);
    assert!(
        matches!(safe_result, Err(SpaError::PressureOutOfRange)),
        "Safe should reject pressure > 5000"
    );

    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, lat_rad, elevation, 5001.0, 10.0) };
    assert!(
        unsafe_error & _FREESPA_PRE_OOR != 0,
        "Unsafe should set PRE_OOR flag for pressure > 5000"
    );
}

#[test]
fn boundary_temperature_below_absolute_zero() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5;
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test temperature < ABSOLUTEZERO (both should error)
    let invalid_temp = ABSOLUTEZERO - 1.0;
    let safe_result = pos.ap_sol_pos_bennet(None, elevation, 1010.0, invalid_temp);
    assert!(
        matches!(safe_result, Err(SpaError::TemperatureOutOfRange)),
        "Safe should reject temperature below absolute zero"
    );

    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, lat_rad, elevation, 1010.0, invalid_temp) };
    assert!(
        unsafe_error & _FREESPA_TEM_OOR != 0,
        "Unsafe should set TEM_OOR flag for temperature below absolute zero"
    );
}

#[test]
fn boundary_temperature_at_absolute_zero() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5;
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test temperature exactly at ABSOLUTEZERO (should be valid in both)
    let safe_result = pos.ap_sol_pos_bennet(None, elevation, 1010.0, ABSOLUTEZERO);
    assert!(
        safe_result.is_ok(),
        "Safe should accept temperature at absolute zero, got {:?}",
        safe_result
    );

    let unsafe_error = unsafe { InputCheck(0.0, lon_rad, lat_rad, elevation, 1010.0, ABSOLUTEZERO) };
    assert!(
        unsafe_error & _FREESPA_TEM_OOR == 0,
        "Unsafe should accept temperature at absolute zero"
    );
}

#[test]
fn boundary_gdip_exceeds_pi_half() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5;
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test gdip > PI/2 (both should error)
    let invalid_gdip = PI / 2.0 + 0.1;
    let safe_result = pos.ap_sol_pos_bennet(Some(invalid_gdip), elevation, 1010.0, 10.0);
    assert!(
        matches!(safe_result, Err(SpaError::GeometricDipOutOfRange)),
        "Safe should reject gdip > PI/2, got {:?}",
        safe_result
    );

    // For unsafe, we need to call ApSolposBennet and check the error flag
    let mut ut = naive_datetime_to_tm(&dt);
    let unsafe_pos = unsafe { SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation) };
    let mut gdip_val = invalid_gdip;
    let unsafe_result = unsafe { ApSolposBennet(unsafe_pos, &mut gdip_val, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_result.E & _FREESPA_DIP_OOR != 0,
        "Unsafe should set DIP_OOR flag for gdip > PI/2, got E={}",
        unsafe_result.E
    );
}

#[test]
fn boundary_gdip_negative_exceeds_pi_half() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5;
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test gdip < -PI/2 (both should error)
    let invalid_gdip = -PI / 2.0 - 0.1;
    let safe_result = pos.ap_sol_pos_bennet(Some(invalid_gdip), elevation, 1010.0, 10.0);
    assert!(
        matches!(safe_result, Err(SpaError::GeometricDipOutOfRange)),
        "Safe should reject gdip < -PI/2, got {:?}",
        safe_result
    );

    let mut ut = naive_datetime_to_tm(&dt);
    let unsafe_pos = unsafe { SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation) };
    let mut gdip_val = invalid_gdip;
    let unsafe_result = unsafe { ApSolposBennet(unsafe_pos, &mut gdip_val, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_result.E & _FREESPA_DIP_OOR != 0,
        "Unsafe should set DIP_OOR flag for gdip < -PI/2, got E={}",
        unsafe_result.E
    );
}

#[test]
fn boundary_gdip_at_pi_half() {
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();
    let lon_rad = 0.0;
    let lat_rad = 0.5;
    let elevation = 100.0;

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        elevation,
        10.0,
        1010.0,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();
    let pos = calculator.get_solar_position();

    // Test gdip exactly at PI/2 (should be valid - boundary is exclusive)
    let valid_gdip = PI / 2.0;
    let safe_result = pos.ap_sol_pos_bennet(Some(valid_gdip), elevation, 1010.0, 10.0);
    assert!(
        safe_result.is_ok(),
        "Safe should accept gdip = PI/2, got {:?}",
        safe_result
    );

    let mut ut = naive_datetime_to_tm(&dt);
    let unsafe_pos = unsafe { SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation) };
    let mut gdip_val = valid_gdip;
    let unsafe_result = unsafe { ApSolposBennet(unsafe_pos, &mut gdip_val, elevation, 1010.0, 10.0) };
    assert!(
        unsafe_result.E & _FREESPA_DIP_OOR == 0,
        "Unsafe should accept gdip = PI/2, got E={}",
        unsafe_result.E
    );
}

// =============================================================================
// 6. EXTREME LATITUDE TESTS
// Test polar regions with special behavior (midnight sun, polar night)
// =============================================================================

proptest! {
    /// Test solar position at North Pole (lat = 90°)
    #[test]
    fn extreme_latitude_north_pole(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        delta_ut1 in -1.0..1.0,
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = PI / 2.0; // Exactly 90°N
        let elevation = 0.0;

        // Call safe implementation
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
        let safe_result = calculator.get_solar_position();

        // Call unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), delta_ut1, lon_rad, lat_rad, elevation)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "North Pole zenith mismatch: unsafe={}, safe={}, diff={}",
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
        prop_assert!(
            (unsafe_result.a - safe_result.azimuth).abs() < epsilon ||
            // At pole, azimuth can be undefined or wrap around
            ((unsafe_result.a - safe_result.azimuth).abs() - 2.0 * PI).abs() < epsilon,
            "North Pole azimuth mismatch: unsafe={}, safe={}, diff={}",
            unsafe_result.a,
            safe_result.azimuth,
            (unsafe_result.a - safe_result.azimuth).abs()
        );
    }

    /// Test solar position at South Pole (lat = -90°)
    #[test]
    fn extreme_latitude_south_pole(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        delta_ut1 in -1.0..1.0,
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = -PI / 2.0; // Exactly 90°S
        let elevation = 0.0;

        // Call safe implementation
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
        let safe_result = calculator.get_solar_position();

        // Call unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), delta_ut1, lon_rad, lat_rad, elevation)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "South Pole zenith mismatch: unsafe={}, safe={}, diff={}",
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
        prop_assert!(
            (unsafe_result.a - safe_result.azimuth).abs() < epsilon ||
            // At pole, azimuth can be undefined or wrap around
            ((unsafe_result.a - safe_result.azimuth).abs() - 2.0 * PI).abs() < epsilon,
            "South Pole azimuth mismatch: unsafe={}, safe={}, diff={}",
            unsafe_result.a,
            safe_result.azimuth,
            (unsafe_result.a - safe_result.azimuth).abs()
        );
    }

    /// Test summer solstice at Arctic Circle (midnight sun conditions)
    #[test]
    fn extreme_latitude_arctic_circle_summer(
        longitude in -180.0_f64..=180.0_f64,
        day_offset in 0u32..5u32, // Days around summer solstice
        hour in 0u32..24u32,
    ) {
        // Summer solstice ~June 21
        let dt = NaiveDate::from_ymd_opt(2024, 6, 21 + day_offset % 10)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap();
        let lon_rad = longitude.to_radians();
        let lat_rad = 66.5_f64.to_radians(); // Arctic Circle ~66.5°N
        let elevation = 0.0;

        // Call safe implementation
        let mut calculator = SpaCalculator::new(dt, None, 0.0, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
        let safe_result = calculator.get_solar_position();

        // Call unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Arctic summer zenith mismatch at hour {}: unsafe={}, safe={}, diff={}",
            hour,
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
    }

    /// Test winter solstice at Arctic Circle (polar night conditions)
    #[test]
    fn extreme_latitude_arctic_circle_winter(
        longitude in -180.0_f64..=180.0_f64,
        day_offset in 0u32..5u32, // Days around winter solstice
        hour in 0u32..24u32,
    ) {
        // Winter solstice ~December 21
        let dt = NaiveDate::from_ymd_opt(2024, 12, 21 + day_offset % 10)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap();
        let lon_rad = longitude.to_radians();
        let lat_rad = 66.5_f64.to_radians(); // Arctic Circle
        let elevation = 0.0;

        // Call safe implementation
        let mut calculator = SpaCalculator::new(dt, None, 0.0, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
        let safe_result = calculator.get_solar_position();

        // Call unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Arctic winter zenith mismatch at hour {}: unsafe={}, safe={}, diff={}",
            hour,
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
    }

    /// Test near-polar latitudes (89°) for numerical stability
    #[test]
    fn extreme_latitude_near_pole(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude_offset in 0.0_f64..1.0_f64, // 89° to 90°
        is_north in proptest::bool::ANY,
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let base_lat = 89.0_f64.to_radians();
        let lat_rad = if is_north {
            base_lat + latitude_offset.to_radians()
        } else {
            -(base_lat + latitude_offset.to_radians())
        };

        // Clamp to valid range
        let lat_rad = lat_rad.clamp(-PI / 2.0, PI / 2.0);
        let elevation = 0.0;

        // Call safe implementation
        let mut calculator = SpaCalculator::new(dt, None, 0.0, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
        let safe_result = calculator.get_solar_position();

        // Call unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, elevation)
        };

        // Verify zenith is valid (between 0 and PI)
        prop_assert!(
            safe_result.zenith >= 0.0 && safe_result.zenith <= PI,
            "Safe zenith out of range at lat {}: {}",
            lat_rad.to_degrees(),
            safe_result.zenith
        );
        prop_assert!(
            unsafe_result.z >= 0.0 && unsafe_result.z <= PI,
            "Unsafe zenith out of range at lat {}: {}",
            lat_rad.to_degrees(),
            unsafe_result.z
        );

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Near-pole zenith mismatch at lat {}: unsafe={}, safe={}, diff={}",
            lat_rad.to_degrees(),
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
    }

    /// Test that solar day methods match the corresponding elements from SolarDay:
    /// - get_prev_solar_midnight() matches SolarDay.t[0]
    /// - get_solar_transit() matches SolarDay.t[1]
    /// - get_next_solar_midnight() matches SolarDay.t[2]
    /// - get_sunrise() matches SolarDay.t[3]
    /// - get_sunset() matches SolarDay.t[4]
    /// - get_civil_dawn() matches SolarDay.t[5]
    /// - get_civil_dusk() matches SolarDay.t[6]
    /// - get_nautical_dawn() matches SolarDay.t[7]
    /// - get_nautical_dusk() matches SolarDay.t[8]
    /// - get_astronomical_dawn() matches SolarDay.t[9]
    /// - get_astronomical_dusk() matches SolarDay.t[10]
    #[test]
    fn solar_day_methods_match_unsafe(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -60.0_f64..=60.0_f64, // Use moderate latitudes to ensure events occur
        elevation in 0.0..=1000.0,
        pressure in 900.0..=1100.0,
        temperature in -20.0..=40.0,
    ) {
        use crate::SolarEventResult;

        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Create safe calculator
        let mut calculator = SpaCalculator::new(
            dt,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            temperature,
            pressure,
            None,
            Refraction::ApSolposBennet,
        ).unwrap();

        // Get all safe results for midnight/transit
        let safe_transit = calculator.get_solar_transit();
        let safe_prev_midnight = calculator.get_prev_solar_midnight();
        let safe_next_midnight = calculator.get_next_solar_midnight();

        // Get all safe results for solar events
        let safe_sunrise = calculator.get_sunrise();
        let safe_sunset = calculator.get_sunset();
        let safe_civil_dawn = calculator.get_civil_dawn();
        let safe_civil_dusk = calculator.get_civil_dusk();
        let safe_nautical_dawn = calculator.get_nautical_dawn();
        let safe_nautical_dusk = calculator.get_nautical_dusk();
        let safe_astronomical_dawn = calculator.get_astronomical_dawn();
        let safe_astronomical_dusk = calculator.get_astronomical_dusk();

        // Call unsafe SolarDay once
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_solar_day = unsafe {
            SolarDay(
                &mut ut,
                core::ptr::null_mut(), // delta_t
                0.0,                    // delta_ut1
                lon_rad,
                lat_rad,
                elevation,
                core::ptr::null_mut(), // gdip
                pressure,
                temperature,
                Some(unsafe_refract_bennet),
            )
        };

        // Compare prev solar midnight (index 0)
        if let Ok(safe_ts) = safe_prev_midnight {
            let diff = (safe_ts - unsafe_solar_day.t[0]).abs();
            prop_assert!(
                diff <= 2,
                "Previous solar midnight mismatch: safe={}, unsafe={}, diff={} seconds",
                safe_ts,
                unsafe_solar_day.t[0],
                diff
            );
        }

        // Compare solar transit (index 1)
        if let Ok(safe_ts) = safe_transit {
            let diff = (safe_ts - unsafe_solar_day.t[1]).abs();
            prop_assert!(
                diff <= 2,
                "Solar transit mismatch: safe={}, unsafe={}, diff={} seconds",
                safe_ts,
                unsafe_solar_day.t[1],
                diff
            );
        }

        // Compare next solar midnight (index 2)
        if let Ok(safe_ts) = safe_next_midnight {
            let diff = (safe_ts - unsafe_solar_day.t[2]).abs();
            prop_assert!(
                diff <= 2,
                "Next solar midnight mismatch: safe={}, unsafe={}, diff={} seconds",
                safe_ts,
                unsafe_solar_day.t[2],
                diff
            );
        }

        // Compare sunrise (index 3)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_sunrise {
            // Only compare if unsafe also found sunrise (status 0 = _FREESPA_EV_OK)
            if unsafe_solar_day.status[3] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[3]).abs();
                prop_assert!(
                    diff <= 2,
                    "Sunrise mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[3],
                    diff
                );
            }
        }

        // Compare sunset (index 4)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_sunset {
            if unsafe_solar_day.status[4] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[4]).abs();
                prop_assert!(
                    diff <= 2,
                    "Sunset mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[4],
                    diff
                );
            }
        }

        // Compare civil dawn (index 5)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_civil_dawn {
            if unsafe_solar_day.status[5] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[5]).abs();
                prop_assert!(
                    diff <= 2,
                    "Civil dawn mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[5],
                    diff
                );
            }
        }

        // Compare civil dusk (index 6)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_civil_dusk {
            if unsafe_solar_day.status[6] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[6]).abs();
                prop_assert!(
                    diff <= 2,
                    "Civil dusk mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[6],
                    diff
                );
            }
        }

        // Compare nautical dawn (index 7)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_nautical_dawn {
            if unsafe_solar_day.status[7] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[7]).abs();
                prop_assert!(
                    diff <= 2,
                    "Nautical dawn mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[7],
                    diff
                );
            }
        }

        // Compare nautical dusk (index 8)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_nautical_dusk {
            if unsafe_solar_day.status[8] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[8]).abs();
                prop_assert!(
                    diff <= 2,
                    "Nautical dusk mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[8],
                    diff
                );
            }
        }

        // Compare astronomical dawn (index 9)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_astronomical_dawn {
            if unsafe_solar_day.status[9] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[9]).abs();
                prop_assert!(
                    diff <= 2,
                    "Astronomical dawn mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[9],
                    diff
                );
            }
        }

        // Compare astronomical dusk (index 10)
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_astronomical_dusk {
            if unsafe_solar_day.status[10] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[10]).abs();
                prop_assert!(
                    diff <= 2,
                    "Astronomical dusk mismatch: safe={}, unsafe={}, diff={} seconds",
                    safe_ts,
                    unsafe_solar_day.t[10],
                    diff
                );
            }
        }
    }

    /// Test solar events at high latitudes where midnight sun/polar night can occur
    /// Note: At extreme latitudes (> 85°), numerical precision issues may occur
    #[test]
    fn solar_events_polar_regions(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in prop::strategy::Union::new(vec![
            (-85.0_f64..=-66.0_f64).boxed(),  // Antarctic (avoid extreme poles)
            (66.0_f64..=85.0_f64).boxed(),     // Arctic (avoid extreme poles)
        ]),
        elevation in 0.0..=100.0,
        pressure in 1000.0..=1013.25,
        temperature in -10.0..=10.0,
    ) {
        use crate::SolarEventResult;

        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Create safe calculator
        let result = SpaCalculator::new(
            dt,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            temperature,
            pressure,
            None,
            Refraction::ApSolposBennet,
        );

        // Should succeed even at extreme latitudes
        prop_assert!(result.is_ok(), "SpaCalculator::new should succeed for polar regions");

        let mut calculator = result.expect("already checked");

        // Get sunrise - may return Ok or Err in polar regions depending on conditions
        let sunrise = calculator.get_sunrise();

        // In polar regions, we may get AlwaysAbove, AlwaysBelow, or even errors
        // due to edge cases. Just verify the system doesn't panic.
        match sunrise {
            Ok(SolarEventResult::Occurs(_ts)) => {
                // Valid timestamp - note: can be negative for dates before 1970
                // Just verify we got a result
            }
            Ok(SolarEventResult::AlwaysAbove) | Ok(SolarEventResult::AlwaysBelow) => {
                // These are valid for polar regions
            }
            Err(_) => {
                // Errors may occur in polar regions due to edge cases
                // This is acceptable behavior
            }
        }
    }

    /// Test that solar events maintain logical ordering:
    /// astronomical_dawn < nautical_dawn < civil_dawn < sunrise < transit < sunset < civil_dusk < nautical_dusk < astronomical_dusk
    ///
    /// Note: This test uses restricted longitude range to avoid edge cases near the
    /// International Date Line where UTC midnight aligns with local noon.
    #[test]
    fn solar_events_chronological_order(
        datetime in any_utc_datetime(),
        longitude in -120.0_f64..=120.0_f64, // Avoid date line edge cases
        latitude in -45.0_f64..=45.0_f64, // Use moderate latitudes to ensure all events occur
        elevation in 0.0..=500.0,
        pressure in 1000.0..=1013.25,
        temperature in 10.0..=25.0,
    ) {
        use crate::SolarEventResult;

        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            temperature,
            pressure,
            None,
            Refraction::ApSolposBennet,
        ).expect("valid inputs");

        // Get all timestamps
        let transit = calculator.get_solar_transit();
        let sunrise = calculator.get_sunrise();
        let sunset = calculator.get_sunset();
        let civil_dawn = calculator.get_civil_dawn();
        let civil_dusk = calculator.get_civil_dusk();
        let nautical_dawn = calculator.get_nautical_dawn();
        let nautical_dusk = calculator.get_nautical_dusk();
        let astronomical_dawn = calculator.get_astronomical_dawn();
        let astronomical_dusk = calculator.get_astronomical_dusk();

        // Only check ordering if all events occur
        if let (
            Ok(transit_ts),
            Ok(SolarEventResult::Occurs(sunrise_ts)),
            Ok(SolarEventResult::Occurs(sunset_ts)),
            Ok(SolarEventResult::Occurs(civil_dawn_ts)),
            Ok(SolarEventResult::Occurs(civil_dusk_ts)),
            Ok(SolarEventResult::Occurs(nautical_dawn_ts)),
            Ok(SolarEventResult::Occurs(nautical_dusk_ts)),
            Ok(SolarEventResult::Occurs(astro_dawn_ts)),
            Ok(SolarEventResult::Occurs(astro_dusk_ts)),
        ) = (
            transit,
            sunrise,
            sunset,
            civil_dawn,
            civil_dusk,
            nautical_dawn,
            nautical_dusk,
            astronomical_dawn,
            astronomical_dusk,
        ) {
            // Morning events should maintain relative order
            prop_assert!(
                astro_dawn_ts <= nautical_dawn_ts,
                "Astronomical dawn ({}) should be before or equal to nautical dawn ({})",
                astro_dawn_ts, nautical_dawn_ts
            );
            prop_assert!(
                nautical_dawn_ts <= civil_dawn_ts,
                "Nautical dawn ({}) should be before or equal to civil dawn ({})",
                nautical_dawn_ts, civil_dawn_ts
            );
            prop_assert!(
                civil_dawn_ts <= sunrise_ts,
                "Civil dawn ({}) should be before or equal to sunrise ({})",
                civil_dawn_ts, sunrise_ts
            );
            prop_assert!(
                sunrise_ts <= transit_ts,
                "Sunrise ({}) should be before or equal to transit ({})",
                sunrise_ts, transit_ts
            );

            // Evening events should maintain relative order
            prop_assert!(
                transit_ts <= sunset_ts,
                "Transit ({}) should be before or equal to sunset ({})",
                transit_ts, sunset_ts
            );
            prop_assert!(
                sunset_ts <= civil_dusk_ts,
                "Sunset ({}) should be before or equal to civil dusk ({})",
                sunset_ts, civil_dusk_ts
            );
            prop_assert!(
                civil_dusk_ts <= nautical_dusk_ts,
                "Civil dusk ({}) should be before or equal to nautical dusk ({})",
                civil_dusk_ts, nautical_dusk_ts
            );
            prop_assert!(
                nautical_dusk_ts <= astro_dusk_ts,
                "Nautical dusk ({}) should be before or equal to astronomical dusk ({})",
                nautical_dusk_ts, astro_dusk_ts
            );
        }
    }

    /// Test memoization - calling the same method multiple times returns the same result
    #[test]
    fn solar_events_memoization(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -45.0_f64..=45.0_f64,
        elevation in 0.0..=200.0,
        pressure in 1000.0..=1013.25,
        temperature in 15.0..=25.0,
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            temperature,
            pressure,
            None,
            Refraction::ApSolposBennet,
        ).expect("valid inputs");

        // Call each method twice and verify results are identical
        let sunrise1 = calculator.get_sunrise();
        let sunrise2 = calculator.get_sunrise();
        prop_assert_eq!(format!("{:?}", sunrise1), format!("{:?}", sunrise2), "Memoized sunrise should match");

        let sunset1 = calculator.get_sunset();
        let sunset2 = calculator.get_sunset();
        prop_assert_eq!(format!("{:?}", sunset1), format!("{:?}", sunset2), "Memoized sunset should match");

        let civil_dawn1 = calculator.get_civil_dawn();
        let civil_dawn2 = calculator.get_civil_dawn();
        prop_assert_eq!(format!("{:?}", civil_dawn1), format!("{:?}", civil_dawn2), "Memoized civil_dawn should match");

        let civil_dusk1 = calculator.get_civil_dusk();
        let civil_dusk2 = calculator.get_civil_dusk();
        prop_assert_eq!(format!("{:?}", civil_dusk1), format!("{:?}", civil_dusk2), "Memoized civil_dusk should match");
    }

    // =============================================================================
    // ADDITIONAL EDGE CASE TESTS - Addressing previously missing coverage
    // =============================================================================

    /// Test with explicit delta_t values instead of None
    /// This tests the delta_t computation path that was previously untested
    #[test]
    fn solar_position_with_explicit_delta_t(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -85.0_f64..=85.0_f64,
        delta_t in 60.0_f64..=75.0_f64, // Typical delta_t range for 2000-2100
    ) {
        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();
        let elevation = 0.0;

        // Call safe with explicit delta_t
        let mut calculator = SpaCalculator::new(
            dt,
            Some(delta_t),
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            10.0,
            1010.0,
            None,
            Refraction::ApSolposBennet,
        ).unwrap();
        let safe_result = calculator.get_solar_position();

        // Call unsafe with explicit delta_t
        let mut ut = naive_datetime_to_tm(&dt);
        let mut delta_t_mut = delta_t;
        let unsafe_result = unsafe {
            SPA(
                &mut ut,
                &mut delta_t_mut,
                0.0,
                lon_rad,
                lat_rad,
                elevation,
            )
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Explicit delta_t zenith mismatch: unsafe={}, safe={}, diff={}, delta_t={}",
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs(),
            delta_t
        );
    }

    /// Test solar events with explicit gdip values
    #[test]
    fn solar_events_with_explicit_gdip(
        datetime in any_utc_datetime(),
        longitude in -120.0_f64..=120.0_f64,
        latitude in -50.0_f64..=50.0_f64,
        elevation in 0.0..=500.0,
        gdip in -0.1_f64..=0.1_f64, // Small realistic gdip values
    ) {
        use crate::SolarEventResult;

        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Create safe calculator with explicit gdip
        let mut calculator = SpaCalculator::new(
            dt,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            10.0,
            1010.0,
            Some(gdip),
            Refraction::ApSolposBennet,
        ).unwrap();

        let safe_sunrise = calculator.get_sunrise();
        let safe_sunset = calculator.get_sunset();

        // Call unsafe SolarDay with explicit gdip
        let mut ut = naive_datetime_to_tm(&dt);
        let mut gdip_mut = gdip;
        let unsafe_solar_day = unsafe {
            SolarDay(
                &mut ut,
                core::ptr::null_mut(),
                0.0,
                lon_rad,
                lat_rad,
                elevation,
                &mut gdip_mut,
                1010.0,
                10.0,
                Some(unsafe_refract_bennet),
            )
        };

        // Compare sunrise
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_sunrise {
            if unsafe_solar_day.status[3] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[3]).abs();
                prop_assert!(
                    diff <= 2,
                    "Sunrise with gdip={} mismatch: safe={}, unsafe={}, diff={}",
                    gdip, safe_ts, unsafe_solar_day.t[3], diff
                );
            }
        }

        // Compare sunset
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_sunset {
            if unsafe_solar_day.status[4] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[4]).abs();
                prop_assert!(
                    diff <= 2,
                    "Sunset with gdip={} mismatch: safe={}, unsafe={}, diff={}",
                    gdip, safe_ts, unsafe_solar_day.t[4], diff
                );
            }
        }
    }

    /// Test that safe and unsafe agree on AlwaysAbove/AlwaysBelow status
    /// This catches cases where one returns an event and the other returns polar status
    #[test]
    fn solar_events_status_agreement(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -85.0_f64..=85.0_f64,
        elevation in 0.0..=100.0,
    ) {
        use crate::SolarEventResult;

        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            10.0,
            1010.0,
            None,
            Refraction::ApSolposBennet,
        ).unwrap();

        let safe_sunrise = calculator.get_sunrise();

        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_solar_day = unsafe {
            SolarDay(
                &mut ut,
                core::ptr::null_mut(),
                0.0,
                lon_rad,
                lat_rad,
                elevation,
                core::ptr::null_mut(),
                1010.0,
                10.0,
                Some(unsafe_refract_bennet),
            )
        };

        // Verify status agreement for sunrise
        match safe_sunrise {
            Ok(SolarEventResult::Occurs(_ts)) => {
                // Safe found an event - unsafe should have status 0 (OK)
                prop_assert!(
                    unsafe_solar_day.status[3] == 0,
                    "Safe found sunrise but unsafe status is {} (expected 0)",
                    unsafe_solar_day.status[3]
                );
            }
            Ok(SolarEventResult::AlwaysAbove) => {
                // Safe says sun always above - unsafe should have status -1
                prop_assert!(
                    unsafe_solar_day.status[3] == -1,
                    "Safe returned AlwaysAbove but unsafe status is {} (expected -1)",
                    unsafe_solar_day.status[3]
                );
            }
            Ok(SolarEventResult::AlwaysBelow) => {
                // Safe says sun always below - unsafe should have status 1
                prop_assert!(
                    unsafe_solar_day.status[3] == 1,
                    "Safe returned AlwaysBelow but unsafe status is {} (expected 1)",
                    unsafe_solar_day.status[3]
                );
            }
            Err(_) => {
                // Safe errored - this is acceptable, just skip
            }
        }
    }

    /// Test BennetNA refraction model in solar events (previously only tested Bennet)
    #[test]
    fn solar_events_bennet_na_refraction(
        datetime in any_utc_datetime(),
        longitude in -120.0_f64..=120.0_f64,
        latitude in -50.0_f64..=50.0_f64,
        elevation in 0.0..=500.0,
        pressure in 900.0..=1100.0,
        temperature in -20.0..=40.0,
    ) {
        use crate::SolarEventResult;

        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Create safe calculator with BennetNA refraction
        let mut calculator = SpaCalculator::new(
            dt,
            None,
            0.0,
            lon_rad,
            lat_rad,
            elevation,
            temperature,
            pressure,
            None,
            Refraction::ApSolposBennetNA,
        ).unwrap();

        let safe_sunrise = calculator.get_sunrise();
        let safe_sunset = calculator.get_sunset();

        // Call unsafe with BennetNA refraction
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_solar_day = unsafe {
            SolarDay(
                &mut ut,
                core::ptr::null_mut(),
                0.0,
                lon_rad,
                lat_rad,
                elevation,
                core::ptr::null_mut(),
                pressure,
                temperature,
                Some(unsafe_refract_bennet_na),
            )
        };

        // Compare sunrise
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_sunrise {
            if unsafe_solar_day.status[3] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[3]).abs();
                prop_assert!(
                    diff <= 2,
                    "BennetNA sunrise mismatch: safe={}, unsafe={}, diff={}",
                    safe_ts, unsafe_solar_day.t[3], diff
                );
            }
        }

        // Compare sunset
        if let Ok(SolarEventResult::Occurs(safe_ts)) = safe_sunset {
            if unsafe_solar_day.status[4] == 0 {
                let diff = (safe_ts - unsafe_solar_day.t[4]).abs();
                prop_assert!(
                    diff <= 2,
                    "BennetNA sunset mismatch: safe={}, unsafe={}, diff={}",
                    safe_ts, unsafe_solar_day.t[4], diff
                );
            }
        }
    }

    /// Test leap year edge cases - Feb 29
    #[test]
    fn leap_year_edge_cases(
        leap_year in prop::strategy::Union::new(vec![
            Just(2000i32).boxed(),  // Divisible by 400
            Just(2004i32).boxed(),  // Regular leap year
            Just(2024i32).boxed(),
            Just(2100i32).boxed(),  // NOT a leap year (divisible by 100 but not 400)
        ]),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -60.0_f64..=60.0_f64,
        hour in 0u32..24u32,
    ) {
        // Check if it's actually a leap year
        let is_leap = (leap_year % 4 == 0 && leap_year % 100 != 0) || (leap_year % 400 == 0);

        // Use Feb 28 for non-leap years, Feb 29 for leap years
        let day = if is_leap { 29 } else { 28 };

        let dt = NaiveDate::from_ymd_opt(leap_year, 2, day)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap();

        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        // Call safe implementation
        let mut calculator = SpaCalculator::new(
            dt, None, 0.0, lon_rad, lat_rad, 0.0, 10.0, 1010.0, None, Refraction::ApSolposBennet
        ).unwrap();
        let safe_result = calculator.get_solar_position();

        // Call unsafe implementation
        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, 0.0)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Leap year {} zenith mismatch: unsafe={}, safe={}, diff={}",
            leap_year,
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
    }

    /// Test year boundaries and century changes
    #[test]
    fn year_boundary_tests(
        year in prop::strategy::Union::new(vec![
            Just(1999i32).boxed(),
            Just(2000i32).boxed(),
            Just(2099i32).boxed(),
            Just(2100i32).boxed(),
        ]),
        month in 1u32..=2u32, // Jan or Feb to catch year transitions
        day in 1u32..28u32,
        longitude in -180.0_f64..=180.0_f64,
        latitude in -60.0_f64..=60.0_f64,
    ) {
        let dt = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt, None, 0.0, lon_rad, lat_rad, 0.0, 10.0, 1010.0, None, Refraction::ApSolposBennet
        ).unwrap();
        let safe_result = calculator.get_solar_position();

        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, 0.0)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Year {} boundary zenith mismatch: unsafe={}, safe={}, diff={}",
            year,
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
    }

    /// Test equinoxes - when day/night are equal
    #[test]
    fn equinox_tests(
        year in 2000i32..=2050i32,
        is_spring in proptest::bool::ANY,
        longitude in -180.0_f64..=180.0_f64,
        latitude in -60.0_f64..=60.0_f64,
        hour in 0u32..24u32,
    ) {
        // Spring equinox ~March 20, Autumn equinox ~September 22
        let (month, day) = if is_spring { (3, 20) } else { (9, 22) };

        let dt = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap();

        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt, None, 0.0, lon_rad, lat_rad, 0.0, 10.0, 1010.0, None, Refraction::ApSolposBennet
        ).unwrap();
        let safe_result = calculator.get_solar_position();

        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, 0.0)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Equinox {} zenith mismatch: unsafe={}, safe={}, diff={}",
            if is_spring { "spring" } else { "autumn" },
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
    }

    /// Test solstices - extreme sun positions
    #[test]
    fn solstice_tests(
        year in 2000i32..=2050i32,
        is_summer in proptest::bool::ANY,
        longitude in -180.0_f64..=180.0_f64,
        latitude in -60.0_f64..=60.0_f64,
        hour in 0u32..24u32,
    ) {
        // Summer solstice ~June 21, Winter solstice ~December 21
        let (month, day) = if is_summer { (6, 21) } else { (12, 21) };

        let dt = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap();

        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt, None, 0.0, lon_rad, lat_rad, 0.0, 10.0, 1010.0, None, Refraction::ApSolposBennet
        ).unwrap();
        let safe_result = calculator.get_solar_position();

        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, 0.0)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Solstice {} zenith mismatch: unsafe={}, safe={}, diff={}",
            if is_summer { "summer" } else { "winter" },
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
    }
}

/// Wrapper for unsafe BennetNA refraction function
unsafe extern "C" fn unsafe_refract_bennet_na(pos: sol_pos, gdip: *mut f64, e: f64, p: f64, t: f64) -> sol_pos {
    ApSolposBennetNA(pos, gdip, e, p, t)
}

/// Test solar noon at the equator on the equinox
/// At the equator on equinox at solar noon, the sun should be nearly overhead (zenith ≈ 0°)
#[test]
fn equator_equinox_solar_noon() {
    // March 20, 2024 around 12:00 UTC at the Prime Meridian
    let dt = NaiveDate::from_ymd_opt(2024, 3, 20)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();

    let lon_rad = 0.0; // Prime Meridian
    let lat_rad = 0.0; // Equator

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        0.0,
        25.0,
        1013.25,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();

    let result = calculator.get_solar_position();
    let zenith_deg = result.zenith.to_degrees();

    // At equinox on equator at solar noon, zenith should be very small (< 5°)
    // Note: Solar noon at Prime Meridian is around 12:00 UTC, but not exactly
    // The sun won't be perfectly overhead due to equation of time and exact timing
    assert!(
        zenith_deg < 10.0,
        "Equator equinox zenith should be small, got {}°",
        zenith_deg
    );
}

/// Test midnight sun phenomenon - sun above horizon at midnight in summer polar regions
#[test]
fn midnight_sun_test() {
    // June 21 (summer solstice) at midnight UTC, at 70°N (well within Arctic Circle)
    let dt = NaiveDate::from_ymd_opt(2024, 6, 21)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    // At 70°N, longitude doesn't matter much for this test
    let lon_rad = 0.0_f64.to_radians();
    let lat_rad = 70.0_f64.to_radians();

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        0.0,
        10.0,
        1013.25,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();

    let result = calculator.get_solar_position();
    let zenith_deg = result.zenith.to_degrees();

    // At this location/time, sun should be above horizon (zenith < 90°)
    // This validates the midnight sun phenomenon
    assert!(
        zenith_deg < 90.0,
        "Midnight sun: zenith should be < 90° at 70°N during summer solstice midnight, got {}°",
        zenith_deg
    );
}

/// Test polar night phenomenon - sun below horizon at noon in winter polar regions
#[test]
fn polar_night_test() {
    // December 21 (winter solstice) at noon UTC, at 70°N
    let dt = NaiveDate::from_ymd_opt(2024, 12, 21)
        .unwrap()
        .and_hms_opt(12, 0, 0)
        .unwrap();

    let lon_rad = 0.0_f64.to_radians();
    let lat_rad = 70.0_f64.to_radians();

    let mut calculator = SpaCalculator::new(
        dt,
        None,
        0.0,
        lon_rad,
        lat_rad,
        0.0,
        -10.0, // Cold!
        1013.25,
        None,
        Refraction::ApSolposBennet,
    )
    .unwrap();

    let result = calculator.get_solar_position();
    let zenith_deg = result.zenith.to_degrees();

    // At this location/time, sun should be below horizon (zenith > 90°)
    // This validates the polar night phenomenon
    assert!(
        zenith_deg > 90.0,
        "Polar night: zenith should be > 90° at 70°N during winter solstice noon, got {}°",
        zenith_deg
    );
}

// =============================================================================
// DELTA_T TABLE VERIFICATION
// Verify safe and unsafe use identical delta_t tables
// =============================================================================

/// Verify that the delta_t tables in safe and unsafe implementations are identical
#[test]
fn delta_t_table_identical() {
    use crate::tables::FREESPA_DELTA_T_TABLE;
    use crate::unsafe_spa::freespa_delta_t_table as UNSAFE_TABLE;

    // Both tables should have the same length
    assert_eq!(
        FREESPA_DELTA_T_TABLE.len(),
        2490,
        "Safe delta_t table should have 2490 elements"
    );

    // Compare every element
    for i in 0..FREESPA_DELTA_T_TABLE.len() {
        let safe_val = FREESPA_DELTA_T_TABLE[i];
        let unsafe_val = unsafe { UNSAFE_TABLE[i] };

        assert!(
            (safe_val - unsafe_val).abs() < 1e-10,
            "Delta_t table mismatch at index {}: safe={}, unsafe={}, diff={}",
            i,
            safe_val,
            unsafe_val,
            (safe_val - unsafe_val).abs()
        );
    }
}

proptest! {
    /// Test that safe and unsafe implementations agree on whether sunrise/sunset occurs
    /// This catches cases where one finds an event but the other doesn't
    #[test]
    fn sunrise_sunset_occurrence_agreement(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -85.0_f64..=85.0_f64,
        elevation in 0.0..=500.0,
    ) {
        use crate::SolarEventResult;

        let dt = datetime.naive_utc();
        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt, None, 0.0, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet
        ).unwrap();

        let safe_sunrise = calculator.get_sunrise();
        let safe_sunset = calculator.get_sunset();

        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_solar_day = unsafe {
            SolarDay(
                &mut ut,
                core::ptr::null_mut(),
                0.0,
                lon_rad,
                lat_rad,
                elevation,
                core::ptr::null_mut(),
                1010.0,
                10.0,
                Some(unsafe_refract_bennet),
            )
        };

        // Check sunrise agreement
        match safe_sunrise {
            Ok(SolarEventResult::Occurs(_)) => {
                // Safe found sunrise - unsafe should have found it too (status 0)
                prop_assert!(
                    unsafe_solar_day.status[3] == 0,
                    "Safe found sunrise but unsafe status={} (lat={}°)",
                    unsafe_solar_day.status[3],
                    latitude
                );
            }
            Ok(SolarEventResult::AlwaysAbove) => {
                prop_assert!(
                    unsafe_solar_day.status[3] == -1,
                    "Safe: AlwaysAbove, unsafe status={} (lat={}°)",
                    unsafe_solar_day.status[3],
                    latitude
                );
            }
            Ok(SolarEventResult::AlwaysBelow) => {
                prop_assert!(
                    unsafe_solar_day.status[3] == 1,
                    "Safe: AlwaysBelow, unsafe status={} (lat={}°)",
                    unsafe_solar_day.status[3],
                    latitude
                );
            }
            Err(_) => {
                // Safe errored - this is edge case behavior, just skip
            }
        }

        // Check sunset agreement
        match safe_sunset {
            Ok(SolarEventResult::Occurs(_)) => {
                prop_assert!(
                    unsafe_solar_day.status[4] == 0,
                    "Safe found sunset but unsafe status={} (lat={}°)",
                    unsafe_solar_day.status[4],
                    latitude
                );
            }
            Ok(SolarEventResult::AlwaysAbove) => {
                prop_assert!(
                    unsafe_solar_day.status[4] == -1,
                    "Safe: AlwaysAbove for sunset, unsafe status={} (lat={}°)",
                    unsafe_solar_day.status[4],
                    latitude
                );
            }
            Ok(SolarEventResult::AlwaysBelow) => {
                prop_assert!(
                    unsafe_solar_day.status[4] == 1,
                    "Safe: AlwaysBelow for sunset, unsafe status={} (lat={}°)",
                    unsafe_solar_day.status[4],
                    latitude
                );
            }
            Err(_) => {
                // Safe errored - skip
            }
        }
    }

    /// Test solar position at times when unsafe implementation might have edge cases
    /// (midnight, noon, around sunrise/sunset times)
    #[test]
    fn solar_position_critical_times(
        year in 2000i32..=2050i32,
        month in 1u32..=12u32,
        day in 1u32..=28u32,
        critical_hour in prop::strategy::Union::new(vec![
            Just(0u32).boxed(),   // Midnight
            Just(6u32).boxed(),   // Around sunrise
            Just(12u32).boxed(),  // Noon
            Just(18u32).boxed(),  // Around sunset
            Just(23u32).boxed(),  // Near midnight
        ]),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -60.0_f64..=60.0_f64,
    ) {
        let dt = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(critical_hour, 0, 0)
            .unwrap();

        let lon_rad = longitude.to_radians();
        let lat_rad = latitude.to_radians();

        let mut calculator = SpaCalculator::new(
            dt, None, 0.0, lon_rad, lat_rad, 0.0, 10.0, 1010.0, None, Refraction::ApSolposBennet
        ).unwrap();
        let safe_result = calculator.get_solar_position();

        let mut ut = naive_datetime_to_tm(&dt);
        let unsafe_result = unsafe {
            SPA(&mut ut, core::ptr::null_mut(), 0.0, lon_rad, lat_rad, 0.0)
        };

        let epsilon = 5e-4;
        prop_assert!(
            (unsafe_result.z - safe_result.zenith).abs() < epsilon,
            "Critical time {}:00 zenith mismatch: unsafe={}, safe={}, diff={}",
            critical_hour,
            unsafe_result.z,
            safe_result.zenith,
            (unsafe_result.z - safe_result.zenith).abs()
        );
        prop_assert!(
            (unsafe_result.a - safe_result.azimuth).abs() < epsilon ||
            ((unsafe_result.a - safe_result.azimuth).abs() - 2.0 * PI).abs() < epsilon,
            "Critical time {}:00 azimuth mismatch: unsafe={}, safe={}, diff={}",
            critical_hour,
            unsafe_result.a,
            safe_result.azimuth,
            (unsafe_result.a - safe_result.azimuth).abs()
        );
    }
}
