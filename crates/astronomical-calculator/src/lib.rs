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
#![no_std]

// use chrono::{DateTime, Datelike, Days, Duration};
// use chrono::{NaiveDate, Utc};
// use thiserror::Error;

// use crate::geo::*;
// use crate::math::floored_mod;
// use crate::terms::{SUN_RISE, SUN_SET, SUN_TRANSIT};
// use crate::time::*;
mod freespa;
#[cfg(test)]
extern crate std;

// mod geo;
// mod math;
// mod terms;
// // #[cfg(all(feature = "__spa-sys", test))]
// #[cfg(test)]
// mod tests;
// mod time;

// pub use time::delta_t;

// // ============================================================================
// // Error Types
// // ============================================================================

// /// Errors that can occur during solar position calculations.
// #[derive(Debug, Error)]
// pub enum Error {
//     /// Longitude value is outside the valid range.
//     #[error("invalid longitude: {0} (expected -180.0..=180.0)")]
//     InvalidLongitude(f64),

//     /// Latitude value is outside the valid range.
//     #[error("invalid latitude: {0} (expected -90.0..=90.0)")]
//     InvalidLatitude(f64),

//     /// Year value is outside the valid range.
//     #[error("invalid year: {0} (expected -2000..=6000)")]
//     InvalidYear(i32),

//     /// Atmospheric pressure value is outside the valid range.
//     #[error("invalid pressure: {0} (expected 0.0..=5000.0)")]
//     InvalidPressure(f64),

//     /// Temperature value is outside the valid range.
//     #[error("invalid temperature: {0} (expected -273.0..=6000.0)")]
//     InvalidTemperature(f64),

//     /// Delta UT1 value is outside the valid range.
//     #[error("invalid delta_ut1: {0} (expected -1.0..=1.0)")]
//     InvalidDeltaUt1(f64),

//     /// Delta T value is outside the valid range.
//     #[error("invalid delta_t: {0} (expected -8000.0..=8000.0)")]
//     InvalidDeltaT(f64),

//     /// Atmospheric refraction value is outside the valid range.
//     #[error("invalid atmos_refract: {0} (expected -5.0..=5.0)")]
//     InvalidAtmosRefract(f64),

//     /// Elevation value is outside the valid range.
//     #[error("invalid elevation: {0} (expected -6500000.0..=6500000.0)")]
//     InvalidElevation(f64),

//     /// Surface slope value is outside the valid range.
//     #[error("invalid slope: {0} (expected -360.0..=360.0)")]
//     InvalidSlope(f64),

//     /// Surface azimuth rotation value is outside the valid range.
//     #[error("invalid azm_rotation: {0} (expected -360.0..=360.0)")]
//     InvalidAzmRotation(f64),

//     /// Unable to estimate delta_t for the given date (year outside estimation range).
//     #[error("unable to estimate delta_t")]
//     UnableToEstimateDeltaT,
// }

// // ============================================================================
// // Input Parameters
// // ============================================================================

// /// Input parameters for solar position calculations.
// ///
// /// This struct contains all the necessary parameters to calculate the solar position
// /// including observer location, atmospheric conditions, and time corrections.
// #[derive(Debug)]
// pub struct AstronomicalCalculator {
//     /// Date and time for the calculation (with timezone information)
//     pub datetime: DateTime<Utc>,

//     /// Difference between UT and UT1 in seconds (fractional second of a day)
//     /// Used for Earth rotation variations. Typical range: -1.0 to +1.0 seconds
//     pub delta_ut1: f64,

//     /// Difference between Terrestrial Time (TT) and Universal Time (UT) in seconds
//     /// Accounts for variations in Earth's rotation. For 2024, approximately 69 seconds
//     pub delta_t: f64,

//     /// Observer's longitude in degrees (positive East, negative West)
//     /// Range: -180.0 to +180.0
//     pub longitude: f64,

//     /// Observer's latitude in degrees (positive North, negative South)
//     /// Range: -90.0 to +90.0
//     pub latitude: f64,

//     /// Observer's elevation above sea level in meters
//     /// Range: -6,500,000.0 to +6,500,000.0
//     pub elevation: f64,

//     /// Local atmospheric pressure in millibars
//     /// Range: 0.0 to 5000.0 (standard pressure at sea level is 1013.25 mb)
//     pub pressure: f64,

