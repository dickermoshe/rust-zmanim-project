#[allow(unused_imports)]
use core_maths::*;
use jiff::{SignedDuration, Timestamp, civil::Date, tz::TimeZone};

use crate::zmanim::types::{error::ZmanimError, location::Location};

const GEOMETRIC_ZENITH: f64 = 90.0;
const REFRACTION: f64 = 34.0 / 60.0;
const EARTH_RADIUS_KM: f64 = 6371.0088;

/// Apparent solar semi-diameter in degrees for each day of year (VSOP87, reference year 2050).
const SOLAR_RADIUS_BY_DAY_OF_YEAR: [f64; 365] = [
    0.27108024, 0.27108486, 0.27108790, 0.27108930, 0.27108899, 0.27108695, 0.27108316, 0.27107762, 0.27107033,
    0.27106133, 0.27105062, 0.27103826, 0.27102427, 0.27100873, 0.27099168, 0.27097320, 0.27095337, 0.27093228,
    0.27091002, 0.27088667, 0.27086231, 0.27083701, 0.27081079, 0.27078369, 0.27075569, 0.27072676, 0.27069684,
    0.27066588, 0.27063378, 0.27060048, 0.27056589, 0.27052995, 0.27049261, 0.27045383, 0.27041359, 0.27037186,
    0.27032864, 0.27028396, 0.27023782, 0.27019025, 0.27014129, 0.27009098, 0.27003938, 0.26998658, 0.26993264,
    0.26987767, 0.26982177, 0.26976506, 0.26970763, 0.26964958, 0.26959099, 0.26953191, 0.26947239, 0.26941242,
    0.26935200, 0.26929108, 0.26922962, 0.26916755, 0.26910482, 0.26904136, 0.26897712, 0.26891206, 0.26884614,
    0.26877935, 0.26871165, 0.26864306, 0.26857358, 0.26850321, 0.26843197, 0.26835989, 0.26828703, 0.26821343,
    0.26813918, 0.26806437, 0.26798910, 0.26791348, 0.26783763, 0.26776167, 0.26768569, 0.26760979, 0.26753404,
    0.26745846, 0.26738308, 0.26730790, 0.26723289, 0.26715800, 0.26708320, 0.26700842, 0.26693363, 0.26685877,
    0.26678380, 0.26670870, 0.26663342, 0.26655796, 0.26648229, 0.26640640, 0.26633030, 0.26625399, 0.26617748,
    0.26610082, 0.26602406, 0.26594728, 0.26587055, 0.26579398, 0.26571769, 0.26564180, 0.26556641, 0.26549164,
    0.26541756, 0.26534425, 0.26527174, 0.26520006, 0.26512920, 0.26505915, 0.26498987, 0.26492132, 0.26485345,
    0.26478622, 0.26471959, 0.26465351, 0.26458794, 0.26452285, 0.26445820, 0.26439396, 0.26433010, 0.26426661,
    0.26420348, 0.26414070, 0.26407832, 0.26401636, 0.26395489, 0.26389400, 0.26383378, 0.26377435, 0.26371580,
    0.26365825, 0.26360179, 0.26354651, 0.26349247, 0.26343971, 0.26338825, 0.26333807, 0.26328918, 0.26324153,
    0.26319510, 0.26314983, 0.26310568, 0.26306261, 0.26302057, 0.26297951, 0.26293938, 0.26290014, 0.26286173,
    0.26282411, 0.26278725, 0.26275111, 0.26271570, 0.26268102, 0.26264710, 0.26261399, 0.26258177, 0.26255053,
    0.26252037, 0.26249137, 0.26246366, 0.26243731, 0.26241239, 0.26238897, 0.26236707, 0.26234671, 0.26232790,
    0.26231061, 0.26229481, 0.26228048, 0.26226756, 0.26225602, 0.26224581, 0.26223687, 0.26222914, 0.26222257,
    0.26221708, 0.26221262, 0.26220912, 0.26220653, 0.26220480, 0.26220392, 0.26220388, 0.26220470, 0.26220642,
    0.26220910, 0.26221282, 0.26221768, 0.26222375, 0.26223114, 0.26223991, 0.26225014, 0.26226187, 0.26227512,
    0.26228992, 0.26230626, 0.26232413, 0.26234349, 0.26236433, 0.26238659, 0.26241024, 0.26243521, 0.26246145,
    0.26248890, 0.26251746, 0.26254708, 0.26257766, 0.26260913, 0.26264141, 0.26267446, 0.26270823, 0.26274270,
    0.26277789, 0.26281382, 0.26285054, 0.26288813, 0.26292666, 0.26296622, 0.26300686, 0.26304868, 0.26309171,
    0.26313601, 0.26318159, 0.26322848, 0.26327666, 0.26332612, 0.26337685, 0.26342881, 0.26348197, 0.26353627,
    0.26359167, 0.26364809, 0.26370545, 0.26376368, 0.26382267, 0.26388233, 0.26394256, 0.26400328, 0.26406442,
    0.26412593, 0.26418777, 0.26424995, 0.26431249, 0.26437542, 0.26443880, 0.26450270, 0.26456718, 0.26463231,
    0.26469815, 0.26476474, 0.26483213, 0.26490034, 0.26496937, 0.26503922, 0.26510990, 0.26518138, 0.26525364,
    0.26532664, 0.26540034, 0.26547467, 0.26554957, 0.26562495, 0.26570072, 0.26577676, 0.26585296, 0.26592922,
    0.26600544, 0.26608154, 0.26615745, 0.26623314, 0.26630859, 0.26638382, 0.26645884, 0.26653372, 0.26660850,
    0.26668323, 0.26675798, 0.26683280, 0.26690773, 0.26698280, 0.26705803, 0.26713345, 0.26720905, 0.26728485,
    0.26736083, 0.26743698, 0.26751326, 0.26758964, 0.26766606, 0.26774245, 0.26781872, 0.26789476, 0.26797045,
    0.26804569, 0.26812035, 0.26819433, 0.26826755, 0.26833992, 0.26841141, 0.26848200, 0.26855170, 0.26862051,
    0.26868849, 0.26875567, 0.26882211, 0.26888786, 0.26895296, 0.26901746, 0.26908139, 0.26914478, 0.26920767,
    0.26927006, 0.26933199, 0.26939345, 0.26945445, 0.26951496, 0.26957497, 0.26963442, 0.26969325, 0.26975136,
    0.26980865, 0.26986502, 0.26992034, 0.26997450, 0.27002740, 0.27007895, 0.27012907, 0.27017773, 0.27022491,
    0.27027060, 0.27031483, 0.27035764, 0.27039906, 0.27043914, 0.27047794, 0.27051549, 0.27055186, 0.27058709,
    0.27062122, 0.27065430, 0.27068636, 0.27071746, 0.27074761, 0.27077684, 0.27080516, 0.27083256, 0.27085899,
    0.27088442, 0.27090875, 0.27093191, 0.27095379, 0.27097427, 0.27099326, 0.27101067, 0.27102640, 0.27104041,
    0.27105266, 0.27106312, 0.27107182, 0.27107876, 0.27108399,
];
const JULIAN_DAY_JAN_1_2000: f64 = 2_451_545.0;
const JULIAN_DAYS_PER_CENTURY: f64 = 36_525.0;
const HOUR_NANOS: f64 = 3_600_000.0 * 1_000_000.0;

