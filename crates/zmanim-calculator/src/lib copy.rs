#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod java_tests;
use astronomical_calculator::{AstronomicalCalculator, Refraction};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeDelta, TimeZone, Utc};
use core::time::Duration as StdDuration;
use time::Duration as TimeDuration;

// Zenith constants for complex zmanim calculations
// GEOMETRIC_ZENITH = 90 degrees
const GEOMETRIC_ZENITH: f64 = 90.0;
const ASTRONOMICAL_ZENITH: f64 = 108.0;

// Complex zmanim zenith constants
const ZENITH_3_POINT_7: f64 = GEOMETRIC_ZENITH + 3.7;
const ZENITH_3_POINT_8: f64 = GEOMETRIC_ZENITH + 3.8;
const ZENITH_4_POINT_37: f64 = GEOMETRIC_ZENITH + 4.37;
const ZENITH_4_POINT_61: f64 = GEOMETRIC_ZENITH + 4.61;
const ZENITH_4_POINT_8: f64 = GEOMETRIC_ZENITH + 4.8;
const ZENITH_5_POINT_88: f64 = GEOMETRIC_ZENITH + 5.88;
const ZENITH_5_POINT_95: f64 = GEOMETRIC_ZENITH + 5.95;
const ZENITH_6_DEGREES: f64 = GEOMETRIC_ZENITH + 6.0;
const ZENITH_6_POINT_45: f64 = GEOMETRIC_ZENITH + 6.45;
const ZENITH_7_POINT_65: f64 = GEOMETRIC_ZENITH + 7.65;
const ZENITH_7_POINT_67: f64 = GEOMETRIC_ZENITH + 7.67;
const ZENITH_7_POINT_083: f64 = GEOMETRIC_ZENITH + 7.0 + (5.0 / 60.0);
const ZENITH_9_POINT_3: f64 = GEOMETRIC_ZENITH + 9.3;
const ZENITH_9_POINT_5: f64 = GEOMETRIC_ZENITH + 9.5;
const ZENITH_9_POINT_75: f64 = GEOMETRIC_ZENITH + 9.75;
const ZENITH_10_POINT_2: f64 = GEOMETRIC_ZENITH + 10.2;
const ZENITH_11_DEGREES: f64 = GEOMETRIC_ZENITH + 11.0;
const ZENITH_11_POINT_5: f64 = GEOMETRIC_ZENITH + 11.5;
const ZENITH_13_POINT_24: f64 = GEOMETRIC_ZENITH + 13.24;
const ZENITH_16_POINT_1: f64 = GEOMETRIC_ZENITH + 16.1;
const ZENITH_16_POINT_9: f64 = GEOMETRIC_ZENITH + 16.9;
const ZENITH_19_DEGREES: f64 = GEOMETRIC_ZENITH + 19.0;
const ZENITH_19_POINT_8: f64 = GEOMETRIC_ZENITH + 19.8;
#[allow(dead_code)]
const ZENITH_26_DEGREES: f64 = GEOMETRIC_ZENITH + 26.0; // Used for deprecated methods
const ZENITH_1_POINT_583: f64 = GEOMETRIC_ZENITH + 1.583;
const ZENITH_MINUS_2_POINT_1: f64 = GEOMETRIC_ZENITH - 2.1;
const ZENITH_MINUS_2_POINT_8: f64 = GEOMETRIC_ZENITH - 2.8;
const ZENITH_MINUS_3_POINT_05: f64 = GEOMETRIC_ZENITH - 3.05;
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
    /// Ateret Torah sunset offset in minutes (default 40).
    ateret_torah_sunset_offset: Duration,
}