//     /// Local temperature in Celsius
//     /// Range: -273.0 to +6000.0
//     pub temperature: f64,

//     /// Surface slope in degrees (0 = horizontal, 90 = vertical)
//     /// Range: -360.0 to +360.0
//     pub slope: f64,

//     /// Surface azimuth rotation in degrees (measured from south)
//     /// Range: -360.0 to +360.0
//     pub azm_rotation: f64,

//     /// Atmospheric refraction at sunrise and sunset in degrees
//     /// Standard value is 0.5667 degrees (34 arcminutes)
//     /// Range: -5.0 to +5.0
//     pub atmos_refract: f64,

//     intermediates: Option<SolarPositionIntermediates>,
// }

// impl AstronomicalCalculator {
//     /// Creates a new `AstronomicalCalculator` with full parameter control.
//     ///
//     /// All parameters are validated. Returns an error if any value is out of range.
//     ///
//     /// # Arguments
//     ///
//     /// * `datetime` - Date and time (UTC). Year must be in (-2000, 6000)
//     /// * `delta_ut1` - UT1-UT correction in seconds. Range: (-1.0, 1.0]. Typical: 0.0
//     /// * `delta_t` - TT-UT correction in seconds. Range: (-8000.0, 8000.0). Use [`delta_t::estimate_from_date_like`] for estimation
//     /// * `longitude` - Observer longitude in degrees. Range: (-180.0, 180.0). Positive = East
//     /// * `latitude` - Observer latitude in degrees. Range: (-90.0, 90.0). Positive = North
//     /// * `elevation` - Elevation above sea level in meters. Range: (-6,500,000.0, 6,500,000.0)
//     /// * `pressure` - Atmospheric pressure in millibars. Range: (0.0, 5000.0). Typical: 1013.25
//     /// * `temperature` - Temperature in Celsius. Range: (-273.0, 6000.0). Typical: 15.0
//     /// * `slope` - Surface slope in degrees. Range: (-360.0, 360.0). 0° = horizontal
//     /// * `azm_rotation` - Surface azimuth in degrees. Range: (-360.0, 360.0). 0° = south-facing (astronomical convention)
//     /// * `atmos_refract` - Atmospheric refraction at horizon in degrees. Range: (-5.0, 5.0). Typical: 0.5667
//     ///
//     /// # Errors
//     ///
//     /// Returns [`Error`] variants for invalid parameters. See individual error types for ranges.
//     ///
//     /// For convenience, use [`Self::standard`] for typical atmospheric conditions with automatic ΔT estimation.
//     #[allow(clippy::too_many_arguments)]
//     pub fn new(
//         datetime: DateTime<Utc>,
//         delta_ut1: f64,
//         delta_t: f64,
//         longitude: f64,
//         latitude: f64,
//         elevation: f64,
//         pressure: f64,
//         temperature: f64,
//         slope: f64,
//         azm_rotation: f64,
//         atmos_refract: f64,
//     ) -> Result<Self, Error> {
//         // Validate year range
//         if datetime.year() < -2000 || datetime.year() > 6000 {
//             return Err(Error::InvalidYear(datetime.year()));
//         }

//         // Validate atmospheric conditions
//         if !(0.0..=5000.0).contains(&pressure) {
//             return Err(Error::InvalidPressure(pressure));
//         }
//         if !(-273.0..=6000.0).contains(&temperature) {
//             return Err(Error::InvalidTemperature(temperature));
//         }

//         // Validate time corrections
//         if delta_ut1 <= -1.0 || delta_ut1 > 1.0 {
//             return Err(Error::InvalidDeltaUt1(delta_ut1));
//         }
//         if delta_t.abs() > 8000.0 {
//             return Err(Error::InvalidDeltaT(delta_t));
//         }

//         // Validate geographic coordinates
//         if longitude.abs() > 180.0 {
//             return Err(Error::InvalidLongitude(longitude));
//         }
//         if latitude.abs() > 90.0 {
//             return Err(Error::InvalidLatitude(latitude));
//         }
//         if elevation < -6500000.0 {
//             return Err(Error::InvalidElevation(elevation));
//         }

