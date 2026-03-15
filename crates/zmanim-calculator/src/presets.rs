//! Predefined zmanim calculations built from reusable primitives.
//!
//! Prefer these presets for standard zmanim usage. Reach for `primitive_zman` only when
//! you need to compose a custom calculation that is not already provided here.

use chrono::{Datelike, TimeZone};
use hebrew_holiday_calendar::MoladCalendar;
use icu_calendar::Date;

use crate::prelude::ZmanimCalculator;
use crate::types::error::ZmanimError;

use crate::{calculator::ZmanLike, primitive_zman::ZmanPrimitive};
use chrono::Duration;
use chrono::{DateTime, Utc};

/// A zman preset built from a low-level [`ZmanPrimitive`] definition.
///
/// Most users should consume these predefined presets directly rather than constructing
/// [`ZmanPrimitive`] values by hand.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ZmanPreset<'a> {
    /// The underlying low-level computation definition for this preset.
    event: ZmanPrimitive<'a>,
    #[cfg(feature = "_java_testing")]
    /// The KosherJava-style preset name used by parity tests.
    pub name: &'a str,
}

impl<'a> ZmanPreset<'a> {
    #[allow(unused)]
    const fn new(event: ZmanPrimitive<'a>, name: &'a str) -> Self {
        #[cfg(feature = "_java_testing")]
        return Self { event, name };
        #[cfg(not(feature = "_java_testing"))]
        return Self { event };
    }
}

impl<'a, Tz: TimeZone> ZmanLike<Tz> for ZmanPreset<'a> {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        self.event.calculate(calculator)
    }
}

/// Sunset (elevation-adjusted).
pub static SUNRISE: ZmanPreset<'static> = ZmanPreset::new(ZmanPrimitive::Sunrise, "getSunrise");
/// Sunrise at sea level (elevation `0m`).
pub static SEA_LEVEL_SUNRISE: ZmanPreset<'static> =
    ZmanPreset::new(ZmanPrimitive::SeaLevelSunrise, "getSeaLevelSunrise");

/// Sunset (elevation-adjusted).
pub static SUNSET: ZmanPreset<'static> = ZmanPreset::new(ZmanPrimitive::Sunset, "getSunset");
/// Sunset at sea level (elevation `0m`).
pub static SEA_LEVEL_SUNSET: ZmanPreset<'static> =
    ZmanPreset::new(ZmanPrimitive::SeaLevelSunset, "getSeaLevelSunset");

/// *Alos* as a fixed `60` minutes before sunrise.
pub static ALOS_60_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-60)),
    "getAlos60",
);
/// *Alos* as a fixed `72` minutes before sunrise.
pub static ALOS_72_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
    "getAlos72",
);
/// *Alos* as `72 zmaniyos` minutes before sunrise (1.2 *shaos zmaniyos*).
pub static ALOS_72_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
    "getAlos72Zmanis",
);
/// *Alos* as a fixed `90` minutes before sunrise.
pub static ALOS_90_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-90)),
    "getAlos90",
);
/// *Alos* as `90 zmaniyos` minutes before sunrise (1.5 *shaos zmaniyos*).
pub static ALOS_90_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.5),
    "getAlos90Zmanis",
);
/// *Alos* as a fixed `96` minutes before sunrise.
pub static ALOS_96_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-96)),
    "getAlos96",
);
/// *Alos* as `96 zmaniyos` minutes before sunrise (1.6 *shaos zmaniyos*).
pub static ALOS_96_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.6),
    "getAlos96Zmanis",
);
/// *Alos* as a fixed `120` minutes before sunrise.
pub static ALOS_120_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-120)),
    "getAlos120",
);
/// *Alos* as `120 zmaniyos` minutes before sunrise (2.0 *shaos zmaniyos*).
pub static ALOS_120_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -2.0),
    "getAlos120Zmanis",
);
/// *Alos* when the sun is `16.1°` below the geometric horizon (degrees-below-horizon dawn).
pub static ALOS_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(16.1),
    "getAlos16Point1Degrees",
);
/// *Alos* when the sun is `18°` below the geometric horizon (degrees-below-horizon dawn).
pub static ALOS_18_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(18.0),
    "getAlos18Degrees",
);
/// *Alos* when the sun is `19°` below the geometric horizon (degrees-below-horizon dawn).
pub static ALOS_19_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(19.0),
    "getAlos19Degrees",
);
/// *Alos* when the sun is `19.8°` below the geometric horizon (degrees-below-horizon dawn).
pub static ALOS_19_POINT_8_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(19.8),
    "getAlos19Point8Degrees",
);
/// *Alos* when the sun is `26°` below the geometric horizon (degrees-below-horizon dawn).
pub static ALOS_26_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(26.0),
    "getAlos26Degrees",
);
/// *Alos* when the sun is `16.9°` below the geometric horizon (degrees-below-horizon dawn).
pub static ALOS_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(16.9),
    "getAlosBaalHatanya",
);

/// Bain hashmashos (Rabbeinu Tam): when the sun is `13.24°` below the geometric horizon (after sunset).
pub static BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(13.24),
    "getBainHashmashosRT13Point24Degrees",
);
/// Bain hashmashos (Rabbeinu Tam): `58.5` minutes after sunset.
pub static BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(
        &ZmanPrimitive::Sunset,
        Duration::milliseconds((58.5 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosRT58Point5Minutes",
);
/// Bain hashmashos (Rabbeinu Tam): `13.5` minutes before when the sun will be `7.083°` below the geometric horizon.
pub static BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES: ZmanPreset<'static> =
    ZmanPreset::new(
        ZmanPrimitive::Offset(
            &ZmanPrimitive::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0)),
            Duration::milliseconds((-13.5 * 60.0 * 1000.0) as i64),
        ),
        "getBainHashmashosRT13Point5MinutesBefore7Point083Degrees",
    );
#[allow(missing_docs)]
pub struct BainHashmashosRt2Stars {
    _private: (),
}

impl<Tz: TimeZone> ZmanLike<Tz> for BainHashmashosRt2Stars {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let alos19_point_8 = ZmanPrimitive::SunriseOffsetByDegrees(19.8).calculate(calculator)?;
        let sunrise = ZmanPrimitive::Sunrise.calculate(calculator)?;
        let sunset = ZmanPrimitive::Sunset.calculate(calculator)?;
        let time_diff = sunrise.signed_duration_since(alos19_point_8);
        let offset = time_diff.num_milliseconds() as f64 * (5.0 / 18.0);
        Ok(sunset + Duration::milliseconds(offset as i64))
    }
}

#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for BainHashmashosRt2Stars {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        true
    }
    fn name(&self) -> &str {
        "getBainHashmashosRT2Stars"
    }
}

/// Bain hashmashos (Rabbeinu Tam, 2-stars): `sunset + (sunrise - alos19.8°) * 5/18`.
pub static BAIN_HASHMASHOS_RT_2_STARS: BainHashmashosRt2Stars =
    BainHashmashosRt2Stars { _private: () };
