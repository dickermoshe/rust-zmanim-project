use std::str::FromStr;

use chrono::{Duration, NaiveDate};
use chrono_tz::{Tz, TZ_VARIANTS};
use flutter_rust_bridge::frb;
use hebrew_holiday_calendar::{HebrewHolidayCalendar, HebrewMonth, Holiday};
use icu_calendar::{
    cal::Hebrew,
    options::{DateAddOptions, Overflow},
    types::DateDuration,
    Date, Gregorian,
};
use lazy_static::lazy_static;
use zmanim_calculator::{prelude::*, presets::ALL};

use tzf_rs::DefaultFinder;

lazy_static! {
    static ref FINDER: DefaultFinder = DefaultFinder::new();
}

#[frb(sync)]
/// Find the timezone for a given longitude and latitude
pub fn find_timezone(longitude: f64, latitude: f64) -> String {
    FINDER.get_tz_name(longitude, latitude).to_string()
}
/// An Opaque wrapper for a ZmanimPreset to be used in the Dart side
#[frb(opaque)]
pub struct ZmanimPreset {
    zman: &'static ZmanPreset<'static>,
}
impl ZmanimPreset {
    /// Get the name of the ZmanimPreset
    /// This is also the method name in the Java side
    #[frb(sync)]
    pub fn name(&self) -> String {
        self.zman.name.to_string()
    }
}
/// Get all the timezones supported by the library
#[frb(sync)]
pub fn timezones() -> Vec<String> {
    TZ_VARIANTS.iter().map(|tz| tz.to_string()).collect()
}
/// Calculate a zman at a given location and date
#[frb(sync)]
pub fn calculate_zman(
    ateret_torah_sunset_offset_minutes: i64,
    candle_lighting_offset_minutes: i64,
    use_astronomical_chatzos_for_other_zmanim: bool,
    latitude: f64,
    longitude: f64,
    elevation: f64,
    timezone: String,
    random_year: i64,
    random_month: i64,
    random_day: i64,
    use_elevation: bool,
    zman: &ZmanimPreset,
) -> Option<(String, i64)> {
    let tz = Tz::from_str(&timezone).unwrap();
    let date = NaiveDate::from_ymd_opt(random_year as i32, random_month as u32, random_day as u32)
        .unwrap();
    let location = Location::new(latitude, longitude, elevation, Some(tz)).ok()?;
    let config = CalculatorConfig {
        use_elevation,
        ateret_torah_sunset_offset: Duration::minutes(ateret_torah_sunset_offset_minutes),
        candle_lighting_offset: Duration::minutes(candle_lighting_offset_minutes),
        use_astronomical_chatzos_for_other_zmanim,
    };
    let mut calculator = ZmanimCalculator::new(location, date, config).ok()?;
    let zman = calculator.calculate(zman.zman).ok()?;
    let at_tz = zman.with_timezone(&tz);
    Some((at_tz.to_string(), zman.timestamp_millis()))
}
/// Get all the ZmanimPresets supported by the library
#[frb(sync)]
pub fn presets() -> Vec<ZmanimPreset> {
    ALL.iter().map(|zman| ZmanimPreset { zman }).collect()
}
/// Creates dates similar to Javas constructor which will clamp a day that is too high
/// instead of failing outright
fn create_clamped_hebrew_date(year: i32, month: u8, day: u8) -> Option<Date<Hebrew>> {
    if let Some(date) =
        Date::<Hebrew>::from_hebrew_date(year, HebrewMonth::try_from(month).unwrap(), day)
    {
        Some(date)
    } else {
        Date::<Hebrew>::from_hebrew_date(year, HebrewMonth::try_from(month).unwrap(), day - 1)
    }
}