//         // Validate surface parameters
//         if slope.abs() > 360.0 {
//             return Err(Error::InvalidSlope(slope));
//         }
//         if azm_rotation.abs() > 360.0 {
//             return Err(Error::InvalidAzmRotation(azm_rotation));
//         }
//         if atmos_refract.abs() > 5.0 {
//             return Err(Error::InvalidAtmosRefract(atmos_refract));
//         }

//         Ok(Self {
//             datetime,
//             delta_ut1,
//             delta_t,
//             longitude,
//             latitude,
//             elevation,
//             pressure,
//             temperature,
//             slope,
//             azm_rotation,
//             atmos_refract,
//             intermediates: None,
//         })
//     }
//     /// Creates a new `AstronomicalCalculator` with standard atmospheric conditions and automatic ΔT estimation.
//     ///
//     /// Convenience constructor using standard values: pressure 1013.25 mbar, temperature 15°C,
//     /// horizontal surface (`slope`=0°, `azm_rotation`=0°), refraction 0.5667°, and automatic ΔT estimation.
//     ///
//     /// # Arguments
//     ///
//     /// * `datetime` - Date and time (UTC). Year must be in (-500, 3000) for ΔT estimation, else use [`Self::new`]
//     /// * `longitude` - Observer longitude in degrees. Range: (-180.0, 180.0). Positive = East
//     /// * `latitude` - Observer latitude in degrees. Range: (-90.0, 90.0). Positive = North
//     /// * `elevation` - Elevation above sea level in meters. Range: (-6,500,000.0, 6,500,000.0)
//     ///
//     /// # Errors
//     ///
//     /// Returns [`Error::UnableToEstimateDeltaT`] if year is outside (-500, 3000), or other [`Error`] variants for invalid parameters.
//     ///
//     /// Use [`Self::new`] for custom atmospheric conditions, tilted surfaces, or dates outside the ΔT estimation range.
//     pub fn standard(datetime: DateTime<Utc>, longitude: f64, latitude: f64, elevation: f64) -> Result<Self, Error> {
//         if let Some(delta_t) = delta_t::estimate_from_date_like(&datetime) {
//             Ok(Self {
//                 datetime,
//                 delta_ut1: 0.0,
//                 delta_t,
//                 longitude,
//                 latitude,
//                 elevation,
//                 pressure: 1013.25,
//                 temperature: 15.0,
//                 slope: 0.0,
//                 azm_rotation: 0.0,
//                 atmos_refract: 0.5667,
//                 intermediates: None,
//             })
//         } else {
//             Err(Error::UnableToEstimateDeltaT)
//         }
//     }

//     /// Calculates solar position and rise/transit/set times.
//     ///
//     /// This is the main entry point for performing complete solar calculations.
//     /// It returns both the instantaneous solar position (zenith, azimuth, incidence)
//     /// and the rise, transit, and set times for the day.
//     ///
//     /// For repeated calculations on the same instance, consider using `calculate_mut()`
//     /// which caches intermediate values for better performance.
//     pub fn calculate(&self) -> Calculations {
//         let solar_position_output = self.calculate_solar_position();
//         let solar_times_output = self.calculate_rise_transit_set_times();
//         Calculations {
//             position: solar_position_output,
//             solar_transit_time: solar_times_output.solar_transit_time,
//             sunrise_time: solar_times_output.sunrise_time,
//             sunset_time: solar_times_output.sunset_time,
//             equation_of_time: solar_times_output.equation_of_time,
//         }
//     }

//     /// Calculates solar position and rise/transit/set times with caching optimization.
//     ///
//     /// This method is similar to `calculate()` but caches intermediate astronomical values
//     /// to avoid redundant calculations when performing multiple operations on the same
//     /// `AstronomicalCalculator` instance. Use this when you need to call multiple methods
//     /// that would otherwise recalculate the same intermediate values.
//     pub fn calculate_mut(&mut self) -> Calculations {
//         let _ = self.calculate_intermediate_solar_values_mut();
//         let solar_position_output = self.calculate_solar_position();
//         let solar_times_output = self.calculate_rise_transit_set_times();
//         Calculations {
//             position: solar_position_output,
//             solar_transit_time: solar_times_output.solar_transit_time,
//             sunrise_time: solar_times_output.sunrise_time,
//             sunset_time: solar_times_output.sunset_time,
//             equation_of_time: solar_times_output.equation_of_time,
//         }
//     }