/// Bain hashmashos (Yereim): `18` minutes before sunset.
pub static BAIN_HASHMASHOS_YEREIM_18_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(-18)),
    "getBainHashmashosYereim18Minutes",
);
/// Bain hashmashos (Yereim): `16.875` minutes before sunset.
pub static BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(
        &ZmanPrimitive::Sunset,
        Duration::milliseconds((-16.875 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosYereim16Point875Minutes",
);
/// Bain hashmashos (Yereim): `13.5` minutes before sunset.
pub static BAIN_HASHMASHOS_YEREIM_13_POINT_5_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(
        &ZmanPrimitive::Sunset,
        Duration::milliseconds((-13.5 * 60.0 * 1000.0) as i64),
    ),
    "getBainHashmashosYereim13Point5Minutes",
);

/// Candle lighting: sea-level sunset minus [`crate::types::config::CalculatorConfig::candle_lighting_offset`].
pub static CANDLE_LIGHTING: ZmanPreset<'static> =
    ZmanPreset::new(ZmanPrimitive::CandleLighting, "getCandleLighting");

// We configure the Java calendar to use astronomical noon  for getChatzos() so that this java method is the equivalent of this rust preset.
/// Chatzos (astronomical noon): solar transit.
pub static CHATZOS_ASTRONOMICAL: ZmanPreset<'static> =
    ZmanPreset::new(ZmanPrimitive::SolarTransit, "getChatzos");
/// Chatzos (half-day): midpoint between sea-level sunrise and sea-level sunset.
pub static CHATZOS_HALF_DAY: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::HalfDayBasedOffset(
        &ZmanPrimitive::SeaLevelSunrise,
        &ZmanPrimitive::SeaLevelSunset,
        3.0,
    ),
    "getChatzosAsHalfDay",
);
/// Chatzos (fixed local): 12:00 local mean time.
pub static CHATZOS_FIXED_LOCAL: ZmanPreset<'static> =
    ZmanPreset::new(ZmanPrimitive::LocalMeanTime(12.0), "getFixedLocalChatzos");

/// Mincha gedola: `6.5` shaos after sunrise (or `0.5` shaah after chatzos if configured).
pub static MINCHA_GEDOLA_SUNRISE_SUNSET: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaGedola(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, true),
    "getMinchaGedola",
);
/// Mincha gedola: `6.5` shaos after alos `16.1°` (or `0.5` shaah after chatzos if configured).
pub static MINCHA_GEDOLA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaGedola(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        true,
    ),
    "getMinchaGedola16Point1Degrees",
);
/// Mincha gedola: `30` minutes after solar transit.
pub static MINCHA_GEDOLA_MINUTES_30: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::SolarTransit, Duration::minutes(30)),
    "getMinchaGedola30Minutes",
);
/// Mincha gedola: `6.5` shaos after alos `72` minutes (or `0.5` shaah after chatzos if configured).
pub static MINCHA_GEDOLA_MINUTES_72: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaGedola(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getMinchaGedola72Minutes",
);

#[allow(missing_docs)]
pub struct MinchaGedolaAhavatShalom {
    _private: (),
}

impl<Tz: TimeZone> ZmanLike<Tz> for MinchaGedolaAhavatShalom {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let chatzos = ZmanPrimitive::SolarTransit.calculate(calculator)?;
        let mincha_gedola_30 = chatzos + Duration::minutes(30);

        let alos = ALOS_16_POINT_1_DEGREES.calculate(calculator)?;
        let tzais = TZAIS_GEONIM_DEGREES_3_POINT_7.calculate(calculator)?;
        let shaah_zmanis = (tzais - alos) / 12;
        let mincha_alternative = chatzos + (shaah_zmanis / 2);
        if mincha_gedola_30 > mincha_alternative {
            Ok(mincha_gedola_30)
        } else {
            Ok(mincha_alternative)
        }
    }
}

#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for MinchaGedolaAhavatShalom {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        true
    }
    fn name(&self) -> &str {
        "getMinchaGedolaAhavatShalom"
    }
}
/// Mincha gedola (Ahavat Shalom): later of `chatzos + 30m` and `chatzos + 1/2 shaah`.
pub static MINCHA_GEDOLA_AHAVAT_SHALOM: MinchaGedolaAhavatShalom =
    MinchaGedolaAhavatShalom { _private: () };
/// Mincha gedola: `6.5` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub static MINCHA_GEDOLA_ATERET_TORAH: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaGedola(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::TzaisAteretTorah,
        false,
    ),
    "getMinchaGedolaAteretTorah",
);
/// Mincha gedola: `6.5` shaos after Baal HaTanya day start (or `0.5` shaah after chatzos if configured).
pub static MINCHA_GEDOLA_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaGedola(
        &ZmanPrimitive::SunriseOffsetByDegrees(1.583),
        &ZmanPrimitive::SunsetOffsetByDegrees(1.583),
        true,
    ),
    "getMinchaGedolaBaalHatanya",
);

#[allow(missing_docs)]
pub struct MinchaGedolaBaalHatanyaGreaterThan30 {
    _private: (),
}

impl<Tz: TimeZone> ZmanLike<Tz> for MinchaGedolaBaalHatanyaGreaterThan30 {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let mincha_30 = MINCHA_GEDOLA_MINUTES_30.calculate(calculator)?;
        let mincha_baal_hatanya = MINCHA_GEDOLA_BAAL_HATANYA.calculate(calculator)?;
        if mincha_30 > mincha_baal_hatanya {
            Ok(mincha_30)
        } else {
            Ok(mincha_baal_hatanya)
        }
    }
}

#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for MinchaGedolaBaalHatanyaGreaterThan30 {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        true
    }
    fn name(&self) -> &str {
        "getMinchaGedolaBaalHatanyaGreaterThan30"
    }
}

/// Mincha gedola: later of Baal HaTanya mincha gedola and `30` minutes after solar transit.
pub static MINCHA_GEDOLA_BAAL_HATANYA_GREATER_THAN_30: MinchaGedolaBaalHatanyaGreaterThan30 =
    MinchaGedolaBaalHatanyaGreaterThan30 { _private: () };
/// Mincha gedola: `30` minutes after fixed local chatzos (12:00 local mean time).
pub static MINCHA_GEDOLA_GRA_FIXED_LOCAL_CHATZOS_30_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::LocalMeanTime(12.0), Duration::minutes(30)),
    "getMinchaGedolaGRAFixedLocalChatzos30Minutes",
);
#[allow(missing_docs)]
pub struct MinchaGedolaGreaterThan30 {
    _private: (),
}

impl<Tz: TimeZone> ZmanLike<Tz> for MinchaGedolaGreaterThan30 {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let mincha_30 = MINCHA_GEDOLA_MINUTES_30.calculate(calculator)?;
        let mincha_regular = MINCHA_GEDOLA_SUNRISE_SUNSET.calculate(calculator)?;
        if mincha_30 > mincha_regular {
            Ok(mincha_30)
        } else {
            Ok(mincha_regular)
        }
    }
}

#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for MinchaGedolaGreaterThan30 {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        true
    }
    fn name(&self) -> &str {
        "getMinchaGedolaGreaterThan30"
    }
}

/// Mincha gedola: later of [`MINCHA_GEDOLA_SUNRISE_SUNSET`] and [`MINCHA_GEDOLA_MINUTES_30`].
pub static MINCHA_GEDOLA_GREATER_THAN_30: MinchaGedolaGreaterThan30 =
    MinchaGedolaGreaterThan30 { _private: () };

