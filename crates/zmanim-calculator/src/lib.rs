#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod java_tests;
use astronomical_calculator::{AstronomicalCalculator, Refraction};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeDelta, TimeZone, Utc};
use core::time::Duration as StdDuration;
use time::Duration as TimeDuration;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location<T: TimeZone = Utc> {
    /// Latitude in degrees (positive North, negative South)
    latitude: f64,
    /// Longitude in degrees (positive East, negative West)
    longitude: f64,
    /// Elevation above sea level in meters
    elevation: f64,
    /// Timezone of the location.
    timezone: Option<T>,
}

impl<T: TimeZone> Location<T> {
    /// Creates a new Location.
    ///
    /// Returns `None` if the location is near the anti-meridian (±150° longitude)
    /// and no timezone is provided, since calculating local noon from UTC using
    /// longitude offset alone becomes unreliable near date boundaries.
    pub fn new(latitude: f64, longitude: f64, elevation: f64, timezone: Option<T>) -> Option<Self> {
        // Near the anti-meridian, require an explicit timezone
        const ANTI_MERIDIAN_THRESHOLD: f64 = 150.0;

        if timezone.is_none() && longitude.abs() > ANTI_MERIDIAN_THRESHOLD {
            return None;
        }

        Some(Self {
            latitude,
            longitude,
            elevation,
            timezone,
        })
    }

    /// Gets the latitude of the location.
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// Gets the longitude of the location.
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    /// Gets the elevation of the location.
    pub fn elevation(&self) -> f64 {
        self.elevation
    }
}

struct AstronomicalCalculators {
    sea_level: AstronomicalCalculator,
    elevation: AstronomicalCalculator,
}

pub struct ZmanimCalculator {
    /// Whether to use astronomical chatzos.
    use_astronomical_chatzos: bool,
    /// Candle lighting offset in seconds.
    candle_lighting_offset: Duration,
    /// Whether to use astronomical chatzos for other zmanim.
    use_astronomical_chatzos_for_other_zmanim: bool,
}

impl Default for ZmanimCalculator {
    fn default() -> Self {
        Self {
            use_astronomical_chatzos: true,
            candle_lighting_offset: Duration::minutes(18),
            use_astronomical_chatzos_for_other_zmanim: false,
        }
    }
}

