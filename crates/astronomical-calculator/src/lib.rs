//! # Astronomical Calculator
//!
//! A high-precision solar position calculator for determining the position of the Sun
//! in the sky at any given time and location on Earth.
//!
//! This library provides accurate calculations for:
//! - Solar position (azimuth and zenith angles)
//! - Sunrise, sunset, and solar transit times
//! - Atmospheric refraction corrections
//! - Topocentric coordinate adjustments
// #![no_std]

use chrono::NaiveDate;
use chrono::{DateTime, Datelike, Offset, TimeZone};
use thiserror::Error;

use crate::geo::*;
use crate::math::*;
use crate::terms::*;
use crate::time::*;

#[cfg(test)]
extern crate std;

mod geo;
mod math;
mod terms;
#[cfg(all(feature = "__spa-sys", test))]
mod tests;
mod time;

pub use time::delta_t;

// ============================================================================
// Constants
// ============================================================================

/// Earth rotation rate for rise/transit/set calculations (degrees per day)
const EARTH_ROTATION_RATE_DEGREES_PER_DAY: f64 = 360.985647;

/// Seconds per day
const SECONDS_PER_DAY: f64 = 86400.0;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during solar position calculations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid longitude: {0} (expected -180.0..=180.0)")]
    InvalidLongitude(f64),

    #[error("invalid latitude: {0} (expected -90.0..=90.0)")]
    InvalidLatitude(f64),

    #[error("invalid year: {0} (expected -2000..=6000)")]
    InvalidYear(i32),

    #[error("invalid pressure: {0} (expected 0.0..=5000.0)")]
    InvalidPressure(f64),

    #[error("invalid temperature: {0} (expected -273.0..=6000.0)")]
    InvalidTemperature(f64),

    #[error("invalid delta_ut1: {0} (expected -1.0..=1.0)")]
    InvalidDeltaUt1(f64),

    #[error("invalid delta_t: {0} (expected -8000.0..=8000.0)")]
    InvalidDeltaT(f64),

    #[error("invalid atmos_refract: {0} (expected -5.0..=5.0)")]
    InvalidAtmosRefract(f64),

    #[error("invalid elevation: {0} (expected -6500000.0..=6500000.0)")]
    InvalidElevation(f64),

    #[error("invalid slope: {0} (expected -360.0..=360.0)")]
    InvalidSlope(f64),

    #[error("invalid azm_rotation: {0} (expected -360.0..=360.0)")]
    InvalidAzmRotation(f64),

    #[error("invalid timezone: {0} (expected -18.0..=18.0 hours)")]
    InvalidTimezone(f64),

    #[error("unable to estimate delta_t")]
    UnableToEstimateDeltaT,
}

// ============================================================================
// Input Parameters
// ============================================================================

/// Input parameters for solar position calculations.
///
/// This struct contains all the necessary parameters to calculate the solar position
/// including observer location, atmospheric conditions, and time corrections.
pub struct AstronomicalCalculator<T: TimeZone> {
    /// Date and time for the calculation (with timezone information)
    pub datetime: DateTime<T>,

    /// Difference between UT and UT1 in seconds (fractional second of a day)
    /// Used for Earth rotation variations. Typical range: -1.0 to +1.0 seconds
    pub delta_ut1: f64,

    /// Difference between Terrestrial Time (TT) and Universal Time (UT) in seconds
    /// Accounts for variations in Earth's rotation. For 2024, approximately 69 seconds
    pub delta_t: f64,

    /// Observer's longitude in degrees (positive East, negative West)
    /// Range: -180.0 to +180.0
    pub longitude: f64,

    /// Observer's latitude in degrees (positive North, negative South)
    /// Range: -90.0 to +90.0
    pub latitude: f64,

    /// Observer's elevation above sea level in meters
    /// Range: -6,500,000.0 to +6,500,000.0
    pub elevation: f64,

    /// Local atmospheric pressure in millibars
    /// Range: 0.0 to 5000.0 (standard pressure at sea level is 1013.25 mb)
    pub pressure: f64,

    /// Local temperature in Celsius
    /// Range: -273.0 to +6000.0
    pub temperature: f64,