/// Mincha ketana: `9.5` shaos after sunrise (or `3.5` shaos after chatzos if configured).
pub static MINCHA_KETANA_SUNRISE_SUNSET: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaKetana(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, true),
    "getMinchaKetana",
);
/// Mincha ketana: `9.5` shaos after alos `16.1°` (or `3.5` shaos after chatzos if configured).
pub static MINCHA_KETANA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaKetana(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        true,
    ),
    "getMinchaKetana16Point1Degrees",
);
/// Mincha ketana: `9.5` shaos after alos `72` minutes (or `3.5` shaos after chatzos if configured).
pub static MINCHA_KETANA_MINUTES_72: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaKetana(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getMinchaKetana72Minutes",
);
#[allow(missing_docs)]
pub struct MinchaKetanaAhavatShalom {
    _private: (),
}

impl<Tz: TimeZone> ZmanLike<Tz> for MinchaKetanaAhavatShalom {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let tzais = TZAIS_GEONIM_DEGREES_3_POINT_8.calculate(calculator)?;
        let alos = ALOS_16_POINT_1_DEGREES.calculate(calculator)?;
        let shaah_zmanis = (tzais - alos) / 12;
        Ok(tzais - (shaah_zmanis * 5 / 2))
    }
}

#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for MinchaKetanaAhavatShalom {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        true
    }
    fn name(&self) -> &str {
        "getMinchaKetanaAhavatShalom"
    }
}

/// Mincha ketana (Ahavat Shalom): `2.5` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
pub static MINCHA_KETANA_AHAVAT_SHALOM: MinchaKetanaAhavatShalom =
    MinchaKetanaAhavatShalom { _private: () };
/// Mincha ketana: `9.5` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub static MINCHA_KETANA_ATERET_TORAH: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaKetana(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::TzaisAteretTorah,
        false,
    ),
    "getMinchaKetanaAteretTorah",
);
/// Mincha ketana: `9.5` shaos after Baal HaTanya day start (or `3.5` shaos after chatzos if configured).
pub static MINCHA_KETANA_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::MinchaKetana(
        &ZmanPrimitive::SunriseOffsetByDegrees(1.583),
        &ZmanPrimitive::SunsetOffsetByDegrees(1.583),
        true,
    ),
    "getMinchaKetanaBaalHatanya",
);
/// Mincha ketana: `3.5` shaos zmaniyos after fixed local chatzos, using fixed-local-chatzos → sunset half-day.
pub static MINCHA_KETANA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::HalfDayBasedOffset(
        &ZmanPrimitive::LocalMeanTime(12.0),
        &ZmanPrimitive::Sunset,
        3.5,
    ),
    "getMinchaKetanaGRAFixedLocalChatzosToSunset",
);

/// Misheyakir when the sun is `10.2°` below the geometric horizon (degrees-below-horizon dawn).
pub static MISHEYAKIR_10_POINT_2_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(10.2),
    "getMisheyakir10Point2Degrees",
);
/// Misheyakir when the sun is `11°` below the geometric horizon (degrees-below-horizon dawn).
pub static MISHEYAKIR_11_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(11.0),
    "getMisheyakir11Degrees",
);
/// Misheyakir when the sun is `11.5°` below the geometric horizon (degrees-below-horizon dawn).
pub static MISHEYAKIR_11_POINT_5_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(11.5),
    "getMisheyakir11Point5Degrees",
);
/// Misheyakir when the sun is `7.65°` below the geometric horizon (degrees-below-horizon dawn).
pub static MISHEYAKIR_7_POINT_65_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(7.65),
    "getMisheyakir7Point65Degrees",
);
/// Misheyakir when the sun is `9.5°` below the geometric horizon (degrees-below-horizon dawn).
pub static MISHEYAKIR_9_POINT_5_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunriseOffsetByDegrees(9.5),
    "getMisheyakir9Point5Degrees",
);

/// Plag hamincha: `10.75` shaos after sunrise (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_SUNRISE_SUNSET: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, true),
    "getPlagHamincha",
);

#[allow(missing_docs)]
pub struct PlagAhavatShalom {
    _private: (),
}

impl<Tz: TimeZone> ZmanLike<Tz> for PlagAhavatShalom {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let tzais = ZmanPrimitive::SunsetOffsetByDegrees(3.8).calculate(calculator)?;
        let alos = ZmanPrimitive::SunriseOffsetByDegrees(16.1).calculate(calculator)?;
        let shaah_zmanis = (tzais - alos) / 12;
        Ok(tzais - (shaah_zmanis * 5 / 4))
    }
}

#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for PlagAhavatShalom {
    #[cfg(feature = "_java_testing")]
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        true
    }
    #[cfg(feature = "_java_testing")]
    fn name(&self) -> &str {
        "getPlagAhavatShalom"
    }
}

/// Plag hamincha (Ahavat Shalom): `1.25` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°).
pub static PLAG_HAMINCHA_AHAVAT_SHALOM: PlagAhavatShalom = PlagAhavatShalom { _private: () };
/// Plag hamincha: `10.75` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais7.083°).
pub static PLAG_HAMINCHA_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083: ZmanPreset<'static> =
    ZmanPreset::new(
        ZmanPrimitive::PlagHamincha(
            &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
            &ZmanPrimitive::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0)),
            false,
        ),
        "getPlagAlos16Point1ToTzaisGeonim7Point083Degrees",
    );
/// Plag hamincha: `10.75` shaos zmaniyos after alos `16.1°` (day end = sunset).
pub static PLAG_HAMINCHA_ALOS_TO_SUNSET: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::Sunset,
        false,
    ),
    "getPlagAlosToSunset",
);
/// Plag hamincha: `10.75` shaos after alos `60` minutes (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_60_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-60)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(60)),
        true,
    ),
    "getPlagHamincha60Minutes",
);
/// Plag hamincha: `10.75` shaos after alos `72` minutes (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_72_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getPlagHamincha72Minutes",
);
/// Plag hamincha: `10.75` shaos after alos `72 zmaniyos` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_72_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.2),
        true,
    ),
    "getPlagHamincha72MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos after alos `90` minutes (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_90_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-90)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(90)),
        true,
    ),
    "getPlagHamincha90Minutes",
);
/// Plag hamincha: `10.75` shaos after alos `90 zmaniyos` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_90_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.5),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.5),
        true,
    ),
    "getPlagHamincha90MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos after alos `96` minutes (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_96_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-96)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(96)),
        true,
    ),
    "getPlagHamincha96Minutes",
);
/// Plag hamincha: `10.75` shaos after alos `96 zmaniyos` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_96_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.6),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.6),
        true,
    ),
    "getPlagHamincha96MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos after alos `120` minutes (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_120_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-120)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(120)),
        true,
    ),
    "getPlagHamincha120Minutes",
);
/// Plag hamincha: `10.75` shaos after alos `120 zmaniyos` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_120_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -2.0),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 2.0),
        true,
    ),
    "getPlagHamincha120MinutesZmanis",
);
/// Plag hamincha: `10.75` shaos after alos `16.1°` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        true,
    ),
    "getPlagHamincha16Point1Degrees",
);
/// Plag hamincha: `10.75` shaos after alos `18°` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_18_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::SunriseOffsetByDegrees(18.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(18.0),
        true,
    ),
    "getPlagHamincha18Degrees",
);
/// Plag hamincha: `10.75` shaos after alos `19.8°` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_19_POINT_8_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::SunriseOffsetByDegrees(19.8),
        &ZmanPrimitive::SunsetOffsetByDegrees(19.8),
        true,
    ),
    "getPlagHamincha19Point8Degrees",
);
/// Plag hamincha: `10.75` shaos after alos `26°` (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_26_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::SunriseOffsetByDegrees(26.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(26.0),
        true,
    ),
    "getPlagHamincha26Degrees",
);
/// Plag hamincha: `10.75` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub static PLAG_HAMINCHA_ATERET_TORAH: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::TzaisAteretTorah,
        false,
    ),
    "getPlagHaminchaAteretTorah",
);
/// Plag hamincha: `10.75` shaos after Baal HaTanya day start (or `4.75` shaos after chatzos if configured).
pub static PLAG_HAMINCHA_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::PlagHamincha(
        &ZmanPrimitive::SunriseOffsetByDegrees(1.583),
        &ZmanPrimitive::SunsetOffsetByDegrees(1.583),
        true,
    ),
    "getPlagHaminchaBaalHatanya",
);
/// Plag hamincha: `4.75` shaos zmaniyos after fixed local chatzos, using fixed-local-chatzos → sunset half-day.
pub static PLAG_HAMINCHA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::HalfDayBasedOffset(
        &ZmanPrimitive::LocalMeanTime(12.0),
        &ZmanPrimitive::Sunset,
        4.75,
    ),
    "getPlagHaminchaGRAFixedLocalChatzosToSunset",
);

