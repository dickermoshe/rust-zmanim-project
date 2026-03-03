//! Low-level zman formulas used to build higher-level presets.
//!
//! [`ZmanPrimitive`] is the internal expression language for zman calculations.
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
pub enum ZmanPrimitive<'a> {
    /// Sunrise at the configured location/date.
    Sunrise,
    /// Sunrise at sea level (no elevation adjustment).
    SeaLevelSunrise,
    /// Solar transit (local apparent noon / astronomical chatzos).
    SolarTransit,
    /// Sunset at the configured location/date.
    Sunset,
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
    /// A fixed time offset from another [`ZmanEvent`].
    Offset(&'a ZmanPrimitive<'a>, Duration),
    /// An offset in "shaos zmaniyos" according to the GRA from another [`ZmanEvent`].
    ZmanisOffset(&'a ZmanPrimitive<'a>, f64),
    /// This calculates a temporal hour from the time between the two events, then adds that many temporal hours to the first event.
    ShaahZmanisBasedOffset(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, f64),
    /// An offset expressed as a fraction of the half-day between two [`ZmanEvent`]s.
    HalfDayBasedOffset(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, f64),
    /// Sof zman shma derived from two bounding [`ZmanEvent`]s.
    Shema(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Mincha gedola derived from two bounding [`ZmanEvent`]s.
    MinchaGedola(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Samuch le-mincha ketana derived from two bounding [`ZmanEvent`]s.
    SamuchLeMinchaKetana(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Mincha ketana derived from two bounding [`ZmanEvent`]s.
    MinchaKetana(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Sof zman tefila derived from two bounding [`ZmanEvent`]s.
    Tefila(&'a ZmanPrimitive<'a>, &'a ZmanPrimitive<'a>, bool),
    /// Plag hamincha derived from two bounding [`ZmanEvent`]s.
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
            ZmanPrimitive::Sunrise => calculator
                .elevation_adjusted_calculator
                .get_sunrise()
                .into_date_time_result(),
            ZmanPrimitive::SeaLevelSunrise => calculator
                .sea_level_calculator
                .get_sea_level_sunrise()
                .into_date_time_result(),
            ZmanPrimitive::SolarTransit => calculator
                .elevation_adjusted_calculator
                .get_solar_transit()
                .into_date_time_result(),
            ZmanPrimitive::Sunset => calculator
                .elevation_adjusted_calculator
                .get_sunset()
                .into_date_time_result(),
            ZmanPrimitive::SeaLevelSunset => calculator
                .sea_level_calculator
                .get_sea_level_sunset()
                .into_date_time_result(),
            ZmanPrimitive::SunriseOffsetByDegrees(degrees) => calculator
                .elevation_adjusted_calculator
                .get_sunrise_offset_by_degrees(degrees, degrees > 0.0)
                .into_date_time_result(),
            ZmanPrimitive::SunsetOffsetByDegrees(degrees) => calculator
                .elevation_adjusted_calculator
                .get_sunset_offset_by_degrees(degrees, degrees > 0.0)
                .into_date_time_result(),
            ZmanPrimitive::LocalMeanTime(hours) => {
                let date = calculator.date;
                let location = calculator.location.clone();
                calculator.local_mean_time(date, &location, hours)
            }
            ZmanPrimitive::CandleLighting => {
                let sunset = calculator.calculate(ZmanPrimitive::SeaLevelSunset)?;
                Ok(sunset - calculator.config.candle_lighting_offset)
            }
            ZmanPrimitive::Offset(event, duration) => {
                let event_time = event.calculate(calculator)?;
                Ok(event_time + duration)
            }
            ZmanPrimitive::ZmanisOffset(event, hours) => {
                let event_time = event.calculate(calculator)?;
                let sunrise = calculator.calculate(ZmanPrimitive::Sunrise)?;
                let sunset = calculator.calculate(ZmanPrimitive::Sunset)?;
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
                    let chatzos = calculator.calculate(ZmanPrimitive::SolarTransit)?;
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
                    let chatzos = calculator.calculate(ZmanPrimitive::SolarTransit)?;
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
                    let chatzos = calculator.calculate(ZmanPrimitive::SolarTransit)?;
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
                    let chatzos = calculator.calculate(ZmanPrimitive::SolarTransit)?;
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
                    let chatzos = calculator.calculate(ZmanPrimitive::SolarTransit)?;
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
                    let chatzos = calculator.calculate(ZmanPrimitive::SolarTransit)?;
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
                let sunset = ZmanPrimitive::Sunset.calculate(calculator)?;
                Ok(sunset + calculator.config.ateret_torah_sunset_offset)
            }
        }
    }
}
#[cfg(test)]
impl<'a> ZmanPrimitive<'a> {
    pub(crate) fn uses_elevation<Tz: TimeZone>(&self, calculator: &ZmanimCalculator<Tz>) -> bool {
        match self {
            ZmanPrimitive::SeaLevelSunrise => false,
            ZmanPrimitive::SeaLevelSunset => false,
            ZmanPrimitive::SunriseOffsetByDegrees(_) => true,
            ZmanPrimitive::SunsetOffsetByDegrees(_) => true,
            ZmanPrimitive::Sunrise => true,
            ZmanPrimitive::SolarTransit => true,
            ZmanPrimitive::Sunset => true,
            ZmanPrimitive::LocalMeanTime(_) => false,
            ZmanPrimitive::CandleLighting => false,
            ZmanPrimitive::ZmanisOffset(_, _) => true,
            ZmanPrimitive::Offset(event, _) => event.uses_elevation(calculator),
            ZmanPrimitive::ShaahZmanisBasedOffset(event1, event2, _)
            | ZmanPrimitive::HalfDayBasedOffset(event1, event2, _) => {
                event1.uses_elevation(calculator) || event2.uses_elevation(calculator)
            }
            ZmanPrimitive::Shema(event1, event2, synchronous)
            | ZmanPrimitive::Tefila(event1, event2, synchronous)
            | ZmanPrimitive::MinchaGedola(event1, event2, synchronous)
            | ZmanPrimitive::SamuchLeMinchaKetana(event1, event2, synchronous)
            | ZmanPrimitive::MinchaKetana(event1, event2, synchronous)
            | ZmanPrimitive::PlagHamincha(event1, event2, synchronous) => {
                if calculator.config.use_astronomical_chatzos_for_other_zmanim && *synchronous {
                    // This path always uses SolarTransit, which is elevation-adjusted.
                    true
                } else {
                    event1.uses_elevation(calculator) || event2.uses_elevation(calculator)
                }
            }
            ZmanPrimitive::TzaisAteretTorah => true,
        }
    }
}
