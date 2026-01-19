use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::ZmanimCalculator;

pub enum Event<'a> {
    Sunrise,
    SeaLevelSunrise,
    Transit,
    Sunset,
    SeaLevelSunset,
    SunriseOffsetByDegrees(f64),
    SunsetOffsetByDegrees(f64),
    LocalMeanTime(f64),
    CandleLighting,
    AteretTorahSunset,
    Offset(&'a Event<'a>, Duration),
    ZmanisOffset(&'a Event<'a>, f64),
    ShaahZmanisBasedOffset(&'a Event<'a>, &'a Event<'a>, f64),
    HalfDayBasedOffset(&'a Event<'a>, &'a Event<'a>, f64),
    Shema(&'a Event<'a>, &'a Event<'a>, bool),
    MinchaGedola(&'a Event<'a>, &'a Event<'a>, bool),
    SamuchLeMinchaKetana(&'a Event<'a>, &'a Event<'a>, bool),
    MinchaKetana(&'a Event<'a>, &'a Event<'a>, bool),
    Tefila(&'a Event<'a>, &'a Event<'a>, bool),
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

pub trait Zman<Tz: TimeZone> {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>>;
    #[cfg(test)]
    fn uses_elevation(&self) -> bool;
    #[cfg(test)]
    fn name(&self) -> &str;
}

pub struct ZmanEvent<'a> {
    pub core: Event<'a>,
    pub name: &'a str,
}

impl<'a> ZmanEvent<'a> {
    pub const fn new(core: Event<'a>, name: &'a str) -> Self {
        Self { core, name }
    }
}
impl<'a, Tz: TimeZone> Zman<Tz> for ZmanEvent<'a> {
    fn calculate(&self, calculator: &mut ZmanimCalculator<Tz>) -> Option<DateTime<Utc>> {
        self.core.calculate(calculator)
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        self.core.uses_elevation()
    }
    #[cfg(test)]
    fn name(&self) -> &str {
        self.name
    }
}

pub struct BainHashmashosRt2Stars {
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

pub struct MinchaGedolaAhavatShalom {
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

pub struct MinchaGedolaGreaterThan30 {
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

pub struct MinchaGedolaBaalHatanyaGreaterThan30 {
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

pub struct MinchaKetanaAhavatShalom {
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

pub struct PlagAhavatShalom {
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

pub const SUNRISE: ZmanEvent<'static> = ZmanEvent::new(Event::Sunrise, "getSunrise");
pub const SEA_LEVEL_SUNRISE: ZmanEvent<'static> =
    ZmanEvent::new(Event::SeaLevelSunrise, "getSeaLevelSunrise");

// ============================================================================
// SUNSET
// ============================================================================

pub const SUNSET: ZmanEvent<'static> = ZmanEvent::new(Event::Sunset, "getSunset");
pub const SEA_LEVEL_SUNSET: ZmanEvent<'static> =
    ZmanEvent::new(Event::SeaLevelSunset, "getSeaLevelSunset");

// ============================================================================
// ALOS
// ============================================================================

pub const CORE_ALOS_60_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-60));
pub const CORE_ALOS_72_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-72));
pub const CORE_ALOS_72_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -1.2);
pub const CORE_ALOS_90_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-90));
pub const CORE_ALOS_90_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -1.5);
pub const CORE_ALOS_96_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-96));
pub const CORE_ALOS_96_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -1.6);
pub const CORE_ALOS_120_MINUTES: Event<'static> =
    Event::Offset(&Event::Sunrise, Duration::minutes(-120));
pub const CORE_ALOS_120_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunrise, -2.0);
pub const CORE_ALOS_16_POINT_1_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(16.1);
pub const CORE_ALOS_18_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(18.0);
pub const CORE_ALOS_19_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(19.0);
pub const CORE_ALOS_19_POINT_8_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(19.8);
pub const CORE_ALOS_26_DEGREES: Event<'static> = Event::SunriseOffsetByDegrees(26.0);
pub const CORE_ALOS_BAAL_HATANYA: Event<'static> = Event::SunriseOffsetByDegrees(16.9);
pub const CORE_BAAL_HATANYA_SUNRISE: Event<'static> = Event::SunriseOffsetByDegrees(1.583);
pub const CORE_BAAL_HATANYA_SUNSET: Event<'static> = Event::SunsetOffsetByDegrees(1.583);