#[derive(Clone, Copy)]
pub(crate) enum SolarEvent {
    Sunrise,
    Sunset,
    Noon,
    Midnight,
}

pub(crate) fn sunrise(date: Date, location: &Location, adjust_for_elevation: bool) -> Result<Timestamp, ZmanimError> {
    rise_set(
        date,
        location,
        GEOMETRIC_ZENITH,
        adjust_for_elevation,
        SolarEvent::Sunrise,
    )
}

pub(crate) fn sunset(date: Date, location: &Location, adjust_for_elevation: bool) -> Result<Timestamp, ZmanimError> {
    rise_set(
        date,
        location,
        GEOMETRIC_ZENITH,
        adjust_for_elevation,
        SolarEvent::Sunset,
    )
}

pub(crate) fn sunrise_offset_by_degrees(
    date: Date,
    location: &Location,
    degrees: f64,
) -> Result<Timestamp, ZmanimError> {
    rise_set(date, location, GEOMETRIC_ZENITH + degrees, false, SolarEvent::Sunrise)
}

pub(crate) fn sunset_offset_by_degrees(
    date: Date,
    location: &Location,
    degrees: f64,
) -> Result<Timestamp, ZmanimError> {
    rise_set(date, location, GEOMETRIC_ZENITH + degrees, false, SolarEvent::Sunset)
}