    /// Surface slope in degrees (0 = horizontal, 90 = vertical)
    /// Range: -360.0 to +360.0
    pub slope: f64,

    /// Surface azimuth rotation in degrees (measured from south)
    /// Range: -360.0 to +360.0
    pub azm_rotation: f64,

    /// Atmospheric refraction at sunrise and sunset in degrees
    /// Standard value is 0.5667 degrees (34 arcminutes)
    /// Range: -5.0 to +5.0
    pub atmos_refract: f64,
}

impl<T: TimeZone> AstronomicalCalculator<T> {
    /// Creates a new `AstronomicalCalculator` struct with validation.
    ///
    /// # Arguments
    ///
    /// * `datetime` - Date and time for the calculation
    /// * `delta_ut1` - UT1-UT correction in seconds
    /// * `delta_t` - TT-UT correction in seconds  
    /// * `longitude` - Observer longitude in degrees (positive East)
    /// * `latitude` - Observer latitude in degrees (positive North)
    /// * `elevation` - Elevation above sea level in meters
    /// * `pressure` - Atmospheric pressure in millibars
    /// * `temperature` - Temperature in Celsius
    /// * `slope` - Surface slope in degrees
    /// * `azm_rotation` - Surface azimuth rotation in degrees
    /// * `atmos_refract` - Atmospheric refraction correction in degrees
    ///
    /// # Returns
    ///
    /// * `Ok(AstronomicalCalculator)` - Valid input parameters
    /// * `Err(Error)` - If any parameter is out of valid range
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        datetime: DateTime<T>,
        delta_ut1: f64,
        delta_t: f64,
        longitude: f64,
        latitude: f64,
        elevation: f64,
        pressure: f64,
        temperature: f64,
        slope: f64,
        azm_rotation: f64,
        atmos_refract: f64,
    ) -> Result<Self, Error> {
        // Validate year range
        if datetime.year() < -2000 || datetime.year() > 6000 {
            return Err(Error::InvalidYear(datetime.year()));
        }

        // Validate atmospheric conditions
        if !(0.0..=5000.0).contains(&pressure) {
            return Err(Error::InvalidPressure(pressure));
        }
        if !(-273.0..=6000.0).contains(&temperature) {
            return Err(Error::InvalidTemperature(temperature));
        }

        // Validate time corrections
        if delta_ut1 <= -1.0 || delta_ut1 > 1.0 {
            return Err(Error::InvalidDeltaUt1(delta_ut1));
        }
        if delta_t.abs() > 8000.0 {
            return Err(Error::InvalidDeltaT(delta_t));
        }

        // Validate geographic coordinates
        if longitude.abs() > 180.0 {
            return Err(Error::InvalidLongitude(longitude));
        }
        if latitude.abs() > 90.0 {
            return Err(Error::InvalidLatitude(latitude));
        }
        if elevation < -6500000.0 {
            return Err(Error::InvalidElevation(elevation));
        }

        // Validate surface parameters
        if slope.abs() > 360.0 {
            return Err(Error::InvalidSlope(slope));
        }
        if azm_rotation.abs() > 360.0 {
            return Err(Error::InvalidAzmRotation(azm_rotation));
        }
        if atmos_refract.abs() > 5.0 {
            return Err(Error::InvalidAtmosRefract(atmos_refract));
        }

        // Validate timezone offset
        let timezone_hours = datetime.offset().fix().local_minus_utc() as f64 / 3600.0;
        if timezone_hours.abs() > 18.0 {
            return Err(Error::InvalidTimezone(timezone_hours));
        }

        Ok(Self {
            datetime,
            delta_ut1,
            delta_t,
            longitude,
            latitude,
            elevation,
            pressure,
            temperature,
            slope,
            azm_rotation,
            atmos_refract,
        })
    }
    /// Creates a new `AstronomicalCalculator` with standard atmospheric conditions and automatic ΔT estimation.
    ///
    /// This convenience constructor uses typical atmospheric values (1013.25 mbar pressure, 15°C temperature,
    /// 0.5667° refraction) and automatically estimates ΔT (the difference between Terrestrial Time and
    /// Universal Time) based on the date.
    ///
    /// # Arguments
    ///
    /// * `datetime` - Date and time for the calculation. **Year must be between -500 and 3000** for ΔT estimation.
    /// * `longitude` - Observer longitude in degrees (positive East, range: -180 to 180)
    /// * `latitude` - Observer latitude in degrees (positive North, range: -90 to 90)
    /// * `elevation` - Elevation above sea level in meters (range: -6500000 to ∞)
    ///
    /// # Returns
    ///
    /// * `Ok(AstronomicalCalculator)` - Valid input parameters
    /// * `Err(Error)` - If any parameter is out of valid range or year is outside [-500, 3000]
    pub fn standard(datetime: DateTime<T>, longitude: f64, latitude: f64, elevation: f64) -> Result<Self, Error> {
        if let Some(delta_t) = delta_t::estimate_from_date_like(&datetime) {
            Ok(Self {
                datetime,
                delta_ut1: 0.0,
                delta_t,
                longitude,
                latitude,
                elevation,
                pressure: 1013.25,
                temperature: 15.0,
                slope: 0.0,
                azm_rotation: 0.0,
                atmos_refract: 0.5667,
            })
        } else {
            Err(Error::UnableToEstimateDeltaT)
        }
    }

    /// Calculates the complete solar position for the given inputs.
    ///
    /// This is the main entry point for solar position calculations. It returns
    /// zenith angle, azimuth, and sunrise/sunset times.
    ///
    /// The solar position (zenith, azimuth, incidence) is always calculated.
    /// However, sunrise/sunset/transit times may be `None` if the sun doesn't
    /// rise or set at this location and date (e.g., polar day/night).
    pub fn calculate_solar_position(&self) -> SolarPositionOutput<T> {
        let intermediates = self.calculate_intermediate_solar_values();

        // Calculate observer's local hour angle
        let hour_angle = calculate_observer_hour_angle(
            intermediates.greenwich_apparent_sidereal_time,
            self.longitude,
            intermediates.geocentric_right_ascension,
        );

        // Calculate parallax correction
        let equatorial_horizontal_parallax =
            calculate_sun_equatorial_horizontal_parallax(intermediates.earth_sun_distance);

        let (topocentric_declination, right_ascension_parallax) =
            calculate_right_ascension_parallax_and_topocentric_declination(
                self.latitude,
                self.elevation,
                equatorial_horizontal_parallax,
                hour_angle,
                intermediates.geocentric_declination,
            );

        // Calculate topocentric coordinates
        let topocentric_hour_angle = calculate_topocentric_local_hour_angle(hour_angle, right_ascension_parallax);
        let elevation_uncorrected =
            calculate_topocentric_elevation_angle(self.latitude, topocentric_declination, topocentric_hour_angle);

        // Apply atmospheric refraction correction
        let refraction_correction = calculate_atmospheric_refraction_correction(
            self.pressure,
            self.temperature,
            self.atmos_refract,
            elevation_uncorrected,
        );
        let elevation_corrected = apply_atmospheric_refraction_correction(elevation_uncorrected, refraction_correction);

        // Calculate final position angles
        let zenith = calculate_zenith_angle_from_elevation(elevation_corrected);
        let azimuth_astronomical =
            calculate_topocentric_azimuth_astronomical(topocentric_hour_angle, self.latitude, topocentric_declination);
        let azimuth = convert_astronomical_to_observer_azimuth(azimuth_astronomical);
        let incidence = calculate_surface_incidence_angle(zenith, azimuth_astronomical, self.azm_rotation, self.slope);

        // Calculate rise, transit, and set times (may be None for polar day/night)
        let rts_times = self.calculate_rise_transit_set_times(&intermediates);
        if let Some(rts_times) = rts_times {
            let solar_transit_time = rts_times.solar_transit_time;
            let sunrise_time = rts_times.sunrise_time;
            let sunset_time = rts_times.sunset_time;
            SolarPositionOutput {
                zenith,
                azimuth_astronomical,
                azimuth,
                incidence,
                solar_transit_time: Some(solar_transit_time),
                sunrise_time: Some(sunrise_time),
                sunset_time: Some(sunset_time),
            }
        } else {
            SolarPositionOutput {
                zenith,
                azimuth_astronomical,
                azimuth,
                incidence,
                solar_transit_time: None,
                sunrise_time: None,
                sunset_time: None,
            }
        }
    }

    /// Calculates intermediate astronomical values needed for solar position.
    ///
    /// This includes Julian dates, Earth's position, nutation, obliquity, and
    /// geocentric solar coordinates.
    fn calculate_intermediate_solar_values(&self) -> SolarPositionIntermediates {
        let julian_day = julian_day(&self.datetime, self.delta_ut1);
        self.calculate_intermediates_for_julian_day(julian_day)
    }

    /// Calculates intermediate values for a specific Julian Day.
    ///
    /// This is used both for the target datetime and for surrounding days
    /// needed for rise/transit/set calculations.
    fn calculate_intermediates_for_julian_day(&self, julian_day: f64) -> SolarPositionIntermediates {
        // Time calculations
        let julian_century = julian_century_from_julian_day(julian_day);
        let julian_ephemeris_day = time::julian_ephemeris_day_from_julian_day(julian_day, self.delta_t);
        let julian_ephemeris_century = julian_ephemeris_century_from_julian_ephemeris_day(julian_ephemeris_day);
        let julian_ephemeris_millennium =
            julian_ephemeris_millennium_from_julian_ephemeris_century(julian_ephemeris_century);

        // Earth's heliocentric position
        let earth_heliocentric_longitude = calculate_earth_heliocentric_longitude(julian_ephemeris_millennium);
        let earth_heliocentric_latitude = calculate_earth_heliocentric_latitude(julian_ephemeris_millennium);
        let earth_sun_distance = calculate_earth_radius_vector(julian_ephemeris_millennium);

        // Convert to geocentric coordinates (Sun's position as seen from Earth's center)
        let sun_geocentric_longitude = convert_heliocentric_to_geocentric_longitude(earth_heliocentric_longitude);
        let sun_geocentric_latitude = convert_heliocentric_to_geocentric_latitude(earth_heliocentric_latitude);

        // Nutation and obliquity calculations
        let mean_elongation = calculate_mean_elongation_moon_sun(julian_ephemeris_century);
        let mean_anomaly_sun = calculate_mean_anomaly_sun(julian_ephemeris_century);
        let mean_anomaly_moon = calculate_mean_anomaly_moon(julian_ephemeris_century);
        let moon_argument_of_latitude = calculate_argument_latitude_moon(julian_ephemeris_century);
        let moon_ascending_node_longitude = calculate_ascending_longitude_moon(julian_ephemeris_century);

        let (nutation_longitude, nutation_obliquity) = calculate_nutation_longitude_and_obliquity(
            julian_ephemeris_century,
            [
                mean_elongation,
                mean_anomaly_sun,
                mean_anomaly_moon,
                moon_argument_of_latitude,
                moon_ascending_node_longitude,
            ],
        );

        let ecliptic_mean_obliquity = calculate_ecliptic_mean_obliquity(julian_ephemeris_millennium);
        let ecliptic_true_obliquity = calculate_ecliptic_true_obliquity(nutation_obliquity, ecliptic_mean_obliquity);

        // Aberration and apparent position
        let aberration = calculate_aberration_correction(earth_sun_distance);
        let sun_apparent_longitude =
            calculate_apparent_sun_longitude(sun_geocentric_longitude, nutation_longitude, aberration);

        // Sidereal time
        let greenwich_mean_sidereal_time = calculate_greenwich_mean_sidereal_time(julian_day, julian_century);
        let greenwich_apparent_sidereal_time = calculate_greenwich_apparent_sidereal_time(
            greenwich_mean_sidereal_time,
            nutation_longitude,
            ecliptic_true_obliquity,
        );

        // Geocentric equatorial coordinates
        let geocentric_right_ascension = calculate_geocentric_right_ascension(
            sun_apparent_longitude,
            ecliptic_true_obliquity,
            sun_geocentric_latitude,
        );
        let geocentric_declination =
            calculate_geocentric_declination(sun_geocentric_latitude, ecliptic_true_obliquity, sun_apparent_longitude);

        SolarPositionIntermediates {
            julian_day,
            julian_century,
            julian_ephemeris_day,
            julian_ephemeris_century,
            julian_ephemeris_millennium,
            earth_heliocentric_longitude,
            earth_heliocentric_latitude,
            earth_sun_distance,
            sun_geocentric_longitude,
            sun_geocentric_latitude,
            mean_elongation,
            mean_anomaly_sun,
            mean_anomaly_moon,
            moon_argument_of_latitude,
            moon_ascending_node_longitude,
            nutation_longitude,
            nutation_obliquity,
            ecliptic_mean_obliquity,
            ecliptic_true_obliquity,
            aberration,
            sun_apparent_longitude,
            greenwich_mean_sidereal_time,
            greenwich_apparent_sidereal_time,
            geocentric_right_ascension,
            geocentric_declination,
        }
    }

    /// Calculates sunrise, sunset, and solar transit (noon) times.
    ///
    /// Uses iterative refinement to calculate accurate rise, transit, and set times
    /// based on interpolated solar coordinates.
    ///
    /// # Returns
    ///
    /// * `Some(RiseTransitSetTimes)` - If the sun rises and sets
    /// * `None` - If the sun doesn't rise or set (polar day/night)
    fn calculate_rise_transit_set_times(
        &self,
        intermediates: &SolarPositionIntermediates,
    ) -> Option<RiseTransitSetTimes<T>> {
        // Calculate equation of time for the target date
        let _equation_of_time = equation_of_time(
            calculate_sun_mean_longitude(intermediates.julian_ephemeris_millennium),
            intermediates.geocentric_right_ascension,
            intermediates.nutation_longitude,
            intermediates.ecliptic_true_obliquity,
        );

        // Create adjusted inputs for midnight of the target date
        let mut midnight_inputs = AstronomicalCalculator {
            // It is not possible to fail to convert a date to a naive date for a naive datetime
            #[allow(clippy::unwrap_used)]
            datetime: NaiveDate::from_ymd_opt(self.datetime.year(), self.datetime.month(), self.datetime.day())
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            delta_ut1: 0.0,
            delta_t: self.delta_t,
            longitude: self.longitude,
            latitude: self.latitude,
            elevation: self.elevation,
            pressure: self.pressure,
            temperature: self.temperature,
            slope: self.slope,
            azm_rotation: self.azm_rotation,
            atmos_refract: self.atmos_refract,
        };

        let midnight_intermediates = midnight_inputs.calculate_intermediate_solar_values();
        let sidereal_time_at_midnight = midnight_intermediates.greenwich_apparent_sidereal_time;

        // Calculate right ascension and declination for 3 days (yesterday, today, tomorrow)
        // This is needed for interpolation in the rise/transit/set algorithm
        midnight_inputs.delta_t = 0.0;
        let mut right_ascension_3days = [0.0; JD_COUNT];
        let mut declination_3days = [0.0; JD_COUNT];
        let mut current_julian_day = midnight_intermediates.julian_day - 1.0;

        for i in 0..JD_COUNT {
            let day_intermediates = midnight_inputs.calculate_intermediates_for_julian_day(current_julian_day);
            right_ascension_3days[i] = day_intermediates.geocentric_right_ascension;
            declination_3days[i] = day_intermediates.geocentric_declination;
            current_julian_day += 1.0;
        }

        // Calculate approximate transit time and hour angle at rise/set
        let mut approx_times: [f64; 3] = [0.0; 3];
        approx_times[SUN_TRANSIT] =
            calculate_approximate_sun_transit_time(right_ascension_3days[1], self.longitude, sidereal_time_at_midnight);

        let hour_angle_at_horizon = calculate_sun_hour_angle_at_rise_set(
            self.latitude,
            declination_3days[1],
            -(SUN_RADIUS + self.atmos_refract),
        )?;

        // Refine the approximate times
        calculate_approximate_sun_rise_and_set(&mut approx_times, hour_angle_at_horizon);
        println!("approx_times: {:?}", approx_times);
        println!("normalized_sunrise: {:?}", approx_times[SUN_RISE] * 24.0);
        // Calculate more precise values for rise, transit, and set times
        let mut sidereal_times_rts: [f64; 3] = [0.0; 3];
        let mut interpolated_right_ascension: [f64; 3] = [0.0; 3];
        let mut interpolated_declination: [f64; 3] = [0.0; 3];
        let mut topocentric_hour_angles: [f64; 3] = [0.0; 3];
        let mut calculated_altitudes: [f64; 3] = [0.0; 3];

        for i in 0..3 {
            sidereal_times_rts[i] = sidereal_time_at_midnight + EARTH_ROTATION_RATE_DEGREES_PER_DAY * approx_times[i];
            let time_fraction = approx_times[i] + self.delta_t / SECONDS_PER_DAY;

            interpolated_right_ascension[i] =
                interpolate_right_ascension_declination_for_rts(&mut right_ascension_3days, time_fraction);
            interpolated_declination[i] =
                interpolate_right_ascension_declination_for_rts(&mut declination_3days, time_fraction);

            topocentric_hour_angles[i] =
                normalize_degrees_180pm(sidereal_times_rts[i] + self.longitude - interpolated_right_ascension[i]);

            calculated_altitudes[i] = calculate_sun_altitude_for_rise_transit_set(
                self.latitude,
                interpolated_declination[i],
                topocentric_hour_angles[i],
            );
        }

        debug_assert!(
            approx_times[SUN_RISE] < approx_times[SUN_TRANSIT] && approx_times[SUN_TRANSIT] < approx_times[SUN_SET],
            "sunrise must be before transit, and transit must be before sunset"
        );

        // Calculate final times in local time
        let solar_transit_fraction = approx_times[SUN_TRANSIT] - topocentric_hour_angles[SUN_TRANSIT] / 360.0;

        let sunrise_fraction = calculate_sun_rise_and_set_time(
            approx_times,
            calculated_altitudes,
            interpolated_declination,
            self.latitude,
            topocentric_hour_angles,
            -(SUN_RADIUS + self.atmos_refract),
            SUN_RISE,
        );
        let sunset_fraction = calculate_sun_rise_and_set_time(
            approx_times,
            calculated_altitudes,
            interpolated_declination,
            self.latitude,
            topocentric_hour_angles,
            -(SUN_RADIUS + self.atmos_refract),
            SUN_SET,
        );
        if sunrise_fraction > solar_transit_fraction || solar_transit_fraction > sunset_fraction {
            return None;
        }

        let mut local_sunrise = foobar(&self.datetime, &midnight_inputs.datetime, sunrise_fraction);
        let mut local_solar_transit = foobar(&self.datetime, &midnight_inputs.datetime, solar_transit_fraction);
        let mut local_sunset = foobar(&self.datetime, &midnight_inputs.datetime, sunset_fraction);

        // The solar transit must occur on the same date as the target date.
        let diff = (local_solar_transit.date_naive() - self.datetime.date_naive()).num_days();
        if diff != 0 {
            local_sunrise = foobar(
                &self.datetime,
                &midnight_inputs.datetime,
                sunrise_fraction - diff as f64,
            );
            local_solar_transit = foobar(
                &self.datetime,
                &midnight_inputs.datetime,
                solar_transit_fraction - diff as f64,
            );
            local_sunset = foobar(&self.datetime, &midnight_inputs.datetime, sunset_fraction - diff as f64);
        }
        debug_assert!(
            local_solar_transit.date_naive() == self.datetime.date_naive(),
            "solar transit must occur on the same date as the target date, got {:?} and {:?}",
            local_solar_transit.date_naive(),
            self.datetime.date_naive()
        );

        if local_sunrise < local_solar_transit && local_solar_transit < local_sunset {
            Some(RiseTransitSetTimes {
                equation_of_time: _equation_of_time,
                solar_transit_time: local_solar_transit,
                sunrise_time: local_sunrise,
                sunset_time: local_sunset,
                sunrise_hour_angle: topocentric_hour_angles[SUN_RISE],
                sunset_hour_angle: topocentric_hour_angles[SUN_SET],
                transit_altitude: calculated_altitudes[SUN_TRANSIT],
            })
        } else {
            None
        }
    }
}

