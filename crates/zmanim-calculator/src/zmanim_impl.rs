use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::{calculator::ZmanimCalculator, types::zman::ZmanLike, zmanim::*};

// ============================================================================
// NEITZ IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for NeitzZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            NeitzZman => calc.sunrise(),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        true // Uses calc.sunrise() which uses elevation_calc
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        "getSunrise"
    }
}

impl ZmanLike for SeaLevelNeitzZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            SeaLevelNeitzZman => calc.sea_level_sunrise(),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false // Uses sea_level_calc
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        "getSeaLevelSunrise"
    }
}

// ============================================================================
// SHKIA IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for ShkiaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            ShkiaZman => calc.sunset(),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        true // Uses calc.sunset() which uses elevation_calc
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        "getSunset"
    }
}

impl ZmanLike for SeaLevelShkiaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            SeaLevelShkiaZman => calc.sea_level_sunset(),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false // Uses sea_level_calc
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        "getSeaLevelSunset"
    }
}

// ============================================================================
// ALOS IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for AlosZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            AlosZman::Minutes60 => {
                let sunrise = calc.sunrise()?;
                Some(sunrise - Duration::minutes(60))
            }
            AlosZman::Minutes72 => {
                let sunrise = calc.sunrise()?;
                Some(sunrise - Duration::minutes(72))
            }
            AlosZman::Minutes72Zmanis => {
                let sunrise = calc.sunrise()?;
                calc.offset_by_shaah_zmanis_gra(sunrise, -1.2)
            }
            AlosZman::Minutes90 => {
                let sunrise = calc.sunrise()?;
                Some(sunrise - Duration::minutes(90))
            }
            AlosZman::Minutes90Zmanis => {
                let sunrise = calc.sunrise()?;
                calc.offset_by_shaah_zmanis_gra(sunrise, -1.5)
            }
            AlosZman::Minutes96 => {
                let sunrise = calc.sunrise()?;
                Some(sunrise - Duration::minutes(96))
            }
            AlosZman::Minutes96Zmanis => {
                let sunrise = calc.sunrise()?;
                calc.offset_by_shaah_zmanis_gra(sunrise, -1.6)
            }
            AlosZman::Minutes120 => {
                let sunrise = calc.sunrise()?;
                Some(sunrise - Duration::minutes(120))
            }
            AlosZman::Minutes120Zmanis => {
                let sunrise = calc.sunrise()?;
                calc.offset_by_shaah_zmanis_gra(sunrise, -2.0)
            }
            AlosZman::Degrees16Point1 => calc.sunrise_offset_by_degrees(16.1),
            AlosZman::Degrees18 => calc.sunrise_offset_by_degrees(18.0), // ASTRONOMICAL_ZENITH
            AlosZman::Degrees19 => calc.sunrise_offset_by_degrees(19.0),
            AlosZman::Degrees19Point8 => calc.sunrise_offset_by_degrees(19.8),
            AlosZman::Degrees26 => calc.sunrise_offset_by_degrees(26.0),
            AlosZman::BaalHatanya => calc.sunrise_offset_by_degrees(16.9),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // Minutes variants use calc.sunrise() which uses elevation_calc
        // Degree variants use sunrise_offset_by_degrees() which uses sea_level_calc
        matches!(
            self,
            AlosZman::Minutes60
                | AlosZman::Minutes72
                | AlosZman::Minutes72Zmanis
                | AlosZman::Minutes90
                | AlosZman::Minutes90Zmanis
                | AlosZman::Minutes96
                | AlosZman::Minutes96Zmanis
                | AlosZman::Minutes120
                | AlosZman::Minutes120Zmanis
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            AlosZman::Minutes60 => "getAlos60",
            AlosZman::Minutes72 => "getAlos72",
            AlosZman::Minutes72Zmanis => "getAlos72Zmanis",
            AlosZman::Minutes90 => "getAlos90",
            AlosZman::Minutes90Zmanis => "getAlos90Zmanis",
            AlosZman::Minutes96 => "getAlos96",
            AlosZman::Minutes96Zmanis => "getAlos96Zmanis",
            AlosZman::Minutes120 => "getAlos120",
            AlosZman::Minutes120Zmanis => "getAlos120Zmanis",
            AlosZman::Degrees16Point1 => "getAlos16Point1Degrees",
            AlosZman::Degrees18 => "getAlos18Degrees",
            AlosZman::Degrees19 => "getAlos19Degrees",
            AlosZman::Degrees19Point8 => "getAlos19Point8Degrees",
            AlosZman::Degrees26 => "getAlos26Degrees",
            AlosZman::BaalHatanya => "getAlosBaalHatanya",
        }
    }
}