pub(crate) fn solar_noon(date: Date, location: &Location) -> Result<Timestamp, ZmanimError> {
    let adjusted_date = adjusted_local_date(date, location)?;
    let noon = solar_noon_midnight_utc(julian_day(adjusted_date), -location.longitude, SolarEvent::Noon);
    instant_from_utc_hours(
        normalize_utc_hours(noon / 60.0),
        adjusted_date,
        location,
        SolarEvent::Noon,
    )
}

pub(crate) fn solar_midnight(date: Date, location: &Location) -> Result<Timestamp, ZmanimError> {
    let adjusted_date = adjusted_local_date(date, location)?;
    let midnight = solar_noon_midnight_utc(julian_day(adjusted_date), -location.longitude, SolarEvent::Midnight);
    instant_from_utc_hours(
        normalize_utc_hours(midnight / 60.0),
        adjusted_date,
        location,
        SolarEvent::Midnight,
    )
}

pub(crate) fn time_at_azimuth(date: Date, location: &Location, target_azimuth: f64) -> Result<Timestamp, ZmanimError> {
    if target_azimuth != 90.0 && target_azimuth != 270.0 {
        return Err(ZmanimError::CalculationError);
    }

    let adjusted_date = adjusted_local_date(date, location)?;
    let julian_day = julian_day(adjusted_date);
    let solar_noon_base = 0.5 - location.longitude / 360.0;
    let mut date_time = solar_noon_base + if target_azimuth == 90.0 { 0.25 } else { 0.75 };

    for _ in 0..3 {
        let julian_centuries = julian_centuries_from_julian_day(julian_day + date_time);
        let ratio = sun_declination(julian_centuries).to_radians().tan() / location.latitude.to_radians().tan();

        if ratio.is_nan() || !(-1.0..=1.0).contains(&ratio) {
            return Err(ZmanimError::CalculationError);
        }

        let direction = if target_azimuth == 90.0 { -1.0 } else { 1.0 };
        let offset = direction * ratio.acos().to_degrees() / 360.0;
        date_time = solar_noon_base + offset - equation_of_time(julian_centuries) / 1440.0;
    }

    let event = if target_azimuth == 90.0 {
        SolarEvent::Sunrise
    } else {
        SolarEvent::Sunset
    };

    instant_from_utc_hours(normalize_utc_hours(date_time * 24.0), adjusted_date, location, event)
}

fn rise_set(
    date: Date,
    location: &Location,
    zenith: f64,
    adjust_for_elevation: bool,
    event: SolarEvent,
) -> Result<Timestamp, ZmanimError> {
    let adjusted_date = adjusted_local_date(date, location)?;
    let elevation = if adjust_for_elevation { location.elevation } else { 0.0 };
    let adjusted_zenith = adjust_zenith(zenith, elevation, adjusted_date);
    let utc_minutes = sun_rise_set_utc(
        adjusted_date,
        location.latitude,
        -location.longitude,
        adjusted_zenith,
        event,
    )?;
    instant_from_utc_hours(normalize_utc_hours(utc_minutes / 60.0), adjusted_date, location, event)
}

pub(crate) fn adjusted_local_date(date: Date, location: &Location) -> Result<Date, ZmanimError> {
    let Some(timezone) = &location.timezone else {
        return Ok(date);
    };

    let midnight = timezone
        .to_ambiguous_timestamp(date.at(0, 0, 0, 0))
        .earlier()
        .map_err(|_| ZmanimError::TimeConversionError)?;
    let offset = midnight.to_zoned(timezone.clone()).offset().seconds();
    let local_hours_offset = (location.longitude * 240.0 - f64::from(offset)) / 3600.0;

    if local_hours_offset >= 20.0 {
        add_days(date, 1)
    } else if local_hours_offset <= -20.0 {
        add_days(date, -1)
    } else {
        Ok(date)
    }
}