// ============================================================================
// Intermediate Values
// ============================================================================

/// Intermediate astronomical values calculated during solar position determination.
///
/// These values represent various astronomical parameters computed from
/// the observer's time and location.
struct SolarPositionIntermediates {
    // Time values
    julian_day: f64,
    #[allow(unused)]
    julian_century: f64,
    #[allow(unused)]
    julian_ephemeris_day: f64,
    #[allow(unused)]
    julian_ephemeris_century: f64,
    julian_ephemeris_millennium: f64,

    // Earth's position
    #[allow(unused)]
    earth_heliocentric_longitude: f64,
    #[allow(unused)]
    earth_heliocentric_latitude: f64,
    earth_sun_distance: f64,

    // Sun's geocentric position
    #[allow(unused)]
    sun_geocentric_longitude: f64,
    #[allow(unused)]
    sun_geocentric_latitude: f64,

    // Lunar orbital parameters (for nutation)
    #[allow(unused)]
    mean_elongation: f64,
    #[allow(unused)]
    mean_anomaly_sun: f64,
    #[allow(unused)]
    mean_anomaly_moon: f64,
    #[allow(unused)]
    moon_argument_of_latitude: f64,
    #[allow(unused)]
    moon_ascending_node_longitude: f64,

    // Nutation and obliquity
    nutation_longitude: f64,
    #[allow(unused)]
    nutation_obliquity: f64,
    #[allow(unused)]
    ecliptic_mean_obliquity: f64,
    ecliptic_true_obliquity: f64,