/// Samuch le-mincha ketana: `9` shaos after sunrise (or `3` shaos after chatzos if configured).
pub static SAMUCH_LE_MINCHA_KETANA_GRA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SamuchLeMinchaKetana(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, true),
    "getSamuchLeMinchaKetanaGRA",
);
/// Samuch le-mincha ketana: `9` shaos after alos `16.1°` (or `3` shaos after chatzos if configured).
pub static SAMUCH_LE_MINCHA_KETANA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SamuchLeMinchaKetana(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        true,
    ),
    "getSamuchLeMinchaKetana16Point1Degrees",
);
/// Samuch le-mincha ketana: `9` shaos after alos `72` minutes (or `3` shaos after chatzos if configured).
pub static SAMUCH_LE_MINCHA_KETANA_72_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SamuchLeMinchaKetana(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getSamuchLeMinchaKetana72Minutes",
);

/// Sof zman achilas chametz: `4` shaos after sunrise (or half-day based if configured).
pub static SOF_ZMAN_ACHILAS_CHAMETZ_GRA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, true),
    "getSofZmanAchilasChametzGRA",
);
/// Sof zman achilas chametz: `4` shaos after alos `72` minutes (or half-day based if configured).
pub static SOF_ZMAN_ACHILAS_CHAMETZ_MGA_72_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getSofZmanAchilasChametzMGA72Minutes",
);
/// Sof zman achilas chametz: `4` shaos after alos `16.1°` (or half-day based if configured).
pub static SOF_ZMAN_ACHILAS_CHAMETZ_MGA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        true,
    ),
    "getSofZmanAchilasChametzMGA16Point1Degrees",
);
/// Sof zman achilas chametz: `4` shaos after Baal HaTanya day start (or half-day based if configured).
pub static SOF_ZMAN_ACHILAS_CHAMETZ_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(1.583),
        &ZmanPrimitive::SunsetOffsetByDegrees(1.583),
        true,
    ),
    "getSofZmanAchilasChametzBaalHatanya",
);

/// Sof zman biur chametz: `5` shaos zmaniyos after sunrise (day = sunrise → sunset).
pub static SOF_ZMAN_BIUR_CHAMETZ_GRA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, 5.0),
    "getSofZmanBiurChametzGRA",
);
/// Sof zman biur chametz: `5` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72).
pub static SOF_ZMAN_BIUR_CHAMETZ_MGA_72_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ShaahZmanisBasedOffset(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        5.0,
    ),
    "getSofZmanBiurChametzMGA72Minutes",
);
/// Sof zman biur chametz: `5` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°).
pub static SOF_ZMAN_BIUR_CHAMETZ_MGA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ShaahZmanisBasedOffset(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        5.0,
    ),
    "getSofZmanBiurChametzMGA16Point1Degrees",
);
/// Sof zman biur chametz: `5` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset).
pub static SOF_ZMAN_BIUR_CHAMETZ_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ShaahZmanisBasedOffset(
        &ZmanPrimitive::SunriseOffsetByDegrees(1.583),
        &ZmanPrimitive::SunsetOffsetByDegrees(1.583),
        5.0,
    ),
    "getSofZmanBiurChametzBaalHatanya",
);

/// Sof zman shma: `3` shaos after sunrise (or half-day based if configured).
pub static SOF_ZMAN_SHMA_GRA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, true),
    "getSofZmanShmaGRA",
);
/// Sof zman shma: `3` shaos after alos `72` minutes (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getSofZmanShmaMGA",
);
/// Sof zman shma: `3` shaos after alos `19.8°` (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_19_POINT_8_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(19.8),
        &ZmanPrimitive::SunsetOffsetByDegrees(19.8),
        true,
    ),
    "getSofZmanShmaMGA19Point8Degrees",
);
/// Sof zman shma: `3` shaos after alos `16.1°` (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        true,
    ),
    "getSofZmanShmaMGA16Point1Degrees",
);
/// Sof zman shma: `3` shaos after alos `18°` (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_18_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(18.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(18.0),
        true,
    ),
    "getSofZmanShmaMGA18Degrees",
);
/// Sof zman shma: `3` shaos after alos `72` minutes (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_72_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getSofZmanShmaMGA72Minutes",
);
/// Sof zman shma: `3` shaos after alos `72 zmaniyos` (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_72_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.2),
        true,
    ),
    "getSofZmanShmaMGA72MinutesZmanis",
);
/// Sof zman shma: `3` shaos after alos `90` minutes (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_90_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-90)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(90)),
        true,
    ),
    "getSofZmanShmaMGA90Minutes",
);
/// Sof zman shma: `3` shaos after alos `90 zmaniyos` (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_90_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.5),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.5),
        true,
    ),
    "getSofZmanShmaMGA90MinutesZmanis",
);
/// Sof zman shma: `3` shaos after alos `96` minutes (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_96_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-96)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(96)),
        true,
    ),
    "getSofZmanShmaMGA96Minutes",
);
/// Sof zman shma: `3` shaos after alos `96 zmaniyos` (or half-day based if configured).
pub static SOF_ZMAN_SHMA_MGA_96_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.6),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.6),
        true,
    ),
    "getSofZmanShmaMGA96MinutesZmanis",
);
/// Sof zman shma: `3` hours before solar transit.
pub static SOF_ZMAN_SHMA_HOURS_3_BEFORE_CHATZOS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::SolarTransit, Duration::minutes(-180)),
    "getSofZmanShma3HoursBeforeChatzos",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `120` minutes (day = alos120 → tzais120).
pub static SOF_ZMAN_SHMA_MGA_120_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-120)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(120)),
        true,
    ),
    "getSofZmanShmaMGA120Minutes",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = sunset).
pub static SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_SUNSET: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::Sunset,
        false,
    ),
    "getSofZmanShmaAlos16Point1ToSunset",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = tzais7.083°).
pub static SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083: ZmanPreset<'static> =
    ZmanPreset::new(
        ZmanPrimitive::Shema(
            &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
            &ZmanPrimitive::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0)),
            false,
        ),
        "getSofZmanShmaAlos16Point1ToTzaisGeonim7Point083Degrees",
    );
