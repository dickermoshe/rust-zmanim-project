//! Low-level zman formulas used to build higher-level presets.
//!
//! [`ZmanPrimitive`](crate::primitive_zman::ZmanPrimitive) is the internal expression language for zman calculations.
//! Variants represent either:
//! - base astronomical events (for example sunrise/sunset),
//! - transformed events (fixed offsets or degree-based offsets), or
//! - derived halachic times computed from two boundary events.
//!
//! Most users should prefer the ready-made constants in [`crate::presets`].
//! Use this module when you need to compose a custom zman definition that is
//! not already provided by a preset.

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use icu_calendar::Date;

use crate::{
    calculator::ZmanLike,
    duration_helper::multiply_duration,
    molad::MoladCalendar,
    prelude::ZmanimCalculator,
    presets::{
        ALOS_16_POINT_1_DEGREES, MINCHA_GEDOLA_BAAL_HATANYA, MINCHA_GEDOLA_MINUTES_30,
        MINCHA_GEDOLA_SUNRISE_SUNSET, TZAIS_GEONIM_DEGREES_3_POINT_7,
        TZAIS_GEONIM_DEGREES_3_POINT_8,
    },
    types::error::{IntoDateTimeResult, ZmanimError},
};

/// A low-level building block for calculating zmanim.
///
/// These should typically not be used directly. Instead, use the presets in [`crate::presets`].
#[derive(Debug, Clone)]
pub enum ZmanPrimitive<'a> {
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
    Offset(&'a ZmanPrimitive<'a>, Duration),
    /// An offset in "shaos zmaniyos" according to the GRA from another [`ZmanPrimitive`].
    ZmanisOffset(&'a ZmanPrimitive<'a>, f64),
    /// This calculates a temporal hour from the time between the two events, then adds that many temporal hours to the first event.
    ShaahZmanisBasedOffset(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, f64),
    /// An offset expressed as a fraction of the half-day between two [`ZmanPrimitive`]s.
    HalfDayBasedOffset(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, f64),
    /// Sof zman shma derived from two bounding [`ZmanPrimitive`]s.
    Shema(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Mincha gedola derived from two bounding [`ZmanPrimitive`]s.
    MinchaGedola(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Samuch le-mincha ketana derived from two bounding [`ZmanPrimitive`]s.
    SamuchLeMinchaKetana(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Mincha ketana derived from two bounding [`ZmanPrimitive`]s.
    MinchaKetana(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Sof zman tefila derived from two bounding [`ZmanPrimitive`]s.
    Tefila(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Plag hamincha derived from two bounding [`ZmanPrimitive`]s.
    PlagHamincha(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
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
    /// Mincha gedola: later of Baal HaTanya mincha gedola and `30` minutes after solar transit.
    MinchaGedolaBaalHatanyaGreaterThan30,
    /// Mincha gedola: later of [`MINCHA_GEDOLA_SUNRISE_SUNSET`] and [`MINCHA_GEDOLA_MINUTES_30`].
    MinchaGedolaGreaterThan30,
    /// Mincha ketana (Ahavat Shalom): `2.5` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
    MinchaKetanaAhavatShalom,
    /// Plag hamincha (Ahavat Shalom): `1.25` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
    PlagAhavatShalom,
    /// Returns the latest time of _Kiddush Levana_ calculated as 15 days after the molad.
    Molad,
}

impl<'a, Tz: TimeZone> ZmanLike<Tz> for ZmanPrimitive<'a> {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        match *self {
            ZmanPrimitive::ConfiguredSunrise => calculator
                .configured_calculator()
                .get_sunrise()
                .into_date_time_result(),
            ZmanPrimitive::ConfiguredSunset => calculator
                .configured_calculator()
                .get_sunset()
                .into_date_time_result(),
            ZmanPrimitive::ElevationAdjustedSunrise => calculator.elevation_adjusted_sunrise(),
            ZmanPrimitive::SeaLevelSunrise => calculator.sea_level_sunrise(),
            ZmanPrimitive::SolarTransit => calculator.solar_transit(),
            ZmanPrimitive::ElevationAdjustedSunset => calculator.elevation_adjusted_sunset(),
            ZmanPrimitive::SeaLevelSunset => calculator.sea_level_sunset(),
            ZmanPrimitive::SunriseOffsetByDegrees(degrees) => calculator
                .sea_level_calculator
                .get_sunrise_offset_by_degrees(degrees, degrees > 0.0)
                .into_date_time_result(),
            ZmanPrimitive::SunsetOffsetByDegrees(degrees) => calculator
                .sea_level_calculator
                .get_sunset_offset_by_degrees(degrees, degrees > 0.0)
                .into_date_time_result(),
            ZmanPrimitive::LocalMeanTime(hours) => {
                let date = calculator.date;
                let location = calculator.location.clone();
                calculator.local_mean_time(date, &location, hours)
            }
            ZmanPrimitive::CandleLighting => {
                // Sea-level sunset occurs earlier than elevation-adjusted sunset.
                // Since candle lighting times are used strictly *l’chumrah* (stringently),
                // we choose the earlier of the two values.
                //
                // This logic is intentionally limited to candle lighting. For other zmanim
                // (e.g., sunset itself), an earlier time is not universally considered
                // *l’chumrah*, so we do not apply this rule there.
                let sunset = calculator.sea_level_sunset()?;
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
                let shaah_zmanis = (sunset - sunrise) / 12;
                let offset = multiply_duration(shaah_zmanis, hours)
                    .ok_or(ZmanimError::TimeConversionError)?;
                Ok(event_time + offset)
            }
            ZmanPrimitive::ShaahZmanisBasedOffset(event1, event2, hours) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator)?;
                let shaah_zmanis = (event2_time - event1_time) / 12;
                let offset = multiply_duration(shaah_zmanis, hours)
                    .ok_or(ZmanimError::TimeConversionError)?;
                Ok(event1_time + offset)
            }
            ZmanPrimitive::HalfDayBasedOffset(event1, event2, hours) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator)?;
                let shaah_zmanis = (event2_time - event1_time) / 6;
                let offset = multiply_duration(shaah_zmanis, hours)
                    .ok_or(ZmanimError::TimeConversionError)?;
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
                    let chatzos = calculator.calculate(&ZmanPrimitive::SolarTransit)?;
                    let shaah_zmanis = (chatzos - event1_time) / 6;
                    let offset = multiply_duration(shaah_zmanis, 3.0)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                } else {
                    let event2_time = event2_time?;
                    let shaah_zmanis = (event2_time - event1_time) / 12;
                    let offset = multiply_duration(shaah_zmanis, 3.0)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::MinchaGedola(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::SolarTransit)?;
                    let shaah_zmanis = (event2_time - chatzos) / 6;
                    let offset = multiply_duration(shaah_zmanis, 0.5)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = (event2_time - event1_time) / 12;
                    let offset = multiply_duration(shaah_zmanis, 6.5)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::SamuchLeMinchaKetana(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::SolarTransit)?;
                    let shaah_zmanis = (event2_time - chatzos) / 6;
                    let offset = multiply_duration(shaah_zmanis, 3.0)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = (event2_time - event1_time) / 12;
                    let offset = multiply_duration(shaah_zmanis, 9.0)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::MinchaKetana(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::SolarTransit)?;
                    let shaah_zmanis = (event2_time - chatzos) / 6;
                    let offset = multiply_duration(shaah_zmanis, 3.5)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = (event2_time - event1_time) / 12;
                    let offset = multiply_duration(shaah_zmanis, 9.5)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::Tefila(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator);
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::SolarTransit)?;
                    let shaah_zmanis = (chatzos - event1_time) / 6;
                    let offset = multiply_duration(shaah_zmanis, 4.0)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                } else {
                    let event2_time = event2_time?;
                    let shaah_zmanis = (event2_time - event1_time) / 12;
                    let offset = multiply_duration(shaah_zmanis, 4.0)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                }
            }
            ZmanPrimitive::PlagHamincha(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
                    let chatzos = calculator.calculate(&ZmanPrimitive::SolarTransit)?;
                    let shaah_zmanis = (event2_time - chatzos) / 6;
                    let offset = multiply_duration(shaah_zmanis, 4.75)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(chatzos + offset)
                } else {
                    let event1_time = event1_time?;
                    let shaah_zmanis = (event2_time - event1_time) / 12;
                    let offset = multiply_duration(shaah_zmanis, 10.75)
                        .ok_or(ZmanimError::TimeConversionError)?;
                    Ok(event1_time + offset)
                }
            }
            Self::TzaisAteretTorah => {
                let sunset = calculator
                    .configured_calculator()
                    .get_sunset()
                    .into_date_time_result()?;
                Ok(sunset + calculator.config.ateret_torah_sunset_offset)
            }
            ZmanPrimitive::SofZmanKidushLevana15Days => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let date = Date::try_new_gregorian(
                    calculator.date.year(),
                    calculator.date.month() as u8,
                    calculator.date.day() as u8,
                )
                .map_err(|_| ZmanimError::TimeConversionError)?;
                date.sof_zman_kidush_levana_15_days(tz)
                    .map(|i| i.0.to_utc())
                    .ok_or(ZmanimError::TimeConversionError)
            }
            ZmanPrimitive::SofZmanKidushLevanaBetweenMoldos => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let date = Date::try_new_gregorian(
                    calculator.date.year(),
                    calculator.date.month() as u8,
                    calculator.date.day() as u8,
                )
                .map_err(|_| ZmanimError::TimeConversionError)?;
                date.sof_zman_kidush_levana_between_moldos(tz)
                    .map(|i| i.0.to_utc())
                    .ok_or(ZmanimError::TimeConversionError)
            }
            ZmanPrimitive::TchilasZmanKidushLevana3Days => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let date = Date::try_new_gregorian(
                    calculator.date.year(),
                    calculator.date.month() as u8,
                    calculator.date.day() as u8,
                )
                .map_err(|_| ZmanimError::TimeConversionError)?;
                date.tchilas_zman_kidush_levana_3_days(tz)
                    .map(|i| i.0.to_utc())
                    .ok_or(ZmanimError::TimeConversionError)
            }
            ZmanPrimitive::TchilasZmanKidushLevana7Days => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let date = Date::try_new_gregorian(
                    calculator.date.year(),
                    calculator.date.month() as u8,
                    calculator.date.day() as u8,
                )
                .map_err(|_| ZmanimError::TimeConversionError)?;
                date.tchilas_zman_kidush_levana_7_days(tz)
                    .map(|i| i.0.to_utc())
                    .ok_or(ZmanimError::TimeConversionError)
            }
            ZmanPrimitive::BainHashmashosRt2Stars => {
                let alos19_point_8 =
                    ZmanPrimitive::SunriseOffsetByDegrees(19.8).calculate(calculator)?;
                let sunrise = ZmanPrimitive::ConfiguredSunrise.calculate(calculator)?;
                let sunset = ZmanPrimitive::ConfiguredSunset.calculate(calculator)?;
                let time_diff = sunrise.signed_duration_since(alos19_point_8);
                let offset = time_diff.num_milliseconds() as f64 * (5.0 / 18.0);
                Ok(sunset + Duration::milliseconds(offset as i64))
            }
            ZmanPrimitive::MinchaGedolaAhavatShalom => {
                let chatzos = ZmanPrimitive::SolarTransit.calculate(calculator)?;
                let mincha_gedola_30 = chatzos + Duration::minutes(30);

                let alos = ALOS_16_POINT_1_DEGREES.calculate(calculator)?;
                let tzais = TZAIS_GEONIM_DEGREES_3_POINT_7.calculate(calculator)?;
                let shaah_zmanis = (tzais - alos) / 12;
                let mincha_alternative = chatzos + (shaah_zmanis / 2);
                if mincha_gedola_30 > mincha_alternative {
                    Ok(mincha_gedola_30)
                } else {
                    Ok(mincha_alternative)
                }
            }
            ZmanPrimitive::MinchaGedolaBaalHatanyaGreaterThan30 => {
                let mincha_30 = MINCHA_GEDOLA_MINUTES_30.calculate(calculator)?;
                let mincha_baal_hatanya = MINCHA_GEDOLA_BAAL_HATANYA.calculate(calculator)?;
                if mincha_30 > mincha_baal_hatanya {
                    Ok(mincha_30)
                } else {
                    Ok(mincha_baal_hatanya)
                }
            }
            ZmanPrimitive::MinchaGedolaGreaterThan30 => {
                let mincha_30 = MINCHA_GEDOLA_MINUTES_30.calculate(calculator)?;
                let mincha_regular = MINCHA_GEDOLA_SUNRISE_SUNSET.calculate(calculator)?;
                if mincha_30 > mincha_regular {
                    Ok(mincha_30)
                } else {
                    Ok(mincha_regular)
                }
            }
            ZmanPrimitive::MinchaKetanaAhavatShalom => {
                let tzais = TZAIS_GEONIM_DEGREES_3_POINT_8.calculate(calculator)?;
                let alos = ALOS_16_POINT_1_DEGREES.calculate(calculator)?;
                let shaah_zmanis = (tzais - alos) / 12;
                Ok(tzais - (shaah_zmanis * 5 / 2))
            }
            ZmanPrimitive::PlagAhavatShalom => {
                let tzais = ZmanPrimitive::SunsetOffsetByDegrees(3.8).calculate(calculator)?;
                let alos = ZmanPrimitive::SunriseOffsetByDegrees(16.1).calculate(calculator)?;
                let shaah_zmanis = (tzais - alos) / 12;
                Ok(tzais - (shaah_zmanis * 5 / 4))
            }
            ZmanPrimitive::Molad => {
                let tz = calculator
                    .location
                    .timezone
                    .as_ref()
                    .ok_or(ZmanimError::TimeZoneRequired)?;
                let date = Date::try_new_gregorian(
                    calculator.date.year(),
                    calculator.date.month() as u8,
                    calculator.date.day() as u8,
                )
                .map_err(|_| ZmanimError::TimeConversionError)?;
                date.molad(tz)
                    .map(|i| i.0.to_utc())
                    .ok_or(ZmanimError::TimeConversionError)
            }
        }
    }
}