pub const ALOS_60_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_60_MINUTES, "getAlos60");
pub const ALOS_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_72_MINUTES, "getAlos72");
pub const ALOS_72_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_72_ZMANIS, "getAlos72Zmanis");
pub const ALOS_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_90_MINUTES, "getAlos90");
pub const ALOS_90_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_90_ZMANIS, "getAlos90Zmanis");
pub const ALOS_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(CORE_ALOS_96_MINUTES, "getAlos96");
pub const ALOS_96_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_96_ZMANIS, "getAlos96Zmanis");
pub const ALOS_120_MINUTES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_120_MINUTES, "getAlos120");
pub const ALOS_120_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_120_ZMANIS, "getAlos120Zmanis");
pub const ALOS_16_POINT_1_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_16_POINT_1_DEGREES, "getAlos16Point1Degrees");
pub const ALOS_18_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_18_DEGREES, "getAlos18Degrees");
pub const ALOS_19_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_19_DEGREES, "getAlos19Degrees");
pub const ALOS_19_POINT_8_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_19_POINT_8_DEGREES, "getAlos19Point8Degrees");
pub const ALOS_26_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_26_DEGREES, "getAlos26Degrees");
pub const ALOS_BAAL_HATANYA: ZmanEvent<'static> =
    ZmanEvent::new(CORE_ALOS_BAAL_HATANYA, "getAlosBaalHatanya");

// ============================================================================
// BAIN HASHMASHOS
// ============================================================================

pub const SUNSET_7_POINT_083_DEGREES: Event<'static> =
    Event::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0));

pub const BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunsetOffsetByDegrees(13.24),
    "getBainHashmashosRT13Point24Degrees",
);
pub const BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(
        &Event::Sunset,
        Duration::milliseconds((58.5 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosRT58Point5Minutes",
);
pub const BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(
        Event::Offset(
            &SUNSET_7_POINT_083_DEGREES,
            Duration::milliseconds((-13.5 * 60.0 * 1000.0) as i64),
        ),
        "getBainHashmashosRT13Point5MinutesBefore7Point083Degrees",
    );
pub const BAIN_HASHMASHOS_RT_2_STARS: BainHashmashosRt2Stars = BainHashmashosRt2Stars {
    name: "getBainHashmashosRT2Stars",
};
pub const BAIN_HASHMASHOS_YEREIM_18_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Sunset, Duration::minutes(-18)),
    "getBainHashmashosYereim18Minutes",
);
pub const BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(
        &Event::Sunset,
        Duration::milliseconds((-16.875 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosYereim16Point875Minutes",
);
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

pub const CANDLE_LIGHTING: ZmanEvent<'static> =
    ZmanEvent::new(Event::CandleLighting, "getCandleLighting");

// ============================================================================
// CHATZOS
// ============================================================================

pub const CORE_FIXED_LOCAL_CHATZOS: Event<'static> = Event::LocalMeanTime(12.0);

pub const CHATZOS_ASTRONOMICAL: ZmanEvent<'static> = ZmanEvent::new(Event::Transit, "getChatzos");
pub const CHATZOS_HALF_DAY: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::SeaLevelSunrise, &Event::SeaLevelSunset, 3.0),
    "getChatzosAsHalfDay",
);
pub const CHATZOS_FIXED_LOCAL: ZmanEvent<'static> =
    ZmanEvent::new(CORE_FIXED_LOCAL_CHATZOS, "getFixedLocalChatzos");

// ============================================================================
// MINCHA GEDOLA
// ============================================================================