/// Sof zman shma: `3` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub static SOF_ZMAN_SHMA_ATERET_TORAH: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::TzaisAteretTorah,
        false,
    ),
    "getSofZmanShmaAteretTorah",
);
/// Sof zman shma: `3` shaos after Baal HaTanya day start (or half-day based if configured).
pub static SOF_ZMAN_SHMA_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Shema(
        &ZmanPrimitive::SunriseOffsetByDegrees(1.583),
        &ZmanPrimitive::SunsetOffsetByDegrees(1.583),
        true,
    ),
    "getSofZmanShmaBaalHatanya",
);

/// Sof zman shma: `3` shaos zmaniyos after sunrise (day end = fixed local chatzos).
pub static SOF_ZMAN_SHMA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::HalfDayBasedOffset(
        &ZmanPrimitive::Sunrise,
        &ZmanPrimitive::LocalMeanTime(12.0),
        3.0,
    ),
    "getSofZmanShmaGRASunriseToFixedLocalChatzos",
);
/// Sof zman shma: `3` shaos zmaniyos after alos `18°` (day end = fixed local chatzos).
pub static SOF_ZMAN_SHMA_MGA_18_DEGREES_TO_FIXED_LOCAL_CHATZOS: ZmanPreset<'static> =
    ZmanPreset::new(
        ZmanPrimitive::HalfDayBasedOffset(
            &ZmanPrimitive::SunriseOffsetByDegrees(18.0),
            &ZmanPrimitive::LocalMeanTime(12.0),
            3.0,
        ),
        "getSofZmanShmaMGA18DegreesToFixedLocalChatzos",
    );
/// Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = fixed local chatzos).
pub static SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES_TO_FIXED_LOCAL_CHATZOS: ZmanPreset<'static> =
    ZmanPreset::new(
        ZmanPrimitive::HalfDayBasedOffset(
            &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
            &ZmanPrimitive::LocalMeanTime(12.0),
            3.0,
        ),
        "getSofZmanShmaMGA16Point1DegreesToFixedLocalChatzos",
    );
/// Sof zman shma: `3` shaos zmaniyos after alos `90` minutes (day end = fixed local chatzos).
pub static SOF_ZMAN_SHMA_MGA_90_MINUTES_TO_FIXED_LOCAL_CHATZOS: ZmanPreset<'static> =
    ZmanPreset::new(
        ZmanPrimitive::HalfDayBasedOffset(
            &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-90)),
            &ZmanPrimitive::LocalMeanTime(12.0),
            3.0,
        ),
        "getSofZmanShmaMGA90MinutesToFixedLocalChatzos",
    );
/// Sof zman shma: `3` shaos zmaniyos after alos `72` minutes (day end = fixed local chatzos).
pub static SOF_ZMAN_SHMA_MGA_72_MINUTES_TO_FIXED_LOCAL_CHATZOS: ZmanPreset<'static> =
    ZmanPreset::new(
        ZmanPrimitive::HalfDayBasedOffset(
            &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
            &ZmanPrimitive::LocalMeanTime(12.0),
            3.0,
        ),
        "getSofZmanShmaMGA72MinutesToFixedLocalChatzos",
    );

/// Sof zman tfila: `4` shaos after sunrise (or half-day based if configured).
pub static SOF_ZMAN_TFILA_GRA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(&ZmanPrimitive::Sunrise, &ZmanPrimitive::Sunset, true),
    "getSofZmanTfilaGRA",
);
/// Sof zman tfila: `4` shaos after alos `72` minutes (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getSofZmanTfilaMGA",
);
/// Sof zman tfila: `4` shaos after alos `19.8°` (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_19_POINT_8_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(19.8),
        &ZmanPrimitive::SunsetOffsetByDegrees(19.8),
        true,
    ),
    "getSofZmanTfilaMGA19Point8Degrees",
);
/// Sof zman tfila: `4` shaos after alos `16.1°` (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(16.1),
        &ZmanPrimitive::SunsetOffsetByDegrees(16.1),
        true,
    ),
    "getSofZmanTfilaMGA16Point1Degrees",
);
/// Sof zman tfila: `4` shaos after alos `18°` (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_18_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(18.0),
        &ZmanPrimitive::SunsetOffsetByDegrees(18.0),
        true,
    ),
    "getSofZmanTfilaMGA18Degrees",
);
/// Sof zman tfila: `4` shaos after alos `72` minutes (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_72_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-72)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
        true,
    ),
    "getSofZmanTfilaMGA72Minutes",
);
/// Sof zman tfila: `4` shaos after alos `72 zmaniyos` (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_72_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.2),
        true,
    ),
    "getSofZmanTfilaMGA72MinutesZmanis",
);
/// Sof zman tfila: `4` shaos after alos `90` minutes (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_90_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-90)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(90)),
        true,
    ),
    "getSofZmanTfilaMGA90Minutes",
);
/// Sof zman tfila: `4` shaos after alos `90 zmaniyos` (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_90_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.5),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.5),
        true,
    ),
    "getSofZmanTfilaMGA90MinutesZmanis",
);
/// Sof zman tfila: `4` shaos after alos `96` minutes (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_96_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-96)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(96)),
        true,
    ),
    "getSofZmanTfilaMGA96Minutes",
);
/// Sof zman tfila: `4` shaos after alos `96 zmaniyos` (or half-day based if configured).
pub static SOF_ZMAN_TFILA_MGA_96_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.6),
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.6),
        true,
    ),
    "getSofZmanTfilaMGA96MinutesZmanis",
);
/// Sof zman tfila: `2` hours before solar transit.
pub static SOF_ZMAN_TFILA_HOURS_2_BEFORE_CHATZOS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::SolarTransit, Duration::minutes(-120)),
    "getSofZmanTfila2HoursBeforeChatzos",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `120` minutes (day = alos120 → tzais120).
pub static SOF_ZMAN_TFILA_MGA_120_MINUTES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunrise, Duration::minutes(-120)),
        &ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(120)),
        true,
    ),
    "getSofZmanTfilaMGA120Minutes",
);
/// Sof zman tfila: `4` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais).
pub static SOF_ZMAN_TFILA_ATERET_TORAH: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunrise, -1.2),
        &ZmanPrimitive::TzaisAteretTorah,
        false,
    ),
    "getSofZmanTfilaAteretTorah",
);
/// Sof zman tfila: `4` shaos after Baal HaTanya day start (or half-day based if configured).
pub static SOF_ZMAN_TFILA_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Tefila(
        &ZmanPrimitive::SunriseOffsetByDegrees(1.583),
        &ZmanPrimitive::SunsetOffsetByDegrees(1.583),
        true,
    ),
    "getSofZmanTfilaBaalHatanya",
);
/// Sof zman tfila: `4` shaos zmaniyos after sunrise (day end = fixed local chatzos).
pub static SOF_ZMAN_TFILA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::HalfDayBasedOffset(
        &ZmanPrimitive::Sunrise,
        &ZmanPrimitive::LocalMeanTime(12.0),
        4.0,
    ),
    "getSofZmanTfilaGRASunriseToFixedLocalChatzos",
);

