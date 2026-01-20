use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::ZmanimCalculator;

/// A low-level building block describing how to compute an astronomical or halachic event.
///
/// Most callers should use the predefined [`ZmanEvent`] constants (such as [`SUNRISE`]) or implement
/// [`Zman`] directly, but exposing [`Event`] allows composing new definitions using offsets.
pub enum Event<'a> {
    /// Sunrise at the configured location/date.
    Sunrise,
    /// Sunrise at sea level (no elevation adjustment).
    SeaLevelSunrise,
    /// Solar transit (local apparent noon / chatzos astronomically).
    Transit,
    /// Sunset at the configured location/date.
    Sunset,
    /// Sunset at sea level (no elevation adjustment).
    SeaLevelSunset,
    /// Time before sunrise when the sun is `degrees` below the geometric horizon.
    ///
    /// This is a *degrees-below-horizon* calculation (twilight-style), not a fixed number of
    /// minutes before the crate’s [`SUNRISE`] value. In particular:
    ///
    /// - It uses the **geometric horizon**, so it is independent of the observer’s elevation.
    /// - It is computed from the sun’s **astronomical position**, not by offsetting an
    ///   elevation-adjusted sunrise time.
    SunriseOffsetByDegrees(f64),
    /// Time after sunset when the sun is `degrees` below the geometric horizon.
    ///
    /// This is a *degrees-below-horizon* calculation (twilight-style), not a fixed number of
    /// minutes after the crate’s [`SUNSET`] value. In particular:
    ///
    /// - It uses the **geometric horizon**, so it is independent of the observer’s elevation.
    /// - It is computed from the sun’s **astronomical position**, not by offsetting an
    ///   elevation-adjusted sunset time.
    SunsetOffsetByDegrees(f64),
    /// Local mean time at the given hour (0.0–24.0).
    LocalMeanTime(f64),
    /// Shabbos/Yom Tov candle lighting time based on configuration.
    CandleLighting,
    /// Ateret Torah sunset based on configuration.
    AteretTorahSunset,
    /// A fixed time offset from another [`Event`].
    Offset(&'a Event<'a>, Duration),
    /// An offset in "shaos zmaniyos" (GRA temporal hours) from another [`Event`].
    ZmanisOffset(&'a Event<'a>, f64),
    /// An offset expressed as a fraction of the day between two [`Event`]s.
    ShaahZmanisBasedOffset(&'a Event<'a>, &'a Event<'a>, f64),
    /// An offset expressed as a fraction of the half-day between two [`Event`]s.
    HalfDayBasedOffset(&'a Event<'a>, &'a Event<'a>, f64),
    /// Sof zman shma derived from two bounding [`Event`]s.
    Shema(&'a Event<'a>, &'a Event<'a>, bool),
    /// Mincha gedola derived from two bounding [`Event`]s.
    MinchaGedola(&'a Event<'a>, &'a Event<'a>, bool),
    /// Samuch le-mincha ketana derived from two bounding [`Event`]s.
    SamuchLeMinchaKetana(&'a Event<'a>, &'a Event<'a>, bool),
    /// Mincha ketana derived from two bounding [`Event`]s.
    MinchaKetana(&'a Event<'a>, &'a Event<'a>, bool),
    /// Sof zman tefila derived from two bounding [`Event`]s.
    Tefila(&'a Event<'a>, &'a Event<'a>, bool),
    /// Plag hamincha derived from two bounding [`Event`]s.
    PlagHamincha(&'a Event<'a>, &'a Event<'a>, bool),
}

impl<'a> Event<'a> {
    fn calculate<T: TimeZone>(
        &self,
        calculator: &mut ZmanimCalculator<T>,
    ) -> Option<DateTime<Utc>> {
        match *self {
            Event::Sunrise => calculator.sunrise(),
            Event::SeaLevelSunrise => calculator.sea_level_sunrise(),
            Event::Transit => calculator.transit(),
            Event::Sunset => calculator.sunset(),
            Event::SeaLevelSunset => calculator.sea_level_sunset(),
            Event::SunriseOffsetByDegrees(degrees) => calculator.sunrise_offset_by_degrees(degrees),
            Event::SunsetOffsetByDegrees(degrees) => calculator.sunset_offset_by_degrees(degrees),
            Event::LocalMeanTime(hours) => {
                let date = calculator.date;
                let location = calculator.location.clone();
                calculator.local_mean_time(date, &location, hours)
            }
            Event::CandleLighting => {
                let sunset = calculator.sea_level_sunset()?;
                Some(sunset - calculator.config.candle_lighting_offset)
            }
            Event::AteretTorahSunset => {
                let sunset = calculator.sunset()?;
                Some(sunset + calculator.config.ateret_torah_sunset_offset)
            }
            Event::Offset(event, duration) => {
                let event_time = event.calculate(calculator)?;
                Some(event_time + duration)
            }
            Event::ZmanisOffset(event, hours) => {
                let event_time = event.calculate(calculator)?;
                calculator.offset_by_shaah_zmanis_gra(event_time, hours)
            }
            Event::ShaahZmanisBasedOffset(event1, event2, hours) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator)?;
                calculator.get_shaah_zmanis_based_zman_from_times(&event1_time, &event2_time, hours)
            }
            Event::HalfDayBasedOffset(event1, event2, hours) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator)?;
                calculator.get_half_day_based_zman_from_times(&event1_time, &event2_time, hours)
            }
            Event::Shema(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator);
                calculator.get_sof_zman_shma_from_times(
                    &event1_time,
                    event2_time.as_ref(),
                    synchronous,
                )
            }
            Event::MinchaGedola(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                calculator.get_mincha_gedola_from_times(
                    event1_time.as_ref(),
                    &event2_time,
                    synchronous,
                )
            }
            Event::SamuchLeMinchaKetana(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                calculator.get_samuch_le_mincha_ketana_from_times(
                    event1_time.as_ref(),
                    &event2_time,
                    synchronous,
                )
            }
            Event::MinchaKetana(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                calculator.get_mincha_ketana_from_times(
                    event1_time.as_ref(),
                    &event2_time,
                    synchronous,
                )
            }
            Event::Tefila(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator)?;
                let event2_time = event2.calculate(calculator);
                calculator.get_sof_zman_tfila_from_times(
                    &event1_time,
                    event2_time.as_ref(),
                    synchronous,
                )
            }
            Event::PlagHamincha(event1, event2, synchronous) => {
                let event1_time = event1.calculate(calculator);
                let event2_time = event2.calculate(calculator)?;
                calculator.get_plag_hamincha_from_times(
                    event1_time.as_ref(),
                    &event2_time,
                    synchronous,
                )
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        match self {
            Event::Sunrise | Event::Sunset => true,
            Event::LocalMeanTime(_) => false,
            Event::CandleLighting => false,
            Event::AteretTorahSunset => true,
            Event::Offset(event, _) => event.uses_elevation(),
            Event::ZmanisOffset(event, _) => event.uses_elevation(),
            Event::ShaahZmanisBasedOffset(event1, event2, _) => {
                event1.uses_elevation() || event2.uses_elevation()
            }
            Event::HalfDayBasedOffset(event1, event2, _) => {
                event1.uses_elevation() || event2.uses_elevation()
            }
            Event::Shema(event1, event2, _) => event1.uses_elevation() || event2.uses_elevation(),
            Event::MinchaGedola(event1, event2, _) => {
                event1.uses_elevation() || event2.uses_elevation()
            }
            Event::SamuchLeMinchaKetana(event1, event2, _) => {
                event1.uses_elevation() || event2.uses_elevation()
            }
            Event::MinchaKetana(event1, event2, _) => {
                event1.uses_elevation() || event2.uses_elevation()
            }
            Event::Tefila(event1, event2, _) => event1.uses_elevation() || event2.uses_elevation(),
            Event::PlagHamincha(event1, event2, _) => {
                event1.uses_elevation() || event2.uses_elevation()
            }
            _ => false,
        }
    }
}