pub const MINCHA_GEDOLA_SUNRISE_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&Event::Sunrise, &Event::Sunset, true),
    "getMinchaGedola",
);
pub const MINCHA_GEDOLA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getMinchaGedola16Point1Degrees",
);
pub const MINCHA_GEDOLA_MINUTES_30: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Transit, Duration::minutes(30)),
    "getMinchaGedola30Minutes",
);
pub const MINCHA_GEDOLA_MINUTES_72: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getMinchaGedola72Minutes",
);
pub const MINCHA_GEDOLA_AHAVAT_SHALOM: MinchaGedolaAhavatShalom = MinchaGedolaAhavatShalom {
    name: "getMinchaGedolaAhavatShalom",
};
pub const MINCHA_GEDOLA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getMinchaGedolaAteretTorah",
);
pub const MINCHA_GEDOLA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaGedola(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getMinchaGedolaBaalHatanya",
);
pub const MINCHA_GEDOLA_BAAL_HATANYA_GREATER_THAN_30: MinchaGedolaBaalHatanyaGreaterThan30 =
    MinchaGedolaBaalHatanyaGreaterThan30 {
        name: "getMinchaGedolaBaalHatanyaGreaterThan30",
    };
pub const MINCHA_GEDOLA_GRA_FIXED_LOCAL_CHATZOS_30_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&CORE_FIXED_LOCAL_CHATZOS, Duration::minutes(30)),
    "getMinchaGedolaGRAFixedLocalChatzos30Minutes",
);
pub const MINCHA_GEDOLA_GREATER_THAN_30: MinchaGedolaGreaterThan30 = MinchaGedolaGreaterThan30 {
    name: "getMinchaGedolaGreaterThan30",
};

// ============================================================================
// MINCHA KETANA
// ============================================================================

pub const MINCHA_KETANA_SUNRISE_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&Event::Sunrise, &Event::Sunset, true),
    "getMinchaKetana",
);
pub const MINCHA_KETANA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getMinchaKetana16Point1Degrees",
);
pub const MINCHA_KETANA_MINUTES_72: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getMinchaKetana72Minutes",
);
pub const MINCHA_KETANA_AHAVAT_SHALOM: MinchaKetanaAhavatShalom = MinchaKetanaAhavatShalom {
    name: "getMinchaKetanaAhavatShalom",
};
pub const MINCHA_KETANA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getMinchaKetanaAteretTorah",
);
pub const MINCHA_KETANA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::MinchaKetana(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getMinchaKetanaBaalHatanya",
);
pub const MINCHA_KETANA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_FIXED_LOCAL_CHATZOS, &Event::Sunset, 3.5),
    "getMinchaKetanaGRAFixedLocalChatzosToSunset",
);

// ============================================================================
// MISHEYAKIR
// ============================================================================

pub const MISHEYAKIR_10_POINT_2_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(10.2),
    "getMisheyakir10Point2Degrees",
);
pub const MISHEYAKIR_11_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(11.0),
    "getMisheyakir11Degrees",
);
pub const MISHEYAKIR_11_POINT_5_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(11.5),
    "getMisheyakir11Point5Degrees",
);
pub const MISHEYAKIR_7_POINT_65_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(7.65),
    "getMisheyakir7Point65Degrees",
);
pub const MISHEYAKIR_9_POINT_5_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SunriseOffsetByDegrees(9.5),
    "getMisheyakir9Point5Degrees",
);

// ============================================================================
// PLAG HAMINCHA
// ============================================================================

