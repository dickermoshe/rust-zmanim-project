extern crate std;

#[allow(unused_imports)]
use core_maths::CoreFloat;

use crate::{
    math::{eval_cubic, normalize_degrees_360},
    terms::{B_TERMS, L_TERMS, PE_TERMS, R_TERMS, Y_TERMS},
};

// ============================================================================
// Constants
// ============================================================================

/// Angular radius of the Sun in degrees (approximately 16 arcminutes)
pub(crate) const SUN_RADIUS: f64 = 0.26667;

/// Earth's equatorial radius in meters
const EARTH_EQUATORIAL_RADIUS_METERS: f64 = 6_378_140.0;

/// Earth's flattening factor (ratio of difference between equatorial and polar radii to equatorial radius)
const EARTH_FLATTENING_FACTOR: f64 = 0.99664719;

/// Constant for equatorial horizontal parallax calculation (in arcseconds)
const PARALLAX_CONSTANT: f64 = 8.794;

/// Constant for aberration correction calculation (in arcseconds)
const ABERRATION_CONSTANT: f64 = 20.4898;

/// Conversion factor from arcseconds to degrees
const ARCSECONDS_TO_DEGREES: f64 = 1.0 / 3600.0;

/// Mean rate of increase of Greenwich sidereal time (degrees per day)
const SIDEREAL_TIME_RATE: f64 = 360.98564736629;

/// Reference Julian Day for J2000.0 epoch
const J2000_EPOCH_JD: f64 = 2451545.0;

/// Base Greenwich mean sidereal time at J2000.0 epoch (degrees)
const GREENWICH_MEAN_SIDEREAL_TIME_BASE: f64 = 280.46061837;

/// Scaling factor for Earth position calculations (10^8)
const EARTH_POSITION_SCALE_FACTOR: f64 = 1.0e8;

/// Scaling factor for nutation calculations (10^7 arcseconds)
const NUTATION_SCALE_FACTOR: f64 = 36_000_000.0;

// ============================================================================
// Solar Position Calculations
// ============================================================================

/// Calculates the mean longitude of the sun.
///
/// Uses Horner's method for polynomial evaluation for efficiency.
///
/// # Arguments
/// * `julian_millennium` - Julian ephemeris millennium
///
/// # Returns
/// Mean longitude in degrees (0° to 360°)
pub(crate) fn calculate_sun_mean_longitude(julian_millennium: f64) -> f64 {
    // Horner's method for: L0 = 280.4664567 + 360007.6982779*jme + 0.03032028*jme^2
    //                          + jme^3/49931 - jme^4/15300 - jme^5/2000000
    let result = -1.0 / 2_000_000.0;
    let result = -1.0 / 15_300.0 + julian_millennium * result;
    let result = 1.0 / 49_931.0 + julian_millennium * result;
    let result = 0.030_320_28 + julian_millennium * result;
    let result = 360_007.698_277_9 + julian_millennium * result;
    let result = 280.466_456_7 + julian_millennium * result;
    normalize_degrees_360(result)
}

/// Calculates the incidence angle on a tilted surface.
///
/// # Arguments
/// * `zenith` - Solar zenith angle in degrees
/// * `azimuth_astro` - Astronomical azimuth (measured from south) in degrees
/// * `surface_azimuth` - Surface azimuth rotation in degrees
/// * `surface_slope` - Surface slope from horizontal in degrees
///
/// # Returns
/// Surface incidence angle in degrees
pub(crate) fn calculate_surface_incidence_angle(
    zenith: f64,
    azimuth_astro: f64,
    surface_azimuth: f64,
    surface_slope: f64,
) -> f64 {
    let zenith_rad = zenith.to_radians();
    let slope_rad = surface_slope.to_radians();
    let azimuth_diff_rad = (azimuth_astro - surface_azimuth).to_radians();

    (zenith_rad.cos() * slope_rad.cos() + slope_rad.sin() * zenith_rad.sin() * azimuth_diff_rad.cos())
        .acos()
        .to_degrees()
}