    // Aberration and apparent position
    #[allow(unused)]
    aberration: f64,
    #[allow(unused)]
    sun_apparent_longitude: f64,

    // Sidereal time
    #[allow(unused)]
    greenwich_mean_sidereal_time: f64,
    greenwich_apparent_sidereal_time: f64,

    // Geocentric equatorial coordinates
    geocentric_right_ascension: f64,
    geocentric_declination: f64,
}

// ============================================================================
// Output Values
// ============================================================================

/// Solar position output containing position angles and rise/transit/set times.
///
/// All angles are in degrees, and times are in hours (decimal format).
#[derive(Debug, PartialEq)]
pub struct SolarPositionOutput<T: TimeZone> {
    /// Solar zenith angle in degrees (angle from vertical)
    /// 0° = directly overhead, 90° = at horizon
    pub zenith: f64,

    /// Astronomical azimuth in degrees (measured from south, eastward positive)
    /// 0° = south, 90° = west, 180° = north, 270° = east
    pub azimuth_astronomical: f64,

    /// Observer azimuth in degrees (measured from north, eastward positive)
    /// 0° = north, 90° = east, 180° = south, 270° = west
    pub azimuth: f64,

    /// Surface incidence angle in degrees (angle between sun and surface normal)
    pub incidence: f64,