fn instant_from_utc_hours(
    time: f64,
    mut date: Date,
    location: &Location,
    event: SolarEvent,
) -> Result<Timestamp, ZmanimError> {
    if time.is_nan() {
        return Err(ZmanimError::CalculationError);
    }

    let local_time_hours = location.longitude / 15.0 + time;
    match event {
        SolarEvent::Sunrise if local_time_hours > 18.0 => date = add_days(date, -1)?,
        SolarEvent::Sunset if local_time_hours < 6.0 => date = add_days(date, 1)?,
        SolarEvent::Midnight if local_time_hours < 12.0 => date = add_days(date, 1)?,
        SolarEvent::Noon if local_time_hours < 0.0 => date = add_days(date, 1)?,
        SolarEvent::Noon if local_time_hours > 24.0 => date = add_days(date, -1)?,
        _ => {}
    }

    let nanos = (time * HOUR_NANOS).round() as i64;
    let date_time = date
        .at(0, 0, 0, 0)
        .checked_add(SignedDuration::from_nanos(nanos))
        .map_err(|_| ZmanimError::TimeConversionError)?;
    date_time
        .to_zoned(TimeZone::UTC)
        .map_err(|_| ZmanimError::TimeConversionError)
        .map(|zdt| zdt.timestamp())
}

fn add_days(date: Date, days: i64) -> Result<Date, ZmanimError> {
    date.checked_add(SignedDuration::from_hours(24 * days))
        .map_err(|_| ZmanimError::TimeConversionError)
}

fn normalize_utc_hours(time: f64) -> f64 {
    (time % 24.0 + 24.0) % 24.0
}

fn apparent_solar_radius(date: Date) -> f64 {
    let month = date.month();
    let day = date.day();
    let reference = if month == 2 && day == 29 {
        Date::new(2050, 2, 28).unwrap()
    } else {
        Date::new(2050, month, day).unwrap()
    };
    let jan1 = Date::new(2050, 1, 1).unwrap();
    let day_index = jan1
        .until(reference)
        .expect("reference date is on or after Jan 1 2050")
        .get_days() as usize;
    SOLAR_RADIUS_BY_DAY_OF_YEAR[day_index]
}

fn adjust_zenith(zenith: f64, elevation: f64, date: Date) -> f64 {
    if zenith == GEOMETRIC_ZENITH {
        zenith + apparent_solar_radius(date) + REFRACTION + elevation_adjustment(elevation)
    } else {
        zenith
    }
}

fn elevation_adjustment(elevation_m: f64) -> f64 {
    (EARTH_RADIUS_KM / (EARTH_RADIUS_KM + elevation_m / 1000.0))
        .acos()
        .to_degrees()
}

fn julian_day(date: Date) -> f64 {
    let mut year = i32::from(date.year());
    let mut month = i32::from(date.month());
    let day = i32::from(date.day());

    if month <= 2 {
        year -= 1;
        month += 12;
    }

    let a = year / 100;
    let b = 2 - a + a / 4;

    (365.25 * f64::from(year + 4716)).floor() + (30.6001 * f64::from(month + 1)).floor() + f64::from(day) + f64::from(b)
        - 1524.5
}

fn julian_centuries_from_julian_day(julian_day: f64) -> f64 {
    (julian_day - JULIAN_DAY_JAN_1_2000) / JULIAN_DAYS_PER_CENTURY
}

fn sun_geometric_mean_longitude(julian_centuries: f64) -> f64 {
    let longitude = 280.46646 + julian_centuries * (36000.76983 + 0.0003032 * julian_centuries);
    (longitude % 360.0 + 360.0) % 360.0
}

fn sun_geometric_mean_anomaly(julian_centuries: f64) -> f64 {
    357.52911 + julian_centuries * (35999.05029 - 0.0001537 * julian_centuries)
}

fn earth_orbit_eccentricity(julian_centuries: f64) -> f64 {
    0.016708634 - julian_centuries * (0.000042037 + 0.0000001267 * julian_centuries)
}

fn sun_equation_of_center(julian_centuries: f64) -> f64 {
    let m = sun_geometric_mean_anomaly(julian_centuries);
    let mrad = m.to_radians();
    let sinm = mrad.sin();
    let sin2m = (mrad + mrad).sin();
    let sin3m = (mrad + mrad + mrad).sin();
    sinm * (1.914602 - julian_centuries * (0.004817 + 0.000014 * julian_centuries))
        + sin2m * (0.019993 - 0.000101 * julian_centuries)
        + sin3m * 0.000289
}