//     /// Calculates the complete solar position for the given inputs.
//     ///
//     /// This is the main entry point for solar position calculations. It returns
//     /// zenith angle, azimuth, and sunrise/sunset times.
//     ///
//     /// The solar position (zenith, azimuth, incidence) is always calculated.
//     /// However, sunrise/sunset/transit times may be `None` if the sun doesn't
//     /// rise or set at this location and date (e.g., polar day/night).
//     fn calculate_solar_position(&self) -> SolarPositionOutput {
//         let intermediates = self.calculate_intermediate_solar_values();

//         // Calculate observer's local hour angle
//         let hour_angle = calculate_observer_hour_angle(
//             intermediates.greenwich_apparent_sidereal_time,
//             self.longitude,
//             intermediates.geocentric_right_ascension,
//         );

//         // Calculate parallax correction
//         let equatorial_horizontal_parallax =
//             calculate_sun_equatorial_horizontal_parallax(intermediates.earth_sun_distance);

//         let (topocentric_declination, right_ascension_parallax) =
//             calculate_right_ascension_parallax_and_topocentric_declination(
//                 self.latitude,
//                 self.elevation,
//                 equatorial_horizontal_parallax,
//                 hour_angle,
//                 intermediates.geocentric_declination,
//             );

//         // Calculate topocentric coordinates
//         let topocentric_hour_angle = calculate_topocentric_local_hour_angle(hour_angle, right_ascension_parallax);
//         let elevation_uncorrected =
//             calculate_topocentric_elevation_angle(self.latitude, topocentric_declination, topocentric_hour_angle);

//         // Apply atmospheric refraction correction
//         let refraction_correction = calculate_atmospheric_refraction_correction(
//             self.pressure,
//             self.temperature,
//             self.atmos_refract,
//             elevation_uncorrected,
//         );
//         let elevation_corrected = apply_atmospheric_refraction_correction(elevation_uncorrected, refraction_correction);
//         // Calculate final position angles
//         let zenith = calculate_zenith_angle_from_elevation(elevation_corrected);
//         let azimuth_astronomical =
//             calculate_topocentric_azimuth_astronomical(topocentric_hour_angle, self.latitude, topocentric_declination);
//         let azimuth = convert_astronomical_to_observer_azimuth(azimuth_astronomical);
//         let incidence = calculate_surface_incidence_angle(zenith, azimuth_astronomical, self.azm_rotation, self.slope);

//         SolarPositionOutput {
//             zenith,
//             azimuth_astronomical,
//             azimuth,
//             incidence,
//         }
//     }

//     /// Calculates intermediate astronomical values needed for solar position.
//     ///
//     /// This includes Julian dates, Earth's position, nutation, obliquity, and
//     /// geocentric solar coordinates.
//     fn calculate_intermediate_solar_values(&self) -> SolarPositionIntermediates {
//         if let Some(intermediates) = self.intermediates {
//             return intermediates;
//         }
//         let julian_day = julian_day(&self.datetime, self.delta_ut1);
//         self.calculate_intermediates_for_julian_day(julian_day)
//     }

//     /// Calculates intermediate astronomical values needed for solar position and stores them in the calculator.
//     ///
//     /// This includes Julian dates, Earth's position, nutation, obliquity, and
//     /// geocentric solar coordinates.
//     fn calculate_intermediate_solar_values_mut(&mut self) -> SolarPositionIntermediates {
//         if let Some(intermediates) = self.intermediates {
//             return intermediates;
//         }
//         let julian_day = julian_day(&self.datetime, self.delta_ut1);
//         let intermediates = self.calculate_intermediates_for_julian_day(julian_day);
//         self.intermediates.replace(intermediates);
//         intermediates
//     }

//     /// Calculates intermediate values for a specific Julian Day.
//     ///
//     /// This is used both for the target datetime and for surrounding days
//     /// needed for rise/transit/set calculations.
//     fn calculate_intermediates_for_julian_day(&self, julian_day: f64) -> SolarPositionIntermediates {
//         // Time calculations
//         let julian_century = julian_century_from_julian_day(julian_day);
//         let julian_ephemeris_day = time::julian_ephemeris_day_from_julian_day(julian_day, self.delta_t);
//         let julian_ephemeris_century = julian_ephemeris_century_from_julian_ephemeris_day(julian_ephemeris_day);
//         let julian_ephemeris_millennium =
//             julian_ephemeris_millennium_from_julian_ephemeris_century(julian_ephemeris_century);