// ============================================================================
// Coordinate Transformations
// ============================================================================

/// Converts astronomical azimuth to observer azimuth.
///
/// Astronomical azimuth is measured from south (0° = south, 90° = west).
/// Observer azimuth is measured from north (0° = north, 90° = east).
///
/// # Arguments
/// * `azimuth_astro` - Astronomical azimuth in degrees
///
/// # Returns
/// Observer azimuth in degrees (0° to 360°)
pub(crate) fn convert_astronomical_to_observer_azimuth(azimuth_astro: f64) -> f64 {
    normalize_degrees_360(azimuth_astro + 180.0)
}

/// Applies atmospheric refraction correction to elevation angle.
///
/// # Arguments
/// * `elevation_uncorrected` - Elevation angle without refraction in degrees
/// * `refraction_correction` - Atmospheric refraction correction in degrees
///
/// # Returns
/// Corrected elevation angle in degrees
pub(crate) fn apply_atmospheric_refraction_correction(elevation_uncorrected: f64, refraction_correction: f64) -> f64 {
    elevation_uncorrected + refraction_correction
}

/// Calculates zenith angle from elevation angle.
///
/// Zenith angle is the complement of elevation angle.
///
/// # Arguments
/// * `elevation` - Elevation angle in degrees
///
/// # Returns
/// Zenith angle in degrees
pub(crate) fn calculate_zenith_angle_from_elevation(elevation: f64) -> f64 {
    90.0 - elevation
}

/// Calculates topocentric astronomical azimuth angle.
///
/// # Arguments
/// * `hour_angle_prime` - Topocentric local hour angle in degrees
/// * `latitude` - Observer's latitude in degrees
/// * `declination_prime` - Topocentric sun declination in degrees
///
/// # Returns
/// Astronomical azimuth in degrees (measured from south)
pub(crate) fn calculate_topocentric_azimuth_astronomical(
    hour_angle_prime: f64,
    latitude: f64,
    declination_prime: f64,
) -> f64 {
    let hour_angle_rad = hour_angle_prime.to_radians();
    let latitude_rad = latitude.to_radians();
    let declination_rad = declination_prime.to_radians();

    normalize_degrees_360(
        hour_angle_rad
            .sin()
            .atan2(hour_angle_rad.cos() * latitude_rad.sin() - declination_rad.tan() * latitude_rad.cos())
            .to_degrees(),
    )
}

/// Converts heliocentric longitude to geocentric longitude.
///
/// Geocentric longitude is heliocentric longitude + 180° (viewed from Earth vs Sun).
///
/// # Arguments
/// * `heliocentric_longitude` - Heliocentric longitude in degrees
///
/// # Returns
/// Geocentric longitude in degrees (0° to 360°)
pub(crate) fn convert_heliocentric_to_geocentric_longitude(heliocentric_longitude: f64) -> f64 {
    let mut geocentric = heliocentric_longitude + 180.0;
    if geocentric >= 360.0 {
        geocentric -= 360.0;
    }
    geocentric
}

/// Converts heliocentric latitude to geocentric latitude.
///
/// Geocentric latitude is the negative of heliocentric latitude.
///
/// # Arguments
/// * `heliocentric_latitude` - Heliocentric latitude in degrees
///
/// # Returns
/// Geocentric latitude in degrees
pub(crate) fn convert_heliocentric_to_geocentric_latitude(heliocentric_latitude: f64) -> f64 {
    -heliocentric_latitude
}

// ============================================================================
// Lunar Orbital Parameters
// ============================================================================

/// Calculates the mean elongation of the Moon from the Sun.
///
/// This is one of the fundamental arguments for nutation calculations.
///
/// # Arguments
/// * `julian_century` - Julian ephemeris century
///
/// # Returns
/// Mean elongation in degrees
pub(crate) fn calculate_mean_elongation_moon_sun(julian_century: f64) -> f64 {
    eval_cubic(1.0 / 189_474.0, -0.0019142, 445_267.111_48, 297.85036, julian_century)
}