#[cfg(test)]
/// A value that can be calculated by a [`ZmanimCalculator`].
///
/// This is implemented for [`ZmanEvent`] and for several custom calculation types.
pub trait Zman<Tz: TimeZone> {
    /// Compute the zman for the current calculator state.
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>>;
    /// Returns whether this zman uses elevation in its calculation (test-only).
    fn uses_elevation(&self) -> bool;
    /// Returns the KosherJava-style method name for this zman (test-only).
    fn name(&self) -> &str;
}

#[cfg(not(test))]
/// A value that can be calculated by a [`ZmanimCalculator`].
///
/// This is implemented for [`ZmanEvent`] and for several custom calculation types.
pub trait Zman<Tz: TimeZone> {
    /// Compute the zman for the current calculator state.
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>>;
}

/// A named zman backed by a low-level [`Event`] definition.
pub struct ZmanEvent<'a> {
    /// The underlying event definition.
    pub event: Event<'a>,
    /// The KosherJava-style method name (used by the Java parity tests).
    #[allow(unused)]
    pub(crate) name: &'a str,
}

impl<'a> ZmanEvent<'a> {
    /// Creates a new named zman backed by `event`.
    /// The name argument is used for internal testing purposes.
    /// Users of this library can safely ignore it.
    pub const fn new(event: Event<'a>, name: &'a str) -> Self {
        Self { event, name }
    }
}
impl<'a, Tz: TimeZone> Zman<Tz> for ZmanEvent<'a> {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        self.event.calculate(calculator)
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        self.event.uses_elevation()
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

/// Bain hashmashos (Rabbeinu Tam, 2-stars variant).
pub struct BainHashmashosRt2Stars {
    #[cfg(test)]
    name: &'static str,
}

impl<Tz: TimeZone> Zman<Tz> for BainHashmashosRt2Stars {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        let alos19_point_8 = calculator.sunrise_offset_by_degrees(19.8)?;
        let sunrise = calculator.sunrise()?;
        let sunset = calculator.sunset()?;
        let time_diff = sunrise.signed_duration_since(alos19_point_8);
        let offset = time_diff.num_milliseconds() as f64 * (5.0 / 18.0);
        Some(sunset + Duration::milliseconds(offset as i64))
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        true
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

/// Mincha gedola computed as the later of 30 minutes after chatzos or chatzos + 1/2 shaah zmanis.
pub struct MinchaGedolaAhavatShalom {
    #[cfg(test)]
    name: &'static str,
}

impl<Tz: TimeZone> Zman<Tz> for MinchaGedolaAhavatShalom {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        let chatzos = calculator.transit()?;
        let mincha_gedola_30 = chatzos + Duration::minutes(30);
        let alos = CORE_ALOS_16_POINT_1_DEGREES.calculate(calculator)?;
        let tzais = CORE_TZAIS_GEONIM_DEGREES_3_POINT_7.calculate(calculator)?;
        let shaah_zmanis = calculator.get_temporal_hour_from_times(&alos, &tzais)?;
        let mincha_alternative = chatzos + (shaah_zmanis / 2);
        if mincha_gedola_30 > mincha_alternative {
            Some(mincha_gedola_30)
        } else {
            Some(mincha_alternative)
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

/// Mincha gedola computed as the later of standard mincha gedola or a fixed 30-minute offset.
pub struct MinchaGedolaGreaterThan30 {
    #[cfg(test)]
    name: &'static str,
}

impl<Tz: TimeZone> Zman<Tz> for MinchaGedolaGreaterThan30 {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        let mincha_30 = MINCHA_GEDOLA_MINUTES_30.calculate(calculator)?;
        let mincha_regular = MINCHA_GEDOLA_SUNRISE_SUNSET.calculate(calculator)?;
        if mincha_30 > mincha_regular {
            Some(mincha_30)
        } else {
            Some(mincha_regular)
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        true
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

/// Mincha gedola computed as the later of 30 minutes after chatzos or the Baal HaTanya definition.
pub struct MinchaGedolaBaalHatanyaGreaterThan30 {
    #[cfg(test)]
    name: &'static str,
}

impl<Tz: TimeZone> Zman<Tz> for MinchaGedolaBaalHatanyaGreaterThan30 {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        let mincha_30 = MINCHA_GEDOLA_MINUTES_30.calculate(calculator)?;
        let mincha_baal_hatanya = MINCHA_GEDOLA_BAAL_HATANYA.calculate(calculator)?;
        if mincha_30 > mincha_baal_hatanya {
            Some(mincha_30)
        } else {
            Some(mincha_baal_hatanya)
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

/// Mincha ketana computed from the Ahavat Shalom definition (using Geonim tzais and alos 16.1°).
pub struct MinchaKetanaAhavatShalom {
    #[cfg(test)]
    name: &'static str,
}

impl<Tz: TimeZone> Zman<Tz> for MinchaKetanaAhavatShalom {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        let tzais = CORE_TZAIS_GEONIM_DEGREES_3_POINT_8.calculate(calculator)?;
        let alos = CORE_ALOS_16_POINT_1_DEGREES.calculate(calculator)?;
        let shaah_zmanis = calculator.get_temporal_hour_from_times(&alos, &tzais)?;
        Some(tzais - (shaah_zmanis * 5 / 2))
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

/// Plag hamincha computed from the Ahavat Shalom definition (using Geonim tzais and alos 16.1°).
pub struct PlagAhavatShalom {
    #[cfg(test)]
    name: &'static str,
}

impl<Tz: TimeZone> Zman<Tz> for PlagAhavatShalom {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        let tzais = CORE_TZAIS_GEONIM_DEGREES_3_POINT_8.calculate(calculator)?;
        let alos = CORE_ALOS_16_POINT_1_DEGREES.calculate(calculator)?;
        let shaah_zmanis = calculator.get_temporal_hour_from_times(&alos, &tzais)?;
        Some(tzais - (shaah_zmanis * 5 / 4))
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

// ============================================================================
// SUNRISE
// ============================================================================

/// Sunrise (elevation-adjusted).
pub const SUNRISE: ZmanEvent<'static> = ZmanEvent::new(Event::Sunrise, "getSunrise");
/// Sunrise at sea level (elevation `0m`).
pub const SEA_LEVEL_SUNRISE: ZmanEvent<'static> =
    ZmanEvent::new(Event::SeaLevelSunrise, "getSeaLevelSunrise");

// ============================================================================
// SUNSET
// ============================================================================

/// Sunset (elevation-adjusted).
pub const SUNSET: ZmanEvent<'static> = ZmanEvent::new(Event::Sunset, "getSunset");
/// Sunset at sea level (elevation `0m`).
pub const SEA_LEVEL_SUNSET: ZmanEvent<'static> =
    ZmanEvent::new(Event::SeaLevelSunset, "getSeaLevelSunset");

// ============================================================================
// ALOS
// ============================================================================

pub(crate) const CORE_ALOS_60_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-60));
pub(crate) const CORE_ALOS_72_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-72));
pub(crate) const CORE_ALOS_72_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -1.2);
pub(crate) const CORE_ALOS_90_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-90));
pub(crate) const CORE_ALOS_90_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -1.5);
pub(crate) const CORE_ALOS_96_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-96));
pub(crate) const CORE_ALOS_96_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -1.6);
pub(crate) const CORE_ALOS_120_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-120));
pub(crate) const CORE_ALOS_120_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -2.0);
pub(crate) const CORE_ALOS_16_POINT_1_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(16.1);
pub(crate) const CORE_ALOS_18_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(18.0);
pub(crate) const CORE_ALOS_19_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(19.0);
pub(crate) const CORE_ALOS_19_POINT_8_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(19.8);
pub(crate) const CORE_ALOS_26_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(26.0);
pub(crate) const CORE_ALOS_BAAL_HATANYA: Event<'static> = Event::SunriseOffsetByDegrees(16.9);
pub(crate) const CORE_BAAL_HATANYA_SUNRISE: Event<'static> = Event::SunriseOffsetByDegrees(1.583);
pub(crate) const CORE_BAAL_HATANYA_SUNSET: Event<'static> = Event::SunsetOffsetByDegrees(1.583);