/// Tzais when the sun is `8.5°` below the geometric horizon (after sunset).
pub static TZAIS_DEGREES_8_POINT_5: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(8.5),
    "getTzaisGeonim8Point5Degrees",
);
/// Tzais: `50` minutes after sunset.
pub static TZAIS_MINUTES_50: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(50)),
    "getTzais50",
);
/// Tzais: `60` minutes after sunset.
pub static TZAIS_MINUTES_60: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(60)),
    "getTzais60",
);
/// Tzais: `72` minutes after sunset.
pub static TZAIS_MINUTES_72: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(72)),
    "getTzais72",
);
/// Tzais: `72 zmaniyos` minutes after sunset (1.2 *shaos zmaniyos*).
pub static TZAIS_72_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.2),
    "getTzais72Zmanis",
);
/// Tzais: `90` minutes after sunset.
pub static TZAIS_MINUTES_90: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(90)),
    "getTzais90",
);
/// Tzais: `90 zmaniyos` minutes after sunset (1.5 *shaos zmaniyos*).
pub static TZAIS_90_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.5),
    "getTzais90Zmanis",
);
/// Tzais: `96` minutes after sunset.
pub static TZAIS_MINUTES_96: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(96)),
    "getTzais96",
);
/// Tzais: `96 zmaniyos` minutes after sunset (1.6 *shaos zmaniyos*).
pub static TZAIS_96_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 1.6),
    "getTzais96Zmanis",
);
/// Tzais: `120` minutes after sunset.
pub static TZAIS_MINUTES_120: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::Offset(&ZmanPrimitive::Sunset, Duration::minutes(120)),
    "getTzais120",
);
/// Tzais: `120 zmaniyos` minutes after sunset (2.0 *shaos zmaniyos*).
pub static TZAIS_120_ZMANIS: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::ZmanisOffset(&ZmanPrimitive::Sunset, 2.0),
    "getTzais120Zmanis",
);
/// Tzais when the sun is `16.1°` below the geometric horizon (after sunset).
pub static TZAIS_16_POINT_1_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(16.1),
    "getTzais16Point1Degrees",
);
/// Tzais when the sun is `18°` below the geometric horizon (after sunset).
pub static TZAIS_18_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(18.0),
    "getTzais18Degrees",
);
/// Tzais when the sun is `19.8°` below the geometric horizon (after sunset).
pub static TZAIS_19_POINT_8_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(19.8),
    "getTzais19Point8Degrees",
);
/// Tzais when the sun is `26°` below the geometric horizon (after sunset).
pub static TZAIS_26_DEGREES: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(26.0),
    "getTzais26Degrees",
);
/// Tzais (Ateret Torah): (elevation-adjusted) sunset plus [`crate::types::config::CalculatorConfig::ateret_torah_sunset_offset`].
pub static TZAIS_ATERET_TORAH: ZmanPreset<'static> =
    ZmanPreset::new(ZmanPrimitive::TzaisAteretTorah, "getTzaisAteretTorah");
/// Tzais (Baal HaTanya): when the sun is `6°` below the geometric horizon (after sunset).
pub static TZAIS_BAAL_HATANYA: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(6.0),
    "getTzaisBaalHatanya",
);
/// Tzais (Geonim): when the sun is `3.7°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_3_POINT_7: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(3.7),
    "getTzaisGeonim3Point7Degrees",
);
/// Tzais (Geonim): when the sun is `3.8°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_3_POINT_8: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(3.8),
    "getTzaisGeonim3Point8Degrees",
);
/// Tzais (Geonim): when the sun is `5.95°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_5_POINT_95: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(5.95),
    "getTzaisGeonim5Point95Degrees",
);
/// Tzais (Geonim): when the sun is `4.61°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_4_POINT_61: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(4.61),
    "getTzaisGeonim4Point61Degrees",
);
/// Tzais (Geonim): when the sun is `4.37°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_4_POINT_37: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(4.37),
    "getTzaisGeonim4Point37Degrees",
);
/// Tzais (Geonim): when the sun is `5.88°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_5_POINT_88: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(5.88),
    "getTzaisGeonim5Point88Degrees",
);
/// Tzais (Geonim): when the sun is `4.8°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_4_POINT_8: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(4.8),
    "getTzaisGeonim4Point8Degrees",
);
/// Tzais (Geonim): when the sun is `6.45°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_6_POINT_45: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(6.45),
    "getTzaisGeonim6Point45Degrees",
);
/// Tzais (Geonim): when the sun is `7.083°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_7_POINT_083: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(7.0 + (5.0 / 60.0)),
    "getTzaisGeonim7Point083Degrees",
);
/// Tzais (Geonim): when the sun is `7.67°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_7_POINT_67: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(7.67),
    "getTzaisGeonim7Point67Degrees",
);
/// Tzais (Geonim): when the sun is `8.5°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_8_POINT_5: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(8.5),
    "getTzaisGeonim8Point5Degrees",
);
/// Tzais (Geonim): when the sun is `9.3°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_9_POINT_3: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(9.3),
    "getTzaisGeonim9Point3Degrees",
);
/// Tzais (Geonim): when the sun is `9.75°` below the geometric horizon (after sunset).
pub static TZAIS_GEONIM_DEGREES_9_POINT_75: ZmanPreset<'static> = ZmanPreset::new(
    ZmanPrimitive::SunsetOffsetByDegrees(9.75),
    "getTzaisGeonim9Point75Degrees",
);

#[allow(missing_docs)]
pub struct SofZmanKidushLevana15Days {
    _private: (),
}
impl<Tz: TimeZone> ZmanLike<Tz> for SofZmanKidushLevana15Days {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let tz = calculator
            .location
            .timezone
            .as_ref()
            .ok_or(ZmanimError::TimeZoneRequired)?;
        let date = Date::try_new_gregorian(
            calculator.date.year(),
            calculator.date.month() as u8,
            calculator.date.day() as u8,
        )
        .map_err(|_| ZmanimError::TimeConversionError)?;
        date.sof_zman_kidush_levana_15_days(tz)
            .map(|i| i.0.to_utc())
            .ok_or(ZmanimError::TimeConversionError)
    }
}
#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for SofZmanKidushLevana15Days {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        false
    }
    fn name(&self) -> &str {
        "getSofZmanKidushLevana15Days"
    }
}

/// Returns the latest time of _Kiddush Levana_ calculated as 15 days after the molad.
///
/// Will return None if the zman will not occur on this day. If the location does not contain
/// a timezone, this will always return None.
pub static SOF_ZMAN_KIDUSH_LEVANA_15_DAYS: SofZmanKidushLevana15Days =
    SofZmanKidushLevana15Days { _private: () };

#[allow(missing_docs)]
pub struct SofZmanKidushLevanaBetweenMoldos {
    _private: (),
}
impl<Tz: TimeZone> ZmanLike<Tz> for SofZmanKidushLevanaBetweenMoldos {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let tz = calculator
            .location
            .timezone
            .as_ref()
            .ok_or(ZmanimError::TimeZoneRequired)?;
        let date = Date::try_new_gregorian(
            calculator.date.year(),
            calculator.date.month() as u8,
            calculator.date.day() as u8,
        )
        .map_err(|_| ZmanimError::TimeConversionError)?;
        date.sof_zman_kidush_levana_between_moldos(tz)
            .map(|i| i.0.to_utc())
            .ok_or(ZmanimError::TimeConversionError)
    }
}
#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for SofZmanKidushLevanaBetweenMoldos {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        false
    }
    fn name(&self) -> &str {
        "getSofZmanKidushLevanaBetweenMoldos"
    }
}