/// Calculates the mean anomaly of the Sun.
///
/// This is one of the fundamental arguments for nutation calculations.
///
/// # Arguments
/// * `julian_century` - Julian ephemeris century
///
/// # Returns
/// Mean anomaly in degrees
pub(crate) fn calculate_mean_anomaly_sun(julian_century: f64) -> f64 {
    eval_cubic(-1.0 / 300_000.0, -0.0001603, 35_999.050_34, 357.52772, julian_century)
}

/// Calculates the mean anomaly of the Moon.
///
/// This is one of the fundamental arguments for nutation calculations.
///
/// # Arguments
/// * `julian_century` - Julian ephemeris century
///
/// # Returns
/// Mean anomaly in degrees
pub(crate) fn calculate_mean_anomaly_moon(julian_century: f64) -> f64 {
    eval_cubic(1.0 / 56_250.0, 0.0086972, 477_198.867398, 134.96298, julian_century)
}

/// Calculates the argument of latitude of the Moon.
///
/// This is one of the fundamental arguments for nutation calculations.
///
/// # Arguments
/// * `julian_century` - Julian ephemeris century
///
/// # Returns
/// Argument of latitude in degrees
pub(crate) fn calculate_argument_latitude_moon(julian_century: f64) -> f64 {
    eval_cubic(1.0 / 327_270.0, -0.0036825, 483_202.017538, 93.27191, julian_century)
}

/// Calculates the longitude of the ascending node of the Moon's mean orbit.
///
/// This is one of the fundamental arguments for nutation calculations.
///
/// # Arguments
/// * `julian_century` - Julian ephemeris century
///
/// # Returns
/// Ascending node longitude in degrees
pub(crate) fn calculate_ascending_longitude_moon(julian_century: f64) -> f64 {
    eval_cubic(1.0 / 450_000.0, 0.0020708, -1934.136261, 125.04452, julian_century)
}

// ============================================================================
// Ecliptic and Nutation
// ============================================================================

/// Calculates the mean obliquity of the ecliptic.
///
/// The obliquity of the ecliptic is the angle between Earth's equatorial plane
/// and the ecliptic plane (Earth's orbital plane).
///
/// # Arguments
/// * `julian_millennium` - Julian ephemeris millennium
///
/// # Returns
/// Mean obliquity in arcseconds
pub(crate) fn calculate_ecliptic_mean_obliquity(julian_millennium: f64) -> f64 {
    let u = julian_millennium / 10.0;

    // Polynomial evaluation using temporary variables for clarity
    let term1 = 5.79 + u * 2.45;
    let term2 = 27.87 + u * term1;
    let term3 = 7.12 + u * term2;
    let term4 = -39.05 + u * term3;
    let term5 = -249.67 + u * term4;
    let term6 = -51.38 + u * term5;
    let term7 = 1999.25 + u * term6;
    let term8 = -1.55 + u * term7;
    let term9 = -4680.93 + u * term8;

    84381.448 + u * term9
}

/// Calculates the true obliquity of the ecliptic.
///
/// True obliquity = mean obliquity + nutation in obliquity
///
/// # Arguments
/// * `nutation_obliquity` - Nutation in obliquity in arcseconds
/// * `mean_obliquity` - Mean obliquity in arcseconds
///
/// # Returns
/// True obliquity in degrees
pub(crate) fn calculate_ecliptic_true_obliquity(nutation_obliquity: f64, mean_obliquity: f64) -> f64 {
    nutation_obliquity + mean_obliquity * ARCSECONDS_TO_DEGREES
}

/// Calculates the aberration correction for the sun's apparent position.
///
/// Stellar aberration is caused by the finite speed of light combined with
/// Earth's orbital motion.
///
/// # Arguments
/// * `earth_sun_distance` - Distance from Earth to Sun in AU
///
/// # Returns
/// Aberration correction in degrees
pub(crate) fn calculate_aberration_correction(earth_sun_distance: f64) -> f64 {
    -ABERRATION_CONSTANT / (3600.0 * earth_sun_distance)
}