/// *Alos* as a fixed `60` minutes before sunrise.
pub const ALOS_60_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_60_MINUTES, "getAlos60");
/// *Alos* as a fixed `72` minutes before sunrise.
pub const ALOS_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_72_MINUTES, "getAlos72");
/// *Alos* as `72 zmaniyos` minutes before sunrise (1.2 *shaos zmaniyos*).
pub const ALOS_72_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_72_ZMANIS, "getAlos72Zmanis");
/// *Alos* as a fixed `90` minutes before sunrise.
pub const ALOS_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_90_MINUTES, "getAlos90");
/// *Alos* as `90 zmaniyos` minutes before sunrise (1.5 *shaos zmaniyos*).
pub const ALOS_90_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_90_ZMANIS, "getAlos90Zmanis");
/// *Alos* as a fixed `96` minutes before sunrise.
pub const ALOS_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_96_MINUTES, "getAlos96");
/// *Alos* as `96 zmaniyos` minutes before sunrise (1.6 *shaos zmaniyos*).
pub const ALOS_96_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_96_ZMANIS, "getAlos96Zmanis");
/// *Alos* as a fixed `120` minutes before sunrise.
pub const ALOS_120_MINUTES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_120_MINUTES, "getAlos120");
/// *Alos* as `120 zmaniyos` minutes before sunrise (2.0 *shaos zmaniyos*).
pub const ALOS_120_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_120_ZMANIS, "getAlos120Zmanis");
/// *Alos* when the sun is `16.1°` below the geometric horizon (degrees-below-horizon dawn).
pub const ALOS_16_POINT_1_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_16_POINT_1_DEGREES, "getAlos16Point1Degrees");
/// *Alos* when the sun is `18°` below the geometric horizon (degrees-below-horizon dawn).
pub const ALOS_18_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_18_DEGREES, "getAlos18Degrees");
/// *Alos* when the sun is `19°` below the geometric horizon (degrees-below-horizon dawn).
pub const ALOS_19_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_19_DEGREES, "getAlos19Degrees");
/// *Alos* when the sun is `19.8°` below the geometric horizon (degrees-below-horizon dawn).
pub const ALOS_19_POINT_8_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_19_POINT_8_DEGREES, "getAlos19Point8Degrees");
/// *Alos* when the sun is `26°` below the geometric horizon (degrees-below-horizon dawn).
pub const ALOS_26_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_26_DEGREES, "getAlos26Degrees");
/// *Alos* when the sun is `16.9°` below the geometric horizon (degrees-below-horizon dawn).
pub const ALOS_BAAL_HATANYA: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_BAAL_HATANYA, "getAlosBaalHatanya");

// ============================================================================
// BAIN HASHMASHOS
// ============================================================================

/// Sunset when the sun is `7.083°` below the geometric horizon (after sunset).
pub(crate) const SUNSET_7_POINT_083_DEGREES: Event<'static> =
    Event::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0));