/// The latest time of _Kiddush Levana_ according to the
/// [Maharil](https://en.wikipedia.org/wiki/Yaakov_ben_Moshe_Levi_Moelin)'s opinion that it
/// is calculated as halfway between molad and molad.
///
/// Will return None if the zman will not occur on this day. If the location does not contain
/// a timezone, this will always return None.
pub static SOF_ZMAN_KIDUSH_LEVANA_BETWEEN_MOLDOS: SofZmanKidushLevanaBetweenMoldos =
    SofZmanKidushLevanaBetweenMoldos { _private: () };

#[allow(missing_docs)]
pub struct TchilasZmanKidushLevana3Days {
    _private: (),
}
impl<Tz: TimeZone> ZmanLike<Tz> for TchilasZmanKidushLevana3Days {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let tz = calculator
            .location
            .timezone
            .as_ref()
            .ok_or(ZmanimError::TimeZoneRequired)?;
        let date = Date::try_new_gregorian(
            calculator.date.year(),
            calculator.date.month() as u8,
            calculator.date.day() as u8,
        )
        .map_err(|_| ZmanimError::TimeConversionError)?;
        date.tchilas_zman_kidush_levana_3_days(tz)
            .map(|i| i.0.to_utc())
            .ok_or(ZmanimError::TimeConversionError)
    }
}
#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for TchilasZmanKidushLevana3Days {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        false
    }
    fn name(&self) -> &str {
        "getTchilasZmanKidushLevana3Days"
    }
}
/// The earliest time of _Kiddush Levana_ according to [Rabbeinu Yonah](https://en.wikipedia.org/wiki/Yonah_Gerondi)'s opinion that it can be said 3 days after the molad.
///
/// Will return None if the zman will not occur on this day. If the location does not contain
/// a timezone, this will always return None.
pub static TCHILAS_ZMAN_KIDUSH_LEVANA_3_DAYS: TchilasZmanKidushLevana3Days =
    TchilasZmanKidushLevana3Days { _private: () };

#[allow(missing_docs)]
pub struct TchilasZmanKidushLevana7Days {
    _private: (),
}
impl<Tz: TimeZone> ZmanLike<Tz> for TchilasZmanKidushLevana7Days {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let tz = calculator
            .location
            .timezone
            .as_ref()
            .ok_or(ZmanimError::TimeZoneRequired)?;
        let date = Date::try_new_gregorian(
            calculator.date.year(),
            calculator.date.month() as u8,
            calculator.date.day() as u8,
        )
        .map_err(|_| ZmanimError::TimeConversionError)?;
        date.tchilas_zman_kidush_levana_7_days(tz)
            .map(|i| i.0.to_utc())
            .ok_or(ZmanimError::TimeConversionError)
    }
}
#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for TchilasZmanKidushLevana7Days {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        false
    }
    fn name(&self) -> &str {
        "getTchilasZmanKidushLevana7Days"
    }
}

/// The earliest time of _Kiddush Levana_ according to the opinions that it should
/// not be said until 7 days after the molad.
///
/// Will return None if the zman will not occur on this day. If the location does not contain
/// a timezone, this will always return None.
pub static TCHILAS_ZMAN_KIDUSH_LEVANA_7_DAYS: TchilasZmanKidushLevana7Days =
    TchilasZmanKidushLevana7Days { _private: () };

#[allow(missing_docs)]
pub struct Molad;

impl<Tz: TimeZone> ZmanLike<Tz> for Molad {
    fn calculate(
        &self,
        calculator: &mut ZmanimCalculator<Tz>,
    ) -> Result<DateTime<Utc>, ZmanimError> {
        let tz = calculator
            .location
            .timezone
            .as_ref()
            .ok_or(ZmanimError::TimeZoneRequired)?;
        let date = Date::try_new_gregorian(
            calculator.date.year(),
            calculator.date.month() as u8,
            calculator.date.day() as u8,
        )
        .map_err(|_| ZmanimError::TimeConversionError)?;
        date.molad(tz)
            .map(|i| i.0.to_utc())
            .ok_or(ZmanimError::TimeConversionError)
    }
}
#[cfg(feature = "_java_testing")]
impl<Tz: TimeZone> ZmanPresetLike<Tz> for Molad {
    fn uses_elevation(&self, _calculator: &ZmanimCalculator<Tz>) -> bool {
        false
    }
    fn name(&self) -> &str {
        "getZmanMolad"
    }
}
/// The time of the molad (new moon) for the current date's Hebrew month.
pub static MOLAD: Molad = Molad;

#[allow(unused)]
#[cfg(feature = "_java_testing")]
pub(crate) trait ZmanPresetLike<Tz: TimeZone>: ZmanLike<Tz> {
    /// Returns whether this zman uses elevation in its calculation (test-only).
    fn uses_elevation(&self, calculator: &ZmanimCalculator<Tz>) -> bool;
    /// Returns the KosherJava-style method name for this zman (test-only).
    fn name(&self) -> &str;
}