pub const PLAG_HAMINCHA_SUNRISE_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&Event::Sunrise, &Event::Sunset, true),
    "getPlagHamincha",
);
pub const PLAG_HAMINCHA_AHAVAT_SHALOM: PlagAhavatShalom = PlagAhavatShalom {
    name: "getPlagAhavatShalom",
};
pub const PLAG_HAMINCHA_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_GEONIM_DEGREES_7_POINT_083,
        false,
    ),
    "getPlagAlos16Point1ToTzaisGeonim7Point083Degrees",
);
pub const PLAG_HAMINCHA_ALOS_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_16_POINT_1_DEGREES, &Event::Sunset, false),
    "getPlagAlosToSunset",
);
pub const PLAG_HAMINCHA_60_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_60_MINUTES, &CORE_TZAIS_MINUTES_60, true),
    "getPlagHamincha60Minutes",
);
pub const PLAG_HAMINCHA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getPlagHamincha72Minutes",
);
pub const PLAG_HAMINCHA_72_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_72_ZMANIS, true),
    "getPlagHamincha72MinutesZmanis",
);
pub const PLAG_HAMINCHA_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_90_MINUTES, &CORE_TZAIS_MINUTES_90, true),
    "getPlagHamincha90Minutes",
);
pub const PLAG_HAMINCHA_90_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_90_ZMANIS, &CORE_TZAIS_90_ZMANIS, true),
    "getPlagHamincha90MinutesZmanis",
);
pub const PLAG_HAMINCHA_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_96_MINUTES, &CORE_TZAIS_MINUTES_96, true),
    "getPlagHamincha96Minutes",
);
pub const PLAG_HAMINCHA_96_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_96_ZMANIS, &CORE_TZAIS_96_ZMANIS, true),
    "getPlagHamincha96MinutesZmanis",
);
pub const PLAG_HAMINCHA_120_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_120_MINUTES, &CORE_TZAIS_MINUTES_120, true),
    "getPlagHamincha120Minutes",
);
pub const PLAG_HAMINCHA_120_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_120_ZMANIS, &CORE_TZAIS_120_ZMANIS, true),
    "getPlagHamincha120MinutesZmanis",
);
pub const PLAG_HAMINCHA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getPlagHamincha16Point1Degrees",
);
pub const PLAG_HAMINCHA_18_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_18_DEGREES, &CORE_TZAIS_18_DEGREES, true),
    "getPlagHamincha18Degrees",
);
pub const PLAG_HAMINCHA_19_POINT_8_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(
        &CORE_ALOS_19_POINT_8_DEGREES,
        &CORE_TZAIS_19_POINT_8_DEGREES,
        true,
    ),
    "getPlagHamincha19Point8Degrees",
);
pub const PLAG_HAMINCHA_26_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_26_DEGREES, &CORE_TZAIS_26_DEGREES, true),
    "getPlagHamincha26Degrees",
);
pub const PLAG_HAMINCHA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getPlagHaminchaAteretTorah",
);
pub const PLAG_HAMINCHA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::PlagHamincha(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getPlagHaminchaBaalHatanya",
);
pub const PLAG_HAMINCHA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_FIXED_LOCAL_CHATZOS, &Event::Sunset, 4.75),
    "getPlagHaminchaGRAFixedLocalChatzosToSunset",
);

// ============================================================================
// SAMUCH LE MINCHA KETANA
// ============================================================================

pub const SAMUCH_LE_MINCHA_KETANA_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::SamuchLeMinchaKetana(&Event::Sunrise, &Event::Sunset, true),
    "getSamuchLeMinchaKetanaGRA",
);
pub const SAMUCH_LE_MINCHA_KETANA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SamuchLeMinchaKetana(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSamuchLeMinchaKetana16Point1Degrees",
);
pub const SAMUCH_LE_MINCHA_KETANA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::SamuchLeMinchaKetana(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSamuchLeMinchaKetana72Minutes",
);

// ============================================================================
// SOF ZMAN ACHILAS CHAMETZ
// ============================================================================

pub const SOF_ZMAN_ACHILAS_CHAMETZ_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&Event::Sunrise, &Event::Sunset, true),
    "getSofZmanAchilasChametzGRA",
);
pub const SOF_ZMAN_ACHILAS_CHAMETZ_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanAchilasChametzMGA72Minutes",
);
pub const SOF_ZMAN_ACHILAS_CHAMETZ_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSofZmanAchilasChametzMGA16Point1Degrees",
);
pub const SOF_ZMAN_ACHILAS_CHAMETZ_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getSofZmanAchilasChametzBaalHatanya",
);

// ============================================================================
// SOF ZMAN BIUR CHAMETZ
// ============================================================================

pub const SOF_ZMAN_BIUR_CHAMETZ_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::ZmanisOffset(&Event::Sunrise, 5.0),
    "getSofZmanBiurChametzGRA",
);
pub const SOF_ZMAN_BIUR_CHAMETZ_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::ShaahZmanisBasedOffset(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, 5.0),
    "getSofZmanBiurChametzMGA72Minutes",
);
pub const SOF_ZMAN_BIUR_CHAMETZ_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::ShaahZmanisBasedOffset(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        5.0,
    ),
    "getSofZmanBiurChametzMGA16Point1Degrees",
);
pub const SOF_ZMAN_BIUR_CHAMETZ_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::ShaahZmanisBasedOffset(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, 5.0),
    "getSofZmanBiurChametzBaalHatanya",
);