// ============================================================================
// BAIN HASHMASHHAS IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for BainHashmashosZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            BainHashmashosZman::RabbeinuTam13Point24Degrees => calc.sunset_offset_by_degrees(13.24),
            BainHashmashosZman::RabbeinuTam58Point5Minutes => {
                let sunset = calc.sunset()?;
                Some(sunset + Duration::milliseconds((58.5 * 60.0 * 1000.0) as i64))
            }
            BainHashmashosZman::RabbeinuTam13Point5MinutesBefore7Point083Degrees => {
                let tzais = calc.sunset_offset_by_degrees(7.0 + (5.0 / 60.0))?;
                Some(tzais - Duration::milliseconds((13.5 * 60.0 * 1000.0) as i64))
            }
            BainHashmashosZman::RabbeinuTam2Stars => {
                let alos19_point_8 = AlosZman::Degrees19Point8.calculate(calc)?;
                let sunrise = calc.sunrise()?;
                let sunset = calc.sunset()?;
                let time_diff = sunrise.signed_duration_since(alos19_point_8);
                let offset = time_diff.num_milliseconds() as f64 * (5.0 / 18.0);
                Some(sunset + Duration::milliseconds(offset as i64))
            }
            BainHashmashosZman::Yereim18Minutes => {
                let sunset = calc.sunset()?;
                Some(sunset - Duration::minutes(18))
            } // BainHashmashosZman::Yereim3Point05Degrees => calc.sunset_offset_by_degrees(-3.05),
            BainHashmashosZman::Yereim16Point875Minutes => {
                let sunset = calc.sunset()?;
                Some(sunset - Duration::milliseconds((16.875 * 60.0 * 1000.0) as i64))
            } // BainHashmashosZman::Yereim2Point8Degrees => calc.sunset_offset_by_degrees(-2.8),
            BainHashmashosZman::Yereim13Point5Minutes => {
                let sunset = calc.sunset()?;
                Some(sunset - Duration::milliseconds((13.5 * 60.0 * 1000.0) as i64))
            } // BainHashmashosZman::Yereim2Point1Degrees => calc.sunset_offset_by_degrees(-2.1),
        }
    }

    #[cfg(test)]
    fn degrees_above_horizon(&self) -> bool {
        false
        // matches!(
        //     self,
        //     BainHashmashosZman::Yereim2Point8Degrees
        //         | BainHashmashosZman::Yereim3Point05Degrees
        //         | BainHashmashosZman::Yereim2Point1Degrees
        // )
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // Minutes variants use calc.sunset() which uses elevation_calc
        // Degree variants use sunset_offset_by_degrees() which uses sea_level_calc
        matches!(
            self,
            BainHashmashosZman::RabbeinuTam58Point5Minutes
                | BainHashmashosZman::RabbeinuTam2Stars
                | BainHashmashosZman::Yereim18Minutes
                | BainHashmashosZman::Yereim16Point875Minutes
                | BainHashmashosZman::Yereim13Point5Minutes
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            BainHashmashosZman::RabbeinuTam13Point24Degrees => {
                "getBainHashmashosRT13Point24Degrees"
            }
            BainHashmashosZman::RabbeinuTam58Point5Minutes => "getBainHashmashosRT58Point5Minutes",
            BainHashmashosZman::RabbeinuTam13Point5MinutesBefore7Point083Degrees => {
                "getBainHashmashosRT13Point5MinutesBefore7Point083Degrees"
            }
            BainHashmashosZman::RabbeinuTam2Stars => "getBainHashmashosRT2Stars",
            BainHashmashosZman::Yereim18Minutes => "getBainHashmashosYereim18Minutes",
            // BainHashmashosZman::Yereim3Point05Degrees => "getBainHashmashosYereim3Point05Degrees",
            BainHashmashosZman::Yereim16Point875Minutes => {
                "getBainHashmashosYereim16Point875Minutes"
            }
            // BainHashmashosZman::Yereim2Point8Degrees => "getBainHashmashosYereim2Point8Degrees",
            BainHashmashosZman::Yereim13Point5Minutes => "getBainHashmashosYereim13Point5Minutes",
            // BainHashmashosZman::Yereim2Point1Degrees => "getBainHashmashosYereim2Point1Degrees",
        }
    }
}

// ============================================================================
// CANDLE LIGHTING IMPLEMENTATION
// ============================================================================

impl ZmanLike for CandleLightingZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        let sunset = calc.sea_level_sunset()?;
        Some(sunset - calc.config.candle_lighting_offset)
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false // Uses sea-level sunset
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        "getCandleLighting"
    }
}

// ============================================================================
// CHATZOS IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for ChatzosZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            ChatzosZman::Astronomical => calc.transit(),
            ChatzosZman::HalfDay => {
                let sunrise = calc.sea_level_sunrise()?;
                let sunset = calc.sea_level_sunset()?;
                calc.get_sun_transit_from_times(&sunrise, &sunset)
            }
            ChatzosZman::FixedLocal => {
                let date = calc.date;
                let location = calc.location.clone();
                calc.local_mean_time(date, &location, 12.0)
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false // All variants use transit or sea_level times
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            ChatzosZman::Astronomical => "getChatzos",
            ChatzosZman::HalfDay => "getChatzosAsHalfDay",
            ChatzosZman::FixedLocal => "getFixedLocalChatzos",
        }
    }
}

// ============================================================================
// MINCHA GEDOLA IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for MinchaGedolaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            MinchaGedolaZman::SunriseSunset => {
                let sunrise = calc.sunrise();
                let sunset = calc.sunset()?;
                calc.get_mincha_gedola_from_times(sunrise.as_ref(), &sunset, true)
            }
            MinchaGedolaZman::Degrees16Point1 => {
                if calc.config.use_astronomical_chatzos_for_other_zmanim {
                    let chatzos = ChatzosZman::Astronomical.calculate(calc)?;
                    let tzais = TzaisZman::Degrees16Point1.calculate(calc)?;
                    calc.get_half_day_based_zman_from_times(&chatzos, &tzais, 0.5)
                } else {
                    let alos = AlosZman::Degrees16Point1.calculate(calc);
                    let tzais = TzaisZman::Degrees16Point1.calculate(calc)?;
                    calc.get_mincha_gedola_from_times(alos.as_ref(), &tzais, true)
                }
            }
            MinchaGedolaZman::Minutes30 => {
                let chatzos = ChatzosZman::Astronomical.calculate(calc)?;
                Some(chatzos + Duration::minutes(30))
            }
            MinchaGedolaZman::Minutes72 => {
                if calc.config.use_astronomical_chatzos_for_other_zmanim {
                    let chatzos = ChatzosZman::Astronomical.calculate(calc)?;
                    let tzais = TzaisZman::Minutes72.calculate(calc)?;
                    calc.get_half_day_based_zman_from_times(&chatzos, &tzais, 0.5)
                } else {
                    let alos = AlosZman::Minutes72.calculate(calc);
                    let tzais = TzaisZman::Minutes72.calculate(calc)?;
                    calc.get_mincha_gedola_from_times(alos.as_ref(), &tzais, true)
                }
            }
            MinchaGedolaZman::AhavatShalom => {
                let chatzos = ChatzosZman::Astronomical.calculate(calc)?;
                let mincha_gedola_30 = MinchaGedolaZman::Minutes30.calculate(calc)?;
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let tzais = TzaisZman::GeonimDegrees3Point7.calculate(calc)?;
                let shaah_zmanis = calc.get_temporal_hour_from_times(&alos, &tzais)?;
                let mincha_alternative = chatzos + (shaah_zmanis / 2);
                if mincha_gedola_30 > mincha_alternative {
                    Some(mincha_gedola_30)
                } else {
                    Some(mincha_alternative)
                }
            }
            MinchaGedolaZman::AteretTorah => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc);
                let tzais = TzaisZman::AteretTorah.calculate(calc)?;
                calc.get_mincha_gedola_from_times(alos.as_ref(), &tzais, false)
            }
            MinchaGedolaZman::BaalHatanya => {
                let sunrise = calc.sunrise_offset_by_degrees(1.583);
                let sunset = calc.sunset_offset_by_degrees(1.583)?;
                calc.get_mincha_gedola_from_times(sunrise.as_ref(), &sunset, true)
            }
            MinchaGedolaZman::BaalHatanyaGreaterThan30 => {
                let mincha_30 = MinchaGedolaZman::Minutes30.calculate(calc)?;
                let mincha_bh = MinchaGedolaZman::BaalHatanya.calculate(calc)?;
                if mincha_30 > mincha_bh {
                    Some(mincha_30)
                } else {
                    Some(mincha_bh)
                }
            }
            MinchaGedolaZman::GRAFixedLocalChatzos30Minutes => {
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                Some(chatzos + Duration::minutes(30))
            }
            MinchaGedolaZman::GreaterThan30 => {
                let mincha_30 = MinchaGedolaZman::Minutes30.calculate(calc)?;
                let mincha_regular = MinchaGedolaZman::SunriseSunset.calculate(calc)?;
                if mincha_30 > mincha_regular {
                    Some(mincha_30)
                } else {
                    Some(mincha_regular)
                }
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // SunriseSunset, Minutes72, Minutes90/96Zmanis variants use elevation-adjusted times
        // GreaterThan30 and BaalHatanyaGreaterThan30 may use elevation-adjusted times
        matches!(
            self,
            MinchaGedolaZman::SunriseSunset
                | MinchaGedolaZman::Minutes72
                | MinchaGedolaZman::AteretTorah
                | MinchaGedolaZman::GreaterThan30
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            MinchaGedolaZman::SunriseSunset => "getMinchaGedola",
            MinchaGedolaZman::Degrees16Point1 => "getMinchaGedola16Point1Degrees",
            MinchaGedolaZman::Minutes30 => "getMinchaGedola30Minutes",
            MinchaGedolaZman::Minutes72 => "getMinchaGedola72Minutes",
            MinchaGedolaZman::AhavatShalom => "getMinchaGedolaAhavatShalom",
            MinchaGedolaZman::AteretTorah => "getMinchaGedolaAteretTorah",
            MinchaGedolaZman::BaalHatanya => "getMinchaGedolaBaalHatanya",
            MinchaGedolaZman::BaalHatanyaGreaterThan30 => "getMinchaGedolaBaalHatanyaGreaterThan30",
            MinchaGedolaZman::GRAFixedLocalChatzos30Minutes => {
                "getMinchaGedolaGRAFixedLocalChatzos30Minutes"
            }
            MinchaGedolaZman::GreaterThan30 => "getMinchaGedolaGreaterThan30",
        }
    }
}