/// Bain hashmashos (Rabbeinu Tam): when the sun is `13.24°` below the geometric horizon (after sunset).
pub const BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunsetOffsetByDegrees(13.24),
    "getBainHashmashosRT13Point24Degrees",
);
/// Bain hashmashos (Rabbeinu Tam): `58.5` minutes after sunset.
pub const BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(
        &Event::Sunset,
        Duration::milliseconds((58.5 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosRT58Point5Minutes",
);
/// Bain hashmashos (Rabbeinu Tam): `13.5` minutes before [`SUNSET_7_POINT_083_DEGREES`].
pub const BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(
        Event::Offset(
            &SUNSET_7_POINT_083_DEGREES,
            Duration::milliseconds((-13.5 * 60.0 * 1000.0) as i64),
        ),
        "getBainHashmashosRT13Point5MinutesBefore7Point083Degrees",
    );
/// Bain hashmashos (Rabbeinu Tam, 2-stars variant).
pub const BAIN_HASHMASHOS_RT_2_STARS: BainHashmashosRt2Stars = BainHashmashosRt2Stars {
    #[cfg(test)]
    name: "getBainHashmashosRT2Stars",
};
/// Bain hashmashos (Yereim): `18` minutes before sunset.
pub const BAIN_HASHMASHOS_YEREIM_18_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Sunset, Duration::minutes(-18)),
    "getBainHashmashosYereim18Minutes",
);
/// Bain hashmashos (Yereim): `16.875` minutes before sunset.
pub const BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(
        &Event::Sunset,
        Duration::milliseconds((-16.875 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosYereim16Point875Minutes",
);
/// Bain hashmashos (Yereim): `13.5` minutes before sunset.
pub const BAIN_HASHMASHOS_YEREIM_13_POINT_5_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(
        &Event::Sunset,
        Duration::milliseconds((-13.5 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosYereim13Point5Minutes",
);

// ============================================================================
// CANDLE LIGHTING
// ============================================================================

/// Candle lighting: sea-level sunset minus [`crate::CalculatorConfig::candle_lighting_offset`].
pub const CANDLE_LIGHTING: ZmanEvent<'static> =
    ZmanEvent::new(Event::CandleLighting, "getCandleLighting");

// ============================================================================
// CHATZOS
// ============================================================================

pub(crate) const CORE_FIXED_LOCAL_CHATZOS: Event<'static> = Event::LocalMeanTime(12.0);

/// Chatzos (astronomical noon): solar transit.
pub const CHATZOS_ASTRONOMICAL: ZmanEvent<'static> = ZmanEvent::new(Event::Transit, "getChatzos");
/// Chatzos (half-day): midpoint between sea-level sunrise and sea-level sunset.
pub const CHATZOS_HALF_DAY: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::SeaLevelSunrise, &Event::SeaLevelSunset, 3.0),
    "getChatzosAsHalfDay",
);
/// Chatzos (fixed local): 12:00 local mean time.
pub const CHATZOS_FIXED_LOCAL: ZmanEvent<'static> =
    ZmanEvent::new(CORE_FIXED_LOCAL_CHATZOS, "getFixedLocalChatzos");

// ============================================================================
// MINCHA GEDOLA
// ============================================================================

/// Mincha gedola: `6.5` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const MINCHA_GEDOLA_SUNRISE_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&Event::Sunrise, &Event::Sunset, true),
    "getMinchaGedola",
);
/// Mincha gedola: `6.5` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const MINCHA_GEDOLA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getMinchaGedola16Point1Degrees",
);
/// Mincha gedola: `30` minutes after chatzos (solar transit).
pub const MINCHA_GEDOLA_MINUTES_30: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Transit, Duration::minutes(30)),
    "getMinchaGedola30Minutes",
);
/// Mincha gedola: `6.5` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const MINCHA_GEDOLA_MINUTES_72: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getMinchaGedola72Minutes",
);
/// Mincha gedola (Ahavat Shalom): later of `chatzos + 30m` and `chatzos + 1/2 shaah zmanis`.
pub const MINCHA_GEDOLA_AHAVAT_SHALOM: MinchaGedolaAhavatShalom = MinchaGedolaAhavatShalom {
    #[cfg(test)]
    name: "getMinchaGedolaAhavatShalom",
};
/// Mincha gedola: `6.5` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub const MINCHA_GEDOLA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getMinchaGedolaAteretTorah",
);
/// Mincha gedola: `6.5` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub const MINCHA_GEDOLA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getMinchaGedolaBaalHatanya",
);
/// Mincha gedola: later of Baal HaTanya mincha gedola and `chatzos + 30m`.
pub const MINCHA_GEDOLA_BAAL_HATANYA_GREATER_THAN_30: MinchaGedolaBaalHatanyaGreaterThan30 =
    MinchaGedolaBaalHatanyaGreaterThan30 {
        #[cfg(test)]
        name: "getMinchaGedolaBaalHatanyaGreaterThan30",
    };
/// Mincha gedola: `30` minutes after fixed local chatzos (12:00 local mean time).
pub const MINCHA_GEDOLA_GRA_FIXED_LOCAL_CHATZOS_30_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&CORE_FIXED_LOCAL_CHATZOS, Duration::minutes(30)),
    "getMinchaGedolaGRAFixedLocalChatzos30Minutes",
);
/// Mincha gedola: later of [`MINCHA_GEDOLA_SUNRISE_SUNSET`] and [`MINCHA_GEDOLA_MINUTES_30`].
pub const MINCHA_GEDOLA_GREATER_THAN_30: MinchaGedolaGreaterThan30 = MinchaGedolaGreaterThan30 {
    #[cfg(test)]
    name: "getMinchaGedolaGreaterThan30",
};

// ============================================================================
// MINCHA KETANA
// ============================================================================

/// Mincha ketana: `9.5` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const MINCHA_KETANA_SUNRISE_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&Event::Sunrise, &Event::Sunset, true),
    "getMinchaKetana",
);
/// Mincha ketana: `9.5` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const MINCHA_KETANA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getMinchaKetana16Point1Degrees",
);
/// Mincha ketana: `9.5` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const MINCHA_KETANA_MINUTES_72: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getMinchaKetana72Minutes",
);
/// Mincha ketana (Ahavat Shalom): `2.5` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
pub const MINCHA_KETANA_AHAVAT_SHALOM: MinchaKetanaAhavatShalom = MinchaKetanaAhavatShalom {
    #[cfg(test)]
    name: "getMinchaKetanaAhavatShalom",
};
/// Mincha ketana: `9.5` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub const MINCHA_KETANA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getMinchaKetanaAteretTorah",
);
/// Mincha ketana: `9.5` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub const MINCHA_KETANA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getMinchaKetanaBaalHatanya",
);
/// Mincha ketana: `3.5` shaos zmaniyos after fixed local chatzos, using fixed-local-chatzos → sunset half-day.
pub const MINCHA_KETANA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_FIXED_LOCAL_CHATZOS, &Event::Sunset, 3.5),
    "getMinchaKetanaGRAFixedLocalChatzosToSunset",
);