//         // Earth's heliocentric position
//         let earth_heliocentric_longitude = calculate_earth_heliocentric_longitude(julian_ephemeris_millennium);
//         let earth_heliocentric_latitude = calculate_earth_heliocentric_latitude(julian_ephemeris_millennium);
//         let earth_sun_distance = calculate_earth_radius_vector(julian_ephemeris_millennium);

//         // Convert to geocentric coordinates (Sun's position as seen from Earth's center)
//         let sun_geocentric_longitude = convert_heliocentric_to_geocentric_longitude(earth_heliocentric_longitude);
//         let sun_geocentric_latitude = convert_heliocentric_to_geocentric_latitude(earth_heliocentric_latitude);

//         // Nutation and obliquity calculations
//         let mean_elongation = calculate_mean_elongation_moon_sun(julian_ephemeris_century);
//         let mean_anomaly_sun = calculate_mean_anomaly_sun(julian_ephemeris_century);
//         let mean_anomaly_moon = calculate_mean_anomaly_moon(julian_ephemeris_century);
//         let moon_argument_of_latitude = calculate_argument_latitude_moon(julian_ephemeris_century);
//         let moon_ascending_node_longitude = calculate_ascending_longitude_moon(julian_ephemeris_century);

//         let (nutation_longitude, nutation_obliquity) = calculate_nutation_longitude_and_obliquity(
//             julian_ephemeris_century,
//             [
//                 mean_elongation,
//                 mean_anomaly_sun,
//                 mean_anomaly_moon,
//                 moon_argument_of_latitude,
//                 moon_ascending_node_longitude,
//             ],
//         );

//         let ecliptic_mean_obliquity = calculate_ecliptic_mean_obliquity(julian_ephemeris_millennium);
//         let ecliptic_true_obliquity = calculate_ecliptic_true_obliquity(nutation_obliquity, ecliptic_mean_obliquity);

//         // Aberration and apparent position
//         let aberration = calculate_aberration_correction(earth_sun_distance);
//         let sun_apparent_longitude =
//             calculate_apparent_sun_longitude(sun_geocentric_longitude, nutation_longitude, aberration);

//         // Sidereal time
//         let greenwich_mean_sidereal_time = calculate_greenwich_mean_sidereal_time(julian_day, julian_century);
//         let greenwich_apparent_sidereal_time = calculate_greenwich_apparent_sidereal_time(
//             greenwich_mean_sidereal_time,
//             nutation_longitude,
//             ecliptic_true_obliquity,
//         );

//         // Geocentric equatorial coordinates
//         let geocentric_right_ascension = calculate_geocentric_right_ascension(
//             sun_apparent_longitude,
//             ecliptic_true_obliquity,
//             sun_geocentric_latitude,
//         );
//         let geocentric_declination =
//             calculate_geocentric_declination(sun_geocentric_latitude, ecliptic_true_obliquity, sun_apparent_longitude);

//         SolarPositionIntermediates {
//             julian_day,
//             julian_century,
//             julian_ephemeris_day,
//             julian_ephemeris_century,
//             julian_ephemeris_millennium,
//             earth_heliocentric_longitude,
//             earth_heliocentric_latitude,
//             earth_sun_distance,
//             sun_geocentric_longitude,
//             sun_geocentric_latitude,
//             mean_elongation,
//             mean_anomaly_sun,
//             mean_anomaly_moon,
//             moon_argument_of_latitude,
//             moon_ascending_node_longitude,
//             nutation_longitude,
//             nutation_obliquity,
//             ecliptic_mean_obliquity,
//             ecliptic_true_obliquity,
//             aberration,
//             sun_apparent_longitude,
//             greenwich_mean_sidereal_time,
//             greenwich_apparent_sidereal_time,
//             geocentric_right_ascension,
//             geocentric_declination,
//         }
//     }