// ============================================================================
// MINCHA KETANA IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for MinchaKetanaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            MinchaKetanaZman::SunriseSunset => {
                let sunrise = calc.sunrise();
                let sunset = calc.sunset()?;
                calc.get_mincha_ketana_from_times(sunrise.as_ref(), &sunset, true)
            }
            MinchaKetanaZman::Degrees16Point1 => {
                let alos = AlosZman::Degrees16Point1.calculate(calc);
                let tzais = TzaisZman::Degrees16Point1.calculate(calc)?;
                calc.get_mincha_ketana_from_times(alos.as_ref(), &tzais, true)
            }
            MinchaKetanaZman::Minutes72 => {
                let alos = AlosZman::Minutes72.calculate(calc);
                let tzais = TzaisZman::Minutes72.calculate(calc)?;
                calc.get_mincha_ketana_from_times(alos.as_ref(), &tzais, true)
            }
            MinchaKetanaZman::AhavatShalom => {
                let tzais = TzaisZman::GeonimDegrees3Point8.calculate(calc)?;
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let tzais_for_shaah = TzaisZman::GeonimDegrees3Point8.calculate(calc)?;
                let shaah_zmanis = calc.get_temporal_hour_from_times(&alos, &tzais_for_shaah)?;
                Some(tzais - (shaah_zmanis * 5 / 2))
            }
            MinchaKetanaZman::AteretTorah => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc);
                let tzais = TzaisZman::AteretTorah.calculate(calc)?;
                calc.get_mincha_ketana_from_times(alos.as_ref(), &tzais, false)
            }
            MinchaKetanaZman::BaalHatanya => {
                let sunrise = calc.sunrise_offset_by_degrees(1.583);
                let sunset = calc.sunset_offset_by_degrees(1.583)?;
                calc.get_mincha_ketana_from_times(sunrise.as_ref(), &sunset, true)
            }
            MinchaKetanaZman::GRAFixedLocalChatzosToSunset => {
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                let sunset = calc.sunset()?;
                calc.get_half_day_based_zman_from_times(&chatzos, &sunset, 3.5)
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // SunriseSunset uses sunrise(); Minutes72 uses Alos72/Tzais72 (both use elevation)
        // AteretTorah uses sunset; GRAFixedLocalChatzosToSunset uses sunset()
        matches!(
            self,
            MinchaKetanaZman::SunriseSunset
                | MinchaKetanaZman::Minutes72
                | MinchaKetanaZman::AteretTorah
                | MinchaKetanaZman::GRAFixedLocalChatzosToSunset
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            MinchaKetanaZman::SunriseSunset => "getMinchaKetana",
            MinchaKetanaZman::Degrees16Point1 => "getMinchaKetana16Point1Degrees",
            MinchaKetanaZman::Minutes72 => "getMinchaKetana72Minutes",
            MinchaKetanaZman::AhavatShalom => "getMinchaKetanaAhavatShalom",
            MinchaKetanaZman::AteretTorah => "getMinchaKetanaAteretTorah",
            MinchaKetanaZman::BaalHatanya => "getMinchaKetanaBaalHatanya",
            MinchaKetanaZman::GRAFixedLocalChatzosToSunset => {
                "getMinchaKetanaGRAFixedLocalChatzosToSunset"
            }
        }
    }
}

// ============================================================================
// MISHEYAKIR IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for MisheyakirZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            MisheyakirZman::Degrees10Point2 => calc.sunrise_offset_by_degrees(10.2),
            MisheyakirZman::Degrees11 => calc.sunrise_offset_by_degrees(11.0),
            MisheyakirZman::Degrees11Point5 => calc.sunrise_offset_by_degrees(11.5),
            MisheyakirZman::Degrees7Point65 => calc.sunrise_offset_by_degrees(7.65),
            MisheyakirZman::Degrees9Point5 => calc.sunrise_offset_by_degrees(9.5),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        false // All use sunrise_offset_by_degrees (sea_level_calc)
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            MisheyakirZman::Degrees10Point2 => "getMisheyakir10Point2Degrees",
            MisheyakirZman::Degrees11 => "getMisheyakir11Degrees",
            MisheyakirZman::Degrees11Point5 => "getMisheyakir11Point5Degrees",
            MisheyakirZman::Degrees7Point65 => "getMisheyakir7Point65Degrees",
            MisheyakirZman::Degrees9Point5 => "getMisheyakir9Point5Degrees",
        }
    }
}