// ============================================================================
// MISHEYAKIR
// ============================================================================

/// Misheyakir when the sun is `10.2°` below the geometric horizon (degrees-below-horizon dawn).
pub const MISHEYAKIR_10_POINT_2_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(10.2),
    "getMisheyakir10Point2Degrees",
);
/// Misheyakir when the sun is `11°` below the geometric horizon (degrees-below-horizon dawn).
pub const MISHEYAKIR_11_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(11.0),
    "getMisheyakir11Degrees",
);
/// Misheyakir when the sun is `11.5°` below the geometric horizon (degrees-below-horizon dawn).
pub const MISHEYAKIR_11_POINT_5_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(11.5),
    "getMisheyakir11Point5Degrees",
);
/// Misheyakir when the sun is `7.65°` below the geometric horizon (degrees-below-horizon dawn).
pub const MISHEYAKIR_7_POINT_65_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(7.65),
    "getMisheyakir7Point65Degrees",
);
/// Misheyakir when the sun is `9.5°` below the geometric horizon (degrees-below-horizon dawn).
pub const MISHEYAKIR_9_POINT_5_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(9.5),
    "getMisheyakir9Point5Degrees",
);

// ============================================================================
// PLAG HAMINCHA
// ============================================================================

/// Plag hamincha: `10.75` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const PLAG_HAMINCHA_SUNRISE_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&Event::Sunrise, &Event::Sunset, true),
    "getPlagHamincha",
);
/// Plag hamincha (Ahavat Shalom): `1.25` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
pub const PLAG_HAMINCHA_AHAVAT_SHALOM: PlagAhavatShalom = PlagAhavatShalom {
    #[cfg(test)]
    name: "getPlagAhavatShalom",
};
/// Plag hamincha: `10.75` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais7.083°).
pub const PLAG_HAMINCHA_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_GEONIM_DEGREES_7_POINT_083,
        false,
    ),
    "getPlagAlos16Point1ToTzaisGeonim7Point083Degrees",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `16.1°` (day end = sunset).
pub const PLAG_HAMINCHA_ALOS_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_16_POINT_1_DEGREES, &Event::Sunset, false),
    "getPlagAlosToSunset",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `60` minutes (day = alos60 → tzais60).
pub const PLAG_HAMINCHA_60_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_60_MINUTES, &CORE_TZAIS_MINUTES_60, true),
    "getPlagHamincha60Minutes",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const PLAG_HAMINCHA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getPlagHamincha72Minutes",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `72 zmaniyos` (day = alos72Z → tzais72Z).
pub const PLAG_HAMINCHA_72_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_72_ZMANIS, true),
    "getPlagHamincha72MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `90` minutes (day = alos90 → tzais90).
pub const PLAG_HAMINCHA_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_90_MINUTES, &CORE_TZAIS_MINUTES_90, true),
    "getPlagHamincha90Minutes",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `90 zmaniyos` (day = alos90Z → tzais90Z).
pub const PLAG_HAMINCHA_90_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_90_ZMANIS, &CORE_TZAIS_90_ZMANIS, true),
    "getPlagHamincha90MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `96` minutes (day = alos96 → tzais96).
pub const PLAG_HAMINCHA_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_96_MINUTES, &CORE_TZAIS_MINUTES_96, true),
    "getPlagHamincha96Minutes",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `96 zmaniyos` (day = alos96Z → tzais96Z).
pub const PLAG_HAMINCHA_96_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_96_ZMANIS, &CORE_TZAIS_96_ZMANIS, true),
    "getPlagHamincha96MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `120` minutes (day = alos120 → tzais120).
pub const PLAG_HAMINCHA_120_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_120_MINUTES, &CORE_TZAIS_MINUTES_120, true),
    "getPlagHamincha120Minutes",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `120 zmaniyos` (day = alos120Z → tzais120Z).
pub const PLAG_HAMINCHA_120_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_120_ZMANIS, &CORE_TZAIS_120_ZMANIS, true),
    "getPlagHamincha120MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const PLAG_HAMINCHA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getPlagHamincha16Point1Degrees",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `18°` (day = alos18° → tzais18°).
pub const PLAG_HAMINCHA_18_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_18_DEGREES, &CORE_TZAIS_18_DEGREES, true),
    "getPlagHamincha18Degrees",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `19.8°` (day = alos19.8° → tzais19.8°).
pub const PLAG_HAMINCHA_19_POINT_8_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(
        &CORE_ALOS_19_POINT_8_DEGREES,
        &CORE_TZAIS_19_POINT_8_DEGREES,
        true,
    ),
    "getPlagHamincha19Point8Degrees",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `26°` (day = alos26° → tzais26°).
pub const PLAG_HAMINCHA_26_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_26_DEGREES, &CORE_TZAIS_26_DEGREES, true),
    "getPlagHamincha26Degrees",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub const PLAG_HAMINCHA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getPlagHaminchaAteretTorah",
);
/// Plag hamincha: `10.75` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub const PLAG_HAMINCHA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getPlagHaminchaBaalHatanya",
);
/// Plag hamincha: `4.75` shaos zmaniyos after fixed local chatzos, using fixed-local-chatzos → sunset half-day.
pub const PLAG_HAMINCHA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_FIXED_LOCAL_CHATZOS, &Event::Sunset, 4.75),
    "getPlagHaminchaGRAFixedLocalChatzosToSunset",
);

// ============================================================================
// SAMUCH LE MINCHA KETANA
// ============================================================================

/// Samuch le-mincha ketana: `9` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const SAMUCH_LE_MINCHA_KETANA_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::SamuchLeMinchaKetana(&Event::Sunrise, &Event::Sunset, true),
    "getSamuchLeMinchaKetanaGRA",
);
/// Samuch le-mincha ketana: `9` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const SAMUCH_LE_MINCHA_KETANA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SamuchLeMinchaKetana(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSamuchLeMinchaKetana16Point1Degrees",
);
/// Samuch le-mincha ketana: `9` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const SAMUCH_LE_MINCHA_KETANA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SamuchLeMinchaKetana(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSamuchLeMinchaKetana72Minutes",
);

// ============================================================================
// SOF ZMAN ACHILAS CHAMETZ
// ============================================================================