/// Calculates the apparent longitude of the sun.
///
/// Apparent longitude = geometric longitude + nutation in longitude + aberration
///
/// # Arguments
/// * `geometric_longitude` - Geometric longitude in degrees
/// * `nutation_longitude` - Nutation in longitude in degrees
/// * `aberration` - Aberration correction in degrees
///
/// # Returns
/// Apparent longitude in degrees
pub(crate) fn calculate_apparent_sun_longitude(
    geometric_longitude: f64,
    nutation_longitude: f64,
    aberration: f64,
) -> f64 {
    geometric_longitude + nutation_longitude + aberration
}

// ============================================================================
// Sidereal Time
// ============================================================================

/// Calculates Greenwich mean sidereal time.
///
/// Sidereal time is the angle between the vernal equinox and the observer's meridian.
///
/// # Arguments
/// * `julian_day` - Julian day
/// * `julian_century` - Julian century
///
/// # Returns
/// Greenwich mean sidereal time in degrees (0° to 360°)
pub(crate) fn calculate_greenwich_mean_sidereal_time(julian_day: f64, julian_century: f64) -> f64 {
    normalize_degrees_360(
        GREENWICH_MEAN_SIDEREAL_TIME_BASE
            + SIDEREAL_TIME_RATE * (julian_day - J2000_EPOCH_JD)
            + julian_century * julian_century * (0.000387933 - julian_century / 38_710_000.0),
    )
}

/// Calculates Greenwich apparent sidereal time.
///
/// Apparent sidereal time = mean sidereal time + equation of the equinoxes
///
/// # Arguments
/// * `mean_sidereal_time` - Greenwich mean sidereal time in degrees
/// * `nutation_longitude` - Nutation in longitude in degrees
/// * `true_obliquity` - True obliquity of the ecliptic in degrees
///
/// # Returns
/// Greenwich apparent sidereal time in degrees
pub(crate) fn calculate_greenwich_apparent_sidereal_time(
    mean_sidereal_time: f64,
    nutation_longitude: f64,
    true_obliquity: f64,
) -> f64 {
    mean_sidereal_time + nutation_longitude * true_obliquity.to_radians().cos()
}

// ============================================================================
// Atmospheric Refraction
// ============================================================================

/// Calculates atmospheric refraction correction.
///
/// Atmospheric refraction causes celestial objects to appear higher in the sky
/// than their true geometric position, with the effect increasing near the horizon.
///
/// # Arguments
/// * `pressure` - Atmospheric pressure in millibars
/// * `temperature` - Temperature in Celsius
/// * `atmos_refract` - Atmospheric refraction parameter in degrees
/// * `elevation_uncorrected` - Uncorrected elevation angle in degrees
///
/// # Returns
/// Refraction correction in degrees (always positive, adds to elevation)
pub(crate) fn calculate_atmospheric_refraction_correction(
    pressure: f64,
    temperature: f64,
    atmos_refract: f64,
    elevation_uncorrected: f64,
) -> f64 {
    // Only calculate refraction if the sun is not too far below the horizon
    if elevation_uncorrected >= -(SUN_RADIUS + atmos_refract) {
        // Bennett's formula for atmospheric refraction with pressure and temperature correction
        (pressure / 1010.0) * (283.0 / (273.0 + temperature)) * 1.02
            / (60.0
                * (elevation_uncorrected + 10.3 / (elevation_uncorrected + 5.11))
                    .to_radians()
                    .tan())
    } else {
        0.0
    }
}

// ============================================================================
// Equatorial Coordinates
// ============================================================================

