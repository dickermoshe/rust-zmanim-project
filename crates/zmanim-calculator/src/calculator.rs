use crate::{
    types::{config::CalculatorConfig, error::ZmanimError, location::Location},
    // zman::{ZmanLike, CHATZOS_HALF_DAY},
};
use astronomical_calculator::{AstronomicalCalculator, Refraction};
use chrono::{
    offset::LocalResult, DateTime, Datelike, Duration, NaiveDate, TimeDelta, TimeZone, Utc,
};
#[allow(unused_imports)]
use core_maths::*;

/// Calculates zmanim for a given [`Location`] and [`NaiveDate`].
///
/// Construct once for a location/date pair, then call [`ZmanimCalculator::calculate`]
/// with one or more values implementing [`ZmanLike`].
///
/// Most users should pass one of the ready-made definitions from [`crate::presets`]
/// (for example `presets::SUNRISE`) instead of implementing custom zman logic.
#[derive(Clone, Debug)]
pub struct ZmanimCalculator<Tz: TimeZone> {
    /// The location to calculate for.
    pub location: Location<Tz>,
    /// The civil date at `location` for which zmanim are calculated.
    pub date: NaiveDate,
    /// Calculation configuration options.
    pub config: CalculatorConfig,
    pub(crate) elevation_adjusted_calculator: AstronomicalCalculator,
    pub(crate) sea_level_calculator: AstronomicalCalculator,
}

impl<Tz: TimeZone> ZmanimCalculator<Tz> {
    /// Creates a new calculator for the given `location`, `date`, and `config`.
    ///
    /// Use this as your main entry point before calculating any zmanim.
    ///
    /// # Errors
    ///
    /// Returns an error when the calculator cannot be initialized from the provided
    /// location/date/config values.
    pub fn new(
        location: Location<Tz>,
        date: NaiveDate,
        config: CalculatorConfig,
    ) -> Result<Self, ZmanimError> {
        let localnoon = Self::local_noon(date, &location)?;
        let elevation_adjusted_calculator = AstronomicalCalculator::new(
            localnoon,
            None,
            0.0,
            location.longitude,
            location.latitude,
            location.elevation,
            22.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .map_err(ZmanimError::AstronomicalCalculatorError)?;
        let sea_level_calculator = AstronomicalCalculator::new(
            localnoon,
            None,
            0.0,
            location.longitude,
            location.latitude,
            0.0,
            22.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .map_err(ZmanimError::AstronomicalCalculatorError)?;
        Ok(Self {
            location,
            date,
            config,
            elevation_adjusted_calculator,
            sea_level_calculator,
        })
    }

    /// Calculates a single zman using the current calculator state.
    ///
    /// Pass any value implementing [`ZmanLike`] and receive the resulting instant in UTC.
    ///
    /// This method takes `&mut self` so repeated calls can reuse internal intermediate
    /// computation state for better performance.
    ///
    /// If borrow rules make your call sites awkward, clone the calculator and use each
    /// clone independently (for example, one clone for sunrise and another for sunset).
    pub fn calculate(&mut self, zman: impl ZmanLike<Tz>) -> Result<DateTime<Utc>, ZmanimError> {
        zman.calculate(self)
    }

    fn local_noon<T: TimeZone>(
        date: NaiveDate,
        location: &Location<T>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        // Preferred: convert 12:00:00 in the location's timezone to UTC.
        if let Some(tz) = location.timezone.as_ref() {
            let result = tz.with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0);
            match result {
                LocalResult::Single(dt) => return Ok(dt.to_utc()),
                // During a DST overlap, noon exists twice; either value is close enough.
                LocalResult::Ambiguous(dt, _) => return Ok(dt.to_utc()),
                // Noon falls inside a DST gap on this date; fall through to the longitude estimate.
                LocalResult::None => {}
            }
        }

        // Fallback: estimate UTC noon from longitude (4 min per degree).
        // Not valid near the anti-meridian where the date itself is ambiguous.
        if !Location::<T>::near_anti_meridian(location.longitude) {
            if let Some(utc_noon) = date.and_hms_micro_opt(12, 0, 0, 0) {
                let offset = TimeDelta::seconds((location.longitude * 4.0 * 60.0) as i64);
                if let Some(dt) = utc_noon.and_utc().checked_sub_signed(offset) {
                    return Ok(dt);
                }
            }
        }

        Err(ZmanimError::LocalNoonError)
    }

    /// Converts local mean time (LMT) hours for a date/location into UTC.
    ///
    /// `hours` is interpreted in the half-open range `[0.0, 24.0)`, where:
    /// - `0.0` is local mean midnight
    /// - `12.0` is local mean noon
    ///
    /// # Errors
    ///
    /// Returns:
    /// - [`ZmanimError::InvalidHours`] when `hours` is outside `[0.0, 24.0)`,
    /// - [`ZmanimError::TimeConversionError`] if midnight construction fails.
    pub(crate) fn local_mean_time(
        &mut self,
        date: NaiveDate,
        location: &Location<Tz>,
        hours: f64,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        if !(0.0..24.0).contains(&hours) {
            return Err(ZmanimError::InvalidHours);
        }

        if let Some(timezone) = &location.timezone {
            #[allow(clippy::unwrap_used)]
            let midnight = date
                .and_hms_opt(0, 0, 0)
                .ok_or(ZmanimError::TimeConversionError)?;

            let lmt_nanos = (hours * 3600.0 * 1_000_000_000.0).round() as i64;
            let lmt_dt = midnight + Duration::nanoseconds(lmt_nanos);

            let offset_nanos = (location.longitude * 240.0 * 1_000_000_000.0).round() as i64;

            let mut utc = Utc.from_utc_datetime(&(lmt_dt - Duration::nanoseconds(offset_nanos)));

            for _ in 0..4 {
                let local_date = utc.with_timezone(timezone).date_naive();
                let diff_days = (local_date - date).num_days();
                if diff_days == 0 {
                    break;
                }
                utc -= Duration::days(diff_days);
            }

            Ok(utc)
        } else {
            let lmt_seconds = (hours * 3600.0).round() as i64;
            #[allow(clippy::unwrap_used)]
            let lmt_dt = date
                .and_hms_opt(0, 0, 0)
                .ok_or(ZmanimError::TimeConversionError)?
                + Duration::seconds(lmt_seconds);
            let offset_seconds = (location.longitude * 240.0).round() as i64;
            Ok((lmt_dt - Duration::seconds(offset_seconds)).and_utc())
        }
    }
}

/// A value that can be calculated by a [`ZmanimCalculator`].
///
/// Most consumers should use predefined values from [`crate::presets`]. Implement this trait
/// when you need a custom zman definition not already provided there.
pub trait ZmanLike<Tz: TimeZone> {
    /// Computes this zman for the current calculator state.
    ///
    /// Implement this trait for custom zman definitions that can run through
    /// [`ZmanimCalculator::calculate`].
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError>;
}

#[cfg(feature = "defmt")]
impl<T: TimeZone> defmt::Format for ZmanimCalculator<T> {
    fn format(&self, fmt: defmt::Formatter) {
        use chrono::Datelike;
        let y = self.date.year();
        let m = self.date.month();
        let d = self.date.day();
        defmt::write!(
            fmt,
            "ZmanimCalculator {{ location: {}, date: {=i32}-{=u32}-{=u32}, config: {} }}",
            self.location,
            y,
            m,
            d,
            self.config
        )
    }
}