/// Sof zman achilas chametz: `4` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const SOF_ZMAN_ACHILAS_CHAMETZ_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&Event::Sunrise, &Event::Sunset, true),
    "getSofZmanAchilasChametzGRA",
);
/// Sof zman achilas chametz: `4` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const SOF_ZMAN_ACHILAS_CHAMETZ_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanAchilasChametzMGA72Minutes",
);
/// Sof zman achilas chametz: `4` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const SOF_ZMAN_ACHILAS_CHAMETZ_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSofZmanAchilasChametzMGA16Point1Degrees",
);
/// Sof zman achilas chametz: `4` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub const SOF_ZMAN_ACHILAS_CHAMETZ_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getSofZmanAchilasChametzBaalHatanya",
);

// ============================================================================
// SOF ZMAN BIUR CHAMETZ
// ============================================================================

/// Sof zman biur chametz: `5` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const SOF_ZMAN_BIUR_CHAMETZ_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::ZmanisOffset(&Event::Sunrise, 5.0),
    "getSofZmanBiurChametzGRA",
);
/// Sof zman biur chametz: `5` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const SOF_ZMAN_BIUR_CHAMETZ_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::ShaahZmanisBasedOffset(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, 5.0),
    "getSofZmanBiurChametzMGA72Minutes",
);
/// Sof zman biur chametz: `5` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const SOF_ZMAN_BIUR_CHAMETZ_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::ShaahZmanisBasedOffset(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        5.0,
    ),
    "getSofZmanBiurChametzMGA16Point1Degrees",
);
/// Sof zman biur chametz: `5` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub const SOF_ZMAN_BIUR_CHAMETZ_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::ShaahZmanisBasedOffset(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, 5.0),
    "getSofZmanBiurChametzBaalHatanya",
);

// ============================================================================
// SOF ZMAN SHMA
// ============================================================================

/// Sof zman shma: `3` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const SOF_ZMAN_SHMA_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&Event::Sunrise, &Event::Sunset, true),
    "getSofZmanShmaGRA",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const SOF_ZMAN_SHMA_MGA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanShmaMGA",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `19.8°` (day = alos19.8° → tzais19.8°).
pub const SOF_ZMAN_SHMA_MGA_19_POINT_8_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(
        &CORE_ALOS_19_POINT_8_DEGREES,
        &CORE_TZAIS_19_POINT_8_DEGREES,
        true,
    ),
    "getSofZmanShmaMGA19Point8Degrees",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSofZmanShmaMGA16Point1Degrees",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `18°` (day = alos18° → tzais18°).
pub const SOF_ZMAN_SHMA_MGA_18_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_18_DEGREES, &CORE_TZAIS_18_DEGREES, true),
    "getSofZmanShmaMGA18Degrees",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const SOF_ZMAN_SHMA_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanShmaMGA72Minutes",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `72 zmaniyos` (day = alos72Z → tzais72Z).
pub const SOF_ZMAN_SHMA_MGA_72_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_72_ZMANIS, true),
    "getSofZmanShmaMGA72MinutesZmanis",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `90` minutes (day = alos90 → tzais90).
pub const SOF_ZMAN_SHMA_MGA_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_90_MINUTES, &CORE_TZAIS_MINUTES_90, true),
    "getSofZmanShmaMGA90Minutes",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `90 zmaniyos` (day = alos90Z → tzais90Z).
pub const SOF_ZMAN_SHMA_MGA_90_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_90_ZMANIS, &CORE_TZAIS_90_ZMANIS, true),
    "getSofZmanShmaMGA90MinutesZmanis",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `96` minutes (day = alos96 → tzais96).
pub const SOF_ZMAN_SHMA_MGA_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_96_MINUTES, &CORE_TZAIS_MINUTES_96, true),
    "getSofZmanShmaMGA96Minutes",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `96 zmaniyos` (day = alos96Z → tzais96Z).
pub const SOF_ZMAN_SHMA_MGA_96_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_96_ZMANIS, &CORE_TZAIS_96_ZMANIS, true),
    "getSofZmanShmaMGA96MinutesZmanis",
);
/// Sof zman shma: `3` hours before chatzos (solar transit).
pub const SOF_ZMAN_SHMA_HOURS_3_BEFORE_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Transit, Duration::minutes(-180)),
    "getSofZmanShma3HoursBeforeChatzos",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `120` minutes (day = alos120 → tzais120).
pub const SOF_ZMAN_SHMA_MGA_120_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_120_MINUTES, &CORE_TZAIS_MINUTES_120, true),
    "getSofZmanShmaMGA120Minutes",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = sunset).
pub const SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_16_POINT_1_DEGREES, &Event::Sunset, false),
    "getSofZmanShmaAlos16Point1ToSunset",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = tzais7.083°).
pub const SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083: ZmanEvent<'static> =
    ZmanEvent::new(
        Event::Shema(
            &CORE_ALOS_16_POINT_1_DEGREES,
            &CORE_TZAIS_GEONIM_DEGREES_7_POINT_083,
            false,
        ),
        "getSofZmanShmaAlos16Point1ToTzaisGeonim7Point083Degrees",
    );
/// Sof zman shma: `3` shaos zmaniyos after sunrise (day end = fixed local chatzos).
pub const SOF_ZMAN_SHMA_KOL_ELIYAHU: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::Sunrise, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaKolEliyahu",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub const SOF_ZMAN_SHMA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getSofZmanShmaAteretTorah",
);
/// Sof zman shma: `3` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub const SOF_ZMAN_SHMA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getSofZmanShmaBaalHatanya",
);
/// Sof zman shma: `3` hours before fixed local chatzos (12:00 local mean time).
pub const SOF_ZMAN_SHMA_FIXED_LOCAL: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&CORE_FIXED_LOCAL_CHATZOS, Duration::minutes(-180)),
    "getSofZmanShmaFixedLocal",
);
/// Sof zman shma: `3` shaos zmaniyos after sunrise (day end = fixed local chatzos).
pub const SOF_ZMAN_SHMA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::Sunrise, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaGRASunriseToFixedLocalChatzos",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `18°` (day end = fixed local chatzos).
pub const SOF_ZMAN_SHMA_MGA_18_DEGREES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_ALOS_18_DEGREES, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaMGA18DegreesToFixedLocalChatzos",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = fixed local chatzos).
pub const SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> =
    ZmanEvent::new(
        Event::HalfDayBasedOffset(
            &CORE_ALOS_16_POINT_1_DEGREES,
            &CORE_FIXED_LOCAL_CHATZOS,
            3.0,
        ),
        "getSofZmanShmaMGA16Point1DegreesToFixedLocalChatzos",
    );