impl ZmanimCalculator {
    pub fn new(
        use_astronomical_chatzos: bool,
        candle_lighting_offset: Duration,
        use_astronomical_chatzos_for_other_zmanim: bool,
    ) -> Self {
        Self {
            use_astronomical_chatzos,
            candle_lighting_offset,
            use_astronomical_chatzos_for_other_zmanim,
        }
    }
    pub fn calculate<T: TimeZone>(
        &self,
        date: NaiveDate,
        location: &Location<T>,
        zman: Zman,
    ) -> Option<DateTime<Utc>> {
        let localnoon = Self::local_noon(date, location)?;
        let mut calculator = Self::astronomical_calculator(localnoon, location)?;
        zman.calculate_with_calculator(&mut calculator, self)
    }
    pub fn calculate_many(
        &self,
        zmanim: VecZman,
        date: NaiveDate,
        location: &Location,
    ) -> VecZmanResult {
        let localnoon = Self::local_noon(date, location);
        if localnoon.is_none() {
            return VecZmanResult::new();
        }
        #[allow(clippy::unwrap_used)]
        let localnoon = localnoon.unwrap();
        let calculator = Self::astronomical_calculator(localnoon, location);
        let mut results: VecZmanResult = VecZmanResult::new();
        if let Some(mut calculator) = calculator {
            for zman in zmanim {
                let result = zman.calculate_with_calculator(&mut calculator, self);
                // Required to avoid lint errors when using Vec types
                #[allow(clippy::let_unit_value)]
                let _ = results.push((zman, result));
            }
        }
        results
    }
    fn transit(&self, calculator: &mut AstronomicalCalculators) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(calculator.sea_level.get_solar_transit().ok()?, 0)
            .single()
    }

    fn sunrise(&self, calculator: &mut AstronomicalCalculators) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(calculator.elevation.get_sunrise().ok()?.timestamp()?, 0)
            .single()
    }
    fn sunset(&self, calculator: &mut AstronomicalCalculators) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(calculator.elevation.get_sunset().ok()?.timestamp()?, 0)
            .single()
    }
    fn sea_level_sunrise(&self, calculator: &mut AstronomicalCalculators) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            calculator
                .sea_level
                .get_sea_level_sunrise()
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    fn sea_level_sunset(&self, calculator: &mut AstronomicalCalculators) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            calculator
                .sea_level
                .get_sea_level_sunset()
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    fn sunrise_offset_by_degrees(
        &self,
        calculator: &mut AstronomicalCalculators,
        offset_zenith: f64,
    ) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            calculator
                .sea_level
                .get_sunrise_offset_by_degrees(offset_zenith)
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    fn sunset_offset_by_degrees(
        &self,
        calculator: &mut AstronomicalCalculators,
        offset_zenith: f64,
    ) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            calculator
                .sea_level
                .get_sunset_offset_by_degrees(offset_zenith)
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    fn temporal_hour(&self, calculator: &mut AstronomicalCalculators) -> Option<Duration> {
        let sea_level_sunrise = self.sea_level_sunrise(calculator)?;
        let sea_level_sunset = self.sea_level_sunset(calculator)?;
        self.get_temporal_hour_from_times(&sea_level_sunrise, &sea_level_sunset)
    }
    fn get_temporal_hour_from_times(
        &self,
        start_of_day: &DateTime<Utc>,
        end_of_day: &DateTime<Utc>,
    ) -> Option<Duration> {
        Some((*end_of_day - start_of_day) / 12)
    }

    fn get_sun_transit_from_times(
        &self,
        start_of_day: &DateTime<Utc>,
        end_of_day: &DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        let temporal_hour = self.get_temporal_hour_from_times(start_of_day, end_of_day)?;
        Some(*start_of_day + (temporal_hour * 6))
    }
    fn get_percent_of_shaah_zmanis_from_degrees(
        &self,
        calculator: &mut AstronomicalCalculators,
        degrees: f64,
        sunset: bool,
    ) -> Option<f64> {
        let sea_level_sunrise = self.sea_level_sunrise(calculator);
        let sea_level_sunset = self.sea_level_sunset(calculator);

        let twilight = if sunset {
            self.sunset_offset_by_degrees(calculator, degrees)
        } else {
            self.sunrise_offset_by_degrees(calculator, degrees)
        };

        match (sea_level_sunrise, sea_level_sunset, twilight) {
            (Some(sunrise), Some(sunset_time), Some(twilight_time)) => {
                let shaah_zmanis =
                    (sunset_time.timestamp_millis() - sunrise.timestamp_millis()) as f64 / 12.0;
                let rise_set_to_twilight = if sunset {
                    twilight_time - sunset_time
                } else {
                    sunrise - twilight_time
                };
                let rise_set_to_twilight_millis = rise_set_to_twilight.num_milliseconds() as f64;
                Some(rise_set_to_twilight_millis / shaah_zmanis)
            }
            _ => None,
        }
    }
    fn get_shaah_zmanis_gra(&self, calculator: &mut AstronomicalCalculators) -> Option<Duration> {
        let sunrise = self.sunrise(calculator)?;
        let sunset = self.sea_level_sunset(calculator)?;
        self.get_temporal_hour_from_times(&sunrise, &sunset)
    }
    fn local_mean_time(date: NaiveDate, location: &Location, hours: f64) -> Option<DateTime<Utc>> {
        // let timezone = location.timezone?;
        if !(0.0..24.0).contains(&hours) {
            return None;
        }
        let offset = Duration::seconds((location.longitude * 4.0 * 60.0) as i64);
        let hours = Duration::seconds((hours * 3600.0) as i64);
        let middnight = Utc
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .single()?;
        Some(middnight + hours - offset)
    }
    fn get_half_day_based_zman_from_times(
        &self,
        start_of_half_day: &DateTime<Utc>,
        end_of_half_day: &DateTime<Utc>,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
        let shaah_zmanis =
            self.get_half_day_based_shaah_zmanis_from_times(start_of_half_day, end_of_half_day)?;
        if hours >= 0.0 {
            Some(*start_of_half_day + multiply_duration(shaah_zmanis, hours)?)
        } else {
            Some(*end_of_half_day + multiply_duration(shaah_zmanis, hours)?)
        }
    }

    fn get_half_day_based_shaah_zmanis_from_times(
        &self,
        start_of_half_day: &DateTime<Utc>,
        end_of_half_day: &DateTime<Utc>,
    ) -> Option<Duration> {
        Some((*end_of_half_day - start_of_half_day) / 6)
    }

    fn get_shaah_zmanis_based_zman_from_times(
        &self,
        start_of_day: &DateTime<Utc>,
        end_of_day: &DateTime<Utc>,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
        let shaah_zmanis = self.get_temporal_hour_from_times(start_of_day, end_of_day)?;

        Some(*start_of_day + multiply_duration(shaah_zmanis, hours)?)
    }

    fn get_sof_zman_shma_from_times(
        &self,
        calculator: &mut AstronomicalCalculators,
        start_of_day: &DateTime<Utc>,
        end_of_day: Option<&DateTime<Utc>>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.use_astronomical_chatzos_for_other_zmanim && synchronous {
            self.get_half_day_based_zman_from_times(
                start_of_day,
                &Zman::Chatzos
                    .calculate_with_calculator(calculator, self)
                    .or_else(|| self.transit(calculator))?,
                3.0,
            )
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day, end_of_day?, 3.0)
        }
    }

    fn get_mincha_gedola_from_times(
        &self,
        calculator: &mut AstronomicalCalculators,
        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.use_astronomical_chatzos_for_other_zmanim && synchronous {
            self.get_half_day_based_zman_from_times(
                &Zman::Chatzos.calculate_with_calculator(calculator, self)?,
                end_of_day,
                0.5,
            )
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 6.5)
        }
    }

    fn get_shaah_zmanis_mga(&self, calculator: &mut AstronomicalCalculators) -> Option<Duration> {
        let alos72 = Zman::Alos72.calculate_with_calculator(calculator, self)?;
        let tzais72 = Zman::Tzais72.calculate_with_calculator(calculator, self)?;
        self.get_temporal_hour_from_times(&alos72, &tzais72)
    }
    fn get_samuch_le_mincha_ketana_from_times(
        &self,
        calculator: &mut AstronomicalCalculators,
        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.use_astronomical_chatzos_for_other_zmanim && synchronous {
            self.get_half_day_based_zman_from_times(
                &Zman::Chatzos.calculate_with_calculator(calculator, self)?,
                end_of_day,
                3.0,
            )
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 9.0)
        }
    }
    fn get_mincha_ketana_from_times(
        &self,
        calculator: &mut AstronomicalCalculators,
        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.use_astronomical_chatzos_for_other_zmanim && synchronous {
            self.get_half_day_based_zman_from_times(
                &Zman::Chatzos.calculate_with_calculator(calculator, self)?,
                end_of_day,
                3.5,
            )
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 9.5)
        }
    }
    fn get_sof_zman_tfila_from_times(
        &self,
        calculator: &mut AstronomicalCalculators,
        start_of_day: &DateTime<Utc>,
        end_of_day: Option<&DateTime<Utc>>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.use_astronomical_chatzos_for_other_zmanim && synchronous {
            self.get_half_day_based_zman_from_times(
                start_of_day,
                &Zman::Chatzos.calculate_with_calculator(calculator, self)?,
                4.0,
            )
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day, end_of_day?, 4.0)
        }
    }

    fn get_plag_hamincha_from_times(
        &self,
        calculator: &mut AstronomicalCalculators,
        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.use_astronomical_chatzos_for_other_zmanim && synchronous {
            self.get_half_day_based_zman_from_times(
                &Zman::Chatzos.calculate_with_calculator(calculator, self)?,
                end_of_day,
                4.75,
            )
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 10.75)
        }
    }

    fn local_noon<T: TimeZone>(date: NaiveDate, location: &Location<T>) -> Option<DateTime<Utc>> {
        if let Some(timezone) = &location.timezone {
            timezone
                .with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0)
                .single()
                .map(|dt| dt.to_utc())
        } else {
            Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0)
                .single()?
                .checked_sub_signed(TimeDelta::seconds((location.longitude * 4.0 * 60.0) as i64))
        }
    }

    fn astronomical_calculator<T: TimeZone>(
        localnoon: DateTime<Utc>,
        location: &Location<T>,
    ) -> Option<AstronomicalCalculators> {
        let elevation_adjusted = AstronomicalCalculator::new(
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
        .ok()?;
        let sea_level_adjusted = AstronomicalCalculator::new(
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
        .ok()?;

        Some(AstronomicalCalculators {
            sea_level: sea_level_adjusted,
            elevation: elevation_adjusted,
        })
    }
}

#[cfg(not(feature = "std"))]
pub type VecZman = heapless::Vec<Zman, 32>;
#[cfg(feature = "std")]
pub type VecZman = Vec<Zman>;

#[cfg(not(feature = "std"))]
pub type VecZmanResult = heapless::Vec<(Zman, Option<DateTime<Utc>>), 32>;
#[cfg(feature = "std")]
pub type VecZmanResult = Vec<(Zman, Option<DateTime<Utc>>)>;

#[derive(Clone, Copy, Debug)]
pub enum Zman {
    Sunrise,
    Sunset,
    SeaLevelSunrise,
    SeaLevelSunset,
    Chatzos,
    ChatzosAsHalfDay,
    Alos72,
    AlosHashachar,
    Tzais72,
    Tzais,
    SofZmanShmaGRA,
    SofZmanShmaMGA,
    SofZmanTfilaGRA,
    SofZmanTfilaMGA,
    PlagHamincha,
    MinchaGedola,
    MinchaKetana,
    CandleLighting,
}
impl Zman {
    fn calculate_with_calculator(
        &self,
        calculator: &mut AstronomicalCalculators,
        zmanim_calculator: &ZmanimCalculator,
    ) -> Option<DateTime<Utc>> {
        match self {
            Zman::Sunrise => zmanim_calculator.sunrise(calculator),
            Zman::Sunset => zmanim_calculator.sunset(calculator),
            Zman::SeaLevelSunrise => zmanim_calculator.sea_level_sunrise(calculator),
            Zman::SeaLevelSunset => zmanim_calculator.sea_level_sunset(calculator),
            Zman::Chatzos => {
                if zmanim_calculator.use_astronomical_chatzos {
                    zmanim_calculator.transit(calculator)
                } else {
                    Zman::ChatzosAsHalfDay
                        .calculate_with_calculator(calculator, zmanim_calculator)
                        .or_else(|| zmanim_calculator.transit(calculator))
                }
            }
            Zman::ChatzosAsHalfDay => {
                let sea_level_sunrise = zmanim_calculator.sea_level_sunrise(calculator)?;
                let sea_level_sunset = zmanim_calculator.sea_level_sunset(calculator)?;
                zmanim_calculator.get_sun_transit_from_times(&sea_level_sunrise, &sea_level_sunset)
            }
            Zman::Alos72 => {
                let sunrise = zmanim_calculator.sunrise(calculator)?;
                Some(sunrise - Duration::minutes(72))
            }
            Zman::Tzais72 => {
                let sunset = zmanim_calculator.sunset(calculator)?;
                Some(sunset + Duration::minutes(72))
            }
            Zman::AlosHashachar => zmanim_calculator.sunrise_offset_by_degrees(calculator, 16.1),
            Zman::Tzais => zmanim_calculator.sunset_offset_by_degrees(calculator, 8.5),
            Zman::SofZmanShmaGRA => {
                let sunrise = zmanim_calculator.sunrise(calculator)?;
                let sunset = zmanim_calculator.sunset(calculator)?;
                zmanim_calculator.get_sof_zman_shma_from_times(
                    calculator,
                    &sunrise,
                    Some(&sunset),
                    true,
                )
            }
            Zman::SofZmanShmaMGA => {
                let alos72 =
                    Zman::Alos72.calculate_with_calculator(calculator, zmanim_calculator)?;
                let tzais72 =
                    Zman::Tzais72.calculate_with_calculator(calculator, zmanim_calculator)?;
                zmanim_calculator.get_sof_zman_shma_from_times(
                    calculator,
                    &alos72,
                    Some(&tzais72),
                    true,
                )
            }
            Zman::SofZmanTfilaGRA => {
                let sunrise = zmanim_calculator.sunrise(calculator)?;
                let sunset = zmanim_calculator.sunset(calculator)?;
                zmanim_calculator.get_sof_zman_tfila_from_times(
                    calculator,
                    &sunrise,
                    Some(&sunset),
                    true,
                )
            }
            Zman::SofZmanTfilaMGA => {
                let alos72 =
                    Zman::Alos72.calculate_with_calculator(calculator, zmanim_calculator)?;
                let tzais72 =
                    Zman::Tzais72.calculate_with_calculator(calculator, zmanim_calculator)?;
                zmanim_calculator.get_sof_zman_tfila_from_times(
                    calculator,
                    &alos72,
                    Some(&tzais72),
                    true,
                )
            }
            Zman::PlagHamincha => {
                let sunrise = zmanim_calculator.sunrise(calculator)?;
                let sunset = zmanim_calculator.sunset(calculator)?;
                zmanim_calculator.get_plag_hamincha_from_times(
                    calculator,
                    Some(&sunrise),
                    &sunset,
                    true,
                )
            }
            Zman::MinchaGedola => {
                let sunrise = zmanim_calculator.sunrise(calculator)?;
                let sunset = zmanim_calculator.sunset(calculator)?;
                zmanim_calculator.get_mincha_gedola_from_times(
                    calculator,
                    Some(&sunrise),
                    &sunset,
                    true,
                )
            }
            Zman::MinchaKetana => {
                let sunrise = zmanim_calculator.sunrise(calculator)?;
                let sunset = zmanim_calculator.sunset(calculator)?;
                zmanim_calculator.get_mincha_ketana_from_times(
                    calculator,
                    Some(&sunrise),
                    &sunset,
                    true,
                )
            }
            Zman::CandleLighting => {
                let sunset = zmanim_calculator.sea_level_sunset(calculator)?;
                Some(sunset - zmanim_calculator.candle_lighting_offset)
            }
        }
    }
}

/// A helper function to multiply a duration by a factor.
/// This uses a clever workaround to handle negative durations which std duration does not support.
fn multiply_duration(core_timedelta: TimeDelta, factor: f64) -> Option<TimeDelta> {
    let is_timedelta_negative = core_timedelta < TimeDelta::zero();
    let factor_is_negative = factor < 0.0;
    let std_duration = core_timedelta.abs().to_std().ok()?;
    let time_duration: TimeDuration = std_duration.try_into().ok()?;
    let std_duration: StdDuration = (time_duration * factor.abs()).try_into().ok()?;
    let core_timedelta = TimeDelta::from_std(std_duration).ok()?;

    if (is_timedelta_negative && !factor_is_negative)
        || (!is_timedelta_negative && factor_is_negative)
    {
        core_timedelta.checked_mul(-1)
    } else {
        Some(core_timedelta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anti_meridian_without_timezone() {
        // Should fail near anti-meridian without timezone
        assert!(Location::new(0.0, 170.0, 0.0, None::<Utc>).is_none());
        assert!(Location::new(0.0, -170.0, 0.0, None::<Utc>).is_none());
        assert!(Location::new(0.0, 180.0, 0.0, None::<Utc>).is_none());

        // Should succeed with explicit timezone
        assert!(Location::new(0.0, 170.0, 0.0, Some(Utc)).is_some());

        // Should succeed far from anti-meridian without timezone
        assert!(Location::new(0.0, 150.0, 0.0, None::<Utc>).is_some());
    }

    #[test]
    fn test_multiply_duration_positive_duration_positive_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::seconds(200)));
    }

    #[test]
    fn test_multiply_duration_positive_duration_negative_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, -2.0);
        assert_eq!(result, Some(TimeDelta::seconds(-200)));
    }

    #[test]
    fn test_multiply_duration_negative_duration_positive_factor() {
        let duration = TimeDelta::seconds(-100);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::seconds(-200)));
    }

    #[test]
    fn test_multiply_duration_negative_duration_negative_factor() {
        let duration = TimeDelta::seconds(-100);
        let result = multiply_duration(duration, -2.0);
        assert_eq!(result, Some(TimeDelta::seconds(200)));
    }

    #[test]
    fn test_multiply_duration_zero_duration() {
        let duration = TimeDelta::zero();
        let result = multiply_duration(duration, 5.0);
        assert_eq!(result, Some(TimeDelta::zero()));

        let result_negative = multiply_duration(duration, -5.0);
        assert_eq!(result_negative, Some(TimeDelta::zero()));
    }

    #[test]
    fn test_multiply_duration_zero_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 0.0);
        assert_eq!(result, Some(TimeDelta::zero()));

        let negative_duration = TimeDelta::seconds(-100);
        let result_negative = multiply_duration(negative_duration, 0.0);
        assert_eq!(result_negative, Some(TimeDelta::zero()));
    }

    #[test]
    fn test_multiply_duration_identity_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 1.0);
        assert_eq!(result, Some(duration));

        let negative_duration = TimeDelta::seconds(-100);
        let result_negative = multiply_duration(negative_duration, 1.0);
        assert_eq!(result_negative, Some(negative_duration));
    }

    #[test]
    fn test_multiply_duration_negation_factor() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, -1.0);
        assert_eq!(result, Some(TimeDelta::seconds(-100)));

        let negative_duration = TimeDelta::seconds(-100);
        let result_negative = multiply_duration(negative_duration, -1.0);
        assert_eq!(result_negative, Some(TimeDelta::seconds(100)));
    }

    #[test]
    fn test_multiply_duration_fractional_factors() {
        let duration = TimeDelta::seconds(100);
        let result = multiply_duration(duration, 0.5);
        assert_eq!(result, Some(TimeDelta::seconds(50)));

        let result = multiply_duration(duration, 1.5);
        assert_eq!(result, Some(TimeDelta::seconds(150)));

        let result = multiply_duration(duration, -0.5);
        assert_eq!(result, Some(TimeDelta::seconds(-50)));
    }

    #[test]
    fn test_multiply_duration_millisecond_precision() {
        let duration = TimeDelta::milliseconds(123);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::milliseconds(246)));

        let result = multiply_duration(duration, -3.0);
        assert_eq!(result, Some(TimeDelta::milliseconds(-369)));
    }

    #[test]
    fn test_multiply_duration_hours() {
        let duration = TimeDelta::hours(1);
        let result = multiply_duration(duration, 3.0);
        assert_eq!(result, Some(TimeDelta::hours(3)));

        let result = multiply_duration(duration, 0.5);
        assert_eq!(result, Some(TimeDelta::minutes(30)));
    }

    #[test]
    fn test_multiply_duration_minutes() {
        let duration = TimeDelta::minutes(72);
        let result = multiply_duration(duration, 2.0);
        assert_eq!(result, Some(TimeDelta::minutes(144)));

        let result = multiply_duration(duration, -1.0);
        assert_eq!(result, Some(TimeDelta::minutes(-72)));
    }

    #[test]
    fn test_multiply_duration_days() {
        let duration = TimeDelta::days(1);
        let result = multiply_duration(duration, 7.0);
        assert_eq!(result, Some(TimeDelta::days(7)));

        let result = multiply_duration(duration, -0.5);
        assert_eq!(result, Some(TimeDelta::hours(-12)));
    }
}
