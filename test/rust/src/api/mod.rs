use std::str::FromStr;

use chrono::{Duration, NaiveDate};
use chrono_tz::{Tz, TZ_VARIANTS};
use flutter_rust_bridge::frb;
use hebrew_holiday_calendar::{HebrewHolidayCalendar, HebrewMonth};
use icu_calendar::{
    cal::Hebrew,
    options::{DateAddOptions, Overflow},
    types::{DateDuration, MonthCode},
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