/// Sof zman shma: `3` shaos zmaniyos after alos `90` minutes (day end = fixed local chatzos).
pub const SOF_ZMAN_SHMA_MGA_90_MINUTES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_ALOS_90_MINUTES, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaMGA90MinutesToFixedLocalChatzos",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `72` minutes (day end = fixed local chatzos).
pub const SOF_ZMAN_SHMA_MGA_72_MINUTES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_ALOS_72_MINUTES, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaMGA72MinutesToFixedLocalChatzos",
);

// ============================================================================
// SOF ZMAN TFILA
// ============================================================================

/// Sof zman tfila: `4` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub const SOF_ZMAN_TFILA_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&Event::Sunrise, &Event::Sunset, true),
    "getSofZmanTfilaGRA",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const SOF_ZMAN_TFILA_MGA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanTfilaMGA",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `19.8°` (day = alos19.8° → tzais19.8°).
pub const SOF_ZMAN_TFILA_MGA_19_POINT_8_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(
        &CORE_ALOS_19_POINT_8_DEGREES,
        &CORE_TZAIS_19_POINT_8_DEGREES,
        true,
    ),
    "getSofZmanTfilaMGA19Point8Degrees",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub const SOF_ZMAN_TFILA_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSofZmanTfilaMGA16Point1Degrees",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `18°` (day = alos18° → tzais18°).
pub const SOF_ZMAN_TFILA_MGA_18_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_18_DEGREES, &CORE_TZAIS_18_DEGREES, true),
    "getSofZmanTfilaMGA18Degrees",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub const SOF_ZMAN_TFILA_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanTfilaMGA72Minutes",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `72 zmaniyos` (day = alos72Z → tzais72Z).
pub const SOF_ZMAN_TFILA_MGA_72_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_72_ZMANIS, true),
    "getSofZmanTfilaMGA72MinutesZmanis",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `90` minutes (day = alos90 → tzais90).
pub const SOF_ZMAN_TFILA_MGA_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_90_MINUTES, &CORE_TZAIS_MINUTES_90, true),
    "getSofZmanTfilaMGA90Minutes",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `90 zmaniyos` (day = alos90Z → tzais90Z).
pub const SOF_ZMAN_TFILA_MGA_90_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_90_ZMANIS, &CORE_TZAIS_90_ZMANIS, true),
    "getSofZmanTfilaMGA90MinutesZmanis",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `96` minutes (day = alos96 → tzais96).
pub const SOF_ZMAN_TFILA_MGA_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_96_MINUTES, &CORE_TZAIS_MINUTES_96, true),
    "getSofZmanTfilaMGA96Minutes",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `96 zmaniyos` (day = alos96Z → tzais96Z).
pub const SOF_ZMAN_TFILA_MGA_96_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_96_ZMANIS, &CORE_TZAIS_96_ZMANIS, true),
    "getSofZmanTfilaMGA96MinutesZmanis",
);
/// Sof zman tfila: `2` hours before chatzos (solar transit).
pub const SOF_ZMAN_TFILA_HOURS_2_BEFORE_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Transit, Duration::minutes(-120)),
    "getSofZmanTfila2HoursBeforeChatzos",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `120` minutes (day = alos120 → tzais120).
pub const SOF_ZMAN_TFILA_MGA_120_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_120_MINUTES, &CORE_TZAIS_MINUTES_120, true),
    "getSofZmanTfilaMGA120Minutes",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub const SOF_ZMAN_TFILA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getSofZmanTfilaAteretTorah",
);
/// Sof zman tfila: `4` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub const SOF_ZMAN_TFILA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getSofZmanTfilaBaalHatanya",
);
/// Sof zman tfila: `2` hours before fixed local chatzos (12:00 local mean time).
pub const SOF_ZMAN_TFILA_FIXED_LOCAL: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&CORE_FIXED_LOCAL_CHATZOS, Duration::minutes(-120)),
    "getSofZmanTfilaFixedLocal",
);
/// Sof zman tfila: `4` shaos zmaniyos after sunrise (day end = fixed local chatzos).
pub const SOF_ZMAN_TFILA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::Sunrise, &CORE_FIXED_LOCAL_CHATZOS, 4.0),
    "getSofZmanTfilaGRASunriseToFixedLocalChatzos",
);

// ============================================================================
// TZAIS
// ============================================================================

pub(crate) const CORE_TZAIS_DEGREES_8_POINT_5: Event<'static> = Event::SunsetOffsetByDegrees(8.5);
pub(crate) const CORE_TZAIS_MINUTES_50: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(50));
pub(crate) const CORE_TZAIS_MINUTES_60: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(60));
pub(crate) const CORE_TZAIS_MINUTES_72: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(72));
pub(crate) const CORE_TZAIS_72_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 1.2);
pub(crate) const CORE_TZAIS_MINUTES_90: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(90));
pub(crate) const CORE_TZAIS_90_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 1.5);
pub(crate) const CORE_TZAIS_MINUTES_96: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(96));
pub(crate) const CORE_TZAIS_96_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 1.6);
pub(crate) const CORE_TZAIS_MINUTES_120: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(120));
pub(crate) const CORE_TZAIS_120_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 2.0);
pub(crate) const CORE_TZAIS_16_POINT_1_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(16.1);
pub(crate) const CORE_TZAIS_18_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(18.0);
pub(crate) const CORE_TZAIS_19_POINT_8_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(19.8);
pub(crate) const CORE_TZAIS_26_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(26.0);
pub(crate) const CORE_TZAIS_ATERET_TORAH: Event<'static> = Event::AteretTorahSunset;
pub(crate) const CORE_TZAIS_BAAL_HATANYA: Event<'static> = Event::SunsetOffsetByDegrees(6.0);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_3_POINT_7: Event<'static> =
    Event::SunsetOffsetByDegrees(3.7);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_3_POINT_8: Event<'static> =
    Event::SunsetOffsetByDegrees(3.8);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_5_POINT_95: Event<'static> =
    Event::SunsetOffsetByDegrees(5.95);
