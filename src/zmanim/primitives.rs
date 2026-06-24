//! Low-level zman formulas used to build higher-level presets.
//!
//! [`ZmanPrimitive`] is the internal expression language for zman calculations.
//! Variants represent either:
//! - base astronomical events (for example sunrise/sunset),
//! - transformed events (fixed offsets or degree-based offsets), or
//! - derived halachic times computed from two boundary events.
//!
//! Most users should prefer the ready-made constants in [`crate::zmanim::presets`].
//! Use this module when you need to compose a custom zman definition that is
//! not already provided by a preset.

use core_maths::CoreFloat;
use icu_calendar::{
    options::{DateAddOptions, Overflow},
    types::DateDuration,
};
use jiff::{SignedDuration as Duration, Timestamp, ToSpan, tz::TimeZone};

use crate::{
    calendar::{HebrewCalendarDate, HebrewHolidayCalendar, holiday::Holiday},
    zmanim::{
        ZmanLike, ZmanimCalculator,
        molad::{is_same_gregorian_day, months_molad},
        types::error::ZmanimError,
    },
};

static CIVIL_ZENITH: f64 = 6.0;
static NAUTICAL_ZENITH: f64 = 12.0;
static ASTRONOMICAL_ZENITH: f64 = 18.0;

/// A low-level building block for calculating zmanim.
///
/// These should typically not be used directly. Instead, use the presets in [`crate::zmanim::presets`].
#[derive(Debug, Clone)]
pub enum ZmanPrimitive {
    /// Sunrise at the configured location/date.
    ElevationAdjustedSunrise,
    /// Sunrise at sea level (no elevation adjustment).
    SeaLevelSunrise,
    /// Sunrise using the configured elevation mode (sea-level or elevation-adjusted).
    ConfiguredSunrise,
    /// Sunset using the configured elevation mode (sea-level or elevation-adjusted).
    ConfiguredSunset,
    /// Solar transit (local apparent noon / astronomical chatzos).
    SolarTransit,
    /// Solar anti-transit (local apparent midnight / astronomical chatzos halayla).
    SolarMidnight,
    /// The midpoint between sunrise and sunset.
    ChatzosHayomAsHalfDay,
    /// Solar Transit or the midpoint between sunrise and sunset depending on the configuration.
    ChatzosHayom,
    /// The midpoint between sunset and the tomorrows sunrise.
    ChatzosHalaylaAsHalfDay,
    /// Solar Midnight or the midpoint between sunset and the tomorrows sunrise depending on the configuration.
    ChatzosHalayla,
    /// Sunset at the configured location/date.
    ElevationAdjustedSunset,
    /// Sunset at sea level (no elevation adjustment).
    SeaLevelSunset,
    /// Time before sunrise when the sun is `degrees` below the geometric horizon (no elevation adjustment).
    SunriseOffsetByDegrees(f64),
    /// Time after sunset when the sun is `degrees` below the geometric horizon (no elevation adjustment).
    SunsetOffsetByDegrees(f64),
    /// Local mean time at the given hour (0.0–24.0).
    LocalMeanTime(f64),
    /// Shabbos/Yom Tov candle lighting time based on configuration.
    CandleLighting,
    /// A fixed time offset from another [`ZmanPrimitive`].
    Offset(&'static ZmanPrimitive, Duration),
    /// An offset in "shaos zmaniyos" according to the GRA from another [`ZmanPrimitive`].
    ZmanisOffset(&'static ZmanPrimitive, f64),
    /// An offset expressed as a fraction of the half-day between two [`ZmanPrimitive`]s.
    HalfDayBasedOffset(&'static ZmanPrimitive, &'static ZmanPrimitive, f64),
    /// Sof zman shma derived from two bounding [`ZmanPrimitive`]s.
    Shema(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Mincha gedola derived from two bounding [`ZmanPrimitive`]s.
    MinchaGedola(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Samuch le-mincha ketana derived from two bounding [`ZmanPrimitive`]s.
    SamuchLeMinchaKetana(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Mincha ketana derived from two bounding [`ZmanPrimitive`]s.
    MinchaKetana(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Sof zman tefila derived from two bounding [`ZmanPrimitive`]s.
    Tefila(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Plag hamincha derived from two bounding [`ZmanPrimitive`]s.
    PlagHamincha(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Sof zman biur chametz derived from two bounding [`ZmanPrimitive`]s.
    SofZmanBiurChametz(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Sof zman achilas chametz derived from two bounding [`ZmanPrimitive`]s.
    SofZmanAchilasChametz(&'static ZmanPrimitive, &'static ZmanPrimitive, bool),
    /// Tzais according to the shita of Yeshivas Ateret Torah
    TzaisAteretTorah,
    /// The latest time of _Kiddush Levana_ calculated as 15 days after the molad.
    SofZmanKidushLevana15Days,
    /// The latest time of _Kiddush Levana_ according to the the opinion of the Maharil
    /// that it is calculated as halfway between molad and molad.
    SofZmanKidushLevanaBetweenMoldos,
    /// The earliest time of _Kiddush Levana_ according to Rabbeinu Yonahs opinion that it can be said 3 days after the molad.
    TchilasZmanKidushLevana3Days,
    /// The earliest time of _Kiddush Levana_ according to the opinions that it should
    /// not be said until 7 days after the molad.
    TchilasZmanKidushLevana7Days,
    /// Bain hashmashos (Rabbeinu Tam, 2-stars): `sunset + (sunrise - alos19.8°) * 5/18`.
    BainHashmashosRt2Stars,
    /// Mincha gedola (Ahavat Shalom): later of `chatzos + 30m` and `chatzos + 1/2 shaah`.
    MinchaGedolaAhavatShalom,
    /// Mincha gedola GRA, but no earlier than 30 minutes after chatzos.
    MinchaGedolaGraGreaterThan30,
    /// Mincha ketana (Ahavat Shalom): `2.5` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
    MinchaKetanaAhavatShalom,
    /// Plag hamincha (Ahavat Shalom): `1.25` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
    PlagAhavatShalom,
    /// The beginning of civil twilight (dawn), when the sun is 6° below the geometric horizon (96° zenith).
    BeginCivilTwilight,
    /// The end of civil twilight, when the sun is 6° below the geometric horizon (96° zenith).
    EndCivilTwilight,
    /// The beginning of nautical twilight, when the sun is 12° below the geometric horizon (102° zenith).
    BeginNauticalTwilight,
    /// The end of nautical twilight, when the sun is 12° below the geometric horizon (102° zenith).
    EndNauticalTwilight,
    /// The beginning of astronomical twilight, when the sun is 18° below the geometric horizon (108° zenith).
    BeginAstronomicalTwilight,
    /// The end of astronomical twilight, when the sun is 18° below the geometric horizon (108° zenith).
    EndAstronomicalTwilight,
    /// Polar sunset (Ben Ish Chai): due west azimuth on polar no-sunset days only.
    PolarSunsetBenIshChai,
    /// Polar sunrise (Ben Ish Chai): due east azimuth on polar no-sunrise days only.
    PolarSunriseBenIshChai,
}

#[cfg(feature = "defmt")]
impl defmt::Format for ZmanPrimitive {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            ZmanPrimitive::ElevationAdjustedSunrise => defmt::write!(fmt, "ElevationAdjustedSunrise"),
            ZmanPrimitive::SeaLevelSunrise => defmt::write!(fmt, "SeaLevelSunrise"),
            ZmanPrimitive::ConfiguredSunrise => defmt::write!(fmt, "ConfiguredSunrise"),
            ZmanPrimitive::ConfiguredSunset => defmt::write!(fmt, "ConfiguredSunset"),
            ZmanPrimitive::SolarTransit => defmt::write!(fmt, "SolarTransit"),
            ZmanPrimitive::SolarMidnight => defmt::write!(fmt, "SolarMidnight"),
            ZmanPrimitive::ElevationAdjustedSunset => defmt::write!(fmt, "ElevationAdjustedSunset"),
            ZmanPrimitive::SeaLevelSunset => defmt::write!(fmt, "SeaLevelSunset"),
            ZmanPrimitive::SunriseOffsetByDegrees(degrees) => {
                defmt::write!(fmt, "SunriseOffsetByDegrees({})", degrees)
            }
            ZmanPrimitive::SunsetOffsetByDegrees(degrees) => {
                defmt::write!(fmt, "SunsetOffsetByDegrees({})", degrees)
            }
            ZmanPrimitive::LocalMeanTime(hour) => defmt::write!(fmt, "LocalMeanTime({})", hour),
            ZmanPrimitive::CandleLighting => defmt::write!(fmt, "CandleLighting"),
            ZmanPrimitive::Offset(primitive, duration) => {
                defmt::write!(fmt, "Offset({}, {}s)", primitive, duration.as_secs_f64())
            }
            ZmanPrimitive::ZmanisOffset(primitive, hours) => {
                defmt::write!(fmt, "ZmanisOffset({}, {})", primitive, hours)
            }
            ZmanPrimitive::HalfDayBasedOffset(start, end, fraction) => {
                defmt::write!(fmt, "HalfDayBasedOffset({}, {}, {})", start, end, fraction)
            }
            ZmanPrimitive::Shema(start, end, fixed) => {
                defmt::write!(fmt, "Shema({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::MinchaGedola(start, end, fixed) => {
                defmt::write!(fmt, "MinchaGedola({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::SamuchLeMinchaKetana(start, end, fixed) => {
                defmt::write!(fmt, "SamuchLeMinchaKetana({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::MinchaKetana(start, end, fixed) => {
                defmt::write!(fmt, "MinchaKetana({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::Tefila(start, end, fixed) => {
                defmt::write!(fmt, "Tefila({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::PlagHamincha(start, end, fixed) => {
                defmt::write!(fmt, "PlagHamincha({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::SofZmanBiurChametz(start, end, fixed) => {
                defmt::write!(fmt, "SofZmanBiurChametz({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::SofZmanAchilasChametz(start, end, fixed) => {
                defmt::write!(fmt, "SofZmanAchilasChametz({}, {}, {})", start, end, fixed)
            }
            ZmanPrimitive::TzaisAteretTorah => defmt::write!(fmt, "TzaisAteretTorah"),
            ZmanPrimitive::SofZmanKidushLevana15Days => {
                defmt::write!(fmt, "SofZmanKidushLevana15Days")
            }
            ZmanPrimitive::SofZmanKidushLevanaBetweenMoldos => {
                defmt::write!(fmt, "SofZmanKidushLevanaBetweenMoldos")
            }
            ZmanPrimitive::TchilasZmanKidushLevana3Days => {
                defmt::write!(fmt, "TchilasZmanKidushLevana3Days")
            }
            ZmanPrimitive::TchilasZmanKidushLevana7Days => {
                defmt::write!(fmt, "TchilasZmanKidushLevana7Days")
            }
            ZmanPrimitive::BainHashmashosRt2Stars => defmt::write!(fmt, "BainHashmashosRt2Stars"),
            ZmanPrimitive::MinchaGedolaAhavatShalom => defmt::write!(fmt, "MinchaGedolaAhavatShalom"),
            ZmanPrimitive::MinchaGedolaGraGreaterThan30 => {
                defmt::write!(fmt, "MinchaGedolaGraGreaterThan30")
            }
            ZmanPrimitive::MinchaKetanaAhavatShalom => defmt::write!(fmt, "MinchaKetanaAhavatShalom"),
            ZmanPrimitive::PlagAhavatShalom => defmt::write!(fmt, "PlagAhavatShalom"),
            ZmanPrimitive::BeginCivilTwilight => defmt::write!(fmt, "BeginCivilTwilight"),
            ZmanPrimitive::EndCivilTwilight => defmt::write!(fmt, "EndCivilTwilight"),
            ZmanPrimitive::BeginNauticalTwilight => defmt::write!(fmt, "BeginNauticalTwilight"),
            ZmanPrimitive::EndNauticalTwilight => defmt::write!(fmt, "EndNauticalTwilight"),
            ZmanPrimitive::BeginAstronomicalTwilight => defmt::write!(fmt, "BeginAstronomicalTwilight"),
            ZmanPrimitive::EndAstronomicalTwilight => defmt::write!(fmt, "EndAstronomicalTwilight"),
            ZmanPrimitive::PolarSunsetBenIshChai => defmt::write!(fmt, "PolarSunsetBenIshChai"),
            ZmanPrimitive::PolarSunriseBenIshChai => defmt::write!(fmt, "PolarSunriseBenIshChai"),
            ZmanPrimitive::ChatzosHayomAsHalfDay => defmt::write!(fmt, "ChatzosHayomAsHalfDay"),
            ZmanPrimitive::ChatzosHayom => defmt::write!(fmt, "ChatzosHayom"),
            ZmanPrimitive::ChatzosHalaylaAsHalfDay => defmt::write!(fmt, "ChatzosHalaylaAsHalfDay"),
            ZmanPrimitive::ChatzosHalayla => defmt::write!(fmt, "ChatzosHalayla"),
        }
    }
}

impl ZmanLike for ZmanPrimitive {
    fn calculate(&self, calculator: &ZmanimCalculator) -> Result<Timestamp, ZmanimError> {
        match *self {
            ZmanPrimitive::ConfiguredSunrise => {
                if calculator.config.use_elevation {
                    ZmanPrimitive::ElevationAdjustedSunrise.calculate(calculator)
                } else {
                    ZmanPrimitive::SeaLevelSunrise.calculate(calculator)
                }
            }
            ZmanPrimitive::ConfiguredSunset => {
                if calculator.config.use_elevation {
                    ZmanPrimitive::ElevationAdjustedSunset.calculate(calculator)
                } else {
                    ZmanPrimitive::SeaLevelSunset.calculate(calculator)
                }
            }
            ZmanPrimitive::ElevationAdjustedSunrise => {
                crate::zmanim::astronomy::sunrise(calculator.date, &calculator.location, true)
            }
            ZmanPrimitive::SeaLevelSunrise => {
                crate::zmanim::astronomy::sunrise(calculator.date, &calculator.location, false)
            }
            ZmanPrimitive::SolarTransit => crate::zmanim::astronomy::solar_noon(calculator.date, &calculator.location),
            ZmanPrimitive::SolarMidnight => {
                crate::zmanim::astronomy::solar_midnight(calculator.date, &calculator.location)
            }
            ZmanPrimitive::ElevationAdjustedSunset => {
                crate::zmanim::astronomy::sunset(calculator.date, &calculator.location, true)
            }
            ZmanPrimitive::SeaLevelSunset => {
                crate::zmanim::astronomy::sunset(calculator.date, &calculator.location, false)
            }
            ZmanPrimitive::SunriseOffsetByDegrees(degrees) => {
                crate::zmanim::astronomy::sunrise_offset_by_degrees(calculator.date, &calculator.location, degrees)
            }
            ZmanPrimitive::SunsetOffsetByDegrees(degrees) => {
                crate::zmanim::astronomy::sunset_offset_by_degrees(calculator.date, &calculator.location, degrees)
            }
            ZmanPrimitive::LocalMeanTime(hours) => {
                let date = calculator.date;
                let location = calculator.location.clone();
                if !(0.0..24.0).contains(&hours) {
                    return Err(ZmanimError::InvalidHours);
                }

                let adjusted_date = crate::zmanim::astronomy::adjusted_local_date(date, &location)?;
                let midnight = adjusted_date.at(0, 0, 0, 0);
                let lmt_nanos = CoreFloat::round(hours * 3600.0 * 1_000_000_000.0) as i64;
                let offset_nanos = CoreFloat::round(location.longitude * 240.0 * 1_000_000_000.0) as i64;
                let lmt_dt = midnight
                    .checked_add(Duration::from_nanos(lmt_nanos))
                    .and_then(|dt| dt.checked_sub(Duration::from_nanos(offset_nanos)))
                    .map_err(|_| ZmanimError::TimeConversionError)?;
                let utc = lmt_dt
                    .to_zoned(TimeZone::UTC)
                    .map_err(|_| ZmanimError::TimeConversionError)?
                    .timestamp();

                Ok(utc)
            }
            ZmanPrimitive::CandleLighting => {
                // Sea-level sunset occurs earlier than elevation-adjusted sunset.
                // Since candle lighting times are used strictly *l’chumrah* (stringently),
                // we choose the earlier of the two values.
                //
                // This logic is intentionally limited to candle lighting. For other zmanim
                // (e.g., sunset itself), an earlier time is not universally considered
                // *l’chumrah*, so we do not apply this rule there.
                let sunset = ZmanPrimitive::SeaLevelSunset.calculate(calculator)?;
                Ok(sunset - calculator.config.candle_lighting_offset)
            }
            ZmanPrimitive::Offset(event, duration) => {
                let event_time = event.calculate(calculator)?;
                Ok(event_time + duration)
            }
            ZmanPrimitive::ZmanisOffset(event, hours) => {
                let event_time = event.calculate(calculator)?;
                let sunrise = calculator.calculate(&ZmanPrimitive::ConfiguredSunrise)?;
                let sunset = calculator.calculate(&ZmanPrimitive::ConfiguredSunset)?;
                let shaah_zmanis = sunset.duration_since(sunrise) / 12;
                let offset = shaah_zmanis.mul_f64(hours);

                Ok(event_time + offset)
            }

            ZmanPrimitive::HalfDayBasedOffset(event1, event2, hours) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator)?;
                let shaah_zmanis = event2_time.duration_since(event1_time) / 6;
                let offset = shaah_zmanis.mul_f64(hours);
                if hours >= 0.0 {
                    Ok(event1_time + offset)
                } else {
                    Ok(event2_time + offset)
                }
            }
            ZmanPrimitive::Shema(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator);
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::ChatzosHayom)?;
                    let shaah_zmanis = chatzos.duration_since(event1_time) / 6;
                    let offset = shaah_zmanis.mul_f64(3.0);
                    Ok(event1_time + offset)
                } else {
                    let event2_time = event2_time?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(3.0);
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::MinchaGedola(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::ChatzosHayom)?;
                    let shaah_zmanis = event2_time.duration_since(chatzos) / 6;
                    let offset = shaah_zmanis.mul_f64(0.5);
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(6.5);
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::SamuchLeMinchaKetana(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::ChatzosHayom)?;
                    let shaah_zmanis = event2_time.duration_since(chatzos) / 6;
                    let offset = shaah_zmanis.mul_f64(3.0);
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(9.0);
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::MinchaKetana(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::ChatzosHayom)?;
                    let shaah_zmanis = event2_time.duration_since(chatzos) / 6;
                    let offset = shaah_zmanis.mul_f64(3.5);
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(9.5);
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::Tefila(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator);
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::ChatzosHayom)?;
                    let shaah_zmanis = chatzos.duration_since(event1_time) / 6;
                    let offset = shaah_zmanis.mul_f64(4.0);
                    Ok(event1_time + offset)
                } else {
                    let event2_time = event2_time?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(4.0);
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::PlagHamincha(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::ChatzosHayom)?;
                    let shaah_zmanis = event2_time.duration_since(chatzos) / 6;
                    let offset = shaah_zmanis.mul_f64(4.75);
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(10.75);
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::SofZmanBiurChametz(event1, event2, synchronous) => {
                if !calculator.date.holidays(false, false).any(|h| h == Holiday::ErevPesach) {
                    return Err(ZmanimError::InvalidForDate);
                }
                let primitive = if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    ZmanPrimitive::HalfDayBasedOffset(event1, &ZmanPrimitive::ChatzosHayom, 5.0)
                } else {
                    let event1_time = event1.calculate(calculator)?;
                    let event2_time = event2.calculate(calculator)?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(5.0);
                    return Ok(event1_time + offset);
                };
                primitive.calculate(calculator)
            }
            ZmanPrimitive::SofZmanAchilasChametz(event1, event2, synchronous) => {
                if !calculator.date.holidays(false, false).any(|h| h == Holiday::ErevPesach) {
                    return Err(ZmanimError::InvalidForDate);
                }
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator);
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::ChatzosHayom)?;
                    let shaah_zmanis = chatzos.duration_since(event1_time) / 6;
                    let offset = shaah_zmanis.mul_f64(4.0);
                    Ok(event1_time + offset)
                } else {
                    let event2_time = event2_time?;
                    let shaah_zmanis = event2_time.duration_since(event1_time) / 12;
                    let offset = shaah_zmanis.mul_f64(4.0);
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::TzaisAteretTorah => {
                let sunset = ZmanPrimitive::ConfiguredSunset.calculate(calculator)?;
                Ok(sunset + calculator.config.ateret_torah_sunset_offset)
            }
            ZmanPrimitive::SofZmanKidushLevana15Days => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let hebrew = calculator.date.hebrew_date();

                if hebrew.day_of_month().0 < 11 || hebrew.day_of_month().0 > 17 {
                    return Err(ZmanimError::InvalidForDate);
                }
                let molad = months_molad(&hebrew)? + Duration::from_hours(24 * 15);
                if is_same_gregorian_day(&hebrew, &molad, tz) {
                    return Ok(molad);
                }
                Err(ZmanimError::InvalidForDate)
            }
            ZmanPrimitive::SofZmanKidushLevanaBetweenMoldos => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let hebrew = calculator.date.hebrew_date();

                if hebrew.day_of_month().0 < 11 || hebrew.day_of_month().0 > 16 {
                    return Err(ZmanimError::InvalidForDate);
                }
                let molad = months_molad(&hebrew)?
                    + Duration::from_hours(24 * 14 + 18)
                    + Duration::from_mins(22)
                    + Duration::from_secs(1)
                    + Duration::from_millis(666);
                if is_same_gregorian_day(&hebrew, &molad, tz) {
                    return Ok(molad);
                }
                Err(ZmanimError::InvalidForDate)
            }
            ZmanPrimitive::TchilasZmanKidushLevana3Days => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let hebrew = calculator.date.hebrew_date();

                if hebrew.day_of_month().0 > 5 && hebrew.day_of_month().0 < 30 {
                    return Err(ZmanimError::InvalidForDate);
                }
                let molad = months_molad(&hebrew)? + Duration::from_hours(72);

                if is_same_gregorian_day(&hebrew, &molad, tz) {
                    return Ok(molad);
                }

                if hebrew.day_of_month().0 == 30 {
                    let mut add_option = DateAddOptions::default();
                    add_option.overflow = Some(Overflow::Constrain);

                    let new = hebrew
                        .try_added_with_options(DateDuration::for_months(1), add_option)
                        .map_err(|_| ZmanimError::TimeConversionError)?;
                    let molad = months_molad(&new)? + Duration::from_hours(72);
                    if is_same_gregorian_day(&hebrew, &molad, tz) {
                        return Ok(molad);
                    }
                }
                Err(ZmanimError::InvalidForDate)
            }
            ZmanPrimitive::TchilasZmanKidushLevana7Days => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let hebrew = calculator.date.hebrew_date();

                if hebrew.day_of_month().0 < 4 || hebrew.day_of_month().0 > 9 {
                    return Err(ZmanimError::InvalidForDate);
                }
                let molad = months_molad(&hebrew)? + Duration::from_hours(168);
                if is_same_gregorian_day(&hebrew, &molad, tz) {
                    return Ok(molad);
                }
                Err(ZmanimError::InvalidForDate)
            }
            ZmanPrimitive::BainHashmashosRt2Stars => {
                let alos19_point_8 = ZmanPrimitive::SunriseOffsetByDegrees(19.8).calculate(calculator)?;
                let sunrise = ZmanPrimitive::ConfiguredSunrise.calculate(calculator)?;
                let sunset = ZmanPrimitive::ConfiguredSunset.calculate(calculator)?;
                let time_diff = sunrise.duration_since(alos19_point_8);
                let offset = time_diff.as_millis() as f64 * (5.0 / 18.0);
                Ok(sunset + Duration::from_millis(offset as i64))
            }
            ZmanPrimitive::MinchaGedolaAhavatShalom => {
                let chatzos = ZmanPrimitive::ChatzosHayom.calculate(calculator)?;
                let mincha_gedola_30 = chatzos + Duration::from_mins(30);

                let alos = ZmanPrimitive::SunriseOffsetByDegrees(16.1).calculate(calculator)?;
                let tzais = ZmanPrimitive::SunsetOffsetByDegrees(3.7).calculate(calculator)?;
                let shaah_zmanis = tzais.duration_since(alos) / 12;
                let mincha_alternative = chatzos + (shaah_zmanis / 2);
                if mincha_gedola_30 > mincha_alternative {
                    Ok(mincha_gedola_30)
                } else {
                    Ok(mincha_alternative)
                }
            }
            ZmanPrimitive::MinchaGedolaGraGreaterThan30 => {
                let mincha_gedola_30 = ZmanPrimitive::Offset(&ZmanPrimitive::ChatzosHayom, Duration::from_mins(30))
                    .calculate(calculator)?;
                let mincha_gedola_gra = ZmanPrimitive::MinchaGedola(
                    &ZmanPrimitive::ConfiguredSunrise,
                    &ZmanPrimitive::ConfiguredSunset,
                    true,
                )
                .calculate(calculator)?;
                Ok(mincha_gedola_30.max(mincha_gedola_gra))
            }
            ZmanPrimitive::MinchaKetanaAhavatShalom => {
                let tzais = ZmanPrimitive::SunsetOffsetByDegrees(3.8).calculate(calculator)?;
                let alos = ZmanPrimitive::SunriseOffsetByDegrees(16.1).calculate(calculator)?;
                let shaah_zmanis = tzais.duration_since(alos) / 12;
                Ok(tzais - (shaah_zmanis * 5 / 2))
            }
            ZmanPrimitive::PlagAhavatShalom => {
                let tzais = ZmanPrimitive::SunsetOffsetByDegrees(3.8).calculate(calculator)?;
                let alos = ZmanPrimitive::SunriseOffsetByDegrees(16.1).calculate(calculator)?;
                let shaah_zmanis = tzais.duration_since(alos) / 12;
                Ok(tzais - (shaah_zmanis * 5 / 4))
            }
            ZmanPrimitive::BeginCivilTwilight => {
                calculator.calculate(&ZmanPrimitive::SunriseOffsetByDegrees(CIVIL_ZENITH))
            }
            ZmanPrimitive::EndCivilTwilight => {
                calculator.calculate(&ZmanPrimitive::SunsetOffsetByDegrees(CIVIL_ZENITH))
            }
            ZmanPrimitive::BeginNauticalTwilight => {
                calculator.calculate(&ZmanPrimitive::SunriseOffsetByDegrees(NAUTICAL_ZENITH))
            }
            ZmanPrimitive::EndNauticalTwilight => {
                calculator.calculate(&ZmanPrimitive::SunsetOffsetByDegrees(NAUTICAL_ZENITH))
            }
            ZmanPrimitive::BeginAstronomicalTwilight => {
                calculator.calculate(&ZmanPrimitive::SunriseOffsetByDegrees(ASTRONOMICAL_ZENITH))
            }
            ZmanPrimitive::EndAstronomicalTwilight => {
                calculator.calculate(&ZmanPrimitive::SunsetOffsetByDegrees(ASTRONOMICAL_ZENITH))
            }
            ZmanPrimitive::PolarSunsetBenIshChai => match ZmanPrimitive::ConfiguredSunset.calculate(calculator) {
                Ok(_) => Err(ZmanimError::CalculationError),
                Err(ZmanimError::AllDay | ZmanimError::AllNight) => {
                    crate::zmanim::astronomy::time_at_azimuth(calculator.date, &calculator.location, 270.0)
                }
                Err(error) => Err(error),
            },
            ZmanPrimitive::PolarSunriseBenIshChai => match ZmanPrimitive::ConfiguredSunrise.calculate(calculator) {
                Ok(_) => Err(ZmanimError::CalculationError),
                Err(ZmanimError::AllDay | ZmanimError::AllNight) => {
                    crate::zmanim::astronomy::time_at_azimuth(calculator.date, &calculator.location, 90.0)
                }
                Err(error) => Err(error),
            },
            ZmanPrimitive::ChatzosHayomAsHalfDay => {
                let sunrise = ZmanPrimitive::SeaLevelSunrise.calculate(calculator)?;
                let sunset = ZmanPrimitive::SeaLevelSunset.calculate(calculator)?;
                let diff = sunset.duration_since(sunrise) / 2;
                Ok(sunrise + diff)
            }
            ZmanPrimitive::ChatzosHayom => {
                if calculator.config.use_astronomical_chatzos {
                    ZmanPrimitive::SolarTransit.calculate(calculator)
                } else {
                    match ZmanPrimitive::ChatzosHayomAsHalfDay.calculate(calculator) {
                        Ok(chatzos) => Ok(chatzos),
                        Err(_) => ZmanPrimitive::SolarTransit.calculate(calculator),
                    }
                }
            }
            ZmanPrimitive::ChatzosHalaylaAsHalfDay => {
                let sunset = ZmanPrimitive::SeaLevelSunset.calculate(calculator)?;
                // Create a copy of the calculator and increment the date by one day
                let mut tomorrow = calculator.clone();
                tomorrow.date = tomorrow
                    .date
                    .checked_add(1.day())
                    .map_err(|_| ZmanimError::TimeConversionError)?;
                let sunrise = ZmanPrimitive::SeaLevelSunrise.calculate(&tomorrow)?;
                let diff = sunrise.duration_since(sunset) / 2;
                Ok(sunset + diff)
            }
            ZmanPrimitive::ChatzosHalayla => {
                if calculator.config.use_astronomical_chatzos {
                    ZmanPrimitive::SolarMidnight.calculate(calculator)
                } else {
                    match ZmanPrimitive::ChatzosHalaylaAsHalfDay.calculate(calculator) {
                        Ok(chatzos) => Ok(chatzos),
                        Err(_) => ZmanPrimitive::SolarMidnight.calculate(calculator),
                    }
                }
            }
        }
    }
}

#[cfg(feature = "alloc")]
/// These methods are used to create more informed user docs for presets.
impl ZmanPrimitive {
    pub(crate) fn uses_astronomical_chatzos_for_other_zmanim_from_config(&self) -> bool {
        match self {
            ZmanPrimitive::Shema(_, _, synchronous)
            | ZmanPrimitive::MinchaGedola(_, _, synchronous)
            | ZmanPrimitive::SamuchLeMinchaKetana(_, _, synchronous)
            | ZmanPrimitive::MinchaKetana(_, _, synchronous)
            | ZmanPrimitive::Tefila(_, _, synchronous)
            | ZmanPrimitive::PlagHamincha(_, _, synchronous)
            | ZmanPrimitive::SofZmanBiurChametz(_, _, synchronous)
            | ZmanPrimitive::SofZmanAchilasChametz(_, _, synchronous) => *synchronous,
            ZmanPrimitive::MinchaGedolaGraGreaterThan30 => true,
            ZmanPrimitive::ElevationAdjustedSunrise
            | ZmanPrimitive::SeaLevelSunrise
            | ZmanPrimitive::ConfiguredSunrise
            | ZmanPrimitive::ConfiguredSunset
            | ZmanPrimitive::SolarTransit
            | ZmanPrimitive::SolarMidnight
            | ZmanPrimitive::ChatzosHayomAsHalfDay
            | ZmanPrimitive::ChatzosHayom
            | ZmanPrimitive::ChatzosHalaylaAsHalfDay
            | ZmanPrimitive::ChatzosHalayla
            | ZmanPrimitive::ElevationAdjustedSunset
            | ZmanPrimitive::SeaLevelSunset
            | ZmanPrimitive::SunriseOffsetByDegrees(_)
            | ZmanPrimitive::SunsetOffsetByDegrees(_)
            | ZmanPrimitive::LocalMeanTime(_)
            | ZmanPrimitive::CandleLighting
            | ZmanPrimitive::Offset(_, _)
            | ZmanPrimitive::ZmanisOffset(_, _)
            | ZmanPrimitive::HalfDayBasedOffset(_, _, _)
            | ZmanPrimitive::TzaisAteretTorah
            | ZmanPrimitive::BainHashmashosRt2Stars
            | ZmanPrimitive::SofZmanKidushLevana15Days
            | ZmanPrimitive::SofZmanKidushLevanaBetweenMoldos
            | ZmanPrimitive::TchilasZmanKidushLevana3Days
            | ZmanPrimitive::TchilasZmanKidushLevana7Days
            | ZmanPrimitive::MinchaGedolaAhavatShalom
            | ZmanPrimitive::MinchaKetanaAhavatShalom
            | ZmanPrimitive::PlagAhavatShalom
            | ZmanPrimitive::BeginCivilTwilight
            | ZmanPrimitive::EndCivilTwilight
            | ZmanPrimitive::BeginNauticalTwilight
            | ZmanPrimitive::EndNauticalTwilight
            | ZmanPrimitive::BeginAstronomicalTwilight
            | ZmanPrimitive::EndAstronomicalTwilight
            | ZmanPrimitive::PolarSunsetBenIshChai
            | ZmanPrimitive::PolarSunriseBenIshChai => false,
        }
    }

    pub(crate) fn uses_astronomical_chatzos_from_config(&self) -> bool {
        match self {
            ZmanPrimitive::ChatzosHayom | ZmanPrimitive::ChatzosHalayla => true,
            ZmanPrimitive::Offset(event, _) => event.uses_astronomical_chatzos_from_config(),
            ZmanPrimitive::ElevationAdjustedSunrise
            | ZmanPrimitive::SeaLevelSunrise
            | ZmanPrimitive::ConfiguredSunrise
            | ZmanPrimitive::ConfiguredSunset
            | ZmanPrimitive::SolarTransit
            | ZmanPrimitive::SolarMidnight
            | ZmanPrimitive::ChatzosHayomAsHalfDay
            | ZmanPrimitive::ChatzosHalaylaAsHalfDay
            | ZmanPrimitive::ElevationAdjustedSunset
            | ZmanPrimitive::SeaLevelSunset
            | ZmanPrimitive::SunriseOffsetByDegrees(_)
            | ZmanPrimitive::SunsetOffsetByDegrees(_)
            | ZmanPrimitive::LocalMeanTime(_)
            | ZmanPrimitive::CandleLighting
            | ZmanPrimitive::ZmanisOffset(_, _)
            | ZmanPrimitive::HalfDayBasedOffset(_, _, _)
            | ZmanPrimitive::Shema(_, _, _)
            | ZmanPrimitive::MinchaGedola(_, _, _)
            | ZmanPrimitive::SamuchLeMinchaKetana(_, _, _)
            | ZmanPrimitive::MinchaKetana(_, _, _)
            | ZmanPrimitive::Tefila(_, _, _)
            | ZmanPrimitive::PlagHamincha(_, _, _)
            | ZmanPrimitive::SofZmanBiurChametz(_, _, _)
            | ZmanPrimitive::SofZmanAchilasChametz(_, _, _)
            | ZmanPrimitive::TzaisAteretTorah
            | ZmanPrimitive::BainHashmashosRt2Stars
            | ZmanPrimitive::MinchaGedolaGraGreaterThan30
            | ZmanPrimitive::SofZmanKidushLevana15Days
            | ZmanPrimitive::SofZmanKidushLevanaBetweenMoldos
            | ZmanPrimitive::TchilasZmanKidushLevana3Days
            | ZmanPrimitive::TchilasZmanKidushLevana7Days
            | ZmanPrimitive::MinchaGedolaAhavatShalom
            | ZmanPrimitive::MinchaKetanaAhavatShalom
            | ZmanPrimitive::PlagAhavatShalom
            | ZmanPrimitive::BeginCivilTwilight
            | ZmanPrimitive::EndCivilTwilight
            | ZmanPrimitive::BeginNauticalTwilight
            | ZmanPrimitive::EndNauticalTwilight
            | ZmanPrimitive::BeginAstronomicalTwilight
            | ZmanPrimitive::EndAstronomicalTwilight
            | ZmanPrimitive::PolarSunsetBenIshChai
            | ZmanPrimitive::PolarSunriseBenIshChai => false,
        }
    }

    pub(crate) fn uses_elevation_from_config(&self) -> bool {
        match self {
            ZmanPrimitive::ConfiguredSunrise | ZmanPrimitive::ConfiguredSunset => true,
            ZmanPrimitive::Offset(event, _) => event.uses_elevation_from_config(),
            ZmanPrimitive::ZmanisOffset(_, _) => true,
            ZmanPrimitive::HalfDayBasedOffset(start, end, _)
            | ZmanPrimitive::Shema(start, end, _)
            | ZmanPrimitive::MinchaGedola(start, end, _)
            | ZmanPrimitive::SamuchLeMinchaKetana(start, end, _)
            | ZmanPrimitive::MinchaKetana(start, end, _)
            | ZmanPrimitive::Tefila(start, end, _)
            | ZmanPrimitive::PlagHamincha(start, end, _)
            | ZmanPrimitive::SofZmanBiurChametz(start, end, _)
            | ZmanPrimitive::SofZmanAchilasChametz(start, end, _) => {
                start.uses_elevation_from_config() || end.uses_elevation_from_config()
            }
            ZmanPrimitive::TzaisAteretTorah
            | ZmanPrimitive::BainHashmashosRt2Stars
            | ZmanPrimitive::MinchaGedolaGraGreaterThan30
            | ZmanPrimitive::PolarSunsetBenIshChai
            | ZmanPrimitive::PolarSunriseBenIshChai => true,
            ZmanPrimitive::ElevationAdjustedSunrise
            | ZmanPrimitive::SeaLevelSunrise
            | ZmanPrimitive::SolarTransit
            | ZmanPrimitive::SolarMidnight
            | ZmanPrimitive::ChatzosHayomAsHalfDay
            | ZmanPrimitive::ChatzosHayom
            | ZmanPrimitive::ChatzosHalaylaAsHalfDay
            | ZmanPrimitive::ChatzosHalayla
            | ZmanPrimitive::ElevationAdjustedSunset
            | ZmanPrimitive::SeaLevelSunset
            | ZmanPrimitive::SunriseOffsetByDegrees(_)
            | ZmanPrimitive::SunsetOffsetByDegrees(_)
            | ZmanPrimitive::LocalMeanTime(_)
            | ZmanPrimitive::CandleLighting
            | ZmanPrimitive::SofZmanKidushLevana15Days
            | ZmanPrimitive::SofZmanKidushLevanaBetweenMoldos
            | ZmanPrimitive::TchilasZmanKidushLevana3Days
            | ZmanPrimitive::TchilasZmanKidushLevana7Days
            | ZmanPrimitive::MinchaGedolaAhavatShalom
            | ZmanPrimitive::MinchaKetanaAhavatShalom
            | ZmanPrimitive::PlagAhavatShalom
            | ZmanPrimitive::BeginCivilTwilight
            | ZmanPrimitive::EndCivilTwilight
            | ZmanPrimitive::BeginNauticalTwilight
            | ZmanPrimitive::EndNauticalTwilight
            | ZmanPrimitive::BeginAstronomicalTwilight
            | ZmanPrimitive::EndAstronomicalTwilight => false,
        }
    }
}
