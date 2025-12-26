//! # Astronomical Calculator
//!
//! A high-precision library for calculating solar position, sunrise/sunset times, and related astronomical phenomena.
//!
//! This library provides accurate calculations of the Sun's position and timing of solar events for any location
//! on Earth. It accounts for atmospheric refraction, parallax, nutation, aberration, and other astronomical
//! phenomena that affect solar position calculations.
//!
//! ## Basic Usage
//!
//! ```
//! use astronomical_calculator::{AstronomicalCalculator, Refraction};
//! use chrono::NaiveDateTime;
//!
//! // Create a datetime (UTC)
//! let dt = NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
//!
//! // Create calculator for New York City
//! // Note: angles must be in radians
//! let mut calc = AstronomicalCalculator::new(
//!     dt,
//!     Some(69.0),        // delta_t: TT-UT in seconds (≈69s for 2024)
//!     0.0,               // delta_ut1: UT1-UTC in seconds
//!     -74.0_f64.to_radians(),  // longitude (negative = West)
//!     40.7_f64.to_radians(),   // latitude (positive = North)
//!     0.0,               // elevation in meters
//!     15.0,              // temperature in Celsius
//!     1013.0,            // pressure in millibars
//!     None,              // optional geometric dip angle
//!     Refraction::ApSolposBennet,  // refraction model
//! ).unwrap();
//!
//! // Get current solar position
//! let position = calc.get_solar_position();
//! println!("Zenith: {:.2}°", position.zenith.to_degrees());
//! println!("Azimuth: {:.2}°", position.azimuth.to_degrees());
//!
//! // Get sunrise and sunset times (as Unix timestamps)
//! use astronomical_calculator::SolarEventResult;
//! match calc.get_sunrise().unwrap() {
//!     SolarEventResult::Occurs(timestamp) => {
//!         println!("Sunrise at Unix timestamp: {}", timestamp);
//!     }
//!     SolarEventResult::AllDay => println!("Sun never sets (midnight sun)"),
//!     SolarEventResult::AllNight => println!("Sun never rises (polar night)"),
//! }
//! ```
#![no_std]

pub(crate) mod tables;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod unsafe_spa;

use chrono::Datelike;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::Utc;
use julian_day_converter::julian_day_to_unix_millis;
use julian_day_converter::unix_millis_to_julian_day;

use core::cell::OnceCell;
use core::f64::consts::PI;
use core::ops::Rem;

#[allow(unused_imports)]
use core_maths::*;
use thiserror::Error;

use crate::tables::*;

// Julian date constants
const JD0: f64 = 2451545.0; // J2000.0 epoch
const ETJD0: i64 = 946728000; // Unix timestamp for J2000.0 epoch

// Physical constants
const SUN_RADIUS: f64 = 4.654_269_516_293_279e-3_f64; // Sun's angular radius in radians
pub(crate) const EARTH_R: f64 = 6378136.6f64; // Earth's radius in meters
pub(crate) const ABSOLUTEZERO: f64 = -273.15f64; // Absolute zero in Celsius

// Standard atmospheric conditions
const AP0: f64 = 1010.0f64; // Standard pressure in millibars
const AT0: f64 = 10.0f64; // Standard temperature in Celsius

// Iteration and convergence parameters
const FRACDAYSEC: f64 = 1.1574074074074073e-05f64; // Fractional day per second
const MAX_FPITER: i64 = 20; // Max iterations for fixed point
const Z_EPS: f64 = PI * 0.05f64 / 180.0f64; // Zenith convergence tolerance
const MAXRAT: i64 = 2; // Max ratio for bisection adjustment
const Z_MAXITER: i64 = 100; // Max iterations for zenith finding

/// Main calculator for solar position and astronomical events.
///
/// This struct computes solar positions, sunrise/sunset times, twilight times, and solar transit times
/// for a specific location and datetime. Results are cached for efficient repeated access.
///
/// # Example
///
/// ```
/// use astronomical_calculator::{AstronomicalCalculator, Refraction};
/// use chrono::NaiveDateTime;
///
/// let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
///
/// let mut calc = AstronomicalCalculator::new(
///     dt,
///     None,                    // calculate delta_t automatically
///     0.0,                     // delta_ut1 in seconds
///     0.0,                     // longitude: 0° (Greenwich)
///     51.5,                    // latitude: 51.5°N (London)
///     0.0,                     // elevation: sea level
///     20.0,                    // temperature: 20°C
///     1013.25,                 // pressure: 1013.25 mb
///     None,                    // geometric dip: None
///     Refraction::ApSolposBennet,
/// ).unwrap();
///
/// // Get solar position
/// let pos = calc.get_solar_position();
/// assert!(pos.zenith >= 0.0 && pos.zenith <= std::f64::consts::PI);
/// ```
pub struct AstronomicalCalculator {
    ut: NaiveDateTime,
    delta_t: Option<f64>,
    delta_ut1: f64,
    lon_radians: f64,
    lat_radians: f64,
    elevation: f64,
    temperature: f64,
    pressure: f64,
    gdip: Option<f64>,
    refraction: Refraction,
    julian_date: OnceCell<JulianDate>,
    geocentric_position: OnceCell<GeoCentricSolPos>,
    solar_position: OnceCell<SolarPosition>,
    solar_transit: OnceCell<Result<SolarInfo, CalculationError>>,
    prev_solar_midnight: OnceCell<Result<SolarInfo, CalculationError>>,
    next_solar_midnight: OnceCell<Result<SolarInfo, CalculationError>>,
    sunrise: OnceCell<Result<SolarEventResult, CalculationError>>,
    sunset: OnceCell<Result<SolarEventResult, CalculationError>>,
    sea_level_sunrise: OnceCell<Result<SolarEventResult, CalculationError>>,
    sea_level_sunset: OnceCell<Result<SolarEventResult, CalculationError>>,
    civil_dawn: OnceCell<Result<SolarEventResult, CalculationError>>,
    civil_dusk: OnceCell<Result<SolarEventResult, CalculationError>>,
    nautical_dawn: OnceCell<Result<SolarEventResult, CalculationError>>,
    nautical_dusk: OnceCell<Result<SolarEventResult, CalculationError>>,
    astronomical_dawn: OnceCell<Result<SolarEventResult, CalculationError>>,
    astronomical_dusk: OnceCell<Result<SolarEventResult, CalculationError>>,
}

#[derive(Copy, Clone, Debug)]
struct SolarInfo {
    position: SolarPosition,
    timestamp: i64,
}

/// Result of a solar event calculation (sunrise, sunset, twilight, etc.)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SolarEventResult {
    /// Event occurs at the given timestamp in seconds since the Unix epoch
    Occurs(i64),
    /// Sun is always above the threshold (e.g., midnight sun)
    AllDay,
    /// Sun is always below the threshold (e.g., polar night)
    AllNight,
}

impl SolarEventResult {
    /// Extracts the timestamp from a solar event result.
    ///
    /// # Returns
    ///
    /// - `Some(timestamp)` if the event occurs at a specific time
    /// - `None` if the sun is always above or always below the threshold
    ///
    /// # Example
    ///
    /// ```rust
    /// use astronomical_calculator::{AstronomicalCalculator, Refraction};
    /// use chrono::NaiveDateTime;
    ///
    /// let datetime = NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    /// let mut calc = AstronomicalCalculator::new(
    ///     datetime, None, 0.0, 0.0, 0.0, 0.0, 20.0, 1013.25, None, Refraction::ApSolposBennet
    /// ).unwrap();
    ///
    /// if let Some(ts) = calc.get_sunrise().unwrap().timestamp() {
    ///     println!("Sunrise at timestamp: {}", ts);
    /// }
    /// ```
    pub fn timestamp(self) -> Option<i64> {
        match self {
            SolarEventResult::Occurs(ts) => Some(ts),
            _ => None,
        }
    }
}