pub(crate) const CORE_TZAIS_GEONIM_3_POINT_65: Event<'static> = Event::SunsetOffsetByDegrees(3.65);
pub(crate) const CORE_TZAIS_GEONIM_3_POINT_676: Event<'static> =
    Event::SunsetOffsetByDegrees(3.676);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_4_POINT_61: Event<'static> =
    Event::SunsetOffsetByDegrees(4.61);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_4_POINT_37: Event<'static> =
    Event::SunsetOffsetByDegrees(4.37);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_5_POINT_88: Event<'static> =
    Event::SunsetOffsetByDegrees(5.88);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_4_POINT_8: Event<'static> =
    Event::SunsetOffsetByDegrees(4.8);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_6_POINT_45: Event<'static> =
    Event::SunsetOffsetByDegrees(6.45);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_7_POINT_083: Event<'static> =
    Event::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0));
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_7_POINT_67: Event<'static> =
    Event::SunsetOffsetByDegrees(7.67);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_8_POINT_5: Event<'static> =
    Event::SunsetOffsetByDegrees(8.5);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_9_POINT_3: Event<'static> =
    Event::SunsetOffsetByDegrees(9.3);
pub(crate) const CORE_TZAIS_GEONIM_DEGREES_9_POINT_75: Event<'static> =
    Event::SunsetOffsetByDegrees(9.75);

/// Tzais when the sun is `8.5°` below the geometric horizon (after sunset).
pub const TZAIS_DEGREES_8_POINT_5: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_DEGREES_8_POINT_5, "getTzaisGeonim8Point5Degrees");
/// Tzais: `50` minutes after sunset.
pub const TZAIS_MINUTES_50: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_50, "getTzais50");
/// Tzais: `60` minutes after sunset.
pub const TZAIS_MINUTES_60: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_60, "getTzais60");
/// Tzais: `72` minutes after sunset.
pub const TZAIS_MINUTES_72: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_72, "getTzais72");
/// Tzais: `72 zmaniyos` minutes after sunset (1.2 *shaos zmaniyos*).
pub const TZAIS_72_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_72_ZMANIS, "getTzais72Zmanis");
/// Tzais: `90` minutes after sunset.
pub const TZAIS_MINUTES_90: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_90, "getTzais90");
/// Tzais: `90 zmaniyos` minutes after sunset (1.5 *shaos zmaniyos*).
pub const TZAIS_90_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_90_ZMANIS, "getTzais90Zmanis");
/// Tzais: `96` minutes after sunset.
pub const TZAIS_MINUTES_96: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_96, "getTzais96");
/// Tzais: `96 zmaniyos` minutes after sunset (1.6 *shaos zmaniyos*).
pub const TZAIS_96_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_96_ZMANIS, "getTzais96Zmanis");
/// Tzais: `120` minutes after sunset.
pub const TZAIS_MINUTES_120: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_120, "getTzais120");
/// Tzais: `120 zmaniyos` minutes after sunset (2.0 *shaos zmaniyos*).
pub const TZAIS_120_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_120_ZMANIS, "getTzais120Zmanis");
/// Tzais when the sun is `16.1°` below the geometric horizon (after sunset).
pub const TZAIS_16_POINT_1_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_16_POINT_1_DEGREES, "getTzais16Point1Degrees");
/// Tzais when the sun is `18°` below the geometric horizon (after sunset).
pub const TZAIS_18_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_18_DEGREES, "getTzais18Degrees");
/// Tzais when the sun is `19.8°` below the geometric horizon (after sunset).
pub const TZAIS_19_POINT_8_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_19_POINT_8_DEGREES, "getTzais19Point8Degrees");
/// Tzais when the sun is `26°` below the geometric horizon (after sunset).
pub const TZAIS_26_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_26_DEGREES, "getTzais26Degrees");
/// Tzais (Ateret Torah): sunset plus [`crate::CalculatorConfig::ateret_torah_sunset_offset`].
pub const TZAIS_ATERET_TORAH: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_ATERET_TORAH, "getTzaisAteretTorah");
/// Tzais (Baal HaTanya): when the sun is `6°` below the geometric horizon (after sunset).
pub const TZAIS_BAAL_HATANYA: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_BAAL_HATANYA, "getTzaisBaalHatanya");
/// Tzais (Geonim): when the sun is `3.7°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_3_POINT_7: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_3_POINT_7,
    "getTzaisGeonim3Point7Degrees",
);
/// Tzais (Geonim): when the sun is `3.8°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_3_POINT_8: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_3_POINT_8,
    "getTzaisGeonim3Point8Degrees",
);
/// Tzais (Geonim): when the sun is `5.95°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_5_POINT_95: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_5_POINT_95,
    "getTzaisGeonim5Point95Degrees",
);
/// Tzais (Geonim): when the sun is `3.65°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_3_POINT_65: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_3_POINT_65,
    "getTzaisGeonim3Point65Degrees",
);
/// Tzais (Geonim): when the sun is `3.676°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_3_POINT_676: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_3_POINT_676,
    "getTzaisGeonim3Point676Degrees",
);
/// Tzais (Geonim): when the sun is `4.61°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_4_POINT_61: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_4_POINT_61,
    "getTzaisGeonim4Point61Degrees",
);
/// Tzais (Geonim): when the sun is `4.37°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_4_POINT_37: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_4_POINT_37,
    "getTzaisGeonim4Point37Degrees",
);
/// Tzais (Geonim): when the sun is `5.88°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_5_POINT_88: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_5_POINT_88,
    "getTzaisGeonim5Point88Degrees",
);
/// Tzais (Geonim): when the sun is `4.8°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_4_POINT_8: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_4_POINT_8,
    "getTzaisGeonim4Point8Degrees",
);
/// Tzais (Geonim): when the sun is `6.45°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_6_POINT_45: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_6_POINT_45,
    "getTzaisGeonim6Point45Degrees",
);
/// Tzais (Geonim): when the sun is `7.083°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_7_POINT_083: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_7_POINT_083,
    "getTzaisGeonim7Point083Degrees",
);
/// Tzais (Geonim): when the sun is `7.67°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_7_POINT_67: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_7_POINT_67,
    "getTzaisGeonim7Point67Degrees",
);
/// Tzais (Geonim): when the sun is `8.5°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_8_POINT_5: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_8_POINT_5,
    "getTzaisGeonim8Point5Degrees",
);
/// Tzais (Geonim): when the sun is `9.3°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_9_POINT_3: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_9_POINT_3,
    "getTzaisGeonim9Point3Degrees",
);
/// Tzais (Geonim): when the sun is `9.75°` below the geometric horizon (after sunset).
pub const TZAIS_GEONIM_DEGREES_9_POINT_75: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_9_POINT_75,
    "getTzaisGeonim9Point75Degrees",
);
