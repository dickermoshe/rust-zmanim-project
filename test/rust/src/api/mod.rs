use std::str::FromStr;

use chrono::{Duration, NaiveDate};
use chrono_tz::{Tz, TZ_VARIANTS};
use flutter_rust_bridge::frb;
use hebrew_holiday_calendar::{HebrewHolidayCalendar, HebrewMonth, Holiday, Parsha};
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
    is_mukaf_choma: bool,
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
}

#[allow(non_snake_case)]
pub struct JavaJewishCalendarTestResults {
    pub isUseModernHolidays: bool,
    pub getInIsrael: bool,
    pub getIsMukafChoma: bool,
    pub isBirkasHachamah: bool,
    pub getParshah: Option<i32>,
    pub getUpcomingParshah: i32,
    pub getSpecialShabbos: Option<i32>,
    pub getYomTovIndex: i32,
    pub isYomTov: bool,
    pub isYomTovAssurBemelacha: bool,
    pub isAssurBemelacha: bool,
    pub hasCandleLighting: bool,
    pub isTomorrowShabbosOrYomTov: bool,
    pub isErevYomTovSheni: bool,
    pub isAseresYemeiTeshuva: bool,
    pub isPesach: bool,
    pub isCholHamoedPesach: bool,
    pub isShavuos: bool,
    pub isRoshHashana: bool,
    pub isYomKippur: bool,
    pub isSuccos: bool,
    pub isHoshanaRabba: bool,
    pub isShminiAtzeres: bool,
    pub isSimchasTorah: bool,
    pub isCholHamoedSuccos: bool,
    pub isCholHamoed: bool,
    pub isErevYomTov: bool,
    pub isErevRoshChodesh: bool,
    pub isYomKippurKatan: bool,
    pub isBeHaB: bool,
    pub isTaanis: bool,
    pub isTaanisBechoros: bool,
    pub getDayOfChanukah: Option<u8>,
    pub isChanukah: bool,
    pub isPurim: bool,
    pub isRoshChodesh: bool,
    pub isMacharChodesh: bool,
    pub isShabbosMevorchim: bool,
    pub getDayOfOmer: Option<u8>,
    pub isTishaBav: bool,
    pub getMoladAsInstant: i64,
    pub getTchilasZmanKidushLevana3Days: i64,
    pub getTchilasZmanKidushLevana7Days: i64,
    pub getSofZmanKidushLevanaBetweenMoldos: i64,
    pub getSofZmanKidushLevana15Days: i64,
    pub getTekufasTishreiElapsedDays: i32,
    pub isIsruChag: bool,
}
impl JavaJewishCalendarTestResults {
    #[allow(non_snake_case)]
    #[frb(sync)]
    pub fn new(
        isUseModernHolidays: bool,
        getInIsrael: bool,
        getIsMukafChoma: bool,
        isBirkasHachamah: bool,
        getParshah: Option<i32>,
        getUpcomingParshah: i32,
        getSpecialShabbos: Option<i32>,
        getYomTovIndex: i32,
        isYomTov: bool,
        isYomTovAssurBemelacha: bool,
        isAssurBemelacha: bool,
        hasCandleLighting: bool,
        isTomorrowShabbosOrYomTov: bool,
        isErevYomTovSheni: bool,
        isAseresYemeiTeshuva: bool,
        isPesach: bool,
        isCholHamoedPesach: bool,
        isShavuos: bool,
        isRoshHashana: bool,
        isYomKippur: bool,
        isSuccos: bool,
        isHoshanaRabba: bool,
        isShminiAtzeres: bool,
        isSimchasTorah: bool,
        isCholHamoedSuccos: bool,
        isCholHamoed: bool,
        isErevYomTov: bool,
        isErevRoshChodesh: bool,
        isYomKippurKatan: bool,
        isBeHaB: bool,
        isTaanis: bool,
        isTaanisBechoros: bool,
        getDayOfChanukah: Option<u8>,
        isChanukah: bool,
        isPurim: bool,
        isRoshChodesh: bool,
        isMacharChodesh: bool,
        isShabbosMevorchim: bool,
        getDayOfOmer: Option<u8>,
        isTishaBav: bool,
        getMoladAsInstant: i64,
        getTchilasZmanKidushLevana3Days: i64,
        getTchilasZmanKidushLevana7Days: i64,
        getSofZmanKidushLevanaBetweenMoldos: i64,
        getSofZmanKidushLevana15Days: i64,
        getTekufasTishreiElapsedDays: i32,
        isIsruChag: bool,
    ) -> Self {
        Self {
            isUseModernHolidays,
            getInIsrael,
            getIsMukafChoma,
            isBirkasHachamah,
            getParshah,
            getUpcomingParshah,
            getSpecialShabbos,
            getYomTovIndex,
            isYomTov,
            isYomTovAssurBemelacha,
            isAssurBemelacha,
            hasCandleLighting,
            isTomorrowShabbosOrYomTov,
            isErevYomTovSheni,
            isAseresYemeiTeshuva,
            isPesach,
            isCholHamoedPesach,
            isShavuos,
            isRoshHashana,
            isYomKippur,

            isSuccos,
            isHoshanaRabba,
            isShminiAtzeres,
            isSimchasTorah,
            isCholHamoedSuccos,
            isCholHamoed,
            isErevYomTov,
            isErevRoshChodesh,
            isYomKippurKatan,
            isBeHaB,
            isTaanis,
            isTaanisBechoros,
            getDayOfChanukah,
            isChanukah,
            isPurim,
            isRoshChodesh,
            isMacharChodesh,
            isShabbosMevorchim,
            getDayOfOmer,
            isTishaBav,
            getMoladAsInstant,
            getTchilasZmanKidushLevana3Days,
            getTchilasZmanKidushLevana7Days,
            getSofZmanKidushLevanaBetweenMoldos,
            getSofZmanKidushLevana15Days,
            getTekufasTishreiElapsedDays,
            isIsruChag,
        }
    }
}