fn sun_true_longitude(julian_centuries: f64) -> f64 {
    sun_geometric_mean_longitude(julian_centuries) + sun_equation_of_center(julian_centuries)
}

fn sun_apparent_longitude(julian_centuries: f64) -> f64 {
    let omega = 125.04 - 1934.136 * julian_centuries;
    sun_true_longitude(julian_centuries) - 0.00569 - 0.00478 * omega.to_radians().sin()
}

fn mean_obliquity_of_ecliptic(julian_centuries: f64) -> f64 {
    let seconds = 21.448 - julian_centuries * (46.8150 + julian_centuries * (0.00059 - julian_centuries * 0.001813));
    23.0 + (26.0 + seconds / 60.0) / 60.0
}

fn obliquity_correction(julian_centuries: f64) -> f64 {
    let omega = 125.04 - 1934.136 * julian_centuries;
    mean_obliquity_of_ecliptic(julian_centuries) + 0.00256 * omega.to_radians().cos()
}

fn sun_declination(julian_centuries: f64) -> f64 {
    let obliquity_correction = obliquity_correction(julian_centuries);
    let lambda = sun_apparent_longitude(julian_centuries);
    let sint = obliquity_correction.to_radians().sin() * lambda.to_radians().sin();
    sint.asin().to_degrees()
}

fn equation_of_time(julian_centuries: f64) -> f64 {
    let epsilon = obliquity_correction(julian_centuries);
    let geom_mean_long_sun = sun_geometric_mean_longitude(julian_centuries);
    let eccentricity_earth_orbit = earth_orbit_eccentricity(julian_centuries);
    let geom_mean_anomaly_sun = sun_geometric_mean_anomaly(julian_centuries);
    let mut y = (epsilon.to_radians() / 2.0).tan();
    y *= y;

    let sin2l0 = (2.0 * geom_mean_long_sun.to_radians()).sin();
    let sinm = geom_mean_anomaly_sun.to_radians().sin();
    let cos2l0 = (2.0 * geom_mean_long_sun.to_radians()).cos();
    let sin4l0 = (4.0 * geom_mean_long_sun.to_radians()).sin();
    let sin2m = (2.0 * geom_mean_anomaly_sun.to_radians()).sin();
    let equation_of_time = y * sin2l0 - 2.0 * eccentricity_earth_orbit * sinm
        + 4.0 * eccentricity_earth_orbit * y * sinm * cos2l0
        - 0.5 * y * y * sin4l0
        - 1.25 * eccentricity_earth_orbit * eccentricity_earth_orbit * sin2m;
    equation_of_time.to_degrees() * 4.0
}

fn sun_hour_angle(latitude: f64, solar_declination: f64, zenith: f64, event: SolarEvent) -> Result<f64, ZmanimError> {
    let lat_rad = latitude.to_radians();
    let sd_rad = solar_declination.to_radians();
    let cos_hour_angle = zenith.to_radians().cos() / (lat_rad.cos() * sd_rad.cos()) - lat_rad.tan() * sd_rad.tan();

    if cos_hour_angle > 1.0 {
        return Err(ZmanimError::AllNight);
    }
    if cos_hour_angle < -1.0 {
        return Err(ZmanimError::AllDay);
    }

    let mut hour_angle = cos_hour_angle.acos();
    if matches!(event, SolarEvent::Sunset) {
        hour_angle = -hour_angle;
    }
    Ok(hour_angle)
}

fn solar_noon_midnight_utc(julian_day: f64, longitude: f64, event: SolarEvent) -> f64 {
    let tnoon = julian_centuries_from_julian_day(julian_day + longitude / 360.0);
    let mut eq_time = equation_of_time(tnoon);
    let mut sol_noon_utc = longitude * 4.0 - eq_time;

    for _ in 0..2 {
        let newt = julian_centuries_from_julian_day(julian_day + sol_noon_utc / 1440.0);
        eq_time = equation_of_time(newt);
        sol_noon_utc = (if matches!(event, SolarEvent::Noon) {
            720.0
        } else {
            1440.0
        }) + longitude * 4.0
            - eq_time;
    }

    sol_noon_utc
}