#[frb(sync)]
pub fn jewish_date_to_gregorian_date(year: i32, month: u8, day: u8) -> Option<(i32, u8, u8)> {
    let date = create_clamped_hebrew_date(year, month, day)?;
    let gregorian_date = date.to_calendar(Gregorian);
    Some((
        gregorian_date.extended_year(),
        gregorian_date.month().month_number(),
        gregorian_date.day_of_month().0,
    ))
}
#[frb(sync)]
pub fn gregorian_date_to_jewish_date(year: i32, month: u8, day: u8) -> Option<(i32, u8, u8)> {
    let date = Date::<Gregorian>::try_new_gregorian(year, month, day).unwrap();
    let hebrew_date = date.to_calendar(Hebrew);
    Some((
        hebrew_date.extended_year(),
        hebrew_date.hebrew_month().into(),
        hebrew_date.day_of_month().0,
    ))
}
#[frb(sync)]
pub fn add_days_to_jewish_date(
    year: i32,
    month: u8,
    day: u8,
    days_to_add: i64,
) -> Option<(i32, u8, u8)> {
    let mut date = create_clamped_hebrew_date(year, month, day)?;
    let mut options = DateAddOptions::default();
    options.overflow = Some(Overflow::Constrain);
    date.try_add_with_options(DateDuration::for_days(days_to_add), options)
        .unwrap();
    Some((
        date.extended_year(),
        date.hebrew_month().into(),
        date.day_of_month().0,
    ))
}
#[frb(sync)]
pub fn add_months_to_jewish_date(
    year: i32,
    month: u8,
    day: u8,
    months_to_add: i32,
) -> Option<(i32, u8, u8)> {
    let mut date = create_clamped_hebrew_date(year, month, day)?;
    let mut options = DateAddOptions::default();
    options.overflow = Some(Overflow::Constrain);
    date.try_add_with_options(DateDuration::for_months(months_to_add), options)
        .unwrap();
    Some((
        date.extended_year(),
        date.hebrew_month().into(),
        date.day_of_month().0,
    ))
}
#[frb(sync)]
pub fn add_years_to_jewish_date(
    year: i32,
    month: u8,
    day: u8,
    years_to_add: i32,
) -> Option<(i32, u8, u8)> {
    let mut date = create_clamped_hebrew_date(year, month, day)?;
    let mut options = DateAddOptions::default();
    options.overflow = Some(Overflow::Constrain);
    date.try_add_with_options(DateDuration::for_years(years_to_add), options)
        .unwrap();

    Some((
        date.extended_year(),
        date.hebrew_month().into(),
        date.day_of_month().0,
    ))
}

