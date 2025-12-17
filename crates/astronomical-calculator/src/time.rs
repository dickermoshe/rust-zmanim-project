use chrono::{DateTime, Datelike, FixedOffset, Offset, TimeZone, Timelike};
#[allow(unused_imports)]
use core_maths::CoreFloat;

use crate::math::normalize_unit_interval;

/// Normalize a time offset (in minutes) to a small range around zero.
///
/// This function assumes that the input value is effectively an offset that
/// might be off by approximately a whole number of days. It "wraps" values
/// that are more than 20 minutes away from zero by adding or subtracting
/// one full day (1440 minutes), so that the result lies in the range
/// \[-20, 20\] minutes.
///
/// This is useful in astronomical calculations such as the equation of time,
/// where angle wrapping can produce results that differ from the expected
/// small offset by almost a full day.
///
/// # Arguments
///
/// * `minutes` - A time offset in minutes, potentially outside the
///   \[-20, 20\] range.
///
/// # Returns
///
/// A time offset in minutes, normalized to be within \[-20, 20\].
pub(crate) fn normalize_time_offset_minutes(minutes: f64) -> f64 {
    let mut limited = minutes;
    if limited < -20.0_f64 {
        limited += 1440.0_f64;
    } else if limited > 20.0_f64 {
        limited -= 1440.0_f64;
    }
    limited
}

/// Compute the equation of time (`EoT`) in minutes, normalized to a small range.
///
/// The equation of time is the difference between apparent solar time
/// (as indicated by a sundial) and mean solar time (clock time), expressed
/// in minutes. This function computes a raw value from the given
/// astronomical parameters and then normalizes it using
/// [`normalize_time_offset_minutes`] so that the final result is near zero.
///
/// # Arguments
///
/// * `mean_longitude_deg`       - Mean longitude of the Sun (degrees).
/// * `apparent_ra_deg`   - Apparent right ascension of the Sun (degrees).
/// * `nutation_longitude_deg` - Nutation in longitude (degrees).
/// * `true_obliquity_deg` - True obliquity of the ecliptic (degrees).
///
/// # Returns
///
/// The equation of time in minutes, normalized to a small range around zero
/// (approximately \[-20, 20\] minutes).
pub(crate) fn equation_of_time(
    mean_longitude_deg: f64,
    apparent_ra_deg: f64,
    nutation_longitude_deg: f64,
    true_obliquity_deg: f64,
) -> f64 {
    normalize_time_offset_minutes(
        4.0 * (mean_longitude_deg - 0.005_718_3 - apparent_ra_deg
            + nutation_longitude_deg * true_obliquity_deg.to_radians().cos()),
    )
}
/// Compute the astronomical Julian Day for a given date/time.
///
/// - `date`: Calendar date/time in any chrono `TimeZone`.
/// - `delta_t`: Additional offset in **seconds** (e.g., ΔT or other correction)
///   that will be added to the UTC-based time.
///
/// The result is a Julian Day in UT (or `UT + delta_t / 86400`), including
/// fractional days.
pub fn julian_day<T: TimeZone>(date: &DateTime<T>, delta_t: f64) -> f64 {
    let mut year = date.year();
    let mut month = date.month();
    let day = date.day();
    let hour = date.hour();
    let minute = date.minute();
    // Seconds including fractional part from nanoseconds
    let seconds = f64::from(date.second()) + f64::from(date.nanosecond()) / 1_000_000_000.0;

    // Time zone offset (UTC - local) in **hours**.
    let tz_hours = date.offset().fix().local_minus_utc() as f64 / 3600.0;

    // Convert calendar day + time-of-day to a fractional day value
    let day_decimal: f64 =
        f64::from(day) + (f64::from(hour) - tz_hours + (f64::from(minute) + (seconds + delta_t) / 60.0) / 60.0) / 24.0;

    // If month is January or February, treat them as month 13/14 of the
    // previous year (standard step in the Julian Day algorithm).
    if month < 3 {
        month += 12;
        year -= 1;
    }

    // Core Julian Day computation (for the proleptic Julian calendar)
    let mut julian_day = ((365.25 * (f64::from(year) + 4716.0)) as i32 as f64
        + (30.6001 * f64::from(month + 1)) as i32 as f64)
        + day_decimal
        - 1524.5;
    // Gregorian calendar correction for dates on/after 1582-10-15
    if julian_day > 2_299_160.0 {
        let a = f64::from(year / 100i32);
        julian_day += 2.0 - a + (a / 4.0) as i32 as f64;
    }
    julian_day
}

/// Julian century (T) from the given Julian Day (JD),
/// measured in Julian centuries since the J2000.0 epoch.
pub(crate) fn julian_century_from_julian_day(julian_day: f64) -> f64 {
    (julian_day - 2_451_545.0) / 36_525.0
}

/// Julian Ephemeris Day (JDE) from Julian Day (JD) and ΔT `delta_t`.
///
/// - `julian_day`: Julian Day in UT.
/// - `delta_t`: Difference TT − UT in **seconds** (ΔT).
pub(crate) fn julian_ephemeris_day_from_julian_day(julian_day: f64, delta_t: f64) -> f64 {
    julian_day + delta_t / 86_400.0
}

/// Julian Ephemeris Century (JCE) from Julian Ephemeris Day (JDE),
/// measured in Julian centuries since the J2000.0 epoch.
pub(crate) fn julian_ephemeris_century_from_julian_ephemeris_day(ephemeris_day: f64) -> f64 {
    (ephemeris_day - 2_451_545.0) / 36_525.0
}

/// Julian Ephemeris Millennium (JME) from Julian Ephemeris Century (JCE).
pub(crate) fn julian_ephemeris_millennium_from_julian_ephemeris_century(ephemeris_century: f64) -> f64 {
    ephemeris_century / 10.0
}

/// Convert a fractional day value to local hour of the day.
///
/// This function takes a day fraction (where 0.0 represents midnight and 1.0
/// represents the next midnight) and a timezone offset, then computes the
/// corresponding local hour of the day (0.0 to 24.0).
///
/// The function normalizes the result to ensure it stays within a single day
/// (wrapping around if the timezone offset pushes it past midnight).
///
/// # Arguments
///
/// * `day_fraction` - A fractional day value, typically in the range \[0.0, 1.0\],
///   where 0.0 is midnight and 0.5 is noon.
/// * `timezone` - A `FixedOffset` representing the timezone offset from UTC.
///
/// # Returns
///
/// The local hour of the day as a floating-point value in the range \[0.0, 24.0\],
/// where 0.0 is midnight, 12.0 is noon, etc.
pub(crate) fn dayfrac_to_local_hr(day_fraction: f64, timezone: FixedOffset) -> f64 {
    let timezone_hours = timezone.local_minus_utc() as f64 / 3600.0;
    24.0 * normalize_unit_interval(day_fraction + timezone_hours / 24.0)
}