impl AstronomicalCalculator {
    /// Creates a new astronomical calculator for the given location, time, and atmospheric conditions.
    ///
    /// All input parameters are validated. If any parameter is out of range, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `ut` - Universal Time as a [`NaiveDateTime`]
    /// * `delta_t` - ΔT (TT-UT) in seconds. Use `Some(value)` for known ΔT, or `None` to calculate automatically
    /// * `delta_ut1` - ΔUT1 (UT1-UTC) in seconds, typically in range [-0.9, 0.9]. Use 0.0 if unknown
    /// * `lon` - Longitude in degrees (positive East, negative West)
    /// * `lat` - Latitude in degrees (positive North, negative South)
    /// * `elevation` - Elevation above sea level in meters
    /// * `temperature` - Temperature in Celsius (affects atmospheric refraction)
    /// * `pressure` - Atmospheric pressure in millibars (affects atmospheric refraction)
    /// * `gdip` - Optional geometric dip angle in radians. Use `None` for standard horizon
    /// * `refraction` - Atmospheric refraction model to use
    ///
    /// # Returns
    ///
    /// A `Result` containing the calculator or a [`CalculationError`] if validation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if any parameter is outside its valid range. See [`CalculationError`] for details.
    ///
    /// # Example
    ///
    /// ```
    /// use astronomical_calculator::{AstronomicalCalculator, Refraction, get_delta_t};
    /// use chrono::NaiveDateTime;
    ///
    /// let dt = NaiveDateTime::parse_from_str("2024-06-21 15:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
    ///
    /// // Paris: 48.8566°N, 2.3522°E
    /// let mut calc = AstronomicalCalculator::new(
    ///     dt,
    ///     Some(get_delta_t(&dt)),     // Calculate ΔT automatically
    ///     0.0,                         // ΔUT1 (use 0.0 if unknown)
    ///     2.3522,                      // longitude
    ///     48.8566,                     // latitude
    ///     35.0,                        // elevation (m)
    ///     22.0,                        // temperature (°C)
    ///     1013.25,                     // pressure (mb)
    ///     None,                        // geometric dip
    ///     Refraction::ApSolposBennet,  // refraction model
    /// ).unwrap();
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ut: NaiveDateTime,
        delta_t: Option<f64>,
        delta_ut1: f64,
        lon: f64,
        lat: f64,
        elevation: f64,
        temperature: f64,
        pressure: f64,
        gdip: Option<f64>,
        refraction: Refraction,
    ) -> Result<Self, CalculationError> {
        // Validate year range (-2000 to 6000)
        let year = ut.year();
        if !(-2000..=6000).contains(&year) {
            return Err(CalculationError::TimeConversionError);
        }

        let lon_radians = lon.to_radians();
        let lat_radians = lat.to_radians();
        if !(-1.0..=1.0).contains(&delta_ut1) {
            return Err(CalculationError::DeltaUt1OutOfRange);
        }
        // Validate delta_t range if explicitly provided
        if let Some(dt) = delta_t {
            if !(-8000.0..=8000.0).contains(&dt) {
                return Err(CalculationError::TimeConversionError);
            }
        }
        if !(-PI..=PI).contains(&lon_radians) {
            return Err(CalculationError::LongitudeOutOfRange);
        }
        if !(-PI / 2.0..=PI / 2.0).contains(&lat_radians) {
            return Err(CalculationError::LatitudeOutOfRange);
        }
        if !(-EARTH_R..=EARTH_R).contains(&elevation) {
            return Err(CalculationError::ElevationOutOfRange);
        }
        if pressure <= 0.0 || pressure > 5000.0 {
            return Err(CalculationError::PressureOutOfRange);
        }
        if !(ABSOLUTEZERO..=6000.0).contains(&temperature) {
            return Err(CalculationError::TemperatureOutOfRange);
        }
        // Validate gdip range if provided
        if let Some(gdip_val) = gdip {
            if gdip_val.abs() > PI / 2.0 {
                return Err(CalculationError::GeometricDipOutOfRange);
            }
        }
        Ok(Self {
            ut,
            delta_t,
            delta_ut1,
            lon_radians,
            lat_radians,
            temperature,
            pressure,
            elevation,
            gdip,
            refraction,
            julian_date: OnceCell::new(),
            geocentric_position: OnceCell::new(),
            solar_position: OnceCell::new(),
            solar_transit: OnceCell::new(),
            prev_solar_midnight: OnceCell::new(),
            next_solar_midnight: OnceCell::new(),
            sunrise: OnceCell::new(),
            sunset: OnceCell::new(),
            sea_level_sunrise: OnceCell::new(),
            sea_level_sunset: OnceCell::new(),
            civil_dawn: OnceCell::new(),
            civil_dusk: OnceCell::new(),
            nautical_dawn: OnceCell::new(),
            nautical_dusk: OnceCell::new(),
            astronomical_dawn: OnceCell::new(),
            astronomical_dusk: OnceCell::new(),
        })
    }
    /// Returns the Julian date and related time values.
    ///
    /// This method computes and caches the Julian date representation of the input datetime.
    /// The result is cached, so subsequent calls return the same reference without recomputation.
    ///
    /// # Returns
    ///
    /// A reference to the computed [`JulianDate`] struct.
    ///
    /// # Example
    ///
    /// ```
    /// use astronomical_calculator::{AstronomicalCalculator, Refraction};
    /// use chrono::NaiveDateTime;
    ///
    /// let dt = NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    /// let mut calc = AstronomicalCalculator::new(
    ///     dt, Some(69.0), 0.0, 0.0, 0.0, 0.0, 15.0, 1013.0,
    ///     None, Refraction::ApSolposBennet
    /// ).unwrap();
    ///
    /// let jd = calc.get_julian_day();
    /// println!("Julian Date: {}", jd.jd);
    /// ```
    pub fn get_julian_day(&mut self) -> &JulianDate {
        self.julian_date
            .get_or_init(|| JulianDate::new(self.ut, self.delta_t, self.delta_ut1))
    }

    pub(crate) fn get_geocentric_position(&mut self) -> &GeoCentricSolPos {
        let julian_date = *self.get_julian_day();
        self.geocentric_position
            .get_or_init(|| GeoCentricSolPos::new(&julian_date))
    }
    /// Returns the topocentric solar position (zenith and azimuth angles).
    ///
    /// Computes the Sun's position as seen from the observer's location, accounting for
    /// atmospheric refraction, parallax, nutation, and aberration.
    ///
    /// # Returns
    ///
    /// A reference to the computed [`SolarPosition`] with zenith and azimuth angles in radians.
    /// The result is cached for efficient repeated access.
    pub fn get_solar_position(&mut self) -> &SolarPosition {
        let julian_date = *self.get_julian_day();
        let gp = *self.get_geocentric_position();
        self.solar_position.get_or_init(|| {
            let dtau = PI * (-20.4898f64 / 3600.0) / 180.0 / gp.rad;
            let (dpsi, deps) = nutation_lon_obliquity(julian_date);
            let eps =
                deps + polynomial(&ECLIPTIC_MEAN_OBLIQUITY, julian_date.jme / 10.0) * (PI * (1.0 / 3600.0) / 180.0);
            let lambda = gp.lon + dpsi + dtau;
            let mut v = polynomial(&GSTA, julian_date.jc) + PI * 360.98564736629f64 / 180.0 * (julian_date.jd - JD0);
            v += dpsi * eps.cos();
            let mut alpha = (lambda.sin() * eps.cos() - gp.lat.tan() * eps.sin()).atan2(lambda.cos());
            if alpha < 0.0 {
                alpha += 2.0 * PI;
            }
            let delta = (gp.lat.sin() * eps.cos() + gp.lat.cos() * eps.sin() * lambda.sin()).asin();
            let hh = v + self.lon_radians - alpha;
            let xi = PI * (8.794f64 / 3600.0) / 180.0 / gp.rad;
            let u = (0.99664719f64 * self.lat_radians.tan()).atan();
            let x = u.cos() + self.elevation * self.lat_radians.cos() / EARTH_R;
            let y = 0.99664719f64 * u.sin() + self.elevation * self.lat_radians.sin() / EARTH_R;
            let dalpha = (-x * xi.sin() * hh.sin()).atan2(delta.cos() - x * xi.sin() * hh.cos());
            let delta_prime =
                ((delta.sin() - y * xi.sin()) * dalpha.cos()).atan2(delta.cos() - x * xi.sin() * hh.cos());
            let h_prime = hh - dalpha;
            let h = (self.lat_radians.sin() * delta_prime.sin()
                + self.lat_radians.cos() * delta_prime.cos() * h_prime.cos())
            .asin();
            let mut z = PI / 2.0 - h;
            let mut a = (PI
                + h_prime
                    .sin()
                    .atan2(h_prime.cos() * self.lat_radians.sin() - delta_prime.tan() * self.lat_radians.cos()))
            .rem(2.0 * PI);
            if z < 0.0 {
                z = -z;
                a = (a + PI).rem(2.0 * PI);
            }
            if z > PI {
                z = 2.0 * PI - z;
                a = (a + 2.0 * PI).rem(2.0 * PI);
            }
            SolarPosition { zenith: z, azimuth: a }
        })
    }
    /// Returns the solar time (apparent solar time) for the given datetime.
    ///
    /// Solar time differs from clock time due to the equation of time and longitude offset.
    /// At solar noon, the Sun crosses the observer's meridian (highest point in the sky).
    ///
    /// # Returns
    ///
    /// A `Result` containing the solar time as a [`NaiveDateTime`], or an error if time conversion fails.
    pub fn get_solar_time(&mut self) -> Result<NaiveDateTime, CalculationError> {
        let e = equation_of_time(*self.get_julian_day(), *self.get_geocentric_position());
        julian_date_to_datetime(self.get_julian_day().jd + (self.lon_radians + e) / PI / 2.0)
    }

    /// Returns the time of solar transit (solar noon).
    ///
    /// Solar transit is the moment when the Sun crosses the observer's meridian,
    /// reaching its highest point in the sky for the day. This is also known as solar noon.
    ///
    /// # Returns
    ///
    /// A `Result` containing the Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
    /// of the solar transit, or an error if the calculation fails.
    pub fn get_solar_transit(&mut self) -> Result<i64, CalculationError> {
        self._get_solar_transit().map(|i| i.timestamp)
    }
    fn _get_solar_transit(&mut self) -> Result<SolarInfo, CalculationError> {
        let r = self.solar_transit.get_or_init(|| {
            let t = datetime_to_unix(self.ut);

            let tc = find_solar_time(t, 12, 0, 0, self.delta_t, self.delta_ut1, self.lon_radians)?;
            let mut calculator = self.with_time(unix_to_datetime(tc)?);
            let pos = *calculator.get_solar_position();
            let pos = match self.refraction {
                Refraction::ApSolposBennet => apply_refraction(
                    bennet_refraction,
                    inverse_bennet_refraction,
                    pos,
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
                Refraction::ApSolposBennetNA => apply_refraction(
                    bennet_na_refraction,
                    inverse_bennet_na_refraction,
                    pos,
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
            };
            Ok(SolarInfo {
                position: pos,
                timestamp: tc,
            })
        });
        *r
    }

    /// Returns the time of the previous solar midnight (solar anti-transit).
    ///
    /// Solar midnight is the moment when the Sun crosses the observer's anti-meridian,
    /// reaching its lowest point below the horizon.
    ///
    /// # Returns
    ///
    /// A `Result` containing the Unix timestamp of the previous solar midnight,
    /// or an error if the calculation fails.
    pub fn get_prev_solar_midnight(&mut self) -> Result<i64, CalculationError> {
        self._get_prev_solar_midnight().map(|i| i.timestamp)
    }

    fn _get_prev_solar_midnight(&mut self) -> Result<SolarInfo, CalculationError> {
        let solar_transit = self.get_solar_transit()?;
        let r = self.prev_solar_midnight.get_or_init(|| {
            let tc = find_solar_time(
                solar_transit - 43200,
                0,
                0,
                0,
                self.delta_t,
                self.delta_ut1,
                self.lon_radians,
            )?;
            let mut calculator = self.with_time(unix_to_datetime(tc)?);
            let pos = *calculator.get_solar_position();
            let pos = match self.refraction {
                Refraction::ApSolposBennet => apply_refraction(
                    bennet_refraction,
                    inverse_bennet_refraction,
                    pos,
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
                Refraction::ApSolposBennetNA => apply_refraction(
                    bennet_na_refraction,
                    inverse_bennet_na_refraction,
                    pos,
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
            };
            Ok(SolarInfo {
                position: pos,
                timestamp: tc,
            })
        });
        *r
    }
    /// Returns the time of the next solar midnight (solar anti-transit).
    ///
    /// Solar midnight is the moment when the Sun crosses the observer's anti-meridian,
    /// reaching its lowest point below the horizon.
    ///
    /// # Returns
    ///
    /// A `Result` containing the Unix timestamp of the next solar midnight,
    /// or an error if the calculation fails.
    pub fn get_next_solar_midnight(&mut self) -> Result<i64, CalculationError> {
        self._get_next_solar_midnight().map(|i| i.timestamp)
    }

    fn _get_next_solar_midnight(&mut self) -> Result<SolarInfo, CalculationError> {
        let solar_transit = self.get_solar_transit()?;
        let r = self.next_solar_midnight.get_or_init(|| {
            let tc = find_solar_time(
                solar_transit + 43200,
                0,
                0,
                0,
                self.delta_t,
                self.delta_ut1,
                self.lon_radians,
            )?;
            let mut calculator = self.with_time(unix_to_datetime(tc)?);
            let pos = *calculator.get_solar_position();
            let pos = match self.refraction {
                Refraction::ApSolposBennet => apply_refraction(
                    bennet_refraction,
                    inverse_bennet_refraction,
                    pos,
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
                Refraction::ApSolposBennetNA => apply_refraction(
                    bennet_na_refraction,
                    inverse_bennet_na_refraction,
                    pos,
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
            };
            Ok(SolarInfo {
                position: pos,
                timestamp: tc,
            })
        });
        *r
    }

    /// Returns the time of sunrise.
    ///
    /// Sunrise is defined as the moment when the top of the Sun's disk appears at the horizon,
    /// using the standard horizon angle of -0.8333° (accounts for Sun's angular radius and atmospheric refraction).
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Sunrise occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun never sets (midnight sun / polar day)
    /// - `AlwaysBelow`: Sun never rises (polar night)
    pub fn get_sunrise(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.sunrise.get() {
            return *r;
        }

        let prev_midnight = self.get_prev_solar_midnight()?;
        let transit = self.get_solar_transit()?;

        // Get solar positions at boundaries - safe because we just computed them
        let z1 = self._get_prev_solar_midnight()?.position.zenith;
        let z2 = self._get_solar_transit()?.position.zenith;
        let dip = self.compute_dip();
        let target_zenith = dip + PI / 2.0 + SUN_RADIUS;

        let result = self.find_solar_event(prev_midnight, transit, z1, z2, target_zenith);
        let _ = self.sunrise.set(result);
        result
    }

    /// Returns the time of sunset.
    ///
    /// Sunset is defined as the moment when the top of the Sun's disk disappears below the horizon,
    /// using the standard horizon angle of -0.8333° (accounts for Sun's angular radius and atmospheric refraction).
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Sunset occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun never sets (midnight sun / polar day)
    /// - `AlwaysBelow`: Sun never rises (polar night)
    pub fn get_sunset(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.sunset.get() {
            return *r;
        }

        let transit = self.get_solar_transit()?;
        let next_midnight = self.get_next_solar_midnight()?;

        // Get solar positions at boundaries
        let z1 = self._get_solar_transit()?.position.zenith;
        let z2 = self._get_next_solar_midnight()?.position.zenith;

        let dip = self.compute_dip();
        let target_zenith = dip + PI / 2.0 + SUN_RADIUS;

        let result = self.find_solar_event(transit, next_midnight, z1, z2, target_zenith);
        let _ = self.sunset.set(result);
        result
    }

    /// Get sunrise time offset by degrees below the horizon.
    ///
    /// Calculates the time when the sun reaches a specific angle below the horizon before sunrise.
    /// This is useful for calculating custom twilight events or dawn times.
    ///
    /// # Arguments
    ///
    /// * `degrees` - The number of degrees below the horizon (e.g., 6.0 for civil dawn, 12.0 for nautical dawn, 18.0 for astronomical dawn)
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: The event occurs at the given Unix timestamp
    /// - `AllDay`: Sun is always above the threshold
    /// - `AllNight`: Sun never reaches the threshold
    ///
    /// # Note
    ///
    /// This function does not cache results. For better performance when calling multiple times,
    /// use the specific methods like `get_civil_dawn()`, `get_nautical_dawn()`, etc.
    pub fn get_sunrise_offset_by_degrees(&mut self, degrees: f64) -> Result<SolarEventResult, CalculationError> {
        let prev_midnight = self.get_prev_solar_midnight()?;
        let transit = self.get_solar_transit()?;

        let z1 = self
            ._get_prev_solar_midnight()
            .map(|i| i.position.zenith)
            .unwrap_or(PI / 2.0);
        let z2 = self._get_solar_transit()?.position.zenith;

        let dip = self.compute_dip();
        let target_zenith = dip + PI / 2.0 + PI * degrees / 180.0;

        self.find_solar_event(prev_midnight, transit, z1, z2, target_zenith)
    }

    /// Get sunset time offset by degrees below the horizon.
    ///
    /// Calculates the time when the sun reaches a specific angle below the horizon after sunset.
    /// This is useful for calculating custom twilight events or dusk times.
    ///
    /// # Arguments
    ///
    /// * `degrees` - The number of degrees below the horizon (e.g., 6.0 for civil dusk, 12.0 for nautical dusk, 18.0 for astronomical dusk)
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: The event occurs at the given Unix timestamp
    /// - `AllDay`: Sun is always above the threshold
    /// - `AllNight`: Sun never reaches the threshold
    ///
    ///
    /// # Note
    ///
    /// This function does not cache results. For better performance when calling multiple times,
    /// use the specific methods like `get_civil_dusk()`, `get_nautical_dusk()`, etc.
    pub fn get_sunset_offset_by_degrees(&mut self, degrees: f64) -> Result<SolarEventResult, CalculationError> {
        let transit = self.get_solar_transit()?;
        let next_midnight = self.get_next_solar_midnight()?;

        let z1 = self._get_solar_transit()?.position.zenith;
        let z2 = self._get_next_solar_midnight()?.position.zenith;

        let dip = self.compute_dip();
        let target_zenith = dip + PI / 2.0 + PI * degrees / 180.0;

        self.find_solar_event(transit, next_midnight, z1, z2, target_zenith)
    }

    /// Get sea-level sunrise time.
    ///
    /// Calculates sunrise at sea level (elevation = 0 meters), without elevation adjustment.
    /// This is important for twilight calculations, as the level of light during twilight
    /// is not affected by elevation. Sea-level sunrise forms the base for dawn calculations
    /// that are calculated as a dip below the horizon before sunrise.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Sea-level sunrise occurs at the given Unix timestamp
    /// - `AllDay`: Sun never sets (midnight sun / polar day)
    /// - `AllNight`: Sun never rises (polar night)
    ///
    /// # See Also
    ///
    /// - [`get_sunrise()`](Self::get_sunrise) - Elevation-adjusted sunrise
    pub fn get_sea_level_sunrise(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.sea_level_sunrise.get() {
            return *r;
        }

        // Create a temporary calculator with elevation=0
        let mut sea_level_calc = self.with_elevation(0.0);
        let result = sea_level_calc.get_sunrise();
        let _ = self.sea_level_sunrise.set(result);
        result
    }

    /// Get sea-level sunset time.
    ///
    /// Calculates sunset at sea level (elevation = 0 meters), without elevation adjustment.
    /// This is important for twilight calculations, as the level of light during twilight
    /// is not affected by elevation. Sea-level sunset forms the base for dusk calculations
    /// that are calculated as a dip below the horizon after sunset.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Sea-level sunset occurs at the given Unix timestamp
    /// - `AllDay`: Sun never sets (midnight sun / polar day)
    /// - `AllNight`: Sun never rises (polar night)
    ///
    /// # See Also
    ///
    /// - [`get_sunset()`](Self::get_sunset) - Elevation-adjusted sunset
    pub fn get_sea_level_sunset(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.sea_level_sunset.get() {
            return *r;
        }

        // Create a temporary calculator with elevation=0
        let mut sea_level_calc = self.with_elevation(0.0);
        let result = sea_level_calc.get_sunset();
        let _ = self.sea_level_sunset.set(result);
        result
    }

    /// Get civil dawn time (sun 6° below horizon)
    /// Uses zenith angle: 90° + 6° + geometric dip
    /// Returns the time of civil dawn (morning civil twilight).
    ///
    /// Civil dawn is when the Sun is 6° below the horizon in the morning.
    /// At this time, there is enough light for most outdoor activities without artificial lighting.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Civil dawn occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun is always above civil twilight threshold
    /// - `AlwaysBelow`: Sun never reaches civil twilight threshold
    pub fn get_civil_dawn(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.civil_dawn.get() {
            return *r;
        }

        let result = self.get_sunrise_offset_by_degrees(6.0);
        let _ = self.civil_dawn.set(result);
        result
    }

    /// Get civil dusk time (sun 6° below horizon)
    /// Uses zenith angle: 90° + 6° + geometric dip
    /// Returns the time of civil dusk (evening civil twilight).
    ///
    /// Civil dusk is when the Sun is 6° below the horizon in the evening.
    /// After this time, artificial lighting is typically needed for outdoor activities.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Civil dusk occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun is always above civil twilight threshold
    /// - `AlwaysBelow`: Sun never reaches civil twilight threshold
    pub fn get_civil_dusk(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.civil_dusk.get() {
            return *r;
        }

        let result = self.get_sunset_offset_by_degrees(6.0);
        let _ = self.civil_dusk.set(result);
        result
    }

    /// Get nautical dawn time (sun 12° below horizon)
    /// Uses zenith angle: 90° + 12° + sun radius + geometric dip
    /// Returns the time of nautical dawn (morning nautical twilight).
    ///
    /// Nautical dawn is when the Sun is 12° below the horizon in the morning.
    /// At this time, the horizon becomes visible at sea, allowing navigation by horizon observations.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Nautical dawn occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun is always above nautical twilight threshold
    /// - `AlwaysBelow`: Sun never reaches nautical twilight threshold
    pub fn get_nautical_dawn(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.nautical_dawn.get() {
            return *r;
        }

        let result = self.get_sunrise_offset_by_degrees(12.0);
        let _ = self.nautical_dawn.set(result);
        result
    }

    /// Get nautical dusk time (sun 12° below horizon)
    /// Uses zenith angle: 90° + 12° + sun radius + geometric dip
    /// Returns the time of nautical dusk (evening nautical twilight).
    ///
    /// Nautical dusk is when the Sun is 12° below the horizon in the evening.
    /// After this time, the horizon is no longer visible at sea.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Nautical dusk occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun is always above nautical twilight threshold
    /// - `AlwaysBelow`: Sun never reaches nautical twilight threshold
    pub fn get_nautical_dusk(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.nautical_dusk.get() {
            return *r;
        }

        let result = self.get_sunset_offset_by_degrees(12.0);
        let _ = self.nautical_dusk.set(result);
        result
    }

    /// Get astronomical dawn time (sun 18° below horizon)
    /// Uses zenith angle: 90° + 18° + sun radius + geometric dip
    /// Returns the time of astronomical dawn (morning astronomical twilight).
    ///
    /// Astronomical dawn is when the Sun is 18° below the horizon in the morning.
    /// This marks the beginning of astronomical twilight, when the sky begins to be illuminated
    /// and faint stars start to disappear.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Astronomical dawn occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun is always above astronomical twilight threshold
    /// - `AlwaysBelow`: Sun never reaches astronomical twilight threshold (true astronomical night)
    pub fn get_astronomical_dawn(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.astronomical_dawn.get() {
            return *r;
        }

        let result = self.get_sunrise_offset_by_degrees(18.0);
        let _ = self.astronomical_dawn.set(result);
        result
    }

    /// Get astronomical dusk time (sun 18° below horizon)
    /// Uses zenith angle: 90° + 18° + sun radius + geometric dip
    /// Returns the time of astronomical dusk (evening astronomical twilight).
    ///
    /// Astronomical dusk is when the Sun is 18° below the horizon in the evening.
    /// After this time, true astronomical night begins, and the sky is completely dark
    /// for astronomical observations.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SolarEventResult`]:
    /// - `Occurs(timestamp)`: Astronomical dusk occurs at the given Unix timestamp
    /// - `AlwaysAbove`: Sun is always above astronomical twilight threshold
    /// - `AlwaysBelow`: Sun never reaches astronomical twilight threshold (true astronomical night)
    pub fn get_astronomical_dusk(&mut self) -> Result<SolarEventResult, CalculationError> {
        if let Some(r) = self.astronomical_dusk.get() {
            return *r;
        }

        let result = self.get_sunset_offset_by_degrees(18.0);
        let _ = self.astronomical_dusk.set(result);
        result
    }
    /// Helper function for creating a copy of this `AstronomicalCalculator` with a new elevation
    fn with_elevation(&self, elevation: f64) -> Self {
        Self {
            ut: self.ut,
            delta_t: self.delta_t,
            delta_ut1: self.delta_ut1,
            lon_radians: self.lon_radians,
            lat_radians: self.lat_radians,
            elevation,
            temperature: self.temperature,
            pressure: self.pressure,
            gdip: None, // Reset gdip when changing elevation
            refraction: self.refraction,
            julian_date: OnceCell::new(),
            geocentric_position: OnceCell::new(),
            solar_position: OnceCell::new(),
            solar_transit: OnceCell::new(),
            prev_solar_midnight: OnceCell::new(),
            next_solar_midnight: OnceCell::new(),
            sunrise: OnceCell::new(),
            sunset: OnceCell::new(),
            sea_level_sunrise: OnceCell::new(),
            sea_level_sunset: OnceCell::new(),
            civil_dawn: OnceCell::new(),
            civil_dusk: OnceCell::new(),
            nautical_dawn: OnceCell::new(),
            nautical_dusk: OnceCell::new(),
            astronomical_dawn: OnceCell::new(),
            astronomical_dusk: OnceCell::new(),
        }
    }

    /// Helper function for creating a copy of this `AstronomicalCalculator` with a new time
    fn with_time(&self, time: NaiveDateTime) -> Self {
        Self {
            ut: time,
            delta_t: self.delta_t,
            delta_ut1: self.delta_ut1,
            lon_radians: self.lon_radians,
            lat_radians: self.lat_radians,
            elevation: self.elevation,
            temperature: self.temperature,
            pressure: self.pressure,
            gdip: self.gdip,
            refraction: self.refraction,
            julian_date: OnceCell::new(),
            geocentric_position: OnceCell::new(),
            solar_position: OnceCell::new(),
            solar_transit: OnceCell::new(),
            prev_solar_midnight: OnceCell::new(),
            next_solar_midnight: OnceCell::new(),
            sunrise: OnceCell::new(),
            sunset: OnceCell::new(),
            sea_level_sunrise: OnceCell::new(),
            sea_level_sunset: OnceCell::new(),
            civil_dawn: OnceCell::new(),
            civil_dusk: OnceCell::new(),
            nautical_dawn: OnceCell::new(),
            nautical_dusk: OnceCell::new(),
            astronomical_dawn: OnceCell::new(),
            astronomical_dusk: OnceCell::new(),
        }
    }

    /// Compute the geometric dip angle based on elevation or explicit gdip
    fn compute_dip(&self) -> f64 {
        if let Some(gdip) = self.gdip {
            gdip
        } else if self.elevation > 0.0 {
            (EARTH_R / (EARTH_R + self.elevation)).acos()
        } else {
            0.0
        }
    }

    /// Helper to find a solar event using bisection search
    fn find_solar_event(
        &mut self,
        t1: i64,
        t2: i64,
        z1: f64,
        z2: f64,
        target_zenith: f64,
    ) -> Result<SolarEventResult, CalculationError> {
        // Check if the sun is always above or below the target zenith
        if target_zenith < z1 && target_zenith < z2 {
            return Ok(SolarEventResult::AllNight);
        }
        if target_zenith > z1 && target_zenith > z2 {
            return Ok(SolarEventResult::AllDay);
        }

        // Use cosine interpolation to estimate crossing time
        let w = PI / (t2 - t1) as f64;
        let b_denom = (t1 as f64 * w).cos() - (t2 as f64 * w).cos();
        let a = -((t2 as f64 * w).cos() * z1 - (t1 as f64 * w).cos() * z2) / b_denom;
        let b = (z1 - z2) / b_denom;
        let direction = if z2 < z1 { 1.0 } else { -1.0 };

        // Initial guess
        let mut timestamp = t1 + ((target_zenith / b - a / b).acos() / w).round() as i64;
        if timestamp < t1 || timestamp > t2 {
            timestamp = (t1 + t2) / 2;
        }

        // Calculate solar position at initial guess
        let datetime = unix_to_datetime(timestamp)?;
        let mut calculator = self.with_time(datetime);
        let mut position = *calculator.get_solar_position();
        position = match self.refraction {
            Refraction::ApSolposBennet => apply_refraction(
                bennet_refraction,
                inverse_bennet_refraction,
                position,
                self.gdip,
                self.elevation,
                self.pressure,
                self.temperature,
            )?,
            Refraction::ApSolposBennetNA => apply_refraction(
                bennet_na_refraction,
                inverse_bennet_na_refraction,
                position,
                self.gdip,
                self.elevation,
                self.pressure,
                self.temperature,
            )?,
        };
        let mut best_timestamp = timestamp;
        let mut best_error = position.zenith - target_zenith;

        if best_error.abs() < Z_EPS {
            return Ok(SolarEventResult::Occurs(best_timestamp));
        }

        // Set up bisection search bounds
        let (mut t_min, mut z_min, mut t_max, mut z_max) = if direction * (position.zenith - target_zenith) > 0.0 {
            (timestamp, position.zenith, t2, z2)
        } else {
            (t1, z1, timestamp, position.zenith)
        };

        // Bisection search with linear interpolation
        let mut iter = 0;
        while t_max - t_min > 1 && iter < Z_MAXITER {
            // Linear interpolation for next guess
            timestamp = (((target_zenith - z_min) * t_max as f64 + (z_max - target_zenith) * t_min as f64)
                / (target_zenith - z_min + (z_max - target_zenith)))
                .round() as i64;

            // Keep within bounds
            if timestamp < t1 || timestamp > t2 {
                timestamp = (t1 + t2) / 2;
            }

            // Avoid skewed divisions
            if timestamp - t_min > MAXRAT * (t_max - timestamp) || MAXRAT * (timestamp - t_min) < t_max - timestamp {
                timestamp = (t_min + t_max) / 2;
            }

            // Calculate solar position
            let datetime = unix_to_datetime(timestamp)?;
            let mut calculator = self.with_time(datetime);
            let position = match self.refraction {
                Refraction::ApSolposBennet => apply_refraction(
                    bennet_refraction,
                    inverse_bennet_refraction,
                    *calculator.get_solar_position(),
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
                Refraction::ApSolposBennetNA => apply_refraction(
                    bennet_na_refraction,
                    inverse_bennet_na_refraction,
                    *calculator.get_solar_position(),
                    self.gdip,
                    self.elevation,
                    self.pressure,
                    self.temperature,
                )?,
            };
            // Update best result
            if (position.zenith - target_zenith).abs() < best_error.abs() {
                best_error = position.zenith - target_zenith;
                best_timestamp = timestamp;
            }

            if best_error.abs() < Z_EPS {
                return Ok(SolarEventResult::Occurs(best_timestamp));
            }

            // Update bisection bounds
            if direction * (position.zenith - target_zenith) > 0.0 {
                t_min = timestamp;
                z_min = position.zenith;
            } else {
                t_max = timestamp;
                z_max = position.zenith;
            }
            iter += 1;
        }

        Ok(SolarEventResult::Occurs(best_timestamp))
    }
}

#[derive(Copy, Clone, Debug)]
/// Solar position in local horizontal coordinates.
///
/// Represents the position of the Sun as seen from an observer's location,
/// specified by zenith and azimuth angles.
///
/// # Fields
///
/// - `zenith`: Zenith angle in radians (0 = directly overhead, π/2 = horizon, π = nadir)
/// - `azimuth`: Azimuth angle in radians, measured clockwise from North (0 = N, π/2 = E, π = S, 3π/2 = W)
pub struct SolarPosition {
    /// Zenith angle in radians (0 = overhead, π/2 = horizon)
    pub zenith: f64,
    /// Azimuth angle in radians, clockwise from North (0 = N, π/2 = E)
    pub azimuth: f64,
}

/// Atmospheric refraction model selection.
///
/// Atmospheric refraction causes the apparent position of the Sun to differ from its geometric position,
/// especially near the horizon. Different models provide varying levels of accuracy.
///
/// # Variants
///
/// - `ApSolposBennet`: Standard Bennett refraction model, suitable for most applications
/// - `ApSolposBennetNA`: Bennett refraction model without atmospheric correction (assumes standard conditions)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Refraction {
    /// Bennett refraction model (recommended for most applications)
    ApSolposBennet,
    /// Bennett refraction model without atmospheric correction
    ApSolposBennetNA,
}

/// Calculate ΔT (Terrestrial Time minus Universal Time) for a given date.
///
/// ΔT represents the difference between Terrestrial Time (TT) and Universal Time (UT).
/// This value changes over time due to variations in Earth's rotation rate.
///
/// # Arguments
///
/// * `ut` - The datetime in Universal Time
///
/// # Returns
///
/// ΔT in seconds. As of 2024, this is approximately 69 seconds.
///
/// # Notes
///
/// - For dates within the table range (1657-2024), uses interpolated historical values
/// - For dates outside this range, uses polynomial approximation
/// - Historical values: ~64s (2000), ~57s (1990)
/// - For high-precision calculations, consult [IERS Bulletin A](https://www.iers.org/IERS/EN/Publications/Bulletins/bulletins.html)
///
/// # Example
///
/// ```
/// use astronomical_calculator::get_delta_t;
/// use chrono::NaiveDateTime;
///
/// let dt = NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// let delta_t = get_delta_t(&dt);
/// println!("ΔT for 2024: {:.2} seconds", delta_t);
/// // Output: approximately 69 seconds
/// ```
pub fn get_delta_t(ut: &NaiveDateTime) -> f64 {
    let mut imin: i64 = 0;
    let mut imax: i64 = 1244;
    let dyear = ut.year() as f64 + ut.month() as f64 / 12.0 + (ut.day() as f64 - 1.0) / 365.0;

    // Use polynomial approximation for dates outside table range
    if FREESPA_DELTA_T_TABLE[0] > dyear || FREESPA_DELTA_T_TABLE[(2 * imax) as usize] < dyear {
        let t = (dyear - 1820.0) / 100.0;
        return 32.0 * t * t - 20.0;
    }

    // Binary search in the table
    while imax - imin > 1 {
        let i = (imin + imax) / 2;
        let table_year = FREESPA_DELTA_T_TABLE[(2 * i) as usize];
        if table_year > dyear {
            imax = i;
        } else if table_year < dyear {
            imin = i;
        } else {
            return FREESPA_DELTA_T_TABLE[(2 * i + 1) as usize];
        }
    }

    // Linear interpolation between table entries
    FREESPA_DELTA_T_TABLE[(2 * imin + 1) as usize]
        + (dyear - FREESPA_DELTA_T_TABLE[(2 * imin) as usize])
            * (FREESPA_DELTA_T_TABLE[(2 * imax + 1) as usize] - FREESPA_DELTA_T_TABLE[(2 * imin + 1) as usize])
            / (FREESPA_DELTA_T_TABLE[(2 * imax) as usize] - FREESPA_DELTA_T_TABLE[(2 * imin) as usize])
}

/// Evaluate polynomial using Horner's method
fn polynomial(coefficients: &[f64], x: f64) -> f64 {
    let mut result = coefficients[0];
    for coeff in coefficients.iter().skip(1) {
        result = coeff + x * result;
    }
    result
}

/// Calculate equation of time
fn equation_of_time(jd: JulianDate, gp: GeoCentricSolPos) -> f64 {
    let dtau = PI * (-20.4898f64 / 3600.0f64) / 180.0f64 / gp.rad;
    let (dpsi, deps) = nutation_lon_obliquity(jd);
    let eps = deps + polynomial(&ECLIPTIC_MEAN_OBLIQUITY, jd.jme / 10.0) * (PI * (1.0 / 3600.0) / 180.0);
    let lambda = gp.lon + dpsi + dtau;
    let alpha = (lambda.sin() * eps.cos() - gp.lat.tan() * eps.sin()).atan2(lambda.cos());
    let m = polynomial(&SMLON, jd.jme);
    let mut e = (m - PI * 0.0057183f64 / 180.0f64 - alpha + dpsi * eps.cos()).rem(2.0 * PI);
    if e > PI * 5.0 / 180.0 {
        e -= 2.0 * PI;
    }
    if e < -(PI * 5.0 / 180.0f64) {
        e += 2.0 * PI;
    }
    e
}

/// Calculate nutation in longitude and obliquity
fn nutation_lon_obliquity(jd: JulianDate) -> (f64, f64) {
    let mut sum_psi: f64 = 0.0;
    let mut sum_eps: f64 = 0.0;

    // Calculate fundamental arguments
    let x = [
        polynomial(&MEAN_ELONGATION_MOON_SUN, jd.jce),
        polynomial(&MEAN_ANOMALY_SUN, jd.jce),
        polynomial(&MEAN_ANOMALY_MOON, jd.jce),
        polynomial(&ARG_LAT_MOON, jd.jce),
        polynomial(&ASC_LON_MOON, jd.jce),
    ];

    // Sum nutation terms
    for i in 0..NY {
        let sum: f64 = x
            .iter()
            .zip(&Y_TERMS[i as usize])
            .map(|(x_val, y_term)| x_val * (*y_term as f64))
            .sum();

        let pe = &PE_TERMS[i as usize];
        sum_psi += (pe[0] + jd.jce * pe[1]) * sum.sin();
        sum_eps += (pe[2] + jd.jce * pe[3]) * sum.cos();
    }

    (
        sum_psi * (PI * (1.0 / 36000000.0) / 180.0),
        sum_eps * (PI * (1.0 / 36000000.0) / 180.0),
    )
}

/// Sum periodic terms for heliocentric calculations
fn sum_periodic_terms(terms: &[PTerm], jd: &JulianDate) -> f64 {
    terms.iter().map(|term| term.a * (term.p + term.w * jd.jme).cos()).sum()
}

/// Calculate heliocentric longitude
fn heliocentric_longitude(jd: &JulianDate) -> f64 {
    let mut lon = sum_periodic_terms(&EARTH_LON0, jd);
    let mut power = jd.jme;

    lon += sum_periodic_terms(&EARTH_LON1, jd) * power;
    power *= jd.jme;
    lon += sum_periodic_terms(&EARTH_LON2, jd) * power;
    power *= jd.jme;
    lon += sum_periodic_terms(&EARTH_LON3, jd) * power;
    power *= jd.jme;
    lon += sum_periodic_terms(&EARTH_LON4, jd) * power;
    power *= jd.jme;
    lon += sum_periodic_terms(&EARTH_LON5, jd) * power;

    lon / 1.0e8
}

/// Calculate heliocentric latitude
fn heliocentric_latitude(jd: &JulianDate) -> f64 {
    let lat = sum_periodic_terms(&EARTH_LAT0, jd) + sum_periodic_terms(&EARTH_LAT1, jd) * jd.jme;
    lat / 1.0e8
}

/// Calculate heliocentric radius vector
fn heliocentric_radius(jd: &JulianDate) -> f64 {
    let mut rad = sum_periodic_terms(&EARTH_RAD0, jd);
    let mut power = jd.jme;

    rad += sum_periodic_terms(&EARTH_RAD1, jd) * power;
    power *= jd.jme;
    rad += sum_periodic_terms(&EARTH_RAD2, jd) * power;
    power *= jd.jme;
    rad += sum_periodic_terms(&EARTH_RAD3, jd) * power;
    power *= jd.jme;
    rad += sum_periodic_terms(&EARTH_RAD4, jd) * power;

    rad / 1.0e8
}

/// Convert Julian date to NaiveDateTime
fn julian_date_to_datetime(julian_day: f64) -> Result<NaiveDateTime, CalculationError> {
    let unix_millis = julian_day_to_unix_millis(julian_day);
    Utc.timestamp_millis_opt(unix_millis)
        .single()
        .map(|dt| dt.naive_utc())
        .ok_or(CalculationError::TimeConversionError)
}

/// Convert Unix timestamp to NaiveDateTime
fn unix_to_datetime(timestamp: i64) -> Result<NaiveDateTime, CalculationError> {
    julian_date_to_datetime((timestamp - ETJD0) as f64 / 86400.0 + JD0)
}

// ============================================================================
// Atmospheric Refraction
// ============================================================================

/// Generic refraction calculation using specified coefficients
fn calculate_refraction(coefficients: &[f64], pressure: f64, temperature: f64, altitude: f64) -> f64 {
    let pressure_ratio = pressure / AP0;
    let temp_ratio = (AT0 - ABSOLUTEZERO) / (temperature - ABSOLUTEZERO);
    let angle_term = (altitude + coefficients[1] / (altitude + coefficients[2])).tan();

    pressure_ratio * temp_ratio * coefficients[0] / angle_term
}

/// Bennet refraction model
fn bennet_refraction(pressure: f64, temperature: f64, altitude: f64) -> f64 {
    calculate_refraction(&BENNET, pressure, temperature, altitude)
}

/// Inverse Bennet refraction model
fn inverse_bennet_refraction(pressure: f64, temperature: f64, altitude: f64) -> f64 {
    calculate_refraction(&IBENNET, pressure, temperature, altitude)
}

/// Bennet refraction model (no atmosphere)
fn bennet_na_refraction(pressure: f64, temperature: f64, altitude: f64) -> f64 {
    calculate_refraction(&BENNETNA, pressure, temperature, altitude)
}

/// Inverse Bennet refraction model (no atmosphere)
fn inverse_bennet_na_refraction(pressure: f64, temperature: f64, altitude: f64) -> f64 {
    calculate_refraction(&IBENNETNA, pressure, temperature, altitude)
}

/// Apply atmospheric refraction to solar position
fn apply_refraction(
    refraction_fn: fn(f64, f64, f64) -> f64,
    inverse_refraction_fn: fn(f64, f64, f64) -> f64,
    mut position: SolarPosition,
    gdip: Option<f64>,
    elevation: f64,
    pressure: f64,
    temperature: f64,
) -> Result<SolarPosition, CalculationError> {
    // Calculate geometric dip
    let dip = if let Some(gdip) = gdip {
        if gdip.abs() > PI / 2.0 {
            return Err(CalculationError::GeometricDipOutOfRange);
        }
        gdip
    } else if elevation > 0.0 {
        (EARTH_R / (EARTH_R + elevation)).acos()
    } else {
        0.0
    };

    // Calculate refraction correction
    let atmospheric_refraction = refraction_fn(pressure, temperature, -dip);
    let altitude = PI / 2.0 - position.zenith;
    let altitude_correction = if altitude >= -atmospheric_refraction - SUN_RADIUS - dip {
        inverse_refraction_fn(pressure, temperature, altitude)
    } else {
        0.0
    };

    // Apply correction
    position.zenith -= altitude_correction;
    position.zenith = position.zenith.rem(2.0 * PI);

    // Normalize zenith angle
    if position.zenith < 0.0 {
        position.zenith = -position.zenith;
        position.azimuth = (position.azimuth + PI).rem(2.0 * PI);
    }
    if position.zenith > PI {
        position.zenith = 2.0 * PI - position.zenith;
        position.azimuth = (position.azimuth + PI).rem(2.0 * PI);
    }

    Ok(position)
}

// ============================================================================
// Julian Date and Geocentric Position
// ============================================================================

impl JulianDate {
    fn new(ut: NaiveDateTime, delta_t: Option<f64>, delta_ut1: f64) -> Self {
        let jd = unix_millis_to_julian_day((ut.and_utc().timestamp_millis() as f64 + (delta_ut1 * 1000.0)) as i64);
        let dt = if let Some(delta_t) = delta_t {
            delta_t
        } else {
            get_delta_t(&ut)
        };
        let jde = jd + dt / 86400.0;
        let jc = (jd - JD0) / 36525.0;
        let jce = (jde - JD0) / 36525.0;
        let jme = jce / 10.0;
        Self { jd, jde, jc, jce, jme }
    }

    fn from_unix_time(unix_time: i64, delta_t: Option<f64>, delta_ut1: f64) -> Result<Self, CalculationError> {
        unix_to_datetime(unix_time).map(|ut| Self::new(ut, delta_t, delta_ut1))
    }
}

/// Geocentric solar position coordinates.
///
/// Represents the Sun's position as seen from Earth's center in ecliptic coordinates.
/// These values are computed using VSOP87 theory and converted from heliocentric to geocentric coordinates.
///
/// # Fields
///
/// - `lat`: Geocentric latitude in radians (ecliptic coordinate system)
/// - `lon`: Geocentric longitude in radians (ecliptic coordinate system)
/// - `rad`: Radius vector (Sun-Earth distance) in Astronomical Units (AU)
#[derive(Copy, Clone, Debug)]
struct GeoCentricSolPos {
    /// Geocentric latitude in radians
    lat: f64,
    /// Geocentric longitude in radians
    lon: f64,
    /// Sun-Earth distance in AU
    rad: f64,
}

impl GeoCentricSolPos {
    fn new(jd: &JulianDate) -> Self {
        // Convert heliocentric to geocentric coordinates
        let lat = (-heliocentric_latitude(jd)).rem(2.0 * PI);
        let mut lon = (heliocentric_longitude(jd) + PI).rem(2.0 * PI);
        if lon < 0.0 {
            lon += 2.0 * PI;
        }
        let rad = heliocentric_radius(jd);
        GeoCentricSolPos { lat, lon, rad }
    }
}

/// Find the time when the sun is at a specific hour angle (solar time)
pub(crate) fn find_solar_time(
    timestamp: i64,
    hour: i64,
    min: i64,
    sec: i64,
    delta_t: Option<f64>,
    delta_ut1: f64,
    longitude: f64,
) -> Result<i64, CalculationError> {
    let mut jd = JulianDate::from_unix_time(timestamp, delta_t, delta_ut1)?;
    #[cfg(test)]
    {
        extern crate std;
        std::println!("jd: {:?}", jd);
    }
    let mut datetime = true_solar_time(unix_to_datetime(timestamp)?, delta_t, delta_ut1, longitude)?;

    // Calculate initial time offset
    // Note: 24.4 is intentional (from original C code), not 24.0
    let mut time_delta = (hour - datetime.hour() as i64) as f64 / 24.0;
    time_delta += (min - datetime.minute() as i64) as f64 / 1440.0;
    time_delta += (sec - datetime.second() as i64) as f64 / 86400.0;

    // Normalize to [-0.5, 0.5] day range
    if time_delta > 0.5 {
        time_delta -= 1.0;
    }
    if time_delta < -0.5 {
        time_delta += 1.0;
    }

    jd.jd += time_delta;
    time_delta = 1.0;

    // Iterate to find exact solar time
    let mut iter = 0;
    while time_delta.abs() > FRACDAYSEC && iter < MAX_FPITER {
        let mut jd_new = jd;
        let geocentric_pos = GeoCentricSolPos::new(&jd);
        let eot = equation_of_time(jd, geocentric_pos);

        jd_new.jd += (longitude + eot) / PI / 2.0;
        datetime = julian_date_to_datetime(jd_new.jd)?;

        // Note: 24.4 is intentional (from original C code), not 24.0
        time_delta = (hour - datetime.hour() as i64) as f64 / 24.0;
        time_delta += (min - datetime.minute() as i64) as f64 / 1440.0;
        time_delta += (sec - datetime.second() as i64) as f64 / 86400.0;

        if time_delta > 0.5 {
            time_delta -= 1.0;
        }
        if time_delta < -0.5 {
            time_delta += 1.0;
        }

        jd.jd += time_delta;
        iter += 1;
    }

    Ok(julian_date_to_unix(&jd))
}

/// Convert Julian date to Unix timestamp
fn julian_date_to_unix(jd: &JulianDate) -> i64 {
    ((jd.jd - JD0) * 86400.0).round() as i64 + ETJD0
}

/// Convert NaiveDateTime to Unix timestamp
fn datetime_to_unix(datetime: NaiveDateTime) -> i64 {
    let jd = JulianDate::new(datetime, None, 0.0);
    ((jd.jd - JD0) * 86400.0).round() as i64 + ETJD0
}

/// Julian date and related time values for astronomical calculations.
///
/// This struct contains various time representations used internally for astronomical computations.
/// All values are computed relative to the J2000.0 epoch (JD 2451545.0).
///
/// # Fields
///
/// - `jd`: Julian Date in Universal Time (UT)
/// - `jde`: Julian Ephemeris Date in Terrestrial Time (TT), used for ephemeris calculations
/// - `jc`: Julian Century from J2000.0 in UT
/// - `jce`: Julian Century from J2000.0 in TT
/// - `jme`: Julian Millennium from J2000.0 in TT
/// - `e`: Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
///
/// # Note
///
/// This struct is primarily used internally but is exposed for advanced use cases.
/// Most users should interact with [`AstronomicalCalculator`] methods instead.
#[derive(Copy, Clone, Debug)]
pub struct JulianDate {
    /// Julian Date (UT)
    pub jd: f64,
    /// Julian Ephemeris Date (TT)
    pub jde: f64,
    /// Julian Century from J2000.0 (UT)
    pub jc: f64,
    /// Julian Century from J2000.0 (TT)
    pub jce: f64,
    /// Julian Millennium from J2000.0 (TT)
    pub jme: f64,
}

/// Errors that can occur during solar position calculations.
///
/// All input parameters are validated when creating an [`AstronomicalCalculator`].
/// These errors indicate that a parameter falls outside its valid range.
///
/// # Variants
///
/// - `DeltaUt1OutOfRange`: ΔUT1 must be in range [-1.0, 1.0] seconds
/// - `LongitudeOutOfRange`: Longitude must be in range [-π, π] radians ([-180°, 180°])
/// - `LatitudeOutOfRange`: Latitude must be in range [-π/2, π/2] radians ([-90°, 90°])
/// - `ElevationOutOfRange`: Elevation must be > -6,500,000 meters
/// - `PressureOutOfRange`: Pressure must be in range (0.0, 5000.0] millibars
/// - `TemperatureOutOfRange`: Temperature must be > -273.15°C (absolute zero)
/// - `GeometricDipOutOfRange`: Geometric dip must be in range [-5.0, 5.0] degrees
/// - `TimeConversionError`: Invalid datetime or timestamp conversion
#[derive(Error, Debug, Clone, Copy)]
pub enum CalculationError {
    /// ΔUT1 parameter out of valid range [-1.0, 1.0] seconds
    #[error("ΔUT1 out of range")]
    DeltaUt1OutOfRange,

    /// Longitude out of valid range [-π, π] radians
    #[error("Longitude out of range")]
    LongitudeOutOfRange,

    /// Latitude out of valid range [-π/2, π/2] radians
    #[error("Latitude out of range")]
    LatitudeOutOfRange,

    /// Elevation below minimum value (-6,500,000 meters)
    #[error("Elevation out of range")]
    ElevationOutOfRange,

    /// Pressure out of valid range (0.0, 5000.0] millibars
    #[error("Pressure out of range")]
    PressureOutOfRange,

    /// Temperature below absolute zero (-273.15°C)
    #[error("Temperature out of range")]
    TemperatureOutOfRange,

    /// Geometric dip angle out of valid range [-5.0, 5.0] degrees
    #[error("Geometric dip out of range")]
    GeometricDipOutOfRange,

    /// Error converting between time representations
    #[error("Time conversion error")]
    TimeConversionError,
}

fn true_solar_time(
    ut: NaiveDateTime,
    delta_t: Option<f64>,
    delta_ut1: f64,
    lon: f64,
) -> Result<NaiveDateTime, CalculationError> {
    let mut jd = JulianDate::new(ut, delta_t, delta_ut1);
    let geocentric_pos = GeoCentricSolPos::new(&jd);
    let eot = equation_of_time(jd, geocentric_pos);
    jd.jd += (lon + eot) / PI / 2.0;
    // jd.jd = (jd.jd - ETJD0 as f64) / 86400.0f64 + JD0;
    let datetime = julian_date_to_datetime(jd.jd)?;
    Ok(datetime)
}