#[frb(sync)]
pub fn test_jewish_calendar(
    year: i32,
    month: u8,
    day: u8,
    in_israel: bool,
    use_modern_holidays: bool,
    java: JavaJewishCalendarTestResults,
) {
    let hebrew_date = create_clamped_hebrew_date(year, month, day).unwrap();

    assert_eq!(
        hebrew_date.is_assur_bemelacha(in_israel),
        java.isAssurBemelacha
    );
    assert_eq!(
        hebrew_date.has_candle_lighting(in_israel),
        java.hasCandleLighting
    );
    assert_eq!(
        hebrew_date.is_aseres_yemei_teshuva(),
        java.isAseresYemeiTeshuva
    );
    assert_eq!(
        hebrew_date
            .todays_parsha(in_israel)
            .map(|parsha| parsha as i32),
        java.getParshah
    );
    assert_eq!(
        hebrew_date
            .special_parsha(in_israel)
            .map(|parsha| parsha as i32),
        java.getSpecialShabbos
    );
    assert_eq!(
        hebrew_date.upcoming_parsha(in_israel) as i32,
        java.getUpcomingParshah
    );
    assert_eq!(hebrew_date.day_of_chanukah(), java.getDayOfChanukah);
    assert_eq!(hebrew_date.day_of_the_omer(), java.getDayOfOmer);

    let rust_holidays = hebrew_date
        .holidays(in_israel, use_modern_holidays)
        .copied()
        .collect::<Vec<Holiday>>();

    let java_yom_tov_index = java_holiday_index_to_rust(java.getYomTovIndex);
    if let Some(expected_holiday) = java_yom_tov_index {
        assert!(rust_holidays.contains(&expected_holiday));
    }
    assert_eq!(
        rust_holidays.contains(&Holiday::FastOfTheFirstborn),
        java.isTaanisBechoros
    );
    assert_eq!(
        rust_holidays.contains(&Holiday::BirchasHachamah),
        java.isBirkasHachamah
    );
    assert_eq!(
        rust_holidays.contains(&Holiday::MacharHachodesh),
        java.isMacharChodesh
    );
    assert_eq!(
        rust_holidays.contains(&Holiday::ShabbosMevarchim),
        java.isShabbosMevorchim
    );
    assert_eq!(
        rust_holidays.contains(&Holiday::RoshChodesh),
        java.isRoshChodesh
    );
    assert_eq!(
        rust_holidays.contains(&Holiday::YomKippurKatan),
        java.isYomKippurKatan
    );
    assert_eq!(rust_holidays.contains(&Holiday::Behab), java.isBeHaB);
}
#[allow(non_snake_case)]
pub struct JavaJewishCalendarTestResults {
    pub isBirkasHachamah: bool,
    pub getParshah: Option<i32>,
    pub getUpcomingParshah: i32,
    pub getSpecialShabbos: Option<i32>,
    pub getYomTovIndex: i32,
    pub isAssurBemelacha: bool,
    pub hasCandleLighting: bool,
    pub isAseresYemeiTeshuva: bool,
    pub isYomKippurKatan: bool,
    pub isBeHaB: bool,
    pub isTaanisBechoros: bool,
    pub getDayOfChanukah: Option<u8>,
    pub isRoshChodesh: bool,
    pub isMacharChodesh: bool,
    pub isShabbosMevorchim: bool,
    pub getDayOfOmer: Option<u8>,
    // pub getTekufasTishreiElapsedDays: i32, TODO
}
impl JavaJewishCalendarTestResults {
    #[allow(non_snake_case)]
    #[frb(sync)]
    pub fn new(
        isBirkasHachamah: bool,
        getParshah: Option<i32>,
        getUpcomingParshah: i32,
        getSpecialShabbos: Option<i32>,
        getYomTovIndex: i32,
        isAssurBemelacha: bool,
        hasCandleLighting: bool,
        isAseresYemeiTeshuva: bool,
        isYomKippurKatan: bool,
        isBeHaB: bool,
        isTaanisBechoros: bool,
        getDayOfChanukah: Option<u8>,
        isRoshChodesh: bool,
        isMacharChodesh: bool,
        isShabbosMevorchim: bool,
        getDayOfOmer: Option<u8>,
    ) -> Self {
        Self {
            isBirkasHachamah,
            getParshah,
            getUpcomingParshah,
            getSpecialShabbos,
            getYomTovIndex,
            isAssurBemelacha,
            hasCandleLighting,
            isAseresYemeiTeshuva,
            isYomKippurKatan,
            isBeHaB,
            isRoshChodesh,
            isMacharChodesh,
            isShabbosMevorchim,
            getDayOfOmer,
            isTaanisBechoros,
            getDayOfChanukah,
            // getTekufasTishreiElapsedDays,
        }
    }
}
fn java_holiday_index_to_rust(index: i32) -> Option<Holiday> {
    match index {
        0 => Some(Holiday::ErevPesach),
        1 => Some(Holiday::Pesach),
        2 => Some(Holiday::CholHamoedPesach),
        3 => Some(Holiday::PesachSheni),
        4 => Some(Holiday::ErevShavuos),
        5 => Some(Holiday::Shavuos),
        6 => Some(Holiday::SeventeenthOfTammuz),
        7 => Some(Holiday::TishahBav),
        8 => Some(Holiday::TuBav),
        9 => Some(Holiday::ErevRoshHashana),
        10 => Some(Holiday::RoshHashana),
        11 => Some(Holiday::FastOfGedalyah),
        12 => Some(Holiday::ErevYomKippur),
        13 => Some(Holiday::YomKippur),
        14 => Some(Holiday::ErevSuccos),
        15 => Some(Holiday::Succos),
        16 => Some(Holiday::CholHamoedSuccos),
        17 => Some(Holiday::HoshanaRabbah),
        18 => Some(Holiday::SheminiAtzeres),
        19 => Some(Holiday::SimchasTorah),
        // 20 => EREV_CHANUKAH (commented out in Java source)
        21 => Some(Holiday::Chanukah),
        22 => Some(Holiday::TenthOfTeves),
        23 => Some(Holiday::TuBshvat),
        24 => Some(Holiday::FastOfEsther),
        25 => Some(Holiday::Purim),
        26 => Some(Holiday::ShushanPurim),
        27 => Some(Holiday::PurimKatan),
        28 => Some(Holiday::RoshChodesh),
        29 => Some(Holiday::YomHaShoah),
        30 => Some(Holiday::YomHazikaron),
        31 => Some(Holiday::YomHaatzmaut),
        32 => Some(Holiday::YomYerushalayim),
        33 => Some(Holiday::LagBomer),
        34 => Some(Holiday::ShushanPurimKatan),
        35 => Some(Holiday::IsruChag),
        36 => Some(Holiday::YomKippurKatan),
        37 => Some(Holiday::Behab),
        _ => None, // Unknown or not mapped
    }
}