    /// Solar transit time (solar noon) in local hours (decimal)
    /// `None` if the sun doesn't rise or set (polar day/night)
    pub solar_transit_time: Option<DateTime<T>>,

    /// Sunrise time in local hours (decimal)
    /// `None` if the sun doesn't rise or set (polar day/night)
    pub sunrise_time: Option<DateTime<T>>,

    /// Sunset time in local hours (decimal)
    /// `None` if the sun doesn't rise or set (polar day/night)
    pub sunset_time: Option<DateTime<T>>,
}

/// Rise, transit, and set times with additional diagnostic information.
struct RiseTransitSetTimes<T: TimeZone> {
    /// Equation of time in minutes
    #[allow(unused)]
    equation_of_time: f64,

    /// Solar transit (noon) time in local hours
    solar_transit_time: DateTime<T>,

    // Solar transit offset from local midnight in hours
    /// Sunrise time in local hours
    sunrise_time: DateTime<T>,

    /// Sunset time in local hours
    sunset_time: DateTime<T>,

    /// Hour angle at sunrise in degrees
    #[allow(unused)]
    sunrise_hour_angle: f64,

    /// Hour angle at sunset in degrees
    #[allow(unused)]
    sunset_hour_angle: f64,

    /// Solar altitude at transit in degrees
    #[allow(unused)]
    transit_altitude: f64,
}