// ============================================================================
// SOF ZMAN SHMA
// ============================================================================

pub const SOF_ZMAN_SHMA_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&Event::Sunrise, &Event::Sunset, true),
    "getSofZmanShmaGRA",
);
pub const SOF_ZMAN_SHMA_MGA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanShmaMGA",
);
pub const SOF_ZMAN_SHMA_MGA_19_POINT_8_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(
        &CORE_ALOS_19_POINT_8_DEGREES,
        &CORE_TZAIS_19_POINT_8_DEGREES,
        true,
    ),
    "getSofZmanShmaMGA19Point8Degrees",
);
pub const SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSofZmanShmaMGA16Point1Degrees",
);
pub const SOF_ZMAN_SHMA_MGA_18_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_18_DEGREES, &CORE_TZAIS_18_DEGREES, true),
    "getSofZmanShmaMGA18Degrees",
);
pub const SOF_ZMAN_SHMA_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanShmaMGA72Minutes",
);
pub const SOF_ZMAN_SHMA_MGA_72_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_72_ZMANIS, true),
    "getSofZmanShmaMGA72MinutesZmanis",
);
pub const SOF_ZMAN_SHMA_MGA_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_90_MINUTES, &CORE_TZAIS_MINUTES_90, true),
    "getSofZmanShmaMGA90Minutes",
);
pub const SOF_ZMAN_SHMA_MGA_90_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_90_ZMANIS, &CORE_TZAIS_90_ZMANIS, true),
    "getSofZmanShmaMGA90MinutesZmanis",
);
pub const SOF_ZMAN_SHMA_MGA_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_96_MINUTES, &CORE_TZAIS_MINUTES_96, true),
    "getSofZmanShmaMGA96Minutes",
);
pub const SOF_ZMAN_SHMA_MGA_96_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_96_ZMANIS, &CORE_TZAIS_96_ZMANIS, true),
    "getSofZmanShmaMGA96MinutesZmanis",
);
pub const SOF_ZMAN_SHMA_HOURS_3_BEFORE_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Transit, Duration::minutes(-180)),
    "getSofZmanShma3HoursBeforeChatzos",
);
pub const SOF_ZMAN_SHMA_MGA_120_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_120_MINUTES, &CORE_TZAIS_MINUTES_120, true),
    "getSofZmanShmaMGA120Minutes",
);
pub const SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_SUNSET: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_16_POINT_1_DEGREES, &Event::Sunset, false),
    "getSofZmanShmaAlos16Point1ToSunset",
);
pub const SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083: ZmanEvent<'static> =
    ZmanEvent::new(
        Event::Shema(
            &CORE_ALOS_16_POINT_1_DEGREES,
            &CORE_TZAIS_GEONIM_DEGREES_7_POINT_083,
            false,
        ),
        "getSofZmanShmaAlos16Point1ToTzaisGeonim7Point083Degrees",
    );
pub const SOF_ZMAN_SHMA_KOL_ELIYAHU: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::Sunrise, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaKolEliyahu",
);
pub const SOF_ZMAN_SHMA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getSofZmanShmaAteretTorah",
);
pub const SOF_ZMAN_SHMA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Shema(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getSofZmanShmaBaalHatanya",
);
pub const SOF_ZMAN_SHMA_FIXED_LOCAL: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&CORE_FIXED_LOCAL_CHATZOS, Duration::minutes(-180)),
    "getSofZmanShmaFixedLocal",
);
pub const SOF_ZMAN_SHMA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::Sunrise, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaGRASunriseToFixedLocalChatzos",
);
pub const SOF_ZMAN_SHMA_MGA_18_DEGREES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_ALOS_18_DEGREES, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaMGA18DegreesToFixedLocalChatzos",
);
pub const SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> =
    ZmanEvent::new(
        Event::HalfDayBasedOffset(
            &CORE_ALOS_16_POINT_1_DEGREES,
            &CORE_FIXED_LOCAL_CHATZOS,
            3.0,
        ),
        "getSofZmanShmaMGA16Point1DegreesToFixedLocalChatzos",
    );
