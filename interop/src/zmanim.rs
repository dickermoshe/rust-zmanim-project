//! Zmanim FFI bridge.

#[diplomat::bridge]
mod ffi {
    use diplomat_runtime::DiplomatStr;
    use jiff::{SignedDuration, civil::Date};
    use kosher_rust::zmanim::{
        ZmanimCalculator,
        types::{config::CalculatorConfig as CoreCalculatorConfig, error::ZmanimError, location::Location},
    };

    use crate::common::{parse_timezone, timestamp_to_epoch_ms};
    use crate::generated::preset_dispatch::preset_by_index;

    pub enum ZmanimErrorCode {
        InvalidLatitude,
        InvalidLongitude,
        InvalidElevation,
        TimeZoneRequired,
        CalculationError,
        AllDay,
        AllNight,
        TimeConversionError,
        InvalidForDate,
        InvalidHours,
    }

    /// Calculator configuration (durations in minutes).
    pub struct CalculatorConfig {
        pub candle_lighting_offset_minutes: i32,
        pub use_astronomical_chatzos_for_other_zmanim: bool,
        pub use_elevation: bool,
        pub ateret_torah_sunset_offset_minutes: i32,
        pub use_astronomical_chatzos: bool,
    }

    /// Gregorian civil date (year, month, day).
    pub struct CivilDate {
        pub year: i32,
        pub month: u8,
        pub day: u8,
    }

    #[diplomat::opaque]
    pub struct FfiLocation(Location);

    impl FfiLocation {
        /// Creates a location. Pass an empty timezone string when unknown.
        pub fn new(
            latitude: f64,
            longitude: f64,
            elevation: f64,
            timezone_iana: &DiplomatStr,
        ) -> Result<Box<FfiLocation>, ZmanimErrorCode> {
            let timezone = core::str::from_utf8(timezone_iana).ok().and_then(parse_timezone);
            let location = Location::new(latitude, longitude, elevation, timezone).map_err(ZmanimErrorCode::from)?;
            Ok(Box::new(FfiLocation(location)))
        }
    }

    #[diplomat::opaque]
    pub struct FfiZmanimCalculator(ZmanimCalculator);

    impl FfiZmanimCalculator {
        pub fn new(
            location: &FfiLocation,
            date: &CivilDate,
            config: &CalculatorConfig,
        ) -> Option<Box<FfiZmanimCalculator>> {
            let date = Date::new(
                i16::try_from(date.year).ok()?,
                i8::try_from(date.month).ok()?,
                i8::try_from(date.day).ok()?,
            )
            .ok()?;
            let config = CoreCalculatorConfig {
                candle_lighting_offset: SignedDuration::from_mins(i64::from(config.candle_lighting_offset_minutes)),
                use_astronomical_chatzos_for_other_zmanim: config.use_astronomical_chatzos_for_other_zmanim,
                use_elevation: config.use_elevation,
                ateret_torah_sunset_offset: SignedDuration::from_mins(i64::from(
                    config.ateret_torah_sunset_offset_minutes,
                )),
                use_astronomical_chatzos: config.use_astronomical_chatzos,
            };
            Some(Box::new(FfiZmanimCalculator(ZmanimCalculator::new(
                location.0.clone(),
                date,
                config,
            ))))
        }

        /// Calculates a zman preset by index, returning UTC epoch milliseconds.
        ///
        /// The index matches [`ZmanPresetId`] in the generated [`crate::zman_preset`] module.
        pub fn calculate_zman_by_index(&self, preset_index: u32) -> Result<i64, ZmanimErrorCode> {
            let preset_ref = preset_by_index(preset_index as usize).ok_or(ZmanimErrorCode::InvalidForDate)?;
            let timestamp = self.0.calculate(preset_ref).map_err(ZmanimErrorCode::from)?;
            Ok(timestamp_to_epoch_ms(timestamp))
        }
    }

    impl From<crate::common::ZmanimErrorCode> for ZmanimErrorCode {
        fn from(value: crate::common::ZmanimErrorCode) -> Self {
            use crate::common::ZmanimErrorCode as C;
            match value {
                C::InvalidLatitude => Self::InvalidLatitude,
                C::InvalidLongitude => Self::InvalidLongitude,
                C::InvalidElevation => Self::InvalidElevation,
                C::TimeZoneRequired => Self::TimeZoneRequired,
                C::CalculationError => Self::CalculationError,
                C::AllDay => Self::AllDay,
                C::AllNight => Self::AllNight,
                C::TimeConversionError => Self::TimeConversionError,
                C::InvalidForDate => Self::InvalidForDate,
                C::InvalidHours => Self::InvalidHours,
            }
        }
    }

    impl From<ZmanimError> for ZmanimErrorCode {
        fn from(error: ZmanimError) -> Self {
            Self::from(crate::common::ZmanimErrorCode::from(error))
        }
    }
}