//     /// Calculates sunrise, sunset, and solar transit (noon) times.
//     ///
//     /// Uses iterative refinement to calculate accurate rise, transit, and set times
//     /// based on interpolated solar coordinates.
//     ///
//     /// # Returns
//     ///
//     /// * `Some(RiseTransitSetTimes)` - If the sun rises and sets
//     /// * `None` - If the sun doesn't rise or set (polar day/night)
//     fn calculate_rise_transit_set_times(&self) -> RiseTransitSetTimes {
//         let intermediates = self.calculate_intermediate_solar_values();
//         let equation_of_time = equation_of_time(
//             calculate_sun_mean_longitude(intermediates.julian_ephemeris_millennium),
//             intermediates.geocentric_right_ascension,
//             intermediates.nutation_longitude,
//             intermediates.ecliptic_true_obliquity,
//         );
//         // It is not possible to fail to convert a date to a naive date for a naive datetime
//         #[allow(clippy::unwrap_used)]
//         let utday_res = NaiveDate::from_ymd_opt(self.datetime.year(), self.datetime.month(), self.datetime.day())
//             .unwrap()
//             .and_hms_opt(0, 0, 0)
//             .unwrap()
//             .and_utc();
//         let ttday0 = utday_res - Duration::milliseconds(self.delta_t.round() as i64);
//         //Naive days can't only fail adding/subtracting days if the overflows.
//         // However, we validate the date in the constructor, so we can safely unwrap.
//         #[allow(clippy::unwrap_used)]
//         let ttdayn1 = ttday0.checked_sub_days(Days::new(1)).unwrap();
//         #[allow(clippy::unwrap_used)]
//         let ttdayp1 = ttday0.checked_add_days(Days::new(1)).unwrap();

//         //Calculate apparent zenith for utday_res
//         let utday_res_calc = AstronomicalCalculator {
//             datetime: utday_res,
//             delta_ut1: self.delta_ut1,
//             delta_t: self.delta_t,
//             atmos_refract: 0.0,
//             azm_rotation: 0.0,
//             elevation: 0.0,
//             latitude: 0.0,
//             longitude: 0.0,
//             pressure: 0.0,
//             slope: 0.0,
//             temperature: 0.0,
//             intermediates: None,
//         };
//         let v = utday_res_calc
//             .calculate_intermediate_solar_values()
//             .greenwich_apparent_sidereal_time;

//         let intermidiate_values = [ttdayn1, ttday0, ttdayp1].map(|i| {
//             let mut calc = AstronomicalCalculator {
//                 datetime: i,
//                 delta_ut1: self.delta_ut1,
//                 delta_t: self.delta_t,
//                 atmos_refract: 0.0,
//                 azm_rotation: 0.0,
//                 elevation: 0.0,
//                 latitude: 0.0,
//                 longitude: 0.0,
//                 pressure: 0.0,
//                 slope: 0.0,
//                 temperature: 0.0,
//                 intermediates: None,
//             };
//             calc.calculate_intermediate_solar_values_mut()
//         });
//         let m0 = (intermidiate_values[1].geocentric_right_ascension - self.longitude - v) / 360.0;

//         let today_declination = intermidiate_values[1].geocentric_declination;

//         let cos_arg = (-0.8333_f64.to_radians().sin()
//             - self.latitude.to_radians().sin() * today_declination.to_radians().sin())
//             / (self.latitude.to_radians().cos() * today_declination.to_radians().cos());
//         let valid_sunrise_set = cos_arg.abs() <= 1.0;

//         let h0 = floored_mod(cos_arg.acos().to_degrees(), 180.0);

//         let mut m = [0.0_f64; 3];
//         m[SUN_TRANSIT] = floored_mod(m0, 1.0);
//         m[SUN_RISE] = m[SUN_TRANSIT] - h0 / 360.0;
//         m[SUN_SET] = m[SUN_TRANSIT] + h0 / 360.0;
//         let add_a_day = m[SUN_SET] >= 1.0;
//         let subtract_a_day = m[SUN_RISE] < 0.0;
//         m[SUN_RISE] = floored_mod(m[SUN_RISE], 1.0);
//         m[SUN_SET] = floored_mod(m[SUN_SET], 1.0);

//         let vs = m.map(|i| v + 360.985647_f64 * i);
//         let n = m.map(|i| i + self.delta_t / 86400.0);