// ============================================================================
// PLAG HAMINCHA IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for PlagHaminchaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            PlagHaminchaZman::SunriseSunset => {
                let sunrise = calc.sunrise();
                let sunset = calc.sunset()?;
                calc.get_plag_hamincha_from_times(sunrise.as_ref(), &sunset, true)
            }
            PlagHaminchaZman::AhavatShalom => {
                let tzais = TzaisZman::GeonimDegrees3Point8.calculate(calc)?;
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let tzais_for_shaah = TzaisZman::GeonimDegrees3Point8.calculate(calc)?;
                let shaah_zmanis = calc.get_temporal_hour_from_times(&alos, &tzais_for_shaah)?;
                Some(tzais - (shaah_zmanis * 5 / 4))
            }
            PlagHaminchaZman::Degrees16Point1ToTzaisGeonim7Point083 => {
                let alos = AlosZman::Degrees16Point1.calculate(calc);
                let tzais = TzaisZman::GeonimDegrees7Point083.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, false)
            }
            PlagHaminchaZman::AlosToSunset => {
                let alos = AlosZman::Degrees16Point1.calculate(calc);
                let sunset = calc.sunset()?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &sunset, false)
            }
            PlagHaminchaZman::Minutes60 => {
                let alos = AlosZman::Minutes60.calculate(calc);
                let tzais = TzaisZman::Minutes60.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes72 => {
                let alos = AlosZman::Minutes72.calculate(calc);
                let tzais = TzaisZman::Minutes72.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes72Zmanis => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc);
                let tzais = TzaisZman::Minutes72Zmanis.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes90 => {
                let alos = AlosZman::Minutes90.calculate(calc);
                let tzais = TzaisZman::Minutes90.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes90Zmanis => {
                let alos = AlosZman::Minutes90Zmanis.calculate(calc);
                let tzais = TzaisZman::Minutes90Zmanis.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes96 => {
                let alos = AlosZman::Minutes96.calculate(calc);
                let tzais = TzaisZman::Minutes96.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes96Zmanis => {
                let alos = AlosZman::Minutes96Zmanis.calculate(calc);
                let tzais = TzaisZman::Minutes96Zmanis.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes120 => {
                let alos = AlosZman::Minutes120.calculate(calc);
                let tzais = TzaisZman::Minutes120.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Minutes120Zmanis => {
                let alos = AlosZman::Minutes120Zmanis.calculate(calc);
                let tzais = TzaisZman::Minutes120Zmanis.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Degrees16Point1 => {
                let alos = AlosZman::Degrees16Point1.calculate(calc);
                let tzais = TzaisZman::Degrees16Point1.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Degrees18 => {
                let alos = AlosZman::Degrees18.calculate(calc);
                let tzais = TzaisZman::Degrees18.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Degrees19Point8 => {
                let alos = AlosZman::Degrees19Point8.calculate(calc);
                let tzais = TzaisZman::Degrees19Point8.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::Degrees26 => {
                let alos = AlosZman::Degrees26.calculate(calc);
                let tzais = TzaisZman::Degrees26.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, true)
            }
            PlagHaminchaZman::AteretTorah => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc);
                let tzais = TzaisZman::AteretTorah.calculate(calc)?;
                calc.get_plag_hamincha_from_times(alos.as_ref(), &tzais, false)
            }
            PlagHaminchaZman::BaalHatanya => {
                let sunrise = calc.sunrise_offset_by_degrees(1.583);
                let sunset = calc.sunset_offset_by_degrees(1.583)?;
                calc.get_plag_hamincha_from_times(sunrise.as_ref(), &sunset, true)
            }
            PlagHaminchaZman::GRAFixedLocalChatzosToSunset => {
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                let sunset = calc.sunset()?;
                calc.get_half_day_based_zman_from_times(&chatzos, &sunset, 4.75)
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // SunriseSunset, Minutes variants, and GRAFixedLocalChatzosToSunset use elevation
        matches!(
            self,
            PlagHaminchaZman::SunriseSunset
                | PlagHaminchaZman::AlosToSunset
                | PlagHaminchaZman::Minutes60
                | PlagHaminchaZman::Minutes72
                | PlagHaminchaZman::Minutes72Zmanis
                | PlagHaminchaZman::Minutes90
                | PlagHaminchaZman::Minutes90Zmanis
                | PlagHaminchaZman::Minutes96
                | PlagHaminchaZman::Minutes96Zmanis
                | PlagHaminchaZman::Minutes120
                | PlagHaminchaZman::Minutes120Zmanis
                | PlagHaminchaZman::AteretTorah
                | PlagHaminchaZman::GRAFixedLocalChatzosToSunset
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            PlagHaminchaZman::SunriseSunset => "getPlagHamincha",
            PlagHaminchaZman::AhavatShalom => "getPlagAhavatShalom",
            PlagHaminchaZman::Degrees16Point1ToTzaisGeonim7Point083 => {
                "getPlagAlos16Point1ToTzaisGeonim7Point083Degrees"
            }
            PlagHaminchaZman::AlosToSunset => "getPlagAlosToSunset",
            PlagHaminchaZman::Minutes60 => "getPlagHamincha60Minutes",
            PlagHaminchaZman::Minutes72 => "getPlagHamincha72Minutes",
            PlagHaminchaZman::Minutes72Zmanis => "getPlagHamincha72MinutesZmanis",
            PlagHaminchaZman::Minutes90 => "getPlagHamincha90Minutes",
            PlagHaminchaZman::Minutes90Zmanis => "getPlagHamincha90MinutesZmanis",
            PlagHaminchaZman::Minutes96 => "getPlagHamincha96Minutes",
            PlagHaminchaZman::Minutes96Zmanis => "getPlagHamincha96MinutesZmanis",
            PlagHaminchaZman::Minutes120 => "getPlagHamincha120Minutes",
            PlagHaminchaZman::Minutes120Zmanis => "getPlagHamincha120MinutesZmanis",
            PlagHaminchaZman::Degrees16Point1 => "getPlagHamincha16Point1Degrees",
            PlagHaminchaZman::Degrees18 => "getPlagHamincha18Degrees",
            PlagHaminchaZman::Degrees19Point8 => "getPlagHamincha19Point8Degrees",
            PlagHaminchaZman::Degrees26 => "getPlagHamincha26Degrees",
            PlagHaminchaZman::AteretTorah => "getPlagHaminchaAteretTorah",
            PlagHaminchaZman::BaalHatanya => "getPlagHaminchaBaalHatanya",
            PlagHaminchaZman::GRAFixedLocalChatzosToSunset => {
                "getPlagHaminchaGRAFixedLocalChatzosToSunset"
            }
        }
    }
}

// ============================================================================
// SAMUCH LE MINCHA KETANA IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for SamuchLeMinchaKetanaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            SamuchLeMinchaKetanaZman::GRA => {
                let sunrise = calc.sunrise();
                let sunset = calc.sunset()?;
                calc.get_samuch_le_mincha_ketana_from_times(sunrise.as_ref(), &sunset, true)
            }
            SamuchLeMinchaKetanaZman::Degrees16Point1 => {
                let alos = AlosZman::Degrees16Point1.calculate(calc);
                let tzais = TzaisZman::Degrees16Point1.calculate(calc)?;
                calc.get_samuch_le_mincha_ketana_from_times(alos.as_ref(), &tzais, true)
            }
            SamuchLeMinchaKetanaZman::Minutes72 => {
                let alos = AlosZman::Minutes72.calculate(calc);
                let tzais = TzaisZman::Minutes72.calculate(calc)?;
                calc.get_samuch_le_mincha_ketana_from_times(alos.as_ref(), &tzais, true)
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // GRA and Minutes72 use elevation-adjusted times
        matches!(
            self,
            SamuchLeMinchaKetanaZman::GRA | SamuchLeMinchaKetanaZman::Minutes72
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            SamuchLeMinchaKetanaZman::GRA => "getSamuchLeMinchaKetanaGRA",
            SamuchLeMinchaKetanaZman::Degrees16Point1 => "getSamuchLeMinchaKetana16Point1Degrees",
            SamuchLeMinchaKetanaZman::Minutes72 => "getSamuchLeMinchaKetana72Minutes",
        }
    }
}

// ============================================================================
// SOF ZMAN ACHILAS CHAMETZ IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for SofZmanAchilasChametzZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            SofZmanAchilasChametzZman::GRA => SofZmanTfilaZman::GRA.calculate(calc),
            SofZmanAchilasChametzZman::MGA72Minutes => {
                SofZmanTfilaZman::MGA72Minutes.calculate(calc)
            }
            SofZmanAchilasChametzZman::MGA16Point1Degrees => {
                SofZmanTfilaZman::MGA16Point1Degrees.calculate(calc)
            }
            SofZmanAchilasChametzZman::BaalHatanya => SofZmanTfilaZman::BaalHatanya.calculate(calc),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // Delegates to SofZmanTfila - GRA and MGA72Minutes use elevation
        matches!(
            self,
            SofZmanAchilasChametzZman::GRA | SofZmanAchilasChametzZman::MGA72Minutes
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            SofZmanAchilasChametzZman::GRA => "getSofZmanAchilasChametzGRA",
            SofZmanAchilasChametzZman::MGA72Minutes => "getSofZmanAchilasChametzMGA72Minutes",
            SofZmanAchilasChametzZman::MGA16Point1Degrees => {
                "getSofZmanAchilasChametzMGA16Point1Degrees"
            }
            SofZmanAchilasChametzZman::BaalHatanya => "getSofZmanAchilasChametzBaalHatanya",
        }
    }
}

// ============================================================================
// SOF ZMAN BIUR CHAMETZ IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for SofZmanBiurChametzZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            SofZmanBiurChametzZman::GRA => {
                let sunrise = calc.sunrise()?;
                let shaah_zmanis = calc.get_shaah_zmanis_gra()?;
                Some(sunrise + (shaah_zmanis * 5))
            }
            SofZmanBiurChametzZman::MGA72Minutes => {
                let alos = AlosZman::Minutes72.calculate(calc)?;
                let shaah_zmanis = calc.get_shaah_zmanis_mga()?;
                Some(alos + (shaah_zmanis * 5))
            }
            SofZmanBiurChametzZman::MGA16Point1Degrees => {
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let tzais = TzaisZman::Degrees16Point1.calculate(calc)?;
                let shaah_zmanis = calc.get_temporal_hour_from_times(&alos, &tzais)?;
                Some(alos + (shaah_zmanis * 5))
            }
            SofZmanBiurChametzZman::BaalHatanya => {
                let sunrise = calc.sunrise_offset_by_degrees(1.583)?;
                let sunset = calc.sunset_offset_by_degrees(1.583)?;
                let shaah_zmanis = calc.get_temporal_hour_from_times(&sunrise, &sunset)?;
                Some(sunrise + (shaah_zmanis * 5))
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // GRA and MGA72Minutes use elevation-adjusted times
        matches!(
            self,
            SofZmanBiurChametzZman::GRA | SofZmanBiurChametzZman::MGA72Minutes
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            SofZmanBiurChametzZman::GRA => "getSofZmanBiurChametzGRA",
            SofZmanBiurChametzZman::MGA72Minutes => "getSofZmanBiurChametzMGA72Minutes",
            SofZmanBiurChametzZman::MGA16Point1Degrees => "getSofZmanBiurChametzMGA16Point1Degrees",
            SofZmanBiurChametzZman::BaalHatanya => "getSofZmanBiurChametzBaalHatanya",
        }
    }
}