/// Calculates geocentric right ascension.
///
/// Right ascension is the celestial equivalent of longitude on the celestial sphere.
///
/// # Arguments
/// * `ecliptic_longitude` - Ecliptic longitude in degrees
/// * `true_obliquity` - True obliquity of the ecliptic in degrees
/// * `ecliptic_latitude` - Ecliptic latitude in degrees
///
/// # Returns
/// Right ascension in degrees (0° to 360°)
pub(crate) fn calculate_geocentric_right_ascension(
    ecliptic_longitude: f64,
    true_obliquity: f64,
    ecliptic_latitude: f64,
) -> f64 {
    let longitude_rad = ecliptic_longitude.to_radians();
    let obliquity_rad = true_obliquity.to_radians();
    let latitude_rad = ecliptic_latitude.to_radians();

    normalize_degrees_360(
        (longitude_rad.sin() * obliquity_rad.cos() - latitude_rad.tan() * obliquity_rad.sin())
            .atan2(longitude_rad.cos())
            .to_degrees(),
    )
}

/// Calculates geocentric declination.
///
/// Declination is the celestial equivalent of latitude on the celestial sphere.
///
/// # Arguments
/// * `ecliptic_latitude` - Ecliptic latitude in degrees
/// * `true_obliquity` - True obliquity of the ecliptic in degrees
/// * `ecliptic_longitude` - Ecliptic longitude in degrees
///
/// # Returns
/// Declination in degrees (-90° to +90°)
pub(crate) fn calculate_geocentric_declination(
    ecliptic_latitude: f64,
    true_obliquity: f64,
    ecliptic_longitude: f64,
) -> f64 {
    let latitude_rad = ecliptic_latitude.to_radians();
    let obliquity_rad = true_obliquity.to_radians();
    let longitude_rad = ecliptic_longitude.to_radians();

    (latitude_rad.sin() * obliquity_rad.cos() + latitude_rad.cos() * obliquity_rad.sin() * longitude_rad.sin())
        .asin()
        .to_degrees()
}

/// Calculates the observer's local hour angle.
///
/// The hour angle is the angular distance along the celestial equator from the
/// observer's meridian to the hour circle passing through the sun.
///
/// # Arguments
/// * `greenwich_sidereal_time` - Greenwich apparent sidereal time in degrees
/// * `longitude` - Observer's longitude in degrees (positive east)
/// * `right_ascension` - Sun's right ascension in degrees
///
/// # Returns
/// Local hour angle in degrees (0° to 360°)
pub(crate) fn calculate_observer_hour_angle(greenwich_sidereal_time: f64, longitude: f64, right_ascension: f64) -> f64 {
    normalize_degrees_360(greenwich_sidereal_time + longitude - right_ascension)
}

// ============================================================================
// Parallax and Topocentric Coordinates
// ============================================================================

/// Calculates the sun's equatorial horizontal parallax.
///
/// Parallax is the apparent displacement of the sun due to the observer being
/// on Earth's surface rather than at its center.
///
/// # Arguments
/// * `earth_sun_distance` - Distance from Earth to Sun in AU
///
/// # Returns
/// Equatorial horizontal parallax in degrees
pub(crate) fn calculate_sun_equatorial_horizontal_parallax(earth_sun_distance: f64) -> f64 {
    PARALLAX_CONSTANT / (3600.0 * earth_sun_distance)
}

/// Calculates topocentric local hour angle.
///
/// # Arguments
/// * `geocentric_hour_angle` - Geocentric local hour angle in degrees
/// * `parallax_correction` - Parallax correction to right ascension in degrees
///
/// # Returns
/// Topocentric local hour angle in degrees
pub(crate) fn calculate_topocentric_local_hour_angle(geocentric_hour_angle: f64, parallax_correction: f64) -> f64 {
    geocentric_hour_angle - parallax_correction
}

