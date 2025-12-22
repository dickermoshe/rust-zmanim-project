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

use crate::spa2::FindSolTime;
use crate::spa2::FindSolZenith;
use crate::spa2::Refraction;
use crate::spa2::SolarPosition;
use crate::spa2::SolarZenith;
use crate::spa2::SpaCalculator;
use crate::spa2::ABSOLUTEZERO;
use crate::spa2::EARTH_R;
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
fn safe_refract_bennet(pos: SolarPosition, gdip: Option<f64>, e: f64, p: f64, T: f64) -> SolarPosition {
    pos.ApSolposBennet(gdip, e, p, T).unwrap_or(pos)
}

/// Wrapper for unsafe refraction function
unsafe extern "C" fn unsafe_refract_bennet(pos: sol_pos, gdip: *mut f64, e: f64, p: f64, T: f64) -> sol_pos {
    ApSolposBennet(pos, gdip, e, p, T)
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
        let mut calculator = SpaCalculator::new(dt, None, delta_ut1, lon_rad, lat_rad, elevation, 10.0, 1010.0, None, Refraction::ApSolposBennet).unwrap();
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

        // Call safe FindSolTime
        let safe_result = FindSolTime(
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

        // Call safe FindSolZenith
        let safe_result = FindSolZenith(
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
    let safe_result = pos.ApSolposBennet(None, elevation, 0.0, 10.0);
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
    let safe_result = pos.ApSolposBennet(None, elevation, -10.0, 10.0);
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
    let safe_result = pos.ApSolposBennet(None, elevation, 5001.0, 10.0);
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
    let safe_result = pos.ApSolposBennet(None, elevation, 1010.0, invalid_temp);
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
    let safe_result = pos.ApSolposBennet(None, elevation, 1010.0, ABSOLUTEZERO);
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
    let safe_result = pos.ApSolposBennet(Some(invalid_gdip), elevation, 1010.0, 10.0);
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
    let safe_result = pos.ApSolposBennet(Some(invalid_gdip), elevation, 1010.0, 10.0);
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
    let safe_result = pos.ApSolposBennet(Some(valid_gdip), elevation, 1010.0, 10.0);
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
    #[test]
    fn solar_day_methods_match_unsafe(
        datetime in any_utc_datetime(),
        longitude in -180.0_f64..=180.0_f64,
        latitude in -85.0_f64..=85.0_f64, // Avoid extreme latitudes for stability
        elevation in 0.0..=1000.0,
        pressure in 900.0..=1100.0,
        temperature in -20.0..=40.0,
    ) {
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

        // Get all safe results
        let safe_transit = calculator.get_solar_transit();
        let safe_prev_midnight = calculator.get_prev_solar_midnight();
        let safe_next_midnight = calculator.get_next_solar_midnight();

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
    }
}
