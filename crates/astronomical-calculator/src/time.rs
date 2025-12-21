use chrono::{DateTime, Datelike, Offset, TimeZone, Timelike};
#[allow(unused_imports)]
use core_maths::CoreFloat;

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
    let tz_hours = f64::from(date.offset().fix().local_minus_utc()) / 3600.0;

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
    let mut julian_day = (f64::from((365.25 * (f64::from(year) + 4716.0)) as i32)
        + f64::from((30.6001 * f64::from(month + 1)) as i32))
        + day_decimal
        - 1524.5;
    // Gregorian calendar correction for dates on/after 1582-10-15
    if julian_day > 2_299_160.0 {
        let a = f64::from(year / 100i32);
        julian_day += 2.0 - a + f64::from((a / 4.0) as i32);
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

/// ΔT (Delta T) estimation functions.
///
/// ΔT represents the difference between Terrestrial Time (TT) and Universal Time (UT1).
/// These estimates are based on Espenak and Meeus polynomial fits updated in 2014.
pub mod delta_t {
    use chrono::Datelike;

    use crate::math::polynomial;

    /// Estimates ΔT for a given decimal year.
    ///
    /// Based on polynomial fits from Espenak & Meeus, updated 2014.
    /// See: <https://www.eclipsewise.com/help/deltatpoly2014.html>
    ///
    /// # Arguments
    /// * `decimal_year` - Year with fractional part (e.g., 2024.5 for mid-2024)
    ///
    /// # Returns
    /// Estimated ΔT in seconds
    ///
    /// # Errors
    /// Returns error for years outside the valid range (-500 to 3000 CE)
    #[allow(clippy::too_many_lines)] // Comprehensive polynomial fit across historical periods
    #[must_use]
    pub fn estimate(decimal_year: f64) -> Option<f64> {
        let year = decimal_year;

        if !year.is_finite() {
            return None;
        }

        if year < -500.0 {
            return None;
        }

        let delta_t = if year < 500.0 {
            let u = year / 100.0;
            polynomial(
                &[
                    10583.6,
                    -1014.41,
                    33.78311,
                    -5.952053,
                    -0.1798452,
                    0.022174192,
                    0.0090316521,
                ],
                u,
            )
        } else if year < 1600.0 {
            let u = (year - 1000.0) / 100.0;
            polynomial(
                &[
                    1574.2,
                    -556.01,
                    71.23472,
                    0.319781,
                    -0.8503463,
                    -0.005050998,
                    0.0083572073,
                ],
                u,
            )
        } else if year < 1700.0 {
            let t = year - 1600.0;
            polynomial(&[120.0, -0.9808, -0.01532, 1.0 / 7129.0], t)
        } else if year < 1800.0 {
            let t = year - 1700.0;
            polynomial(&[8.83, 0.1603, -0.0059285, 0.00013336, -1.0 / 1_174_000.0], t)
        } else if year < 1860.0 {
            let t = year - 1800.0;
            polynomial(
                &[
                    13.72,
                    -0.332447,
                    0.0068612,
                    0.0041116,
                    -0.00037436,
                    0.0000121272,
                    -0.0000001699,
                    0.000000000875,
                ],
                t,
            )
        } else if year < 1900.0 {
            let t = year - 1860.0;
            polynomial(
                &[7.62, 0.5737, -0.251754, 0.01680668, -0.0004473624, 1.0 / 233_174.0],
                t,
            )
        } else if year < 1920.0 {
            let t = year - 1900.0;
            polynomial(&[-2.79, 1.494119, -0.0598939, 0.0061966, -0.000197], t)
        } else if year < 1941.0 {
            let t = year - 1920.0;
            polynomial(&[21.20, 0.84493, -0.076100, 0.0020936], t)
        } else if year < 1961.0 {
            let t = year - 1950.0;
            polynomial(&[29.07, 0.407, -1.0 / 233.0, 1.0 / 2547.0], t)
        } else if year < 1986.0 {
            let t = year - 1975.0;
            polynomial(&[45.45, 1.067, -1.0 / 260.0, -1.0 / 718.0], t)
        } else if year < 2005.0 {
            let t = year - 2000.0;
            polynomial(&[63.86, 0.3345, -0.060374, 0.0017275, 0.000651814, 0.00002373599], t)
        } else if year < 2015.0 {
            let t = year - 2005.0;
            polynomial(&[64.69, 0.2930], t)
        } else if year <= 3000.0 {
            let t = year - 2015.0;
            polynomial(&[67.62, 0.3645, 0.0039755], t)
        } else {
            return None;
        };

        Some(delta_t)
    }

    /// Estimates ΔT from year and month.
    ///
    /// Calculates decimal year as: year + (month - 0.5) / 12
    ///
    /// # Arguments
    /// * `year` - Year
    /// * `month` - Month (1-12)
    ///
    /// # Returns
    /// Returns estimated ΔT in seconds.
    ///
    /// # Errors
    /// Returns error if month is outside the range 1-12.
    ///
    /// # Panics
    /// This function does not panic.
    #[must_use]
    pub fn estimate_from_date(year: i32, month: u32) -> Option<f64> {
        if !(1..=12).contains(&month) {
            return None;
        }

        let decimal_year = f64::from(year) + (f64::from(month) - 0.5) / 12.0;
        estimate(decimal_year)
    }

    /// Estimates ΔT from any date-like type.
    ///
    /// Convenience method that extracts the year and month from any chrono type
    /// that implements `Datelike` (`DateTime`, `NaiveDateTime`, `NaiveDate`, etc.).
    ///
    /// # Arguments
    /// * `date` - Any date-like type
    ///
    /// # Returns
    /// Returns estimated ΔT in seconds.
    ///
    /// # Errors
    /// Returns error if the date components are invalid.
    ///
    #[allow(clippy::needless_pass_by_value)]
    pub fn estimate_from_date_like<D: Datelike>(date: &D) -> Option<f64> {
        estimate_from_date(date.year(), date.month())
    }
    #[allow(clippy::unwrap_used, unused_imports)]
    mod tests {
        use crate::delta_t::{estimate, estimate_from_date, estimate_from_date_like};

        #[test]
        fn delta_t_modern_estimates() {
            // Test some known ranges
            let delta_t_2000 = estimate(2000.0).unwrap();
            let delta_t_2020 = estimate(2020.0).unwrap();

            assert!(delta_t_2000 > 60.0 && delta_t_2000 < 70.0);
            assert!(delta_t_2020 > 65.0 && delta_t_2020 < 75.0);
            assert!(delta_t_2020 > delta_t_2000); // ΔT is generally increasing
        }

        #[test]
        fn delta_t_historical_estimates() {
            let delta_t_1900 = estimate(1900.0).unwrap();
            let delta_t_1950 = estimate(1950.0).unwrap();

            assert!(delta_t_1900 < 0.0); // Negative in early 20th century
            assert!(delta_t_1950 > 25.0 && delta_t_1950 < 35.0);
        }

        #[test]
        fn delta_t_boundary_conditions() {
            // Test edge cases
            assert!(estimate(-500.0).is_some());
            assert!(estimate(3000.0).is_some());
            assert!(estimate(-501.0).is_none());
            assert!(estimate(3001.0).is_none()); // Should fail beyond 3000
        }

        #[test]
        fn delta_t_from_date() {
            let delta_t = estimate_from_date(2024, 6).unwrap();
            let delta_t_decimal = estimate(2024.5 - 1.0 / 24.0).unwrap(); // June = month 6, so (6-0.5)/12 ≈ 0.458

            // Should be very close
            assert!((delta_t - delta_t_decimal).abs() < 0.01);

            // Test invalid month
            assert!(estimate_from_date(2024, 13).is_none());
            assert!(estimate_from_date(2024, 0).is_none());
        }

        #[test]
        fn delta_t_from_date_like() {
            use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

            // Test with DateTime<FixedOffset>
            let datetime_fixed = "2024-06-15T12:00:00-07:00".parse::<DateTime<FixedOffset>>().unwrap();
            let delta_t_fixed = estimate_from_date_like(&datetime_fixed).unwrap();

            // Test with DateTime<Utc>
            let datetime_utc = "2024-06-15T19:00:00Z".parse::<DateTime<Utc>>().unwrap();
            let delta_t_utc = estimate_from_date_like(&datetime_utc).unwrap();

            // Test with NaiveDate
            let naive_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
            let delta_t_naive_date = estimate_from_date_like(&naive_date).unwrap();

            // Test with NaiveDateTime
            let naive_datetime = naive_date.and_hms_opt(12, 0, 0).unwrap();
            let delta_t_naive_datetime = estimate_from_date_like(&naive_datetime).unwrap();

            // Should all be identical since we only use year/month
            assert_eq!(delta_t_fixed, delta_t_utc);
            assert_eq!(delta_t_fixed, delta_t_naive_date);
            assert_eq!(delta_t_fixed, delta_t_naive_datetime);

            // Should match estimate_from_date
            let delta_t_date = estimate_from_date(2024, 6).unwrap();
            assert_eq!(delta_t_fixed, delta_t_date);

            // Verify reasonable range for 2024
            assert!(delta_t_fixed > 60.0 && delta_t_fixed < 80.0);
        }
    }
}