/// Calculates topocentric elevation angle.
///
/// # Arguments
/// * `latitude` - Observer's latitude in degrees
/// * `declination_prime` - Topocentric declination in degrees
/// * `hour_angle_prime` - Topocentric local hour angle in degrees
///
/// # Returns
/// Topocentric elevation angle in degrees
pub(crate) fn calculate_topocentric_elevation_angle(
    latitude: f64,
    declination_prime: f64,
    hour_angle_prime: f64,
) -> f64 {
    let latitude_rad = latitude.to_radians();
    let declination_rad = declination_prime.to_radians();
    let hour_angle_rad = hour_angle_prime.to_radians();

    (latitude_rad.sin() * declination_rad.sin() + latitude_rad.cos() * declination_rad.cos() * hour_angle_rad.cos())
        .asin()
        .to_degrees()
}

/// Calculates parallax correction and topocentric declination.
///
/// This function accounts for the observer's position on Earth's surface,
/// which affects the apparent position of the sun.
///
/// # Arguments
/// * `latitude` - Observer's latitude in degrees
/// * `elevation_meters` - Observer's elevation above sea level in meters
/// * `equatorial_horizontal_parallax` - Sun's equatorial horizontal parallax in degrees
/// * `geocentric_hour_angle` - Geocentric local hour angle in degrees
/// * `geocentric_declination` - Geocentric declination in degrees
///
/// # Returns
/// Tuple of (`topocentric_declination`, `parallax_correction_to_right_ascension`) in degrees
pub(crate) fn calculate_right_ascension_parallax_and_topocentric_declination(
    latitude: f64,
    elevation_meters: f64,
    equatorial_horizontal_parallax: f64,
    geocentric_hour_angle: f64,
    geocentric_declination: f64,
) -> (f64, f64) {
    let latitude_rad = latitude.to_radians();
    let parallax_rad = equatorial_horizontal_parallax.to_radians();
    let hour_angle_rad = geocentric_hour_angle.to_radians();
    let declination_rad = geocentric_declination.to_radians();

    // Calculate observer's geocentric coordinates
    let u = (EARTH_FLATTENING_FACTOR * latitude_rad.tan()).atan();
    let rho_sin_phi_prime =
        EARTH_FLATTENING_FACTOR * u.sin() + elevation_meters * latitude_rad.sin() / EARTH_EQUATORIAL_RADIUS_METERS;
    let rho_cos_phi_prime = u.cos() + elevation_meters * latitude_rad.cos() / EARTH_EQUATORIAL_RADIUS_METERS;

    // Calculate parallax corrections
    let parallax_correction_to_right_ascension_rad = (-rho_cos_phi_prime * parallax_rad.sin() * hour_angle_rad.sin())
        .atan2(declination_rad.cos() - rho_cos_phi_prime * parallax_rad.sin() * hour_angle_rad.cos());

    let topocentric_declination_rad = ((declination_rad.sin() - rho_sin_phi_prime * parallax_rad.sin())
        * parallax_correction_to_right_ascension_rad.cos())
    .atan2(declination_rad.cos() - rho_cos_phi_prime * parallax_rad.sin() * hour_angle_rad.cos());

    (
        topocentric_declination_rad.to_degrees(),
        parallax_correction_to_right_ascension_rad.to_degrees(),
    )
}

// ============================================================================
// Earth Heliocentric Position
// ============================================================================

/// Calculates Earth's heliocentric position component from periodic terms.
///
/// # Arguments
/// * `term_sums` - Iterator over sums of periodic terms for each power of JME
/// * `julian_millennium` - Julian ephemeris millennium
///
/// # Returns
/// Position component value
pub(crate) fn calculate_earth_position_from_terms(term_sums: impl Iterator<Item = f64>, julian_millennium: f64) -> f64 {
    let sum = term_sums.enumerate().fold(0.0, |accumulator, (power, term_sum)| {
        accumulator + term_sum * julian_millennium.powf(power as f64)
    });
    sum / EARTH_POSITION_SCALE_FACTOR
}