fn sun_rise_set_utc(
    local_date: Date,
    latitude: f64,
    longitude: f64,
    zenith: f64,
    event: SolarEvent,
) -> Result<f64, ZmanimError> {
    let julian_day = julian_day(local_date);
    let noon_min = solar_noon_midnight_utc(julian_day, longitude, SolarEvent::Noon);
    let tnoon = julian_centuries_from_julian_day(julian_day + noon_min / 1440.0);
    let mut eq_time = equation_of_time(tnoon);
    let mut solar_declination = sun_declination(tnoon);
    let mut hour_angle = sun_hour_angle(latitude, solar_declination, zenith, event)?;
    let mut delta = longitude - hour_angle.to_degrees();
    let mut time_diff = 4.0 * delta;
    let mut time_utc = 720.0 + time_diff - eq_time;

    let newt = julian_centuries_from_julian_day(julian_day + time_utc / 1440.0);
    eq_time = equation_of_time(newt);
    solar_declination = sun_declination(newt);
    hour_angle = sun_hour_angle(latitude, solar_declination, zenith, event)?;
    delta = longitude - hour_angle.to_degrees();
    time_diff = 4.0 * delta;
    time_utc = 720.0 + time_diff - eq_time;
    Ok(time_utc)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use jiff::tz::TimeZone;

    fn test_location() -> Location {
        Location::new(
            40.0721087,
            -74.2400243,
            0.0,
            Some(TimeZone::get("America/New_York").unwrap()),
        )
        .unwrap()
    }

    #[test]
    fn normalize_utc_hours_zero_stays_zero() {
        assert_eq!(normalize_utc_hours(0.0), 0.0);
    }

    #[test]
    fn normalize_utc_hours_negative_and_overflow() {
        assert!((normalize_utc_hours(-1.0) - 23.0).abs() < f64::EPSILON);
        assert!((normalize_utc_hours(25.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn apparent_solar_radius_varies_through_year() {
        let perihelion = Date::new(2024, 1, 3).unwrap();
        let aphelion = Date::new(2024, 7, 5).unwrap();
        let fixed = 16.0 / 60.0;
        assert!(apparent_solar_radius(perihelion) > fixed);
        assert!(apparent_solar_radius(aphelion) < fixed);
    }

    #[test]
    fn apparent_solar_radius_changes_sunrise_vs_fixed_radius() {
        let location = test_location();
        let date = Date::new(2024, 1, 3).unwrap();
        let apparent = sunrise(date, &location, false).unwrap();

        let fixed_zenith = GEOMETRIC_ZENITH + (16.0 / 60.0) + REFRACTION;
        let adjusted_date = adjusted_local_date(date, &location).unwrap();
        let utc_minutes = sun_rise_set_utc(
            adjusted_date,
            location.latitude,
            -location.longitude,
            fixed_zenith,
            SolarEvent::Sunrise,
        )
        .unwrap();
        let fixed = instant_from_utc_hours(
            normalize_utc_hours(utc_minutes / 60.0),
            adjusted_date,
            &location,
            SolarEvent::Sunrise,
        )
        .unwrap();

        assert_ne!(apparent, fixed);
        assert!(apparent < fixed);
    }

    #[test]
    fn solar_midnight_succeeds_near_antimeridian() {
        let location = Location::new(-14.2750, -170.7020, 0.0, Some(TimeZone::get("Pacific/Apia").unwrap())).unwrap();
        let date = Date::new(2011, 12, 29).unwrap();
        assert!(solar_midnight(date, &location).is_ok());
    }

    #[test]
    fn time_at_azimuth_normalizes_hours_before_timestamp() {
        let location = Location::new(69.6492, 18.9553, 0.0, Some(TimeZone::get("Europe/Oslo").unwrap())).unwrap();
        let date = Date::new(2017, 6, 21).unwrap();
        let azimuth_time = time_at_azimuth(date, &location, 90.0).unwrap();
        let local = azimuth_time.to_zoned(TimeZone::get("Europe/Oslo").unwrap());
        assert_eq!(local.date(), Date::new(2017, 6, 21).unwrap());
    }
}