// ============================================================================
// SOF ZMAN SHMA IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for SofZmanShmaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            SofZmanShmaZman::GRA => {
                let sunrise = calc.sunrise()?;
                let sunset = calc.sunset();
                calc.get_sof_zman_shma_from_times(&sunrise, sunset.as_ref(), true)
            }
            SofZmanShmaZman::MGA => {
                let alos = AlosZman::Minutes72.calculate(calc)?;
                let tzais = TzaisZman::Minutes72.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA19Point8Degrees => {
                let alos = AlosZman::Degrees19Point8.calculate(calc)?;
                let tzais = TzaisZman::Degrees19Point8.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA16Point1Degrees => {
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let tzais = TzaisZman::Degrees16Point1.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA18Degrees => {
                let alos = AlosZman::Degrees18.calculate(calc)?;
                let tzais = TzaisZman::Degrees18.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA72Minutes => {
                let alos = AlosZman::Minutes72.calculate(calc)?;
                let tzais = TzaisZman::Minutes72.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA72MinutesZmanis => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc)?;
                let tzais = TzaisZman::Minutes72Zmanis.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA90Minutes => {
                let alos = AlosZman::Minutes90.calculate(calc)?;
                let tzais = TzaisZman::Minutes90.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA90MinutesZmanis => {
                let alos = AlosZman::Minutes90Zmanis.calculate(calc)?;
                let tzais = TzaisZman::Minutes90Zmanis.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA96Minutes => {
                let alos = AlosZman::Minutes96.calculate(calc)?;
                let tzais = TzaisZman::Minutes96.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::MGA96MinutesZmanis => {
                let alos = AlosZman::Minutes96Zmanis.calculate(calc)?;
                let tzais = TzaisZman::Minutes96Zmanis.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::Hours3BeforeChatzos => {
                let chatzos = ChatzosZman::Astronomical.calculate(calc)?;
                Some(chatzos - Duration::minutes(180))
            }
            SofZmanShmaZman::MGA120Minutes => {
                let alos = AlosZman::Minutes120.calculate(calc)?;
                let tzais = TzaisZman::Minutes120.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanShmaZman::Alos16Point1ToSunset => {
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let sunset = calc.sunset();
                calc.get_sof_zman_shma_from_times(&alos, sunset.as_ref(), false)
            }
            SofZmanShmaZman::Alos16Point1ToTzaisGeonim7Point083Degrees => {
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let tzais = TzaisZman::GeonimDegrees7Point083.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), false)
            }
            SofZmanShmaZman::KolEliyahu => {
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                let sunrise = calc.sunrise()?;
                let diff = chatzos.signed_duration_since(sunrise) / 2;
                Some(chatzos - diff)
            }
            SofZmanShmaZman::AteretTorah => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc)?;
                let tzais = TzaisZman::AteretTorah.calculate(calc);
                calc.get_sof_zman_shma_from_times(&alos, tzais.as_ref(), false)
            }
            SofZmanShmaZman::BaalHatanya => {
                let sunrise = calc.sunrise_offset_by_degrees(1.583)?;
                let sunset = calc.sunset_offset_by_degrees(1.583);
                calc.get_sof_zman_shma_from_times(&sunrise, sunset.as_ref(), true)
            }
            SofZmanShmaZman::FixedLocal => {
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                Some(chatzos - Duration::minutes(180))
            }
            SofZmanShmaZman::GRASunriseToFixedLocalChatzos => {
                let sunrise = calc.sunrise()?;
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                calc.get_half_day_based_zman_from_times(&sunrise, &chatzos, 3.0)
            }
            SofZmanShmaZman::MGA18DegreesToFixedLocalChatzos => {
                let alos = AlosZman::Degrees18.calculate(calc)?;
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                calc.get_half_day_based_zman_from_times(&alos, &chatzos, 3.0)
            }
            SofZmanShmaZman::MGA16Point1DegreesToFixedLocalChatzos => {
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                calc.get_half_day_based_zman_from_times(&alos, &chatzos, 3.0)
            }
            SofZmanShmaZman::MGA90MinutesToFixedLocalChatzos => {
                let alos = AlosZman::Minutes90.calculate(calc)?;
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                calc.get_half_day_based_zman_from_times(&alos, &chatzos, 3.0)
            }
            SofZmanShmaZman::MGA72MinutesToFixedLocalChatzos => {
                let alos = AlosZman::Minutes72.calculate(calc)?;
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                calc.get_half_day_based_zman_from_times(&alos, &chatzos, 3.0)
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // Most variants use elevation-adjusted times (Minutes variants, sunrise-based)
        // Degree-based variants don't use elevation
        matches!(
            self,
            SofZmanShmaZman::GRA
                | SofZmanShmaZman::MGA
                | SofZmanShmaZman::MGA72Minutes
                | SofZmanShmaZman::MGA72MinutesZmanis
                | SofZmanShmaZman::MGA90Minutes
                | SofZmanShmaZman::MGA90MinutesZmanis
                | SofZmanShmaZman::MGA96Minutes
                | SofZmanShmaZman::MGA96MinutesZmanis
                | SofZmanShmaZman::MGA120Minutes
                | SofZmanShmaZman::Alos16Point1ToSunset
                | SofZmanShmaZman::KolEliyahu
                | SofZmanShmaZman::AteretTorah
                | SofZmanShmaZman::GRASunriseToFixedLocalChatzos
                | SofZmanShmaZman::MGA90MinutesToFixedLocalChatzos
                | SofZmanShmaZman::MGA72MinutesToFixedLocalChatzos
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            SofZmanShmaZman::GRA => "getSofZmanShmaGRA",
            SofZmanShmaZman::MGA => "getSofZmanShmaMGA",
            SofZmanShmaZman::MGA19Point8Degrees => "getSofZmanShmaMGA19Point8Degrees",
            SofZmanShmaZman::MGA16Point1Degrees => "getSofZmanShmaMGA16Point1Degrees",
            SofZmanShmaZman::MGA18Degrees => "getSofZmanShmaMGA18Degrees",
            SofZmanShmaZman::MGA72Minutes => "getSofZmanShmaMGA72Minutes",
            SofZmanShmaZman::MGA72MinutesZmanis => "getSofZmanShmaMGA72MinutesZmanis",
            SofZmanShmaZman::MGA90Minutes => "getSofZmanShmaMGA90Minutes",
            SofZmanShmaZman::MGA90MinutesZmanis => "getSofZmanShmaMGA90MinutesZmanis",
            SofZmanShmaZman::MGA96Minutes => "getSofZmanShmaMGA96Minutes",
            SofZmanShmaZman::MGA96MinutesZmanis => "getSofZmanShmaMGA96MinutesZmanis",
            SofZmanShmaZman::Hours3BeforeChatzos => "getSofZmanShma3HoursBeforeChatzos",
            SofZmanShmaZman::MGA120Minutes => "getSofZmanShmaMGA120Minutes",
            SofZmanShmaZman::Alos16Point1ToSunset => "getSofZmanShmaAlos16Point1ToSunset",
            SofZmanShmaZman::Alos16Point1ToTzaisGeonim7Point083Degrees => {
                "getSofZmanShmaAlos16Point1ToTzaisGeonim7Point083Degrees"
            }
            SofZmanShmaZman::KolEliyahu => "getSofZmanShmaKolEliyahu",
            SofZmanShmaZman::AteretTorah => "getSofZmanShmaAteretTorah",
            SofZmanShmaZman::BaalHatanya => "getSofZmanShmaBaalHatanya",
            SofZmanShmaZman::FixedLocal => "getSofZmanShmaFixedLocal",
            SofZmanShmaZman::GRASunriseToFixedLocalChatzos => {
                "getSofZmanShmaGRASunriseToFixedLocalChatzos"
            }
            SofZmanShmaZman::MGA18DegreesToFixedLocalChatzos => {
                "getSofZmanShmaMGA18DegreesToFixedLocalChatzos"
            }
            SofZmanShmaZman::MGA16Point1DegreesToFixedLocalChatzos => {
                "getSofZmanShmaMGA16Point1DegreesToFixedLocalChatzos"
            }
            SofZmanShmaZman::MGA90MinutesToFixedLocalChatzos => {
                "getSofZmanShmaMGA90MinutesToFixedLocalChatzos"
            }
            SofZmanShmaZman::MGA72MinutesToFixedLocalChatzos => {
                "getSofZmanShmaMGA72MinutesToFixedLocalChatzos"
            }
        }
    }
}

// ============================================================================
// SOF ZMAN TFILA IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for SofZmanTfilaZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            SofZmanTfilaZman::GRA => {
                let sunrise = calc.sunrise()?;
                let sunset = calc.sunset();
                calc.get_sof_zman_tfila_from_times(&sunrise, sunset.as_ref(), true)
            }
            SofZmanTfilaZman::MGA => {
                let alos = AlosZman::Minutes72.calculate(calc)?;
                let tzais = TzaisZman::Minutes72.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA19Point8Degrees => {
                let alos = AlosZman::Degrees19Point8.calculate(calc)?;
                let tzais = TzaisZman::Degrees19Point8.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA16Point1Degrees => {
                let alos = AlosZman::Degrees16Point1.calculate(calc)?;
                let tzais = TzaisZman::Degrees16Point1.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA18Degrees => {
                let alos = AlosZman::Degrees18.calculate(calc)?;
                let tzais = TzaisZman::Degrees18.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA72Minutes => {
                let alos = AlosZman::Minutes72.calculate(calc)?;
                let tzais = TzaisZman::Minutes72.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA72MinutesZmanis => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc)?;
                let tzais = TzaisZman::Minutes72Zmanis.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA90Minutes => {
                let alos = AlosZman::Minutes90.calculate(calc)?;
                let tzais = TzaisZman::Minutes90.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA90MinutesZmanis => {
                let alos = AlosZman::Minutes90Zmanis.calculate(calc)?;
                let tzais = TzaisZman::Minutes90Zmanis.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA96Minutes => {
                let alos = AlosZman::Minutes96.calculate(calc)?;
                let tzais = TzaisZman::Minutes96.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::MGA96MinutesZmanis => {
                let alos = AlosZman::Minutes96Zmanis.calculate(calc)?;
                let tzais = TzaisZman::Minutes96Zmanis.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::Hours2BeforeChatzos => {
                let chatzos = ChatzosZman::Astronomical.calculate(calc)?;
                Some(chatzos - Duration::minutes(120))
            }
            SofZmanTfilaZman::MGA120Minutes => {
                let alos = AlosZman::Minutes120.calculate(calc)?;
                let tzais = TzaisZman::Minutes120.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), true)
            }
            SofZmanTfilaZman::AteretTorah => {
                let alos = AlosZman::Minutes72Zmanis.calculate(calc)?;
                let tzais = TzaisZman::AteretTorah.calculate(calc);
                calc.get_sof_zman_tfila_from_times(&alos, tzais.as_ref(), false)
            }
            SofZmanTfilaZman::BaalHatanya => {
                let sunrise = calc.sunrise_offset_by_degrees(1.583)?;
                let sunset = calc.sunset_offset_by_degrees(1.583);
                calc.get_sof_zman_tfila_from_times(&sunrise, sunset.as_ref(), true)
            }
            SofZmanTfilaZman::FixedLocal => {
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                Some(chatzos - Duration::minutes(120))
            }
            SofZmanTfilaZman::GRASunriseToFixedLocalChatzos => {
                let sunrise = calc.sunrise()?;
                let chatzos = ChatzosZman::FixedLocal.calculate(calc)?;
                calc.get_half_day_based_zman_from_times(&sunrise, &chatzos, 4.0)
            }
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // Minutes variants and GRA use elevation-adjusted times
        matches!(
            self,
            SofZmanTfilaZman::GRA
                | SofZmanTfilaZman::MGA
                | SofZmanTfilaZman::MGA72Minutes
                | SofZmanTfilaZman::MGA72MinutesZmanis
                | SofZmanTfilaZman::MGA90Minutes
                | SofZmanTfilaZman::MGA90MinutesZmanis
                | SofZmanTfilaZman::MGA96Minutes
                | SofZmanTfilaZman::MGA96MinutesZmanis
                | SofZmanTfilaZman::MGA120Minutes
                | SofZmanTfilaZman::AteretTorah
                | SofZmanTfilaZman::GRASunriseToFixedLocalChatzos
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            SofZmanTfilaZman::GRA => "getSofZmanTfilaGRA",
            SofZmanTfilaZman::MGA => "getSofZmanTfilaMGA",
            SofZmanTfilaZman::MGA19Point8Degrees => "getSofZmanTfilaMGA19Point8Degrees",
            SofZmanTfilaZman::MGA16Point1Degrees => "getSofZmanTfilaMGA16Point1Degrees",
            SofZmanTfilaZman::MGA18Degrees => "getSofZmanTfilaMGA18Degrees",
            SofZmanTfilaZman::MGA72Minutes => "getSofZmanTfilaMGA72Minutes",
            SofZmanTfilaZman::MGA72MinutesZmanis => "getSofZmanTfilaMGA72MinutesZmanis",
            SofZmanTfilaZman::MGA90Minutes => "getSofZmanTfilaMGA90Minutes",
            SofZmanTfilaZman::MGA90MinutesZmanis => "getSofZmanTfilaMGA90MinutesZmanis",
            SofZmanTfilaZman::MGA96Minutes => "getSofZmanTfilaMGA96Minutes",
            SofZmanTfilaZman::MGA96MinutesZmanis => "getSofZmanTfilaMGA96MinutesZmanis",
            SofZmanTfilaZman::Hours2BeforeChatzos => "getSofZmanTfila2HoursBeforeChatzos",
            SofZmanTfilaZman::MGA120Minutes => "getSofZmanTfilaMGA120Minutes",
            SofZmanTfilaZman::AteretTorah => "getSofZmanTfilaAteretTorah",
            SofZmanTfilaZman::BaalHatanya => "getSofZmanTfilaBaalHatanya",
            SofZmanTfilaZman::FixedLocal => "getSofZmanTfilaFixedLocal",
            SofZmanTfilaZman::GRASunriseToFixedLocalChatzos => {
                "getSofZmanTfilaGRASunriseToFixedLocalChatzos"
            }
        }
    }
}

// ============================================================================
// TZAIS IMPLEMENTATIONS
// ============================================================================

impl ZmanLike for TzaisZman {
    fn calculate<T: TimeZone>(&self, calc: &mut ZmanimCalculator<T>) -> Option<DateTime<Utc>> {
        match self {
            TzaisZman::Degrees8Point5 => calc.sunset_offset_by_degrees(8.5),
            TzaisZman::Minutes50 => {
                let sunset = calc.sunset()?;
                Some(sunset + Duration::minutes(50))
            }
            TzaisZman::Minutes60 => {
                let sunset = calc.sunset()?;
                Some(sunset + Duration::minutes(60))
            }
            TzaisZman::Minutes72 => {
                let sunset = calc.sunset()?;
                Some(sunset + Duration::minutes(72))
            }
            TzaisZman::Minutes72Zmanis => {
                let sunset = calc.sunset()?;
                calc.offset_by_shaah_zmanis_gra(sunset, 1.2)
            }
            TzaisZman::Minutes90 => {
                let sunset = calc.sunset()?;
                Some(sunset + Duration::minutes(90))
            }
            TzaisZman::Minutes90Zmanis => {
                let sunset = calc.sunset()?;
                calc.offset_by_shaah_zmanis_gra(sunset, 1.5)
            }
            TzaisZman::Minutes96 => {
                let sunset = calc.sunset()?;
                Some(sunset + Duration::minutes(96))
            }
            TzaisZman::Minutes96Zmanis => {
                let sunset = calc.sunset()?;
                calc.offset_by_shaah_zmanis_gra(sunset, 1.6)
            }
            TzaisZman::Minutes120 => {
                let sunset = calc.sunset()?;
                Some(sunset + Duration::minutes(120))
            }
            TzaisZman::Minutes120Zmanis => {
                let sunset = calc.sunset()?;
                calc.offset_by_shaah_zmanis_gra(sunset, 2.0)
            }
            TzaisZman::Degrees16Point1 => calc.sunset_offset_by_degrees(16.1),
            TzaisZman::Degrees18 => calc.sunset_offset_by_degrees(18.0), // ASTRONOMICAL_ZENITH
            TzaisZman::Degrees19Point8 => calc.sunset_offset_by_degrees(19.8),
            TzaisZman::Degrees26 => calc.sunset_offset_by_degrees(26.0),
            TzaisZman::AteretTorah => {
                let sunset = calc.sunset()?;
                Some(sunset + calc.config.ateret_torah_sunset_offset)
            }
            TzaisZman::BaalHatanya => calc.sunset_offset_by_degrees(6.0),
            TzaisZman::GeonimDegrees3Point7 => calc.sunset_offset_by_degrees(3.7),
            TzaisZman::GeonimDegrees3Point8 => calc.sunset_offset_by_degrees(3.8),
            TzaisZman::GeonimDegrees5Point95 => calc.sunset_offset_by_degrees(5.95),
            TzaisZman::Geonim3Point65 => calc.sunset_offset_by_degrees(3.65),
            TzaisZman::Geonim3Point676 => calc.sunset_offset_by_degrees(3.676),
            TzaisZman::GeonimDegrees4Point61 => calc.sunset_offset_by_degrees(4.61),
            TzaisZman::GeonimDegrees4Point37 => calc.sunset_offset_by_degrees(4.37),
            TzaisZman::GeonimDegrees5Point88 => calc.sunset_offset_by_degrees(5.88),
            TzaisZman::GeonimDegrees4Point8 => calc.sunset_offset_by_degrees(4.8),
            TzaisZman::GeonimDegrees6Point45 => calc.sunset_offset_by_degrees(6.45),
            TzaisZman::GeonimDegrees7Point083 => calc.sunset_offset_by_degrees(7.0 + (5.0 / 60.0)),
            TzaisZman::GeonimDegrees7Point67 => calc.sunset_offset_by_degrees(7.67),
            TzaisZman::GeonimDegrees8Point5 => calc.sunset_offset_by_degrees(8.5),
            TzaisZman::GeonimDegrees9Point3 => calc.sunset_offset_by_degrees(9.3),
            TzaisZman::GeonimDegrees9Point75 => calc.sunset_offset_by_degrees(9.75),
        }
    }
    #[cfg(test)]
    fn uses_elevation(&self) -> bool {
        // Minutes variants and AteretTorah use calc.sunset() which uses elevation_calc
        // Degree variants use sunset_offset_by_degrees() which uses sea_level_calc
        matches!(
            self,
            TzaisZman::Minutes50
                | TzaisZman::Minutes60
                | TzaisZman::Minutes72
                | TzaisZman::Minutes72Zmanis
                | TzaisZman::Minutes90
                | TzaisZman::Minutes90Zmanis
                | TzaisZman::Minutes96
                | TzaisZman::Minutes96Zmanis
                | TzaisZman::Minutes120
                | TzaisZman::Minutes120Zmanis
                | TzaisZman::AteretTorah
        )
    }
    #[cfg(test)]
    fn java_function_name(&self) -> &str {
        match self {
            TzaisZman::Degrees8Point5 => "getTzaisGeonim8Point5Degrees",
            TzaisZman::Minutes50 => "getTzais50",
            TzaisZman::Minutes60 => "getTzais60",
            TzaisZman::Minutes72 => "getTzais72",
            TzaisZman::Minutes72Zmanis => "getTzais72Zmanis",
            TzaisZman::Minutes90 => "getTzais90",
            TzaisZman::Minutes90Zmanis => "getTzais90Zmanis",
            TzaisZman::Minutes96 => "getTzais96",
            TzaisZman::Minutes96Zmanis => "getTzais96Zmanis",
            TzaisZman::Minutes120 => "getTzais120",
            TzaisZman::Minutes120Zmanis => "getTzais120Zmanis",
            TzaisZman::Degrees16Point1 => "getTzais16Point1Degrees",
            TzaisZman::Degrees18 => "getTzais18Degrees",
            TzaisZman::Degrees19Point8 => "getTzais19Point8Degrees",
            TzaisZman::Degrees26 => "getTzais26Degrees",
            TzaisZman::AteretTorah => "getTzaisAteretTorah",
            TzaisZman::BaalHatanya => "getTzaisBaalHatanya",
            TzaisZman::GeonimDegrees3Point7 => "getTzaisGeonim3Point7Degrees",
            TzaisZman::GeonimDegrees3Point8 => "getTzaisGeonim3Point8Degrees",
            TzaisZman::GeonimDegrees5Point95 => "getTzaisGeonim5Point95Degrees",
            TzaisZman::Geonim3Point65 => "getTzaisGeonim3Point65Degrees",
            TzaisZman::Geonim3Point676 => "getTzaisGeonim3Point676Degrees",
            TzaisZman::GeonimDegrees4Point61 => "getTzaisGeonim4Point61Degrees",
            TzaisZman::GeonimDegrees4Point37 => "getTzaisGeonim4Point37Degrees",
            TzaisZman::GeonimDegrees5Point88 => "getTzaisGeonim5Point88Degrees",
            TzaisZman::GeonimDegrees4Point8 => "getTzaisGeonim4Point8Degrees",
            TzaisZman::GeonimDegrees6Point45 => "getTzaisGeonim6Point45Degrees",
            TzaisZman::GeonimDegrees7Point083 => "getTzaisGeonim7Point083Degrees",
            TzaisZman::GeonimDegrees7Point67 => "getTzaisGeonim7Point67Degrees",
            TzaisZman::GeonimDegrees8Point5 => "getTzaisGeonim8Point5Degrees",
            TzaisZman::GeonimDegrees9Point3 => "getTzaisGeonim9Point3Degrees",
            TzaisZman::GeonimDegrees9Point75 => "getTzaisGeonim9Point75Degrees",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::zman::ZmanLike;

    #[test]
    fn test_bain_hashmashos_degrees_above_horizon() {
        assert!(!BainHashmashosZman::RabbeinuTam13Point24Degrees.degrees_above_horizon());
    }
}
