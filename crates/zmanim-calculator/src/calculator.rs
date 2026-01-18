use astronomical_calculator::{AstronomicalCalculator, Refraction};
#[allow(unused_imports)]
use core_maths::*;

use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeDelta, TimeZone, Utc};

use crate::{
    math::multiply_duration,
    types::{config::CalculatorConfig, location::Location, zman::ZmanLike},
    ChatzosZman,
};

#[derive(Clone, Debug)]
pub struct ZmanimCalculator<Tz: TimeZone> {
    pub location: Location<Tz>,
    pub date: NaiveDate,
    pub config: CalculatorConfig,
    pub(crate) a_calc: AstronomicalCalculator,
    pub(crate) sl_calc: AstronomicalCalculator,
}

impl<Tz: TimeZone> ZmanimCalculator<Tz> {
    pub fn new(location: Location<Tz>, date: NaiveDate, config: CalculatorConfig) -> Option<Self> {
        let localnoon = Self::local_noon(date, &location)?;
        let elevation_calc = AstronomicalCalculator::new(
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
        let sea_level_calc = AstronomicalCalculator::new(
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
        Some(Self {
            location,
            date,
            config,
            a_calc: elevation_calc,
            sl_calc: sea_level_calc,
        })
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
    pub fn calculate(&mut self, zman: impl ZmanLike) -> Option<DateTime<Utc>> {
        zman.calculate(self)
    }
    pub(crate) fn transit(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.a_calc.get_solar_transit().ok()?, 0)
            .single()
    }

    pub(crate) fn sunrise(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.a_calc.get_sunrise().ok()?.timestamp()?, 0)
            .single()
    }
    pub(crate) fn sunset(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.a_calc.get_sunset().ok()?.timestamp()?, 0)
            .single()
    }
    pub(crate) fn sea_level_sunrise(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.sl_calc.get_sea_level_sunrise().ok()?.timestamp()?, 0)
            .single()
    }
    pub(crate) fn sea_level_sunset(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.sl_calc.get_sea_level_sunset().ok()?.timestamp()?, 0)
            .single()
    }
    pub(crate) fn sunrise_offset_by_degrees(
        &mut self,
        offset_zenith: f64,
    ) -> Option<DateTime<Utc>> {
        // If we are calculating a position above the horizon, we need to use the astronomical calculator
        // to offset from the real sunrise.
        Utc.timestamp_opt(
            self.a_calc
                .get_sunrise_offset_by_degrees(offset_zenith, offset_zenith > 0.0)
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    pub(crate) fn sunset_offset_by_degrees(&mut self, offset_zenith: f64) -> Option<DateTime<Utc>> {
        // If we are calculating a position above the horizon, we need to use the astronomical calculator
        // to offset from the real sunset.
        Utc.timestamp_opt(
            self.a_calc
                .get_sunset_offset_by_degrees(offset_zenith, offset_zenith > 0.0)
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    #[allow(unused)]
    pub(crate) fn temporal_hour(&mut self) -> Option<Duration> {
        let sea_level_sunrise = self.sea_level_sunrise()?;
        let sea_level_sunset = self.sea_level_sunset()?;
        self.get_temporal_hour_from_times(&sea_level_sunrise, &sea_level_sunset)
    }
    pub(crate) fn get_temporal_hour_from_times(
        &mut self,
        start_of_day: &DateTime<Utc>,
        end_of_day: &DateTime<Utc>,
    ) -> Option<Duration> {
        Some((*end_of_day - start_of_day) / 12)
    }

    pub(crate) fn get_sun_transit_from_times(
        &mut self,
        start_of_day: &DateTime<Utc>,
        end_of_day: &DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        let temporal_hour = self.get_temporal_hour_from_times(start_of_day, end_of_day)?;
        Some(*start_of_day + (temporal_hour * 6))
    }

    pub(crate) fn get_shaah_zmanis_gra(&mut self) -> Option<Duration> {
        let sunrise = self.sunrise()?;
        let sunset = self.sunset()?;
        self.get_temporal_hour_from_times(&sunrise, &sunset)
    }

    pub(crate) fn offset_by_shaah_zmanis_gra(
        &mut self,
        base: DateTime<Utc>,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
        let shaah_zmanis = self.get_shaah_zmanis_gra()?;
        Some(base + multiply_duration(shaah_zmanis, hours)?)
    }
    #[allow(unused)]
    pub(crate) fn get_shaah_zmanis_from_zmanim(
        &mut self,
        alos: impl ZmanLike,
        tzais: impl ZmanLike,
    ) -> Option<Duration> {
        let alos_time = alos.calculate(self)?;
        let tzais_time = tzais.calculate(self)?;
        self.get_temporal_hour_from_times(&alos_time, &tzais_time)
    }

    pub(crate) fn local_mean_time(
        &mut self,
        date: NaiveDate,
        location: &Location<Tz>,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
        if !(0.0..24.0).contains(&hours) {
            return None;
        }

        if let Some(timezone) = &location.timezone {
            #[allow(clippy::unwrap_used)]
            let midnight = date.and_hms_opt(0, 0, 0).unwrap();

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

            Some(utc)
        } else {
            let lmt_seconds = (hours * 3600.0).round() as i64;
            #[allow(clippy::unwrap_used)]
            let lmt_dt = date.and_hms_opt(0, 0, 0).unwrap() + Duration::seconds(lmt_seconds);
            let offset_seconds = (location.longitude * 240.0).round() as i64;
            Some((lmt_dt - Duration::seconds(offset_seconds)).and_utc())
        }
    }
    pub(crate) fn get_half_day_based_zman_from_times(
        &mut self,
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

    pub(crate) fn get_half_day_based_shaah_zmanis_from_times(
        &mut self,
        start_of_half_day: &DateTime<Utc>,
        end_of_half_day: &DateTime<Utc>,
    ) -> Option<Duration> {
        Some((*end_of_half_day - start_of_half_day) / 6)
    }

    pub(crate) fn get_shaah_zmanis_based_zman_from_times(
        &mut self,
        start_of_day: &DateTime<Utc>,
        end_of_day: &DateTime<Utc>,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
        let shaah_zmanis = self.get_temporal_hour_from_times(start_of_day, end_of_day)?;

        Some(*start_of_day + multiply_duration(shaah_zmanis, hours)?)
    }

    pub(crate) fn get_sof_zman_shma_from_times(
        &mut self,
        start_of_day: &DateTime<Utc>,
        end_of_day: Option<&DateTime<Utc>>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
            let chatzos = ChatzosZman::Astronomical
                .calculate(self)
                .or_else(|| self.transit())?;

            self.get_half_day_based_zman_from_times(start_of_day, &chatzos, 3.0)
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day, end_of_day?, 3.0)
        }
    }

    pub(crate) fn get_mincha_gedola_from_times(
        &mut self,

        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
            let chatzos = ChatzosZman::Astronomical.calculate(self)?;
            self.get_half_day_based_zman_from_times(&chatzos, end_of_day, 0.5)
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 6.5)
        }
    }

    pub(crate) fn get_shaah_zmanis_mga(&mut self) -> Option<Duration> {
        let sunrise = self.sunrise()?;
        let sunset = self.sunset()?;
        let alos72 = sunrise - Duration::minutes(72);
        let tzais72 = sunset + Duration::minutes(72);
        self.get_temporal_hour_from_times(&alos72, &tzais72)
    }
    pub(crate) fn get_samuch_le_mincha_ketana_from_times(
        &mut self,

        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
            let chatzos = ChatzosZman::Astronomical.calculate(self)?;

            self.get_half_day_based_zman_from_times(&chatzos, end_of_day, 3.0)
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 9.0)
        }
    }
    pub(crate) fn get_mincha_ketana_from_times(
        &mut self,

        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
            let chatzos = ChatzosZman::Astronomical.calculate(self)?;

            self.get_half_day_based_zman_from_times(&chatzos, end_of_day, 3.5)
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 9.5)
        }
    }
    pub(crate) fn get_sof_zman_tfila_from_times(
        &mut self,
        start_of_day: &DateTime<Utc>,
        end_of_day: Option<&DateTime<Utc>>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
            let chatzos = ChatzosZman::Astronomical.calculate(self)?;

            self.get_half_day_based_zman_from_times(start_of_day, &chatzos, 4.0)
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day, end_of_day?, 4.0)
        }
    }

    pub(crate) fn get_plag_hamincha_from_times(
        &mut self,
        start_of_day: Option<&DateTime<Utc>>,
        end_of_day: &DateTime<Utc>,
        synchronous: bool,
    ) -> Option<DateTime<Utc>> {
        if self.config.use_astronomical_chatzos_for_other_zmanim && synchronous {
            let chatzos = ChatzosZman::Astronomical.calculate(self)?;

            self.get_half_day_based_zman_from_times(&chatzos, end_of_day, 4.75)
        } else {
            self.get_shaah_zmanis_based_zman_from_times(start_of_day?, end_of_day, 10.75)
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ZmanimCalculator<Utc> {
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
#[cfg(all(feature = "tz", feature = "defmt"))]
impl defmt::Format for ZmanimCalculator<chrono_tz::Tz> {
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