/// Calculates the sum of periodic terms for Earth's position.
///
/// Each term is of the form: A * cos(B + C * JME)
///
/// # Arguments
/// * `terms` - Iterator over periodic term coefficients [A, B, C]
/// * `julian_millennium` - Julian ephemeris millennium
///
/// # Returns
/// Sum of all periodic terms
pub(crate) fn calculate_earth_periodic_term_sum(terms: core::slice::Iter<[f64; 3]>, julian_millennium: f64) -> f64 {
    terms.fold(0.0, |accumulator, term| {
        accumulator + term[0] * (term[1] + term[2] * julian_millennium).cos()
    })
}

/// Calculates Earth's heliocentric longitude.
///
/// # Arguments
/// * `julian_millennium` - Julian ephemeris millennium
///
/// # Returns
/// Heliocentric longitude in degrees (0° to 360°)
pub(crate) fn calculate_earth_heliocentric_longitude(julian_millennium: f64) -> f64 {
    let term_sums = L_TERMS
        .iter()
        .map(|terms| calculate_earth_periodic_term_sum(terms.iter(), julian_millennium));
    normalize_degrees_360(calculate_earth_position_from_terms(term_sums, julian_millennium).to_degrees())
}

/// Calculates Earth's heliocentric latitude.
///
/// # Arguments
/// * `julian_millennium` - Julian ephemeris millennium
///
/// # Returns
/// Heliocentric latitude in degrees
pub(crate) fn calculate_earth_heliocentric_latitude(julian_millennium: f64) -> f64 {
    let term_sums = B_TERMS
        .iter()
        .map(|terms| calculate_earth_periodic_term_sum(terms.iter(), julian_millennium));
    calculate_earth_position_from_terms(term_sums, julian_millennium).to_degrees()
}

/// Calculates Earth's radius vector (distance from Sun).
///
/// # Arguments
/// * `julian_millennium` - Julian ephemeris millennium
///
/// # Returns
/// Earth-Sun distance in astronomical units (AU)
pub(crate) fn calculate_earth_radius_vector(julian_millennium: f64) -> f64 {
    let term_sums = R_TERMS
        .iter()
        .map(|terms| calculate_earth_periodic_term_sum(terms.iter(), julian_millennium));
    calculate_earth_position_from_terms(term_sums, julian_millennium)
}

// ============================================================================
// Nutation Calculations
// ============================================================================

/// Calculates nutation in longitude and obliquity.
///
/// Nutation is the periodic oscillation of Earth's axis of rotation caused by
/// the gravitational pull of the Moon and Sun on Earth's equatorial bulge.
///
/// # Arguments
/// * `julian_century` - Julian ephemeris century
/// * `fundamental_arguments` - Array of [X0, X1, X2, X3, X4] fundamental arguments in degrees
///   - X0: Mean elongation of Moon from Sun
///   - X1: Mean anomaly of Sun
///   - X2: Mean anomaly of Moon
///   - X3: Argument of latitude of Moon
///   - X4: Longitude of ascending node of Moon
///
/// # Returns
/// Tuple of (`nutation_in_longitude`, `nutation_in_obliquity`) in degrees
pub(crate) fn calculate_nutation_longitude_and_obliquity(
    julian_century: f64,
    fundamental_arguments: [f64; 5],
) -> (f64, f64) {
    let mut sum_longitude = 0.0;
    let mut sum_obliquity = 0.0;

    for i in 0..Y_TERMS.len() {
        // Calculate the argument for this nutation term
        let argument_sum: f64 = Y_TERMS[i]
            .iter()
            .enumerate()
            .fold(0.0, |accumulator, (j, &multiplier)| {
                accumulator + multiplier as f64 * fundamental_arguments[j]
            })
            .to_radians();

        // Add contributions to nutation in longitude and obliquity
        sum_longitude += (PE_TERMS[i][0] + julian_century * PE_TERMS[i][1]) * argument_sum.sin();
        sum_obliquity += (PE_TERMS[i][2] + julian_century * PE_TERMS[i][3]) * argument_sum.cos();
    }

    (
        sum_longitude / NUTATION_SCALE_FACTOR,
        sum_obliquity / NUTATION_SCALE_FACTOR,
    )
}
