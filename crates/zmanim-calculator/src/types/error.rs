use astronomical_calculator::{CalculationError, SolarEventResult};
use chrono::{DateTime, TimeZone, Utc};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ZmanimError {
    #[error("The provided latitude is invalid. Must be between -90 and 90 degrees.")]
    InvalidLatitude,
    #[error("The provided longitude is invalid. Must be between -180 and 180 degrees.")]
    InvalidLongitude,
    #[error("The provided elevation is invalid. Must be greater than or equal to 0 meters.")]
    InvalidElevation,
    #[error("The provided timezone is required for locations near the anti-meridian.")]
    TimeZoneRequired,
    #[error("The astronomical calculator failed to initialize.")]
    AstronomicalCalculatorError(astronomical_calculator::CalculationError),
    #[error(
        "Noon on the requested date does not exist in the location's timezone (e.g. a DST gap)."
    )]
    LocalNoonError,
    #[error("No zmanim are available for this date at this location because it is all day.")]
    AllDay,
    #[error("No zmanim are available for this date at this location because it is all night.")]
    AllNight,
    #[error(
        "A date/time conversion failed during zman calculation. This is a rare edge case; please report it if encountered."
    )]
    TimeConversionError,
    #[error("The provided hours are invalid. Must be between 0 and 24.")]
    InvalidHours,
}

pub(crate) trait IntoDateTimeResult {
    fn into_date_time_result(self) -> Result<DateTime<Utc>, ZmanimError>;
}

impl IntoDateTimeResult for Result<SolarEventResult, CalculationError> {
    fn into_date_time_result(self) -> Result<DateTime<Utc>, ZmanimError> {
        match self {
            Ok(result) => match result {
                SolarEventResult::Occurs(timestamp) => Ok(Utc
                    .timestamp_opt(timestamp, 0)
                    .single()
                    .ok_or(ZmanimError::TimeConversionError)?),
                SolarEventResult::AllDay => Err(ZmanimError::AllDay),
                SolarEventResult::AllNight => Err(ZmanimError::AllNight),
            },
            Err(e) => Err(ZmanimError::AstronomicalCalculatorError(e)),
        }
    }
}
impl IntoDateTimeResult for Result<i64, CalculationError> {
    fn into_date_time_result(self) -> Result<DateTime<Utc>, ZmanimError> {
        match self {
            Ok(result) => Ok(Utc
                .timestamp_opt(result, 0)
                .single()
                .ok_or(ZmanimError::TimeConversionError)?),
            Err(e) => Err(ZmanimError::AstronomicalCalculatorError(e)),
        }
    }
}
