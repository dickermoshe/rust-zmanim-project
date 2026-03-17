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

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::{
    calculator::ZmanLike,
    duration_helper::multiply_duration,
    prelude::ZmanimCalculator,
    types::error::{IntoDateTimeResult, ZmanimError},
};

/// A low-level building block for calculating zmanim.
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
                .configured_calculator()
                .get_sunrise_offset_by_degrees(degrees, degrees > 0.0)
                .into_date_time_result(),
            ZmanPrimitive::SunsetOffsetByDegrees(degrees) => calculator
                .configured_calculator()
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
        }
    }
}