#[cfg(feature = "_java_testing")]
impl<'a, Tz: TimeZone> ZmanPresetLike<Tz> for ZmanPreset<'a> {
    fn uses_elevation(&self, calculator: &ZmanimCalculator<Tz>) -> bool {
        self.event.uses_elevation(calculator)
    }
    fn name(&self) -> &str {
        self.name
    }
}
#[allow(missing_docs)]
pub static ALL: &[&'static ZmanPreset<'static>] = &[
    &SUNRISE,
    &SEA_LEVEL_SUNRISE,
    &SUNSET,
    &SEA_LEVEL_SUNSET,
    &ALOS_60_MINUTES,
    &ALOS_72_MINUTES,
    &ALOS_72_ZMANIS,
    &ALOS_90_MINUTES,
    &ALOS_90_ZMANIS,
    &ALOS_96_MINUTES,
    &ALOS_96_ZMANIS,
    &ALOS_120_MINUTES,
    &ALOS_120_ZMANIS,
    &ALOS_16_POINT_1_DEGREES,
    &ALOS_18_DEGREES,
    &ALOS_19_DEGREES,
    &ALOS_19_POINT_8_DEGREES,
    &ALOS_26_DEGREES,
    &ALOS_BAAL_HATANYA,
    &BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES,
    &BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES,
    &BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES,
    &BAIN_HASHMASHOS_YEREIM_18_MINUTES,
    &BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES,
    &BAIN_HASHMASHOS_YEREIM_13_POINT_5_MINUTES,
    &CANDLE_LIGHTING,
    &CHATZOS_ASTRONOMICAL,
    &CHATZOS_HALF_DAY,
    &CHATZOS_FIXED_LOCAL,
    &MINCHA_GEDOLA_SUNRISE_SUNSET,
    &MINCHA_GEDOLA_16_POINT_1_DEGREES,
    &MINCHA_GEDOLA_MINUTES_30,
    &MINCHA_GEDOLA_MINUTES_72,
    &MINCHA_GEDOLA_ATERET_TORAH,
    &MINCHA_GEDOLA_BAAL_HATANYA,
    &MINCHA_GEDOLA_GRA_FIXED_LOCAL_CHATZOS_30_MINUTES,
    &MINCHA_KETANA_SUNRISE_SUNSET,
    &MINCHA_KETANA_16_POINT_1_DEGREES,
    &MINCHA_KETANA_MINUTES_72,
    &MINCHA_KETANA_ATERET_TORAH,
    &MINCHA_KETANA_BAAL_HATANYA,
    &MINCHA_KETANA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET,
    &MISHEYAKIR_10_POINT_2_DEGREES,
    &MISHEYAKIR_11_DEGREES,
    &MISHEYAKIR_11_POINT_5_DEGREES,
    &MISHEYAKIR_7_POINT_65_DEGREES,
    &MISHEYAKIR_9_POINT_5_DEGREES,
    &PLAG_HAMINCHA_SUNRISE_SUNSET,
    &PLAG_HAMINCHA_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083,
    &PLAG_HAMINCHA_ALOS_TO_SUNSET,
    &PLAG_HAMINCHA_60_MINUTES,
    &PLAG_HAMINCHA_72_MINUTES,
    &PLAG_HAMINCHA_72_ZMANIS,
    &PLAG_HAMINCHA_90_MINUTES,
    &PLAG_HAMINCHA_90_ZMANIS,
    &PLAG_HAMINCHA_96_MINUTES,
    &PLAG_HAMINCHA_96_ZMANIS,
    &PLAG_HAMINCHA_120_MINUTES,
    &PLAG_HAMINCHA_120_ZMANIS,
    &PLAG_HAMINCHA_16_POINT_1_DEGREES,
    &PLAG_HAMINCHA_18_DEGREES,
    &PLAG_HAMINCHA_19_POINT_8_DEGREES,
    &PLAG_HAMINCHA_26_DEGREES,
    &PLAG_HAMINCHA_ATERET_TORAH,
    &PLAG_HAMINCHA_BAAL_HATANYA,
    &PLAG_HAMINCHA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET,
    &SAMUCH_LE_MINCHA_KETANA_GRA,
    &SAMUCH_LE_MINCHA_KETANA_16_POINT_1_DEGREES,
    &SAMUCH_LE_MINCHA_KETANA_72_MINUTES,
    &SOF_ZMAN_ACHILAS_CHAMETZ_GRA,
    &SOF_ZMAN_ACHILAS_CHAMETZ_MGA_72_MINUTES,
    &SOF_ZMAN_ACHILAS_CHAMETZ_MGA_16_POINT_1_DEGREES,
    &SOF_ZMAN_ACHILAS_CHAMETZ_BAAL_HATANYA,
    &SOF_ZMAN_BIUR_CHAMETZ_GRA,
    &SOF_ZMAN_BIUR_CHAMETZ_MGA_72_MINUTES,
    &SOF_ZMAN_BIUR_CHAMETZ_MGA_16_POINT_1_DEGREES,
    &SOF_ZMAN_BIUR_CHAMETZ_BAAL_HATANYA,
    &SOF_ZMAN_SHMA_GRA,
    &SOF_ZMAN_SHMA_MGA,
    &SOF_ZMAN_SHMA_MGA_19_POINT_8_DEGREES,
    &SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES,
    &SOF_ZMAN_SHMA_MGA_18_DEGREES,
    &SOF_ZMAN_SHMA_MGA_72_MINUTES,
    &SOF_ZMAN_SHMA_MGA_72_ZMANIS,
    &SOF_ZMAN_SHMA_MGA_90_MINUTES,
    &SOF_ZMAN_SHMA_MGA_90_ZMANIS,
    &SOF_ZMAN_SHMA_MGA_96_MINUTES,
    &SOF_ZMAN_SHMA_MGA_96_ZMANIS,
    &SOF_ZMAN_SHMA_HOURS_3_BEFORE_CHATZOS,
    &SOF_ZMAN_SHMA_MGA_120_MINUTES,
    &SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_SUNSET,
    &SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083,
    &SOF_ZMAN_SHMA_ATERET_TORAH,
    &SOF_ZMAN_SHMA_BAAL_HATANYA,
    &SOF_ZMAN_SHMA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS,
    &SOF_ZMAN_SHMA_MGA_18_DEGREES_TO_FIXED_LOCAL_CHATZOS,
    &SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES_TO_FIXED_LOCAL_CHATZOS,
    &SOF_ZMAN_SHMA_MGA_90_MINUTES_TO_FIXED_LOCAL_CHATZOS,
    &SOF_ZMAN_SHMA_MGA_72_MINUTES_TO_FIXED_LOCAL_CHATZOS,
    &SOF_ZMAN_TFILA_GRA,
    &SOF_ZMAN_TFILA_MGA,
    &SOF_ZMAN_TFILA_MGA_19_POINT_8_DEGREES,
    &SOF_ZMAN_TFILA_MGA_16_POINT_1_DEGREES,
    &SOF_ZMAN_TFILA_MGA_18_DEGREES,
    &SOF_ZMAN_TFILA_MGA_72_MINUTES,
    &SOF_ZMAN_TFILA_MGA_72_ZMANIS,
    &SOF_ZMAN_TFILA_MGA_90_MINUTES,
    &SOF_ZMAN_TFILA_MGA_90_ZMANIS,
    &SOF_ZMAN_TFILA_MGA_96_MINUTES,
    &SOF_ZMAN_TFILA_MGA_96_ZMANIS,
    &SOF_ZMAN_TFILA_HOURS_2_BEFORE_CHATZOS,
    &SOF_ZMAN_TFILA_MGA_120_MINUTES,
    &SOF_ZMAN_TFILA_ATERET_TORAH,
    &SOF_ZMAN_TFILA_BAAL_HATANYA,
    &SOF_ZMAN_TFILA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS,
    &TZAIS_DEGREES_8_POINT_5,
    &TZAIS_MINUTES_50,
    &TZAIS_MINUTES_60,
    &TZAIS_MINUTES_72,
    &TZAIS_72_ZMANIS,
    &TZAIS_MINUTES_90,
    &TZAIS_90_ZMANIS,
    &TZAIS_MINUTES_96,
    &TZAIS_96_ZMANIS,
    &TZAIS_MINUTES_120,
    &TZAIS_120_ZMANIS,
    &TZAIS_16_POINT_1_DEGREES,
    &TZAIS_18_DEGREES,
    &TZAIS_19_POINT_8_DEGREES,
    &TZAIS_26_DEGREES,
    &TZAIS_ATERET_TORAH,
    &TZAIS_BAAL_HATANYA,
    &TZAIS_GEONIM_DEGREES_3_POINT_7,
    &TZAIS_GEONIM_DEGREES_3_POINT_8,
    &TZAIS_GEONIM_DEGREES_5_POINT_95,
    &TZAIS_GEONIM_DEGREES_4_POINT_61,
    &TZAIS_GEONIM_DEGREES_4_POINT_37,
    &TZAIS_GEONIM_DEGREES_5_POINT_88,
    &TZAIS_GEONIM_DEGREES_4_POINT_8,
    &TZAIS_GEONIM_DEGREES_6_POINT_45,
    &TZAIS_GEONIM_DEGREES_7_POINT_083,
    &TZAIS_GEONIM_DEGREES_7_POINT_67,
    &TZAIS_GEONIM_DEGREES_8_POINT_5,
    &TZAIS_GEONIM_DEGREES_9_POINT_3,
    &TZAIS_GEONIM_DEGREES_9_POINT_75,
];
