use astronomical_calculator::{AstronomicalCalculator, Refraction};
use chrono::offset::Offset;
use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, TimeDelta, TimeZone, Utc};

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
    pub(crate) sea_level_calc: AstronomicalCalculator,
    pub(crate) sea_level_no_refraction_calc: AstronomicalCalculator,
    pub(crate) elevation_calc: AstronomicalCalculator,
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
        let sea_level_no_refraction_calc = AstronomicalCalculator::new(
            localnoon,
            None,
            0.0,
            location.longitude,
            location.latitude,
            0.0,
            22.0,
            1013.25,
            None,
            Refraction::NoRefraction,
        )
        .ok()?;
        Some(Self {
            location,
            date,
            config,
            elevation_calc,
            sea_level_calc,
            sea_level_no_refraction_calc,
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
        Utc.timestamp_opt(self.sea_level_calc.get_solar_transit().ok()?, 0)
            .single()
    }

    pub(crate) fn sunrise(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.elevation_calc.get_sunrise().ok()?.timestamp()?, 0)
            .single()
    }
    pub(crate) fn sunset(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(self.elevation_calc.get_sunset().ok()?.timestamp()?, 0)
            .single()
    }
    pub(crate) fn sea_level_sunrise(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            self.sea_level_calc
                .get_sea_level_sunrise()
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    pub(crate) fn sea_level_sunset(&mut self) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            self.sea_level_calc
                .get_sea_level_sunset()
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    pub(crate) fn sunrise_offset_by_degrees(
        &mut self,
        offset_zenith: f64,
    ) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            self.sea_level_no_refraction_calc
                .get_sunrise_offset_by_degrees(offset_zenith)
                .ok()?
                .timestamp()?,
            0,
        )
        .single()
    }
    pub(crate) fn sunset_offset_by_degrees(&mut self, offset_zenith: f64) -> Option<DateTime<Utc>> {
        Utc.timestamp_opt(
            self.sea_level_no_refraction_calc
                .get_sunset_offset_by_degrees(offset_zenith)
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
    #[allow(unused)]
    pub(crate) fn get_percent_of_shaah_zmanis_from_degrees(
        &mut self,
        degrees: f64,
        sunset: bool,
    ) -> Option<f64> {
        let sea_level_sunrise = self.sea_level_sunrise();
        let sea_level_sunset = self.sea_level_sunset();

        let twilight = if sunset {
            self.sunset_offset_by_degrees(degrees)
        } else {
            self.sunrise_offset_by_degrees(degrees)
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
    #[allow(unused)]
    pub(crate) fn local_mean_time(
        &mut self,
        date: NaiveDate,
        location: &Location<Tz>,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
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
            let mut chatzos = ChatzosZman::Astronomical.calculate(self);
            if chatzos.is_none() {
                chatzos = self.transit();
            }
            let chatzos = chatzos?;

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