pub const SOF_ZMAN_SHMA_MGA_90_MINUTES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_ALOS_90_MINUTES, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaMGA90MinutesToFixedLocalChatzos",
);
pub const SOF_ZMAN_SHMA_MGA_72_MINUTES_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&CORE_ALOS_72_MINUTES, &CORE_FIXED_LOCAL_CHATZOS, 3.0),
    "getSofZmanShmaMGA72MinutesToFixedLocalChatzos",
);

// ============================================================================
// SOF ZMAN TFILA
// ============================================================================

pub const SOF_ZMAN_TFILA_GRA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&Event::Sunrise, &Event::Sunset, true),
    "getSofZmanTfilaGRA",
);
pub const SOF_ZMAN_TFILA_MGA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanTfilaMGA",
);
pub const SOF_ZMAN_TFILA_MGA_19_POINT_8_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(
        &CORE_ALOS_19_POINT_8_DEGREES,
        &CORE_TZAIS_19_POINT_8_DEGREES,
        true,
    ),
    "getSofZmanTfilaMGA19Point8Degrees",
);
pub const SOF_ZMAN_TFILA_MGA_16_POINT_1_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(
        &CORE_ALOS_16_POINT_1_DEGREES,
        &CORE_TZAIS_16_POINT_1_DEGREES,
        true,
    ),
    "getSofZmanTfilaMGA16Point1Degrees",
);
pub const SOF_ZMAN_TFILA_MGA_18_DEGREES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_18_DEGREES, &CORE_TZAIS_18_DEGREES, true),
    "getSofZmanTfilaMGA18Degrees",
);
pub const SOF_ZMAN_TFILA_MGA_72_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_MINUTES, &CORE_TZAIS_MINUTES_72, true),
    "getSofZmanTfilaMGA72Minutes",
);
pub const SOF_ZMAN_TFILA_MGA_72_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_72_ZMANIS, true),
    "getSofZmanTfilaMGA72MinutesZmanis",
);
pub const SOF_ZMAN_TFILA_MGA_90_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_90_MINUTES, &CORE_TZAIS_MINUTES_90, true),
    "getSofZmanTfilaMGA90Minutes",
);
pub const SOF_ZMAN_TFILA_MGA_90_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_90_ZMANIS, &CORE_TZAIS_90_ZMANIS, true),
    "getSofZmanTfilaMGA90MinutesZmanis",
);
pub const SOF_ZMAN_TFILA_MGA_96_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_96_MINUTES, &CORE_TZAIS_MINUTES_96, true),
    "getSofZmanTfilaMGA96Minutes",
);
pub const SOF_ZMAN_TFILA_MGA_96_ZMANIS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_96_ZMANIS, &CORE_TZAIS_96_ZMANIS, true),
    "getSofZmanTfilaMGA96MinutesZmanis",
);
pub const SOF_ZMAN_TFILA_HOURS_2_BEFORE_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&Event::Transit, Duration::minutes(-120)),
    "getSofZmanTfila2HoursBeforeChatzos",
);
pub const SOF_ZMAN_TFILA_MGA_120_MINUTES: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_120_MINUTES, &CORE_TZAIS_MINUTES_120, true),
    "getSofZmanTfilaMGA120Minutes",
);
pub const SOF_ZMAN_TFILA_ATERET_TORAH: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_ALOS_72_ZMANIS, &CORE_TZAIS_ATERET_TORAH, false),
    "getSofZmanTfilaAteretTorah",
);
pub const SOF_ZMAN_TFILA_BAAL_HATANYA: ZmanEvent<'static> = ZmanEvent::new(
    Event::Tefila(&CORE_BAAL_HATANYA_SUNRISE, &CORE_BAAL_HATANYA_SUNSET, true),
    "getSofZmanTfilaBaalHatanya",
);
pub const SOF_ZMAN_TFILA_FIXED_LOCAL: ZmanEvent<'static> = ZmanEvent::new(
    Event::Offset(&CORE_FIXED_LOCAL_CHATZOS, Duration::minutes(-120)),
    "getSofZmanTfilaFixedLocal",
);
pub const SOF_ZMAN_TFILA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS: ZmanEvent<'static> = ZmanEvent::new(
    Event::HalfDayBasedOffset(&Event::Sunrise, &CORE_FIXED_LOCAL_CHATZOS, 4.0),
    "getSofZmanTfilaGRASunriseToFixedLocalChatzos",
);