impl Default for ZmanimCalculator {
    fn default() -> Self {
        Self {
            use_astronomical_chatzos: true,
            candle_lighting_offset: Duration::minutes(18),
            use_astronomical_chatzos_for_other_zmanim: false,
            ateret_torah_sunset_offset: Duration::minutes(40),
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
            ateret_torah_sunset_offset: Duration::minutes(40),
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

    fn get_zmanis_based_offset(
        &self,
        calculator: &mut AstronomicalCalculators,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
        let shaah_zmanis = self.get_shaah_zmanis_gra(calculator)?;
        if hours == 0.0 {
            return None;
        }

        if hours > 0.0 {
            let sunset = self.sunset(calculator)?;
            Some(sunset + multiply_duration(shaah_zmanis, hours)?)
        } else {
            let sunrise = self.sunrise(calculator)?;
            Some(sunrise + multiply_duration(shaah_zmanis, hours)?)
        }
    }

    fn get_shaah_zmanis_from_zmanim(
        &self,
        calculator: &mut AstronomicalCalculators,
        alos: Zman,
        tzais: Zman,
    ) -> Option<Duration> {
        let alos_time = alos.calculate_with_calculator(calculator, self)?;
        let tzais_time = tzais.calculate_with_calculator(calculator, self)?;
        self.get_temporal_hour_from_times(&alos_time, &tzais_time)
    }
    fn local_mean_time(
        &self,
        date: NaiveDate,
        location: &Location,
        hours: f64,
    ) -> Option<DateTime<Utc>> {
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
    // Alos variants
    Alos60,
    Alos72Zmanis,
    Alos90,
    Alos90Zmanis,
    Alos96,
    Alos96Zmanis,
    Alos16Point1Degrees,
    Alos18Degrees,
    Alos19Degrees,
    Alos19Point8Degrees,
    // Misheyakir variants
    Misheyakir7Point65Degrees,
    Misheyakir9Point5Degrees,
    Misheyakir10Point2Degrees,
    Misheyakir11Degrees,
    Misheyakir11Point5Degrees,
    // Sof Zman Shma variants
    SofZmanShmaMGA16Point1Degrees,
    SofZmanShmaMGA18Degrees,
    SofZmanShmaMGA19Point8Degrees,
    SofZmanShmaMGA72MinutesZmanis,
    SofZmanShmaMGA90Minutes,
    SofZmanShmaMGA90MinutesZmanis,
    SofZmanShmaMGA96Minutes,
    SofZmanShmaMGA96MinutesZmanis,
    SofZmanShmaMGA120Minutes,
    SofZmanShma3HoursBeforeChatzos,
    SofZmanShmaAlos16Point1ToSunset,
    SofZmanShmaAlos16Point1ToTzaisGeonim7Point083Degrees,
    SofZmanShmaAteretTorah,
    // Sof Zman Tfila variants
    SofZmanTfilaMGA16Point1Degrees,
    SofZmanTfilaMGA18Degrees,
    SofZmanTfilaMGA19Point8Degrees,
    SofZmanTfilaMGA72MinutesZmanis,
    SofZmanTfilaMGA90Minutes,
    SofZmanTfilaMGA90MinutesZmanis,
    SofZmanTfilaMGA96Minutes,
    SofZmanTfilaMGA96MinutesZmanis,
    SofZmanTfilaMGA120Minutes,
    SofZmanTfila2HoursBeforeChatzos,
    SofZmanTfilaAteretTorah,
    // Mincha Gedola variants
    MinchaGedola30Minutes,
    MinchaGedola72Minutes,
    MinchaGedola16Point1Degrees,
    MinchaGedolaAhavatShalom,
    MinchaGedolaGreaterThan30,
    MinchaGedolaAteretTorah,
    // Mincha Ketana variants
    MinchaKetana16Point1Degrees,
    MinchaKetana72Minutes,
    MinchaKetanaAhavatShalom,
    MinchaKetanaAteretTorah,
    // Plag Hamincha variants
    PlagHamincha60Minutes,
    PlagAlos16Point1ToTzaisGeonim7Point083Degrees,
    PlagAhavatShalom,
    PlagHaminchaAteretTorah,
    // Bain Hashmashos variants
    BainHashmashosRT13Point24Degrees,
    BainHashmashosRT58Point5Minutes,
    BainHashmashosRT13Point5MinutesBefore7Point083Degrees,
    BainHashmashosRT2Stars,
    BainHashmashosYereim18Minutes,
    BainHashmashosYereim3Point05Degrees,
    BainHashmashosYereim16Point875Minutes,
    BainHashmashosYereim2Point8Degrees,
    BainHashmashosYereim13Point5Minutes,
    BainHashmashosYereim2Point1Degrees,
    // Tzais variants
    Tzais60,
    Tzais90,
    Tzais96,
    Tzais72Zmanis,
    Tzais90Zmanis,
    Tzais96Zmanis,
    Tzais16Point1Degrees,
    Tzais18Degrees,
    Tzais19Point8Degrees,
    TzaisGeonim3Point7Degrees,
    TzaisGeonim3Point8Degrees,
    TzaisGeonim4Point37Degrees,
    TzaisGeonim4Point61Degrees,
    TzaisGeonim4Point8Degrees,
    TzaisGeonim5Point88Degrees,
    TzaisGeonim5Point95Degrees,
    TzaisGeonim6Point45Degrees,
    TzaisGeonim7Point083Degrees,
    TzaisGeonim7Point67Degrees,
    TzaisGeonim8Point5Degrees,
    TzaisGeonim9Point3Degrees,
    TzaisGeonim9Point75Degrees,
    TzaisAteretTorah,
    // Fixed Local Chatzos variants
    FixedLocalChatzos,
    SofZmanShmaMGA18DegreesToFixedLocalChatzos,
    SofZmanShmaMGA16Point1DegreesToFixedLocalChatzos,
    SofZmanShmaMGA90MinutesToFixedLocalChatzos,
    SofZmanShmaMGA72MinutesToFixedLocalChatzos,
    SofZmanShmaGRASunriseToFixedLocalChatzos,
    SofZmanTfilaGRASunriseToFixedLocalChatzos,
    MinchaGedolaGRAFixedLocalChatzos30Minutes,
    MinchaKetanaGRAFixedLocalChatzosToSunset,
    PlagHaminchaGRAFixedLocalChatzosToSunset,
    // Chametz variants
    SofZmanAchilasChametzGRA,
    SofZmanAchilasChametzMGA72Minutes,
    SofZmanAchilasChametzMGA16Point1Degrees,
    SofZmanBiurChametzGRA,
    SofZmanBiurChametzMGA72Minutes,
    SofZmanBiurChametzMGA16Point1Degrees,
    // Baal HaTanya variants
    AlosBaalHatanya,
    SofZmanShmaBaalHatanya,
    SofZmanTfilaBaalHatanya,
    SofZmanAchilasChametzBaalHatanya,
    SofZmanBiurChametzBaalHatanya,
    MinchaGedolaBaalHatanya,
    MinchaGedolaBaalHatanyaGreaterThan30,
    MinchaKetanaBaalHatanya,
    PlagHaminchaBaalHatanya,
    TzaisBaalHatanya,
    // Samuch LeMinchaKetana variants
    SamuchLeMinchaKetanaGRA,
    SamuchLeMinchaKetana16Point1Degrees,
    SamuchLeMinchaKetana72Minutes,
    // Other
    Tzais50,
}
impl Zman {
    fn calculate_with_calculator(
        &self,
        a_calc: &mut AstronomicalCalculators,
        z_calc: &ZmanimCalculator,
    ) -> Option<DateTime<Utc>> {
        match self {
            Zman::Sunrise => z_calc.sunrise(a_calc),
            Zman::Sunset => z_calc.sunset(a_calc),
            Zman::SeaLevelSunrise => z_calc.sea_level_sunrise(a_calc),
            Zman::SeaLevelSunset => z_calc.sea_level_sunset(a_calc),
            Zman::Chatzos => {
                if z_calc.use_astronomical_chatzos {
                    z_calc.transit(a_calc)
                } else {
                    Zman::ChatzosAsHalfDay
                        .calculate_with_calculator(a_calc, z_calc)
                        .or_else(|| z_calc.transit(a_calc))
                }
            }
            Zman::ChatzosAsHalfDay => {
                let sea_level_sunrise = z_calc.sea_level_sunrise(a_calc)?;
                let sea_level_sunset = z_calc.sea_level_sunset(a_calc)?;
                z_calc.get_sun_transit_from_times(&sea_level_sunrise, &sea_level_sunset)
            }
            Zman::Alos72 => {
                let sunrise = z_calc.sunrise(a_calc)?;
                Some(sunrise - Duration::minutes(72))
            }
            Zman::Tzais72 => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset + Duration::minutes(72))
            }
            Zman::AlosHashachar => z_calc.sunrise_offset_by_degrees(a_calc, 16.1),
            Zman::Tzais => z_calc.sunset_offset_by_degrees(a_calc, 8.5),
            Zman::SofZmanShmaGRA => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &sunrise, Some(&sunset), true)
            }
            Zman::SofZmanShmaMGA => {
                let alos72 = Zman::Alos72.calculate_with_calculator(a_calc, z_calc)?;
                let tzais72 = Zman::Tzais72.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos72, Some(&tzais72), true)
            }
            Zman::SofZmanTfilaGRA => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &sunrise, Some(&sunset), true)
            }
            Zman::SofZmanTfilaMGA => {
                let alos72 = Zman::Alos72.calculate_with_calculator(a_calc, z_calc)?;
                let tzais72 = Zman::Tzais72.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos72, Some(&tzais72), true)
            }
            Zman::PlagHamincha => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                z_calc.get_plag_hamincha_from_times(a_calc, Some(&sunrise), &sunset, true)
            }
            Zman::MinchaGedola => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                z_calc.get_mincha_gedola_from_times(a_calc, Some(&sunrise), &sunset, true)
            }
            Zman::MinchaKetana => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                z_calc.get_mincha_ketana_from_times(a_calc, Some(&sunrise), &sunset, true)
            }
            Zman::CandleLighting => {
                let sunset = z_calc.sea_level_sunset(a_calc)?;
                Some(sunset - z_calc.candle_lighting_offset)
            }
            // Alos variants
            Zman::Alos60 => {
                let sunrise = z_calc.sunrise(a_calc)?;
                Some(sunrise - Duration::minutes(60))
            }
            Zman::Alos72Zmanis => z_calc.get_zmanis_based_offset(a_calc, -1.2),
            Zman::Alos90 => {
                let sunrise = z_calc.sunrise(a_calc)?;
                Some(sunrise - Duration::minutes(90))
            }
            Zman::Alos90Zmanis => z_calc.get_zmanis_based_offset(a_calc, -1.5),
            Zman::Alos96 => {
                let sunrise = z_calc.sunrise(a_calc)?;
                Some(sunrise - Duration::minutes(96))
            }
            Zman::Alos96Zmanis => z_calc.get_zmanis_based_offset(a_calc, -1.6),
            Zman::Alos16Point1Degrees => {
                z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_16_POINT_1)
            }
            Zman::Alos18Degrees => z_calc.sunrise_offset_by_degrees(a_calc, ASTRONOMICAL_ZENITH),
            Zman::Alos19Degrees => z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_19_DEGREES),
            Zman::Alos19Point8Degrees => {
                z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_19_POINT_8)
            }
            // Misheyakir variants
            Zman::Misheyakir7Point65Degrees => {
                z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_7_POINT_65)
            }
            Zman::Misheyakir9Point5Degrees => {
                z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_9_POINT_5)
            }
            Zman::Misheyakir10Point2Degrees => {
                z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_10_POINT_2)
            }
            Zman::Misheyakir11Degrees => {
                z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_11_DEGREES)
            }
            Zman::Misheyakir11Point5Degrees => {
                z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_11_POINT_5)
            }
            // Tzais variants
            Zman::Tzais60 => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset + Duration::minutes(60))
            }
            Zman::Tzais90 => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset + Duration::minutes(90))
            }
            Zman::Tzais96 => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset + Duration::minutes(96))
            }
            Zman::Tzais72Zmanis => z_calc.get_zmanis_based_offset(a_calc, 1.2),
            Zman::Tzais90Zmanis => z_calc.get_zmanis_based_offset(a_calc, 1.5),
            Zman::Tzais96Zmanis => z_calc.get_zmanis_based_offset(a_calc, 1.6),
            Zman::Tzais16Point1Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_16_POINT_1)
            }
            Zman::Tzais18Degrees => z_calc.sunset_offset_by_degrees(a_calc, ASTRONOMICAL_ZENITH),
            Zman::Tzais19Point8Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_19_POINT_8)
            }
            Zman::TzaisGeonim3Point7Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_3_POINT_7)
            }
            Zman::TzaisGeonim3Point8Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_3_POINT_8)
            }
            Zman::TzaisGeonim4Point37Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_4_POINT_37)
            }
            Zman::TzaisGeonim4Point61Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_4_POINT_61)
            }
            Zman::TzaisGeonim4Point8Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_4_POINT_8)
            }
            Zman::TzaisGeonim5Point88Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_5_POINT_88)
            }
            Zman::TzaisGeonim5Point95Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_5_POINT_95)
            }
            Zman::TzaisGeonim6Point45Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_6_POINT_45)
            }
            Zman::TzaisGeonim7Point083Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_7_POINT_083)
            }
            Zman::TzaisGeonim7Point67Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_7_POINT_67)
            }
            Zman::TzaisGeonim8Point5Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, GEOMETRIC_ZENITH + 8.5)
            }
            Zman::TzaisGeonim9Point3Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_9_POINT_3)
            }
            Zman::TzaisGeonim9Point75Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_9_POINT_75)
            }
            Zman::TzaisAteretTorah => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset + z_calc.ateret_torah_sunset_offset)
            }
            Zman::Tzais50 => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset + Duration::minutes(50))
            }
            // Fixed Local Chatzos - Note: This needs location which isn't available here
            // Will need to be handled specially or location needs to be stored in calculator
            Zman::FixedLocalChatzos => {
                // For now, return None - this needs location access
                // TODO: Store location in AstronomicalCalculators or pass it differently
                None
            }
            // Sof Zman Shma variants
            Zman::SofZmanShmaMGA16Point1Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA18Degrees => {
                let alos = Zman::Alos18Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais18Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA19Point8Degrees => {
                let alos = Zman::Alos19Point8Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais19Point8Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA72MinutesZmanis => {
                let alos = Zman::Alos72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA90Minutes => {
                let alos = Zman::Alos90.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais90.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA90MinutesZmanis => {
                let alos = Zman::Alos90Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais90Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA96Minutes => {
                let alos = Zman::Alos96.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais96.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA96MinutesZmanis => {
                let alos = Zman::Alos96Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais96Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanShmaMGA120Minutes => {
                // Note: Alos120 and Tzais120 are deprecated, but we need them for this calculation
                // For now, we'll calculate them inline
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                let alos120 = sunrise - Duration::minutes(120);
                let tzais120 = sunset + Duration::minutes(120);
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos120, Some(&tzais120), true)
            }
            Zman::SofZmanShma3HoursBeforeChatzos => {
                let chatzos = Zman::Chatzos.calculate_with_calculator(a_calc, z_calc)?;
                Some(chatzos - Duration::minutes(180))
            }
            Zman::SofZmanShmaAlos16Point1ToSunset => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&sunset), false)
            }
            Zman::SofZmanShmaAlos16Point1ToTzaisGeonim7Point083Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais =
                    Zman::TzaisGeonim7Point083Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), false)
            }
            Zman::SofZmanShmaAteretTorah => {
                let alos = Zman::Alos72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::TzaisAteretTorah.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &alos, Some(&tzais), false)
            }
            // Sof Zman Tfila variants
            Zman::SofZmanTfilaMGA16Point1Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA18Degrees => {
                let alos = Zman::Alos18Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais18Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA19Point8Degrees => {
                let alos = Zman::Alos19Point8Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais19Point8Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA72MinutesZmanis => {
                let alos = Zman::Alos72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA90Minutes => {
                let alos = Zman::Alos90.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais90.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA90MinutesZmanis => {
                let alos = Zman::Alos90Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais90Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA96Minutes => {
                let alos = Zman::Alos96.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais96.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA96MinutesZmanis => {
                let alos = Zman::Alos96Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais96Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), true)
            }
            Zman::SofZmanTfilaMGA120Minutes => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                let alos120 = sunrise - Duration::minutes(120);
                let tzais120 = sunset + Duration::minutes(120);
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos120, Some(&tzais120), true)
            }
            Zman::SofZmanTfila2HoursBeforeChatzos => {
                let chatzos = Zman::Chatzos.calculate_with_calculator(a_calc, z_calc)?;
                Some(chatzos - Duration::minutes(120))
            }
            Zman::SofZmanTfilaAteretTorah => {
                let alos = Zman::Alos72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::TzaisAteretTorah.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &alos, Some(&tzais), false)
            }
            // Mincha Gedola variants
            Zman::MinchaGedola30Minutes => {
                let chatzos = Zman::Chatzos.calculate_with_calculator(a_calc, z_calc)?;
                Some(chatzos + Duration::minutes(30))
            }
            Zman::MinchaGedola72Minutes => {
                let alos = Zman::Alos72.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais72.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_mincha_gedola_from_times(a_calc, Some(&alos), &tzais, true)
            }
            Zman::MinchaGedola16Point1Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_mincha_gedola_from_times(a_calc, Some(&alos), &tzais, true)
            }
            Zman::MinchaGedolaAhavatShalom => {
                let chatzos = Zman::Chatzos.calculate_with_calculator(a_calc, z_calc)?;
                let mincha_gedola_30 =
                    Zman::MinchaGedola30Minutes.calculate_with_calculator(a_calc, z_calc)?;
                let shaah_zmanis = z_calc.get_shaah_zmanis_from_zmanim(
                    a_calc,
                    Zman::Alos16Point1Degrees,
                    Zman::TzaisGeonim3Point7Degrees,
                )?;
                let alternative = chatzos + multiply_duration(shaah_zmanis, 0.5)?;
                Some(if mincha_gedola_30 > alternative {
                    mincha_gedola_30
                } else {
                    alternative
                })
            }
            Zman::MinchaGedolaGreaterThan30 => {
                let mincha_gedola_30 =
                    Zman::MinchaGedola30Minutes.calculate_with_calculator(a_calc, z_calc)?;
                let mincha_gedola = Zman::MinchaGedola.calculate_with_calculator(a_calc, z_calc)?;
                Some(if mincha_gedola_30 > mincha_gedola {
                    mincha_gedola_30
                } else {
                    mincha_gedola
                })
            }
            Zman::MinchaGedolaAteretTorah => {
                let alos = Zman::Alos72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::TzaisAteretTorah.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_mincha_gedola_from_times(a_calc, Some(&alos), &tzais, false)
            }
            // Mincha Ketana variants
            Zman::MinchaKetana16Point1Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_mincha_ketana_from_times(a_calc, Some(&alos), &tzais, true)
            }
            Zman::MinchaKetana72Minutes => {
                let alos = Zman::Alos72.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais72.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_mincha_ketana_from_times(a_calc, Some(&alos), &tzais, true)
            }
            Zman::MinchaKetanaAhavatShalom => {
                let tzais_geonim_3_8 =
                    Zman::TzaisGeonim3Point8Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let shaah_zmanis = z_calc.get_shaah_zmanis_from_zmanim(
                    a_calc,
                    Zman::Alos16Point1Degrees,
                    Zman::TzaisGeonim3Point8Degrees,
                )?;
                Some(tzais_geonim_3_8 - multiply_duration(shaah_zmanis, 2.5)?)
            }
            Zman::MinchaKetanaAteretTorah => {
                let alos = Zman::Alos72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::TzaisAteretTorah.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_mincha_ketana_from_times(a_calc, Some(&alos), &tzais, false)
            }
            // Plag Hamincha variants
            Zman::PlagHamincha60Minutes => {
                let alos = Zman::Alos60.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais60.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_plag_hamincha_from_times(a_calc, Some(&alos), &tzais, true)
            }
            Zman::PlagAlos16Point1ToTzaisGeonim7Point083Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais =
                    Zman::TzaisGeonim7Point083Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_plag_hamincha_from_times(a_calc, Some(&alos), &tzais, false)
            }
            Zman::PlagAhavatShalom => {
                let tzais_geonim_3_8 =
                    Zman::TzaisGeonim3Point8Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let shaah_zmanis = z_calc.get_shaah_zmanis_from_zmanim(
                    a_calc,
                    Zman::Alos16Point1Degrees,
                    Zman::TzaisGeonim3Point8Degrees,
                )?;
                Some(tzais_geonim_3_8 - multiply_duration(shaah_zmanis, 1.25)?)
            }
            Zman::PlagHaminchaAteretTorah => {
                let alos = Zman::Alos72Zmanis.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::TzaisAteretTorah.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_plag_hamincha_from_times(a_calc, Some(&alos), &tzais, false)
            }
            // Bain Hashmashos variants
            Zman::BainHashmashosRT13Point24Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_13_POINT_24)
            }
            Zman::BainHashmashosRT58Point5Minutes => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset + Duration::seconds((58.5 * 60.0) as i64))
            }
            Zman::BainHashmashosRT13Point5MinutesBefore7Point083Degrees => {
                let tzais_7_083 =
                    Zman::TzaisGeonim7Point083Degrees.calculate_with_calculator(a_calc, z_calc)?;
                Some(tzais_7_083 - Duration::seconds((13.5 * 60.0) as i64))
            }
            Zman::BainHashmashosRT2Stars => {
                let alos_19_8 =
                    Zman::Alos19Point8Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                let diff_millis =
                    (sunrise.timestamp_millis() - alos_19_8.timestamp_millis()) as f64;
                let offset_millis = (diff_millis * (5.0 / 18.0)) as i64;
                Some(sunset + Duration::milliseconds(offset_millis))
            }
            Zman::BainHashmashosYereim18Minutes => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset - Duration::minutes(18))
            }
            Zman::BainHashmashosYereim3Point05Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_MINUS_3_POINT_05)
            }
            Zman::BainHashmashosYereim16Point875Minutes => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset - Duration::seconds((16.875 * 60.0) as i64))
            }
            Zman::BainHashmashosYereim2Point8Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_MINUS_2_POINT_8)
            }
            Zman::BainHashmashosYereim13Point5Minutes => {
                let sunset = z_calc.sunset(a_calc)?;
                Some(sunset - Duration::seconds((13.5 * 60.0) as i64))
            }
            Zman::BainHashmashosYereim2Point1Degrees => {
                z_calc.sunset_offset_by_degrees(a_calc, ZENITH_MINUS_2_POINT_1)
            }
            // Fixed Local Chatzos variants (need location - will return None for now)
            Zman::SofZmanShmaMGA18DegreesToFixedLocalChatzos => None, // Needs FixedLocalChatzos
            Zman::SofZmanShmaMGA16Point1DegreesToFixedLocalChatzos => None, // Needs FixedLocalChatzos
            Zman::SofZmanShmaMGA90MinutesToFixedLocalChatzos => None, // Needs FixedLocalChatzos
            Zman::SofZmanShmaMGA72MinutesToFixedLocalChatzos => None, // Needs FixedLocalChatzos
            Zman::SofZmanShmaGRASunriseToFixedLocalChatzos => None,   // Needs FixedLocalChatzos
            Zman::SofZmanTfilaGRASunriseToFixedLocalChatzos => None,  // Needs FixedLocalChatzos
            Zman::MinchaGedolaGRAFixedLocalChatzos30Minutes => None,  // Needs FixedLocalChatzos
            Zman::MinchaKetanaGRAFixedLocalChatzosToSunset => None,   // Needs FixedLocalChatzos
            Zman::PlagHaminchaGRAFixedLocalChatzosToSunset => None,   // Needs FixedLocalChatzos
            // Chametz variants
            Zman::SofZmanAchilasChametzGRA => {
                Zman::SofZmanTfilaGRA.calculate_with_calculator(a_calc, z_calc)
            }
            Zman::SofZmanAchilasChametzMGA72Minutes => {
                Zman::SofZmanTfilaMGA.calculate_with_calculator(a_calc, z_calc)
            }
            Zman::SofZmanAchilasChametzMGA16Point1Degrees => {
                Zman::SofZmanTfilaMGA16Point1Degrees.calculate_with_calculator(a_calc, z_calc)
            }
            Zman::SofZmanBiurChametzGRA => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let shaah_zmanis = z_calc.get_shaah_zmanis_gra(a_calc)?;
                Some(sunrise + multiply_duration(shaah_zmanis, 5.0)?)
            }
            Zman::SofZmanBiurChametzMGA72Minutes => {
                let alos = Zman::Alos72.calculate_with_calculator(a_calc, z_calc)?;
                let shaah_zmanis = z_calc.get_shaah_zmanis_mga(a_calc)?;
                Some(alos + multiply_duration(shaah_zmanis, 5.0)?)
            }
            Zman::SofZmanBiurChametzMGA16Point1Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let shaah_zmanis = z_calc.get_shaah_zmanis_from_zmanim(
                    a_calc,
                    Zman::Alos16Point1Degrees,
                    Zman::Tzais16Point1Degrees,
                )?;
                Some(alos + multiply_duration(shaah_zmanis, 5.0)?)
            }
            // Baal HaTanya variants
            Zman::AlosBaalHatanya => z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_16_POINT_9),
            Zman::SofZmanShmaBaalHatanya => {
                let sunrise_bht = z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                let sunset_bht = z_calc.sunset_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                z_calc.get_sof_zman_shma_from_times(a_calc, &sunrise_bht, Some(&sunset_bht), true)
            }
            Zman::SofZmanTfilaBaalHatanya => {
                let sunrise_bht = z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                let sunset_bht = z_calc.sunset_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                z_calc.get_sof_zman_tfila_from_times(a_calc, &sunrise_bht, Some(&sunset_bht), true)
            }
            Zman::SofZmanAchilasChametzBaalHatanya => {
                Zman::SofZmanTfilaBaalHatanya.calculate_with_calculator(a_calc, z_calc)
            }
            Zman::SofZmanBiurChametzBaalHatanya => {
                let sunrise_bht = z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                let sunset_bht = z_calc.sunset_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                let shaah_zmanis =
                    z_calc.get_temporal_hour_from_times(&sunrise_bht, &sunset_bht)?;
                Some(sunrise_bht + multiply_duration(shaah_zmanis, 5.0)?)
            }
            Zman::MinchaGedolaBaalHatanya => {
                let sunrise_bht = z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                let sunset_bht = z_calc.sunset_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                z_calc.get_mincha_gedola_from_times(a_calc, Some(&sunrise_bht), &sunset_bht, true)
            }
            Zman::MinchaGedolaBaalHatanyaGreaterThan30 => {
                let mincha_gedola_30 =
                    Zman::MinchaGedola30Minutes.calculate_with_calculator(a_calc, z_calc)?;
                let mincha_gedola_bht =
                    Zman::MinchaGedolaBaalHatanya.calculate_with_calculator(a_calc, z_calc)?;
                Some(if mincha_gedola_30 > mincha_gedola_bht {
                    mincha_gedola_30
                } else {
                    mincha_gedola_bht
                })
            }
            Zman::MinchaKetanaBaalHatanya => {
                let sunrise_bht = z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                let sunset_bht = z_calc.sunset_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                z_calc.get_mincha_ketana_from_times(a_calc, Some(&sunrise_bht), &sunset_bht, true)
            }
            Zman::PlagHaminchaBaalHatanya => {
                let sunrise_bht = z_calc.sunrise_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                let sunset_bht = z_calc.sunset_offset_by_degrees(a_calc, ZENITH_1_POINT_583)?;
                z_calc.get_plag_hamincha_from_times(a_calc, Some(&sunrise_bht), &sunset_bht, true)
            }
            Zman::TzaisBaalHatanya => z_calc.sunset_offset_by_degrees(a_calc, ZENITH_6_DEGREES),
            // Samuch LeMinchaKetana variants
            Zman::SamuchLeMinchaKetanaGRA => {
                let sunrise = z_calc.sunrise(a_calc)?;
                let sunset = z_calc.sunset(a_calc)?;
                z_calc.get_samuch_le_mincha_ketana_from_times(a_calc, Some(&sunrise), &sunset, true)
            }
            Zman::SamuchLeMinchaKetana16Point1Degrees => {
                let alos = Zman::Alos16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais16Point1Degrees.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_samuch_le_mincha_ketana_from_times(a_calc, Some(&alos), &tzais, true)
            }
            Zman::SamuchLeMinchaKetana72Minutes => {
                let alos = Zman::Alos72.calculate_with_calculator(a_calc, z_calc)?;
                let tzais = Zman::Tzais72.calculate_with_calculator(a_calc, z_calc)?;
                z_calc.get_samuch_le_mincha_ketana_from_times(a_calc, Some(&alos), &tzais, true)
            }
        }
    }
}

struct InternalCalculator {
    location: Location,
    date: NaiveDate,
    z_calculator: ZmanimCalculator,
    a_calculator: AstronomicalCalculator,
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
