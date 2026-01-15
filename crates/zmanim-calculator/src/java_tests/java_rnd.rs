//! A set of functions which generate random pairs of Java and Rust objects.
//! This is used in our testing framework to create random test cases.
use std::str::FromStr;

use crate::{
    java_tests::java_bindings::{JavaTimeAndPlace, JavaZmanimCalendar},
    Location, ZmanimCalculator,
};

use super::*;
use chrono::{Datelike, Duration, TimeZone, Timelike, Utc};
use j4rs::Jvm;
use lazy_static::lazy_static;
use rand::Rng;
use tzf_rs::DefaultFinder;

lazy_static! {
    static ref FINDER: DefaultFinder = DefaultFinder::new();
}

/// Default number of years to test.
static DEFAULT_TEST_YEARS: i64 = 100;

/// Default number of milliseconds in the given number of years.
static DEFAULT_TEST_YEARS_IN_MILLISECONDS: i64 = 1000 * 3600 * 24 * 365 * DEFAULT_TEST_YEARS;

/// Generates a random DateTime in the range 1870-2070 with the given timezone.
fn random_date_time(rng: &mut impl Rng, tz: &chrono_tz::Tz) -> DateTime<chrono_tz::Tz> {
    let milliseconds_since_epoch: i64 = rng.gen_range(
        -DEFAULT_TEST_YEARS_IN_MILLISECONDS..=DEFAULT_TEST_YEARS_IN_MILLISECONDS, // 1870 to 2070
    );
    tz.timestamp_millis_opt(milliseconds_since_epoch).unwrap()
}

pub fn random_time_and_place<Rng: rand::Rng>(
    jvm: &Jvm,
    rng: &mut Rng,
) -> Option<(
    Location<chrono_tz::Tz>,
    DateTime<chrono_tz::Tz>,
    JavaTimeAndPlace,
)> {
    // We are using a different algorithm to calculate sunrise and sunset.
    // The difference between these 2 algorithms are small under most cases. However as
    // you get closer to the poles, these results can vary signifigantly.
    // We are allowing for a n second difference between results. If we test for locations
    // too close to the poles, we would need to allow for a much larger room for error
    // which would start to affect the effectiveness of the tests.
    let latitude = rng.gen_range(-50.0..=50.0);
    let longitude = rng.gen_range(-180.0..=180.0);
    let elevation = rng.gen_range(-0.0..=400.0);
    let timezone_id = FINDER.get_tz_name(longitude, latitude);
    let timezone = chrono_tz::Tz::from_str(timezone_id).ok()?;
    let date_time = random_date_time(rng, &timezone);
    let rust_time_and_place = Location::new(latitude, longitude, elevation, Some(timezone))?;
    let java_time_and_place = JavaTimeAndPlace::new(jvm, &rust_time_and_place, &date_time)?;
    Some((rust_time_and_place, date_time, java_time_and_place))
}

pub fn random_zmanim_calendars<'a>(
    jvm: &'a Jvm,
    rng: &mut impl Rng,
) -> Option<(
    ZmanimCalculator,
    Location<chrono_tz::Tz>,
    DateTime<chrono_tz::Tz>,
    JavaZmanimCalendar<'a>,
)> {
    let (rust_time_and_place, date_time, java_time_and_place) = random_time_and_place(jvm, rng)?;
    let candle_lighting_offset = Duration::minutes(rng.gen_range(0..=60));
    let use_astronomical_chatzos = rng.gen_bool(0.5);
    let use_astronomical_chatzos_for_other_zmanim = rng.gen_bool(0.5);
    let ateret_torah_sunset_offset = Duration::minutes(rng.gen_range(0..=60));
    let rust_calculator = ZmanimCalculator::new(
        use_astronomical_chatzos,
        candle_lighting_offset,
        use_astronomical_chatzos_for_other_zmanim,
    );
    let java_calendar = JavaZmanimCalendar::new(
        jvm,
        java_time_and_place,
        candle_lighting_offset,
        use_astronomical_chatzos,
        use_astronomical_chatzos_for_other_zmanim,
        ateret_torah_sunset_offset,
    )?;

    Some((
        rust_calculator,
        rust_time_and_place,
        date_time,
        java_calendar,
    ))
}