// ============================================================================
// TZAIS
// ============================================================================

pub const CORE_TZAIS_DEGREES_8_POINT_5: Event<'static> = Event::SunsetOffsetByDegrees(8.5);
pub const CORE_TZAIS_MINUTES_50: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(50));
pub const CORE_TZAIS_MINUTES_60: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(60));
pub const CORE_TZAIS_MINUTES_72: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(72));
pub const CORE_TZAIS_72_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 1.2);
pub const CORE_TZAIS_MINUTES_90: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(90));
pub const CORE_TZAIS_90_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 1.5);
pub const CORE_TZAIS_MINUTES_96: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(96));
pub const CORE_TZAIS_96_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 1.6);
pub const CORE_TZAIS_MINUTES_120: Event<'static> =
    Event::Offset(&Event::Sunset, Duration::minutes(120));
pub const CORE_TZAIS_120_ZMANIS: Event<'static> = Event::ZmanisOffset(&Event::Sunset, 2.0);
pub const CORE_TZAIS_16_POINT_1_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(16.1);
pub const CORE_TZAIS_18_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(18.0);
pub const CORE_TZAIS_19_POINT_8_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(19.8);
pub const CORE_TZAIS_26_DEGREES: Event<'static> = Event::SunsetOffsetByDegrees(26.0);
pub const CORE_TZAIS_ATERET_TORAH: Event<'static> = Event::AteretTorahSunset;
pub const CORE_TZAIS_BAAL_HATANYA: Event<'static> = Event::SunsetOffsetByDegrees(6.0);
pub const CORE_TZAIS_GEONIM_DEGREES_3_POINT_7: Event<'static> = Event::SunsetOffsetByDegrees(3.7);
pub const CORE_TZAIS_GEONIM_DEGREES_3_POINT_8: Event<'static> = Event::SunsetOffsetByDegrees(3.8);
pub const CORE_TZAIS_GEONIM_DEGREES_5_POINT_95: Event<'static> = Event::SunsetOffsetByDegrees(5.95);
pub const CORE_TZAIS_GEONIM_3_POINT_65: Event<'static> = Event::SunsetOffsetByDegrees(3.65);
pub const CORE_TZAIS_GEONIM_3_POINT_676: Event<'static> = Event::SunsetOffsetByDegrees(3.676);
pub const CORE_TZAIS_GEONIM_DEGREES_4_POINT_61: Event<'static> = Event::SunsetOffsetByDegrees(4.61);
pub const CORE_TZAIS_GEONIM_DEGREES_4_POINT_37: Event<'static> = Event::SunsetOffsetByDegrees(4.37);
pub const CORE_TZAIS_GEONIM_DEGREES_5_POINT_88: Event<'static> = Event::SunsetOffsetByDegrees(5.88);
pub const CORE_TZAIS_GEONIM_DEGREES_4_POINT_8: Event<'static> = Event::SunsetOffsetByDegrees(4.8);
pub const CORE_TZAIS_GEONIM_DEGREES_6_POINT_45: Event<'static> = Event::SunsetOffsetByDegrees(6.45);
pub const CORE_TZAIS_GEONIM_DEGREES_7_POINT_083: Event<'static> =
    Event::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0));
pub const CORE_TZAIS_GEONIM_DEGREES_7_POINT_67: Event<'static> = Event::SunsetOffsetByDegrees(7.67);
pub const CORE_TZAIS_GEONIM_DEGREES_8_POINT_5: Event<'static> = Event::SunsetOffsetByDegrees(8.5);
pub const CORE_TZAIS_GEONIM_DEGREES_9_POINT_3: Event<'static> = Event::SunsetOffsetByDegrees(9.3);
pub const CORE_TZAIS_GEONIM_DEGREES_9_POINT_75: Event<'static> = Event::SunsetOffsetByDegrees(9.75);

