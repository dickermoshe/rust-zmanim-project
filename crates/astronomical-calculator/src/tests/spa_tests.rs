use super::unsafe_spa::*;
use crate::*;
use chrono::prelude::*;
use proptest::prelude::*;
use proptest::proptest;

/// Wrapper for unsafe BennetNA refraction function
unsafe extern "C" fn unsafe_refract_bennet(pos: sol_pos, gdip: *mut f64, e: f64, p: f64, t: f64) -> sol_pos {
    ApSolposBennet(pos, gdip, e, p, t)
}

/// Wrapper for unsafe BennetNA refraction function
unsafe extern "C" fn unsafe_refract_bennet_na(pos: sol_pos, gdip: *mut f64, e: f64, p: f64, t: f64) -> sol_pos {
    ApSolposBennetNA(pos, gdip, e, p, t)
}

/// Compares the results of a solar event calculation between the safe and unsafe SPA algorithms.
/// Panics if the results do not match.
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
            panic!(
                "Solar event results don't match: safe={:?}, unsafe={:?}, for method {}",
                safe_result, unsafe_result, name,
            );
        }
    }
}
/// A helper method to convert the results of the unsafe solar day calculation to a SolarEventResult.
fn solar_day_to_event(day: solar_day, index: usize) -> SolarEventResult {
    if day.status[index] == 0 {
        SolarEventResult::Occurs(day.t[index])
    } else if day.status[index] == 1 {
        SolarEventResult::AllDay
    } else if day.status[index] == -1 {
        SolarEventResult::AllNight
    } else {
        panic!("Invalid status: {}", day.status[index]);
    }
}

/// Compares the results of the safe and unsafe SPA algorithms
/// with the given parameters.
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
    let mut ut = tm {
        timestamp: naive_datetime.timestamp_millis(),
    };
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

/// A proptest strategy to generate UTC datetimes in
/// the range 1900-2100.
fn any_utc_datetime() -> impl Strategy<Value = DateTime<Utc>> {
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

/// A proptest strategy to generate refraction models.
fn refraction_strategy() -> impl Strategy<Value = Refraction> {
    prop_oneof![Just(Refraction::ApSolposBennet), Just(Refraction::ApSolposBennetNA)]
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