//         let mut a =
//             intermidiate_values[1].geocentric_right_ascension - intermidiate_values[0].geocentric_right_ascension;
//         a = if a.abs() > 2.0 { floored_mod(a, 1.0) } else { a };
//         let mut ap = intermidiate_values[1].geocentric_declination - intermidiate_values[0].geocentric_declination;
//         ap = if ap.abs() > 2.0 { floored_mod(ap, 1.0) } else { ap };
//         let mut b =
//             intermidiate_values[2].geocentric_right_ascension - intermidiate_values[1].geocentric_right_ascension;
//         b = if b.abs() > 2.0 { floored_mod(b, 1.0) } else { b };
//         let mut bp = intermidiate_values[2].geocentric_declination - intermidiate_values[1].geocentric_declination;
//         bp = if bp.abs() > 2.0 { floored_mod(bp, 1.0) } else { bp };
//         let c = b - a;
//         let cp = bp - ap;

//         let alpha_prime = n.map(|n| intermidiate_values[1].geocentric_right_ascension + (n * (a + b + c * n)) / 2.0);
//         let delta_prime = n.map(|n| intermidiate_values[1].geocentric_declination + (n * (ap + bp + cp * n)) / 2.0);

//         let mut hp = [0_f64; 3];
//         for i in 0..3 {
//             let z = floored_mod(vs[i] + self.longitude - alpha_prime[i], 360.0);
//             hp[i] = if z >= 180.0 { z - 360.0 } else { z };
//         }

//         let mut h = [0_f64; 3];
//         for i in 0..3 {
//             h[i] = self.latitude.to_radians().sin() * delta_prime[i].to_radians().sin()
//                 + self.latitude.to_radians().cos() * delta_prime[i].to_radians().cos() * hp[i].to_radians().cos();
//             h[i] = h[i].asin().to_degrees();
//         }

//         let t = m[SUN_TRANSIT] - hp[SUN_TRANSIT] / 360.0;
//         let mut r = m[SUN_RISE]
//             + (h[SUN_RISE] + 0.8333)
//                 / (360.0
//                     * delta_prime[SUN_RISE].to_radians().cos()
//                     * self.latitude.to_radians().cos()
//                     * hp[SUN_RISE].to_radians().sin());
//         let mut s = m[SUN_SET]
//             + (h[SUN_SET] + 0.8333)
//                 / (360.0
//                     * delta_prime[SUN_SET].to_radians().cos()
//                     * self.latitude.to_radians().cos()
//                     * hp[SUN_SET].to_radians().sin());

//         if add_a_day {
//             s += 1.0
//         }
//         if subtract_a_day {
//             r -= 1.0
//         }

//         let sunrise = utday_res + Duration::milliseconds((r * 24.0 * 3600.0 * 1000.0) as i64);
//         let solar_transit = utday_res + Duration::milliseconds((t * 24.0 * 3600.0 * 1000.0) as i64);
//         let sunset = utday_res + Duration::milliseconds((s * 24.0 * 3600.0 * 1000.0) as i64);

//         RiseTransitSetTimes {
//             equation_of_time,
//             solar_transit_time: solar_transit,
//             sunrise_time: if sunrise < solar_transit && valid_sunrise_set {
//                 Some(sunrise)
//             } else {
//                 None
//             },
//             sunset_time: if solar_transit < sunset && valid_sunrise_set {
//                 Some(sunset)
//             } else {
//                 None
//             },
//         }
//     }
// }

// // ============================================================================
// // Intermediate Values
// // ============================================================================

// /// Intermediate astronomical values calculated during solar position determination.
// ///
// /// These values represent various astronomical parameters computed from
// /// the observer's time and location.
// #[derive(Debug, Copy, Clone)]
// struct SolarPositionIntermediates {
//     // Time values
//     #[allow(unused)]
//     julian_day: f64,
//     #[allow(unused)]
//     julian_century: f64,
//     #[allow(unused)]
//     julian_ephemeris_day: f64,
//     #[allow(unused)]
//     julian_ephemeris_century: f64,
//     #[allow(unused)]
//     julian_ephemeris_millennium: f64,

//     // Earth's position
//     #[allow(unused)]
//     earth_heliocentric_longitude: f64,
//     #[allow(unused)]
//     earth_heliocentric_latitude: f64,
//     earth_sun_distance: f64,

