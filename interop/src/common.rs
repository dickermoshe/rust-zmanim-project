//! Shared conversion helpers used by bridge modules (not exported over FFI directly).
#![allow(missing_docs)]

use icu_calendar::{Date, cal::Hebrew};
use jiff::{Timestamp, civil::Date as JiffDate, tz::TimeZone};
use jiff_icu::ConvertTryFrom;
use kosher_rust::{
    calendar::{
        HebrewCalendarDate, YearLengthType,
        holiday::Holiday,
        month::{ADAR, ADARI, AV, ELUL, IYYAR, KISLEV, NISAN, SHEVAT, SIVAN, TAMMUZ, TEVET, TISHREI, ḤESHVAN},
        parsha::Parsha,
    },
    limudim::{Side, Tractate},
    zmanim::types::error::ZmanimError,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// Gregorian civil date passed across the FFI boundary.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CivilDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

/// Hebrew calendar date passed across the FFI boundary.
///
/// Month codes follow ICU Hebrew month numbering used in this crate:
/// 1 = Tishrei … 12 = Elul, and 25 = Adar I (leap years only).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HebrewDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

pub fn civil_date_to_jiff(date: CivilDate) -> Option<JiffDate> {
    JiffDate::new(
        i16::try_from(date.year).ok()?,
        i8::try_from(date.month).ok()?,
        i8::try_from(date.day).ok()?,
    )
    .ok()
}

pub fn jiff_to_civil_date(date: JiffDate) -> CivilDate {
    CivilDate {
        year: i32::from(date.year()),
        month: u8::try_from(date.month()).unwrap_or(0),
        day: u8::try_from(date.day()).unwrap_or(0),
    }
}

pub fn encode_hebrew_month(month: icu_calendar::types::Month) -> u8 {
    if month == ADARI {
        25
    } else if month == TISHREI {
        1
    } else if month == ḤESHVAN {
        2
    } else if month == KISLEV {
        3
    } else if month == TEVET {
        4
    } else if month == SHEVAT {
        5
    } else if month == ADAR {
        6
    } else if month == NISAN {
        7
    } else if month == IYYAR {
        8
    } else if month == SIVAN {
        9
    } else if month == TAMMUZ {
        10
    } else if month == AV {
        11
    } else if month == ELUL {
        12
    } else {
        0
    }
}

pub fn decode_hebrew_month(code: u8) -> Option<icu_calendar::types::Month> {
    match code {
        1 => Some(TISHREI),
        2 => Some(ḤESHVAN),
        3 => Some(KISLEV),
        4 => Some(TEVET),
        5 => Some(SHEVAT),
        6 => Some(ADAR),
        7 => Some(NISAN),
        8 => Some(IYYAR),
        9 => Some(SIVAN),
        10 => Some(TAMMUZ),
        11 => Some(AV),
        12 => Some(ELUL),
        25 => Some(ADARI),
        _ => None,
    }
}

pub fn gregorian_to_hebrew(date: CivilDate) -> Option<HebrewDate> {
    let jiff = civil_date_to_jiff(date)?;
    let hebrew = jiff.hebrew_date();
    Some(HebrewDate {
        year: hebrew.year().extended_year(),
        month: encode_hebrew_month(hebrew.month().to_input()),
        day: hebrew.day_of_month().0,
    })
}

pub fn hebrew_to_gregorian(date: HebrewDate) -> Option<CivilDate> {
    let month = decode_hebrew_month(date.month)?;
    let hebrew = Date::try_new_hebrew_v2(date.year, month, date.day).ok()?;
    let iso: Date<icu_calendar::cal::Iso> = hebrew.to_calendar(icu_calendar::cal::Iso);
    let jiff = JiffDate::convert_try_from(iso).ok()?;
    Some(jiff_to_civil_date(jiff))
}

pub fn hebrew_date_from_civil(date: CivilDate) -> Option<Date<Hebrew>> {
    let jiff = civil_date_to_jiff(date)?;
    Some(jiff.hebrew_date())
}

pub fn parse_timezone(iana: &str) -> Option<TimeZone> {
    if iana.is_empty() {
        return None;
    }
    TimeZone::get(iana).ok()
}

pub fn timestamp_to_epoch_ms(timestamp: Timestamp) -> i64 {
    timestamp.as_millisecond()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ZmanimErrorCode {
    InvalidLatitude = 0,
    InvalidLongitude = 1,
    InvalidElevation = 2,
    TimeZoneRequired = 3,
    CalculationError = 4,
    AllDay = 5,
    AllNight = 6,
    TimeConversionError = 7,
    InvalidForDate = 8,
    InvalidHours = 9,
}

impl From<ZmanimError> for ZmanimErrorCode {
    fn from(error: ZmanimError) -> Self {
        match error {
            ZmanimError::InvalidLatitude => Self::InvalidLatitude,
            ZmanimError::InvalidLongitude => Self::InvalidLongitude,
            ZmanimError::InvalidElevation => Self::InvalidElevation,
            ZmanimError::TimeZoneRequired => Self::TimeZoneRequired,
            ZmanimError::CalculationError => Self::CalculationError,
            ZmanimError::AllDay => Self::AllDay,
            ZmanimError::AllNight => Self::AllNight,
            ZmanimError::TimeConversionError => Self::TimeConversionError,
            ZmanimError::InvalidForDate => Self::InvalidForDate,
            ZmanimError::InvalidHours => Self::InvalidHours,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum YearLengthTypeCode {
    Chaserim = 0,
    Kesidran = 1,
    Shelaimim = 2,
}

impl From<YearLengthType> for YearLengthTypeCode {
    fn from(value: YearLengthType) -> Self {
        match value {
            YearLengthType::Chaserim => Self::Chaserim,
            YearLengthType::Kesidran => Self::Kesidran,
            YearLengthType::Shelaimim => Self::Shelaimim,
        }
    }
}

pub fn holiday_to_codes(holiday: Holiday) -> (HolidayCode, u8) {
    match holiday {
        Holiday::ErevPesach => (HolidayCode::ErevPesach, 0),
        Holiday::Pesach => (HolidayCode::Pesach, 0),
        Holiday::CholHamoedPesach => (HolidayCode::CholHamoedPesach, 0),
        Holiday::PesachSheni => (HolidayCode::PesachSheni, 0),
        Holiday::ErevShavuos => (HolidayCode::ErevShavuos, 0),
        Holiday::Shavuos => (HolidayCode::Shavuos, 0),
        Holiday::SeventeenthOfTammuz => (HolidayCode::SeventeenthOfTammuz, 0),
        Holiday::TishahBav => (HolidayCode::TishahBav, 0),
        Holiday::TuBav => (HolidayCode::TuBav, 0),
        Holiday::ErevRoshHashana => (HolidayCode::ErevRoshHashana, 0),
        Holiday::RoshHashana => (HolidayCode::RoshHashana, 0),
        Holiday::FastOfGedalyah => (HolidayCode::FastOfGedalyah, 0),
        Holiday::ErevYomKippur => (HolidayCode::ErevYomKippur, 0),
        Holiday::YomKippur => (HolidayCode::YomKippur, 0),
        Holiday::ErevSuccos => (HolidayCode::ErevSuccos, 0),
        Holiday::Succos => (HolidayCode::Succos, 0),
        Holiday::CholHamoedSuccos => (HolidayCode::CholHamoedSuccos, 0),
        Holiday::HoshanaRabbah => (HolidayCode::HoshanaRabbah, 0),
        Holiday::SheminiAtzeres => (HolidayCode::SheminiAtzeres, 0),
        Holiday::SimchasTorah => (HolidayCode::SimchasTorah, 0),
        Holiday::Chanukah(day) => (HolidayCode::Chanukah, day),
        Holiday::TenthOfTeves => (HolidayCode::TenthOfTeves, 0),
        Holiday::TuBshvat => (HolidayCode::TuBshvat, 0),
        Holiday::FastOfEsther => (HolidayCode::FastOfEsther, 0),
        Holiday::Purim => (HolidayCode::Purim, 0),
        Holiday::ShushanPurim => (HolidayCode::ShushanPurim, 0),
        Holiday::PurimKatan => (HolidayCode::PurimKatan, 0),
        Holiday::RoshChodesh => (HolidayCode::RoshChodesh, 0),
        Holiday::YomHaShoah => (HolidayCode::YomHaShoah, 0),
        Holiday::YomHazikaron => (HolidayCode::YomHazikaron, 0),
        Holiday::YomHaatzmaut => (HolidayCode::YomHaatzmaut, 0),
        Holiday::YomYerushalayim => (HolidayCode::YomYerushalayim, 0),
        Holiday::LagBomer => (HolidayCode::LagBomer, 0),
        Holiday::ShushanPurimKatan => (HolidayCode::ShushanPurimKatan, 0),
        Holiday::IsruChag => (HolidayCode::IsruChag, 0),
        Holiday::YomKippurKatan => (HolidayCode::YomKippurKatan, 0),
        Holiday::Behab => (HolidayCode::Behab, 0),
        Holiday::FastOfTheFirstborn => (HolidayCode::FastOfTheFirstborn, 0),
        Holiday::CountOfTheOmer(day) => (HolidayCode::CountOfTheOmer, day),
        Holiday::BirchasHachamah => (HolidayCode::BirchasHachamah, 0),
        Holiday::MacharHachodesh => (HolidayCode::MacharHachodesh, 0),
        Holiday::ShabbosMevarchim => (HolidayCode::ShabbosMevarchim, 0),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum HolidayCode {
    ErevPesach = 0,
    Pesach = 1,
    CholHamoedPesach = 2,
    PesachSheni = 3,
    ErevShavuos = 4,
    Shavuos = 5,
    SeventeenthOfTammuz = 6,
    TishahBav = 7,
    TuBav = 8,
    ErevRoshHashana = 9,
    RoshHashana = 10,
    FastOfGedalyah = 11,
    ErevYomKippur = 12,
    YomKippur = 13,
    ErevSuccos = 14,
    Succos = 15,
    CholHamoedSuccos = 16,
    HoshanaRabbah = 17,
    SheminiAtzeres = 18,
    SimchasTorah = 19,
    Chanukah = 20,
    TenthOfTeves = 21,
    TuBshvat = 22,
    FastOfEsther = 23,
    Purim = 24,
    ShushanPurim = 25,
    PurimKatan = 26,
    RoshChodesh = 27,
    YomHaShoah = 28,
    YomHazikaron = 29,
    YomHaatzmaut = 30,
    YomYerushalayim = 31,
    LagBomer = 32,
    ShushanPurimKatan = 33,
    IsruChag = 34,
    YomKippurKatan = 35,
    Behab = 36,
    FastOfTheFirstborn = 37,
    CountOfTheOmer = 38,
    BirchasHachamah = 39,
    MacharHachodesh = 40,
    ShabbosMevarchim = 41,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ParshaCode {
    Bereshis = 0,
    Noach = 1,
    LechLecha = 2,
    Vayera = 3,
    ChayeiSara = 4,
    Toldos = 5,
    Vayetzei = 6,
    Vayishlach = 7,
    Vayeshev = 8,
    Miketz = 9,
    Vayigash = 10,
    Vayechi = 11,
    Shemos = 12,
    Vaera = 13,
    Bo = 14,
    Beshalach = 15,
    Yisro = 16,
    Mishpatim = 17,
    Terumah = 18,
    Tetzaveh = 19,
    KiSisa = 20,
    Vayakhel = 21,
    Pekudei = 22,
    Vayikra = 23,
    Tzav = 24,
    Shmini = 25,
    Tazria = 26,
    Metzora = 27,
    AchreiMos = 28,
    Kedoshim = 29,
    Emor = 30,
    Behar = 31,
    Bechukosai = 32,
    Bamidbar = 33,
    Nasso = 34,
    Behaaloscha = 35,
    Shlach = 36,
    Korach = 37,
    Chukas = 38,
    Balak = 39,
    Pinchas = 40,
    Matos = 41,
    Masei = 42,
    Devarim = 43,
    Vaeschanan = 44,
    Eikev = 45,
    Reeh = 46,
    Shoftim = 47,
    KiSeitzei = 48,
    KiSavo = 49,
    Nitzavim = 50,
    Vayeilech = 51,
    HaAzinu = 52,
    VezosHabracha = 53,
    VayakhelPekudei = 54,
    TazriaMetzora = 55,
    AchreiMosKedoshim = 56,
    BeharBechukosai = 57,
    ChukasBalak = 58,
    MatosMasei = 59,
    NitzavimVayeilech = 60,
    Shekalim = 61,
    Zachor = 62,
    Parah = 63,
    Hachodesh = 64,
    Shuva = 65,
    Shira = 66,
    Hagadol = 67,
    Chazon = 68,
    Nachamu = 69,
}

impl From<Parsha> for ParshaCode {
    fn from(value: Parsha) -> Self {
        Self::try_from(u8::from(value)).unwrap_or(Self::Bereshis)
    }
}

impl TryFrom<ParshaCode> for Parsha {
    type Error = ();

    fn try_from(value: ParshaCode) -> Result<Self, Self::Error> {
        Parsha::try_from(u8::from(value)).map_err(|_| ())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TractateCode {
    Berachos,
    Peah,
    Demai,
    Kilayim,
    Sheviis,
    Terumos,
    Maasros,
    MaaserSheni,
    Chalah,
    Orlah,
    Bikurim,
    Shabbos,
    Eruvin,
    Pesachim,
    Shekalim,
    Yoma,
    Sukkah,
    Beitzah,
    RoshHashanah,
    Taanis,
    Megillah,
    MoedKatan,
    Chagigah,
    Yevamos,
    Kesubos,
    Nedarim,
    Nazir,
    Sotah,
    Gitin,
    Kiddushin,
    BavaKamma,
    BavaMetzia,
    BavaBasra,
    Sanhedrin,
    Makkos,
    Shevuos,
    Eduyos,
    AvodahZarah,
    Avos,
    Horiyos,
    Zevachim,
    Menachos,
    Chullin,
    Bechoros,
    Arachin,
    Temurah,
    Kerisos,
    Meilah,
    Tamid,
    Midos,
    Kinnim,
    Keilim,
    Ohalos,
    Negaim,
    Parah,
    Taharos,
    Mikvaos,
    Niddah,
    Machshirin,
    Zavim,
    TevulYom,
    Yadayim,
    Uktzin,
}

pub fn tractate_to_code(value: Tractate) -> TractateCode {
    match value {
        Tractate::Berachos => TractateCode::Berachos,
        Tractate::Peah => TractateCode::Peah,
        Tractate::Demai => TractateCode::Demai,
        Tractate::Kilayim => TractateCode::Kilayim,
        Tractate::Sheviis => TractateCode::Sheviis,
        Tractate::Terumos => TractateCode::Terumos,
        Tractate::Maasros => TractateCode::Maasros,
        Tractate::MaaserSheni => TractateCode::MaaserSheni,
        Tractate::Chalah => TractateCode::Chalah,
        Tractate::Orlah => TractateCode::Orlah,
        Tractate::Bikurim => TractateCode::Bikurim,
        Tractate::Shabbos => TractateCode::Shabbos,
        Tractate::Eruvin => TractateCode::Eruvin,
        Tractate::Pesachim => TractateCode::Pesachim,
        Tractate::Shekalim => TractateCode::Shekalim,
        Tractate::Yoma => TractateCode::Yoma,
        Tractate::Sukkah => TractateCode::Sukkah,
        Tractate::Beitzah => TractateCode::Beitzah,
        Tractate::RoshHashanah => TractateCode::RoshHashanah,
        Tractate::Taanis => TractateCode::Taanis,
        Tractate::Megillah => TractateCode::Megillah,
        Tractate::MoedKatan => TractateCode::MoedKatan,
        Tractate::Chagigah => TractateCode::Chagigah,
        Tractate::Yevamos => TractateCode::Yevamos,
        Tractate::Kesubos => TractateCode::Kesubos,
        Tractate::Nedarim => TractateCode::Nedarim,
        Tractate::Nazir => TractateCode::Nazir,
        Tractate::Sotah => TractateCode::Sotah,
        Tractate::Gitin => TractateCode::Gitin,
        Tractate::Kiddushin => TractateCode::Kiddushin,
        Tractate::BavaKamma => TractateCode::BavaKamma,
        Tractate::BavaMetzia => TractateCode::BavaMetzia,
        Tractate::BavaBasra => TractateCode::BavaBasra,
        Tractate::Sanhedrin => TractateCode::Sanhedrin,
        Tractate::Makkos => TractateCode::Makkos,
        Tractate::Shevuos => TractateCode::Shevuos,
        Tractate::Eduyos => TractateCode::Eduyos,
        Tractate::AvodahZarah => TractateCode::AvodahZarah,
        Tractate::Avos => TractateCode::Avos,
        Tractate::Horiyos => TractateCode::Horiyos,
        Tractate::Zevachim => TractateCode::Zevachim,
        Tractate::Menachos => TractateCode::Menachos,
        Tractate::Chullin => TractateCode::Chullin,
        Tractate::Bechoros => TractateCode::Bechoros,
        Tractate::Arachin => TractateCode::Arachin,
        Tractate::Temurah => TractateCode::Temurah,
        Tractate::Kerisos => TractateCode::Kerisos,
        Tractate::Meilah => TractateCode::Meilah,
        Tractate::Tamid => TractateCode::Tamid,
        Tractate::Midos => TractateCode::Midos,
        Tractate::Kinnim => TractateCode::Kinnim,
        Tractate::Keilim => TractateCode::Keilim,
        Tractate::Ohalos => TractateCode::Ohalos,
        Tractate::Negaim => TractateCode::Negaim,
        Tractate::Parah => TractateCode::Parah,
        Tractate::Taharos => TractateCode::Taharos,
        Tractate::Mikvaos => TractateCode::Mikvaos,
        Tractate::Niddah => TractateCode::Niddah,
        Tractate::Machshirin => TractateCode::Machshirin,
        Tractate::Zavim => TractateCode::Zavim,
        Tractate::TevulYom => TractateCode::TevulYom,
        Tractate::Yadayim => TractateCode::Yadayim,
        Tractate::Uktzin => TractateCode::Uktzin,
    }
}

pub fn code_to_tractate(value: TractateCode) -> Tractate {
    match value {
        TractateCode::Berachos => Tractate::Berachos,
        TractateCode::Peah => Tractate::Peah,
        TractateCode::Demai => Tractate::Demai,
        TractateCode::Kilayim => Tractate::Kilayim,
        TractateCode::Sheviis => Tractate::Sheviis,
        TractateCode::Terumos => Tractate::Terumos,
        TractateCode::Maasros => Tractate::Maasros,
        TractateCode::MaaserSheni => Tractate::MaaserSheni,
        TractateCode::Chalah => Tractate::Chalah,
        TractateCode::Orlah => Tractate::Orlah,
        TractateCode::Bikurim => Tractate::Bikurim,
        TractateCode::Shabbos => Tractate::Shabbos,
        TractateCode::Eruvin => Tractate::Eruvin,
        TractateCode::Pesachim => Tractate::Pesachim,
        TractateCode::Shekalim => Tractate::Shekalim,
        TractateCode::Yoma => Tractate::Yoma,
        TractateCode::Sukkah => Tractate::Sukkah,
        TractateCode::Beitzah => Tractate::Beitzah,
        TractateCode::RoshHashanah => Tractate::RoshHashanah,
        TractateCode::Taanis => Tractate::Taanis,
        TractateCode::Megillah => Tractate::Megillah,
        TractateCode::MoedKatan => Tractate::MoedKatan,
        TractateCode::Chagigah => Tractate::Chagigah,
        TractateCode::Yevamos => Tractate::Yevamos,
        TractateCode::Kesubos => Tractate::Kesubos,
        TractateCode::Nedarim => Tractate::Nedarim,
        TractateCode::Nazir => Tractate::Nazir,
        TractateCode::Sotah => Tractate::Sotah,
        TractateCode::Gitin => Tractate::Gitin,
        TractateCode::Kiddushin => Tractate::Kiddushin,
        TractateCode::BavaKamma => Tractate::BavaKamma,
        TractateCode::BavaMetzia => Tractate::BavaMetzia,
        TractateCode::BavaBasra => Tractate::BavaBasra,
        TractateCode::Sanhedrin => Tractate::Sanhedrin,
        TractateCode::Makkos => Tractate::Makkos,
        TractateCode::Shevuos => Tractate::Shevuos,
        TractateCode::Eduyos => Tractate::Eduyos,
        TractateCode::AvodahZarah => Tractate::AvodahZarah,
        TractateCode::Avos => Tractate::Avos,
        TractateCode::Horiyos => Tractate::Horiyos,
        TractateCode::Zevachim => Tractate::Zevachim,
        TractateCode::Menachos => Tractate::Menachos,
        TractateCode::Chullin => Tractate::Chullin,
        TractateCode::Bechoros => Tractate::Bechoros,
        TractateCode::Arachin => Tractate::Arachin,
        TractateCode::Temurah => Tractate::Temurah,
        TractateCode::Kerisos => Tractate::Kerisos,
        TractateCode::Meilah => Tractate::Meilah,
        TractateCode::Tamid => Tractate::Tamid,
        TractateCode::Midos => Tractate::Midos,
        TractateCode::Kinnim => Tractate::Kinnim,
        TractateCode::Keilim => Tractate::Keilim,
        TractateCode::Ohalos => Tractate::Ohalos,
        TractateCode::Negaim => Tractate::Negaim,
        TractateCode::Parah => Tractate::Parah,
        TractateCode::Taharos => Tractate::Taharos,
        TractateCode::Mikvaos => Tractate::Mikvaos,
        TractateCode::Niddah => Tractate::Niddah,
        TractateCode::Machshirin => Tractate::Machshirin,
        TractateCode::Zavim => Tractate::Zavim,
        TractateCode::TevulYom => Tractate::TevulYom,
        TractateCode::Yadayim => Tractate::Yadayim,
        TractateCode::Uktzin => Tractate::Uktzin,
    }
}

pub fn side_to_code(value: Side) -> SideCode {
    match value {
        Side::Aleph => SideCode::Aleph,
        Side::Bet => SideCode::Bet,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SideCode {
    Aleph,
    Bet,
}
