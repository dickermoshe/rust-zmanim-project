#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeitzZman;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShkiaZman;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeaLevelNeitzZman;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeaLevelShkiaZman;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlosZman {
    Minutes120,
    Minutes120Zmanis,
    Degrees16Point1,
    Degrees18,
    Degrees19,
    Degrees19Point8,
    Degrees26,
    Minutes60,
    Minutes72,
    Minutes72Zmanis,
    Minutes90,
    Minutes90Zmanis,
    Minutes96,
    Minutes96Zmanis,
    BaalHatanya,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BainHashmashosZman {
    RabbeinuTam13Point24Degrees,
    RabbeinuTam13Point5MinutesBefore7Point083Degrees,
    RabbeinuTam2Stars,
    RabbeinuTam58Point5Minutes,
    Yereim13Point5Minutes,
    Yereim16Point875Minutes,
    Yereim18Minutes,
    // This is not supported by our library until
    // we can figure out how to handle refraction/elevation for degree
    // calculations for when the sun is above the horizon.
    // Yereim2Point1Degrees,
    // Yereim2Point8Degrees,
    // Yereim3Point05Degrees,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandleLightingZman;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChatzosZman {
    Astronomical,
    HalfDay,
    FixedLocal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MinchaGedolaZman {
    SunriseSunset,
    Degrees16Point1,
    Minutes30,
    Minutes72,
    AhavatShalom,
    AteretTorah,
    BaalHatanya,
    BaalHatanyaGreaterThan30,
    GRAFixedLocalChatzos30Minutes,
    GreaterThan30,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MinchaKetanaZman {
    SunriseSunset,
    Degrees16Point1,
    Minutes72,
    AhavatShalom,
    AteretTorah,
    BaalHatanya,
    GRAFixedLocalChatzosToSunset,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MisheyakirZman {
    Degrees10Point2,
    Degrees11,
    Degrees11Point5,
    Degrees7Point65,
    Degrees9Point5,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlagHaminchaZman {
    AhavatShalom,
    Degrees16Point1ToTzaisGeonim7Point083,
    AlosToSunset,
    SunriseSunset,
    Minutes120,
    Minutes120Zmanis,
    Degrees16Point1,
    Degrees18,
    Degrees19Point8,
    Degrees26,
    Minutes60,
    Minutes72,
    Minutes72Zmanis,
    Minutes90,
    Minutes90Zmanis,
    Minutes96,
    Minutes96Zmanis,
    AteretTorah,
    BaalHatanya,
    GRAFixedLocalChatzosToSunset,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamuchLeMinchaKetanaZman {
    Degrees16Point1,
    Minutes72,
    GRA,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SofZmanAchilasChametzZman {
    BaalHatanya,
    GRA,
    MGA16Point1Degrees,
    MGA72Minutes,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SofZmanBiurChametzZman {
    BaalHatanya,
    GRA,
    MGA16Point1Degrees,
    MGA72Minutes,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SofZmanShmaZman {
    Hours3BeforeChatzos,
    Alos16Point1ToSunset,
    Alos16Point1ToTzaisGeonim7Point083Degrees,
    AteretTorah,
    BaalHatanya,
    FixedLocal,
    GRA,
    GRASunriseToFixedLocalChatzos,
    KolEliyahu,
    MGA,
    MGA120Minutes,
    MGA16Point1Degrees,
    MGA16Point1DegreesToFixedLocalChatzos,
    MGA18Degrees,
    MGA18DegreesToFixedLocalChatzos,
    MGA19Point8Degrees,
    MGA72Minutes,
    MGA72MinutesToFixedLocalChatzos,
    MGA72MinutesZmanis,
    MGA90Minutes,
    MGA90MinutesToFixedLocalChatzos,
    MGA90MinutesZmanis,
    MGA96Minutes,
    MGA96MinutesZmanis,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SofZmanTfilaZman {
    Hours2BeforeChatzos,
    AteretTorah,
    BaalHatanya,
    FixedLocal,
    GRA,
    GRASunriseToFixedLocalChatzos,
    MGA,
    MGA120Minutes,
    MGA16Point1Degrees,
    MGA18Degrees,
    MGA19Point8Degrees,
    MGA72Minutes,
    MGA72MinutesZmanis,
    MGA90Minutes,
    MGA90MinutesZmanis,
    MGA96Minutes,
    MGA96MinutesZmanis,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TzaisZman {
    Degrees8Point5,
    Minutes120,
    Minutes120Zmanis,
    Degrees16Point1,
    Degrees18,
    Degrees19Point8,
    Degrees26,
    Minutes50,
    Minutes60,
    Minutes72,
    Minutes72Zmanis,
    Minutes90,
    Minutes90Zmanis,
    Minutes96,
    Minutes96Zmanis,
    AteretTorah,
    BaalHatanya,
    Geonim3Point65,
    Geonim3Point676,
    GeonimDegrees3Point7,
    GeonimDegrees3Point8,
    GeonimDegrees4Point37,
    GeonimDegrees4Point61,
    GeonimDegrees4Point8,
    GeonimDegrees5Point88,
    GeonimDegrees5Point95,
    GeonimDegrees6Point45,
    GeonimDegrees7Point083,
    GeonimDegrees7Point67,
    GeonimDegrees8Point5,
    GeonimDegrees9Point3,
    GeonimDegrees9Point75,
}