//     // Sun's geocentric position
//     #[allow(unused)]
//     sun_geocentric_longitude: f64,
//     #[allow(unused)]
//     sun_geocentric_latitude: f64,

//     // Lunar orbital parameters (for nutation)
//     #[allow(unused)]
//     mean_elongation: f64,
//     #[allow(unused)]
//     mean_anomaly_sun: f64,
//     #[allow(unused)]
//     mean_anomaly_moon: f64,
//     #[allow(unused)]
//     moon_argument_of_latitude: f64,
//     #[allow(unused)]
//     moon_ascending_node_longitude: f64,

//     // Nutation and obliquity
//     #[allow(unused)]
//     nutation_longitude: f64,
//     #[allow(unused)]
//     nutation_obliquity: f64,
//     #[allow(unused)]
//     ecliptic_mean_obliquity: f64,
//     #[allow(unused)]
//     ecliptic_true_obliquity: f64,

//     // Aberration and apparent position
//     #[allow(unused)]
//     aberration: f64,
//     #[allow(unused)]
//     sun_apparent_longitude: f64,

//     // Sidereal time
//     #[allow(unused)]
//     greenwich_mean_sidereal_time: f64,
//     greenwich_apparent_sidereal_time: f64,

//     // Geocentric equatorial coordinates
//     geocentric_right_ascension: f64,
//     geocentric_declination: f64,
// }

// // ============================================================================
// // Output Values
// // ============================================================================

// /// Solar position output containing position angles and rise/transit/set times.
// ///
// /// All angles are in degrees, and times are in hours (decimal format).
// #[derive(Debug, PartialEq)]
// pub struct SolarPositionOutput {
//     /// Solar zenith angle in degrees (angle from vertical)
//     /// 0° = directly overhead, 90° = at horizon
//     pub zenith: f64,

//     /// Astronomical azimuth in degrees (measured from south, eastward positive)
//     /// 0° = south, 90° = west, 180° = north, 270° = east
//     pub azimuth_astronomical: f64,

//     /// Observer azimuth in degrees (measured from north, eastward positive)
//     /// 0° = north, 90° = east, 180° = south, 270° = west
//     pub azimuth: f64,

//     /// Surface incidence angle in degrees (angle between sun and surface normal)
//     pub incidence: f64,
// }

// /// Solar position output containing position angles and rise/transit/set times.
// ///
// /// All angles are in degrees, and times are in hours (decimal format).
// #[derive(Debug, PartialEq)]
// pub struct SolarTimesOutput {
//     /// Solar transit time (solar noon) in UTC
//     /// `None` if the sun doesn't rise or set (polar day/night)
//     pub solar_transit_time: DateTime<Utc>,

//     /// Sunrise time in UTC
//     /// `None` if the sun doesn't rise or set (polar day/night)
//     pub sunrise_time: Option<DateTime<Utc>>,

//     /// Sunset time in UTC
//     /// `None` if the sun doesn't rise or set (polar day/night)
//     pub sunset_time: Option<DateTime<Utc>>,
// }

// /// Combined solar position and rise/transit/set time calculations.
// ///
// /// Contains both the instantaneous solar position and the rise/transit/set times for the day.
// #[derive(Debug, PartialEq)]
// pub struct Calculations {
//     /// Solar position output containing zenith, azimuth, and incidence angles
//     pub position: SolarPositionOutput,
//     /// Solar transit (noon) time in UTC
//     pub solar_transit_time: DateTime<Utc>,

//     /// Sunrise time in UTC, will be None if the sun doesn't rise or set (polar day/night)
//     pub sunrise_time: Option<DateTime<Utc>>,

//     /// Sunset time in UTC, will be None if the sun doesn't rise or set (polar day/night)
//     pub sunset_time: Option<DateTime<Utc>>,

//     /// Equation of time in minutes
//     pub equation_of_time: f64,
// }

// /// Rise, transit, and set times with additional diagnostic information.
// #[derive(Debug, PartialEq)]
// struct RiseTransitSetTimes {
//     pub equation_of_time: f64,
//     pub solar_transit_time: DateTime<Utc>,
//     pub sunrise_time: Option<DateTime<Utc>>,
//     pub sunset_time: Option<DateTime<Utc>>,
// }