pub const TZAIS_DEGREES_8_POINT_5: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_DEGREES_8_POINT_5, "getTzaisGeonim8Point5Degrees");
pub const TZAIS_MINUTES_50: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_50, "getTzais50");
pub const TZAIS_MINUTES_60: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_60, "getTzais60");
pub const TZAIS_MINUTES_72: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_72, "getTzais72");
pub const TZAIS_72_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_72_ZMANIS, "getTzais72Zmanis");
pub const TZAIS_MINUTES_90: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_90, "getTzais90");
pub const TZAIS_90_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_90_ZMANIS, "getTzais90Zmanis");
pub const TZAIS_MINUTES_96: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_96, "getTzais96");
pub const TZAIS_96_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_96_ZMANIS, "getTzais96Zmanis");
pub const TZAIS_MINUTES_120: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_MINUTES_120, "getTzais120");
pub const TZAIS_120_ZMANIS: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_120_ZMANIS, "getTzais120Zmanis");
pub const TZAIS_16_POINT_1_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_16_POINT_1_DEGREES, "getTzais16Point1Degrees");
pub const TZAIS_18_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_18_DEGREES, "getTzais18Degrees");
pub const TZAIS_19_POINT_8_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_19_POINT_8_DEGREES, "getTzais19Point8Degrees");
pub const TZAIS_26_DEGREES: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_26_DEGREES, "getTzais26Degrees");
pub const TZAIS_ATERET_TORAH: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_ATERET_TORAH, "getTzaisAteretTorah");
pub const TZAIS_BAAL_HATANYA: ZmanEvent<'static> =
    ZmanEvent::new(CORE_TZAIS_BAAL_HATANYA, "getTzaisBaalHatanya");
pub const TZAIS_GEONIM_DEGREES_3_POINT_7: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_3_POINT_7,
    "getTzaisGeonim3Point7Degrees",
);
pub const TZAIS_GEONIM_DEGREES_3_POINT_8: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_3_POINT_8,
    "getTzaisGeonim3Point8Degrees",
);
pub const TZAIS_GEONIM_DEGREES_5_POINT_95: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_5_POINT_95,
    "getTzaisGeonim5Point95Degrees",
);
pub const TZAIS_GEONIM_3_POINT_65: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_3_POINT_65,
    "getTzaisGeonim3Point65Degrees",
);
pub const TZAIS_GEONIM_3_POINT_676: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_3_POINT_676,
    "getTzaisGeonim3Point676Degrees",
);
pub const TZAIS_GEONIM_DEGREES_4_POINT_61: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_4_POINT_61,
    "getTzaisGeonim4Point61Degrees",
);
pub const TZAIS_GEONIM_DEGREES_4_POINT_37: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_4_POINT_37,
    "getTzaisGeonim4Point37Degrees",
);
pub const TZAIS_GEONIM_DEGREES_5_POINT_88: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_5_POINT_88,
    "getTzaisGeonim5Point88Degrees",
);
pub const TZAIS_GEONIM_DEGREES_4_POINT_8: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_4_POINT_8,
    "getTzaisGeonim4Point8Degrees",
);
pub const TZAIS_GEONIM_DEGREES_6_POINT_45: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_6_POINT_45,
    "getTzaisGeonim6Point45Degrees",
);
pub const TZAIS_GEONIM_DEGREES_7_POINT_083: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_7_POINT_083,
    "getTzaisGeonim7Point083Degrees",
);
pub const TZAIS_GEONIM_DEGREES_7_POINT_67: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_7_POINT_67,
    "getTzaisGeonim7Point67Degrees",
);
pub const TZAIS_GEONIM_DEGREES_8_POINT_5: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_8_POINT_5,
    "getTzaisGeonim8Point5Degrees",
);
pub const TZAIS_GEONIM_DEGREES_9_POINT_3: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_9_POINT_3,
    "getTzaisGeonim9Point3Degrees",
);
pub const TZAIS_GEONIM_DEGREES_9_POINT_75: ZmanEvent<'static> = ZmanEvent::new(
    CORE_TZAIS_GEONIM_DEGREES_9_POINT_75,
    "getTzaisGeonim9Point75Degrees",
);
