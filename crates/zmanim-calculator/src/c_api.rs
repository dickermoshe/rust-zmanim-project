use crate::prelude::{
    CalculatorConfig as RustCalculatorConfig, Location as RustLocation,
    ZmanimCalculator as RustZmanimCalculator,
};
use chrono::{Duration, FixedOffset, NaiveDate};
use interoptopus::patterns::option::FFIOption;
use interoptopus::{ffi_function, ffi_type, function, Inventory, InventoryBuilder};
extern crate alloc;

#[derive(Default)]
#[ffi_type]
#[repr(C)]
/// A geographic location (latitude/longitude/elevation) and an optional timezone.
pub struct Location {
    /// Latitude in degrees. Valid range: `[-90.0, 90.0]` (positive = North).
    pub latitude: f64,
    /// Longitude in degrees. Valid range: `[-180.0, 180.0]` (positive = East).
    pub longitude: f64,
    /// Elevation above sea level in meters. Must be `>= 0.0`.
    pub elevation: f64,
    /// Timezone offset of the location in seconds on the date in question. Required when `abs(longitude) > 150.0`
    pub timezone_offset: FFIOption<i32>,
    _private: [u8; 0],
}

impl Location {
    fn timezone(&self) -> Option<FixedOffset> {
        if self.timezone_offset.is_none() {
            None
        } else {
            let offset = self.timezone_offset.unwrap();
            match FixedOffset::east_opt(offset) {
                Some(tz) => Some(tz),
                None => None,
            }
        }
    }
    fn into_rust_location(&self) -> Option<RustLocation<FixedOffset>> {
        RustLocation::new(
            self.latitude,
            self.longitude,
            self.elevation,
            self.timezone(),
        )
        .ok()
    }
}

/// Creates a geographic location for zmanim calculations.
///
/// This mirrors [`crate::types::location::Location::new`]:
/// - latitude must be within `[-90.0, 90.0]`
/// - longitude must be within `[-180.0, 180.0]`
/// - elevation must be `>= 0.0`
/// - timezone is required when `abs(longitude) > 150.0`
///
/// `timezone_offset` is the UTC offset in seconds. Pass `FFIOption::none()` when unknown.
///
/// Returns `FFIOption::none()` if the timezone offset is invalid or if any input fails validation.
#[no_mangle]
#[ffi_function]
pub extern "C" fn new_location<'a>(
    latitude: f64,
    longitude: f64,
    elevation: f64,
    timezone_offset: FFIOption<i32>,
) -> FFIOption<Location> {
    let tz = if timezone_offset.is_none() {
        None
    } else {
        let offset = timezone_offset.unwrap();
        match FixedOffset::east_opt(offset) {
            Some(tz) => Some(tz),
            None => return FFIOption::<Location>::none(),
        }
    };
    let location = RustLocation::new(latitude, longitude, elevation, tz);
    if let Ok(location) = location {
        FFIOption::some(Location {
            latitude: location.latitude,
            longitude: location.longitude,
            elevation: location.elevation,
            timezone_offset: timezone_offset,
            _private: [0; 0],
        })
    } else {
        FFIOption::<Location>::none()
    }
}

#[ffi_type]
#[repr(C)]
/// Parameters that control how zmanim are calculated.
pub struct CalculatorConfig {
    /// Offset subtracted from sea-level sunset to produce [`crate::CANDLE_LIGHTING`]. Default: 18 min.
    pub candle_lighting_offset_seconds: i32,
    /// When `true`, derived zmanim (sof zman shma, mincha gedola, etc.) are computed
    /// relative to solar noon rather than as a fraction of the sunrise–sunset interval. Default: `false`.
    pub use_astronomical_chatzos_for_other_zmanim: bool,
    /// Offset added to elevation-adjusted sunset for the Ateret Torah opinion (see [`crate::TZAIS_ATERET_TORAH`]). Default: 40 min.
    pub ateret_torah_sunset_offset_seconds: i32,
    _private: [u8; 0],
}

impl Default for CalculatorConfig {
    fn default() -> Self {
        Self {
            candle_lighting_offset_seconds: 18 * 60,
            use_astronomical_chatzos_for_other_zmanim: false,
            ateret_torah_sunset_offset_seconds: 40 * 60,
            _private: [0; 0],
        }
    }
}
impl CalculatorConfig {
    fn into_rust_config(&self) -> RustCalculatorConfig {
        RustCalculatorConfig {
            candle_lighting_offset: Duration::seconds(self.candle_lighting_offset_seconds as i64),
            use_astronomical_chatzos_for_other_zmanim: self
                .use_astronomical_chatzos_for_other_zmanim,
            ateret_torah_sunset_offset: Duration::seconds(
                self.ateret_torah_sunset_offset_seconds as i64,
            ),
        }
    }
}

#[ffi_type]
#[derive(Default)]
#[repr(C)]
/// A calculator instance for one location/date/config combination.
pub struct ZmanimCalculator {
    /// The location to calculate for.
    pub location: Location,
    /// The year of the date to calculate for.
    pub year: i32,
    /// The month of the date to calculate for.
    pub month: u32,
    /// The day of the date to calculate for.
    pub day: u32,
    /// The configuration to use for the calculator.
    pub config: CalculatorConfig,
    _private: [u8; 0],
}

impl ZmanimCalculator {
    fn into_rust_calculator(&self) -> Option<RustZmanimCalculator<FixedOffset>> {
        let date = NaiveDate::from_ymd_opt(self.year, self.month, self.day)?;
        let location = self.location.into_rust_location()?;
        let config = self.config.into_rust_config();
        RustZmanimCalculator::new(location, date, config).ok()
    }
}
/// Creates a calculator instance for one location/date/config combination.
///
/// This mirrors [`crate::calculator::ZmanimCalculator::new`] and validates:
/// - `year` / `month` / `day` form a valid Gregorian date
/// - `location` and `config` are valid according to their constructors
///
/// Returns `FFIOption::none()` if any validation fails.
#[no_mangle]
#[ffi_function]
pub extern "C" fn new_calculator(
    location: Location,
    year: i32,
    month: u32,
    day: u32,
    config: CalculatorConfig,
) -> FFIOption<ZmanimCalculator> {
    let calculator = ZmanimCalculator {
        location: location,
        year: year,
        month: month,
        day: day,
        config: config,
        _private: [0; 0],
    };
    let rust_calculator = calculator.into_rust_calculator();
    match rust_calculator {
        Some(_) => FFIOption::some(calculator),
        None => FFIOption::<ZmanimCalculator>::none(),
    }
}

// Generated from C:/Users/Moshe/DickerSystems/zmanim-group/zmanim-calendar/src/presets.rs (161 presets)
fn calculate_preset_timestamp<T>(calculator: &ZmanimCalculator, preset: T) -> FFIOption<i64>
where
    T: crate::calculator::ZmanLike<FixedOffset>,
{
    let Some(mut rust_calculator) = calculator.into_rust_calculator() else {
        return FFIOption::none();
    };

    rust_calculator
        .calculate(preset)
        .map(|datetime| datetime.timestamp())
        .ok()
        .into()
}

#[doc = "Sunset (elevation-adjusted)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sunrise(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SUNRISE)
}

#[doc = "Sunrise at sea level (elevation `0m`)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sea_level_sunrise(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SEA_LEVEL_SUNRISE)
}

#[doc = "Sunset (elevation-adjusted)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SUNSET)
}

#[doc = "Sunset at sea level (elevation `0m`)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sea_level_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SEA_LEVEL_SUNSET)
}

#[doc = "*Alos* as a fixed `60` minutes before sunrise."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_60_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_60_MINUTES)
}

#[doc = "*Alos* as a fixed `72` minutes before sunrise."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_72_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_72_MINUTES)
}

#[doc = "*Alos* as `72 zmaniyos` minutes before sunrise (1.2 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_72_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_72_ZMANIS)
}

#[doc = "*Alos* as a fixed `90` minutes before sunrise."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_90_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_90_MINUTES)
}

#[doc = "*Alos* as `90 zmaniyos` minutes before sunrise (1.5 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_90_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_90_ZMANIS)
}

#[doc = "*Alos* as a fixed `96` minutes before sunrise."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_96_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_96_MINUTES)
}

#[doc = "*Alos* as `96 zmaniyos` minutes before sunrise (1.6 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_96_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_96_ZMANIS)
}

#[doc = "*Alos* as a fixed `120` minutes before sunrise."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_120_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_120_MINUTES)
}

#[doc = "*Alos* as `120 zmaniyos` minutes before sunrise (2.0 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_120_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_120_ZMANIS)
}

#[doc = "*Alos* when the sun is `16.1°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_16_POINT_1_DEGREES)
}

#[doc = "*Alos* when the sun is `18°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_18_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_18_DEGREES)
}

#[doc = "*Alos* when the sun is `19°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_19_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_19_DEGREES)
}

#[doc = "*Alos* when the sun is `19.8°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_19_point_8_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_19_POINT_8_DEGREES)
}

#[doc = "*Alos* when the sun is `26°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_26_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_26_DEGREES)
}

#[doc = "*Alos* when the sun is `16.9°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn alos_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::ALOS_BAAL_HATANYA)
}

#[doc = "Calculates preset `BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES`."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn bain_hashmashos_rt_13_point_24_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES)
}

#[doc = "Bain hashmashos (Rabbeinu Tam): `58.5` minutes after sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn bain_hashmashos_rt_58_point_5_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES)
}

#[doc = "Bain hashmashos (Rabbeinu Tam): `13.5` minutes before when the sun will be `7.083°` below the geometric horizon."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn bain_hashmashos_rt_13_point_5_minutes_before_7_point_083_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES)
}

#[doc = "Bain hashmashos (Rabbeinu Tam, 2-stars): `sunset + (sunrise - alos19.8°) * 5/18`."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn bain_hashmashos_rt_2_stars(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::BAIN_HASHMASHOS_RT_2_STARS)
}

#[doc = "Bain hashmashos (Yereim): `18` minutes before sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn bain_hashmashos_yereim_18_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::BAIN_HASHMASHOS_YEREIM_18_MINUTES)
}

#[doc = "Bain hashmashos (Yereim): `16.875` minutes before sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn bain_hashmashos_yereim_16_point_875_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES)
}

#[doc = "Bain hashmashos (Yereim): `13.5` minutes before sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn bain_hashmashos_yereim_13_point_5_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::BAIN_HASHMASHOS_YEREIM_13_POINT_5_MINUTES)
}

#[doc = "Candle lighting: sea-level sunset minus [`crate::CalculatorConfig::candle_lighting_offset`]."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn candle_lighting(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::CANDLE_LIGHTING)
}

#[doc = "Chatzos (astronomical noon): solar transit."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn chatzos_astronomical(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::CHATZOS_ASTRONOMICAL)
}

#[doc = "Chatzos (half-day): midpoint between sea-level sunrise and sea-level sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn chatzos_half_day(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::CHATZOS_HALF_DAY)
}

#[doc = "Chatzos (fixed local): 12:00 local mean time."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn chatzos_fixed_local(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::CHATZOS_FIXED_LOCAL)
}

#[doc = "Mincha gedola: `6.5` shaos after sunrise (or `0.5` shaah after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_sunrise_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_SUNRISE_SUNSET)
}

#[doc = "Mincha gedola: `6.5` shaos after alos `16.1°` (or `0.5` shaah after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_16_POINT_1_DEGREES)
}

#[doc = "Mincha gedola: `30` minutes after solar transit."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_minutes_30(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_MINUTES_30)
}

#[doc = "Mincha gedola: `6.5` shaos after alos `72` minutes (or `0.5` shaah after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_minutes_72(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_MINUTES_72)
}

#[doc = "Mincha gedola (Ahavat Shalom): later of `chatzos + 30m` and `chatzos + 1/2 shaah`."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_ahavat_shalom(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_AHAVAT_SHALOM)
}

#[doc = "Mincha gedola: `6.5` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_ateret_torah(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_ATERET_TORAH)
}

#[doc = "Mincha gedola: `6.5` shaos after Baal HaTanya day start (or `0.5` shaah after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_BAAL_HATANYA)
}

#[doc = "Mincha gedola: later of Baal HaTanya mincha gedola and `30` minutes after solar transit."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_baal_hatanya_greater_than_30(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_BAAL_HATANYA_GREATER_THAN_30)
}

#[doc = "Mincha gedola: `30` minutes after fixed local chatzos (12:00 local mean time)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_gra_fixed_local_chatzos_30_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_GRA_FIXED_LOCAL_CHATZOS_30_MINUTES)
}

#[doc = "Mincha gedola: later of [`MINCHA_GEDOLA_SUNRISE_SUNSET`] and [`MINCHA_GEDOLA_MINUTES_30`]."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_gedola_greater_than_30(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_GEDOLA_GREATER_THAN_30)
}

#[doc = "Mincha ketana: `9.5` shaos after sunrise (or `3.5` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_ketana_sunrise_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_KETANA_SUNRISE_SUNSET)
}

#[doc = "Mincha ketana: `9.5` shaos after alos `16.1°` (or `3.5` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_ketana_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_KETANA_16_POINT_1_DEGREES)
}

#[doc = "Mincha ketana: `9.5` shaos after alos `72` minutes (or `3.5` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_ketana_minutes_72(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_KETANA_MINUTES_72)
}

#[doc = "Mincha ketana (Ahavat Shalom): `2.5` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_ketana_ahavat_shalom(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_KETANA_AHAVAT_SHALOM)
}

#[doc = "Mincha ketana: `9.5` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_ketana_ateret_torah(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_KETANA_ATERET_TORAH)
}

#[doc = "Mincha ketana: `9.5` shaos after Baal HaTanya day start (or `3.5` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_ketana_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_KETANA_BAAL_HATANYA)
}

#[doc = "Mincha ketana: `3.5` shaos zmaniyos after fixed local chatzos, using fixed-local-chatzos → sunset half-day."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn mincha_ketana_gra_fixed_local_chatzos_to_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MINCHA_KETANA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET)
}

#[doc = "Misheyakir when the sun is `10.2°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn misheyakir_10_point_2_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MISHEYAKIR_10_POINT_2_DEGREES)
}

#[doc = "Misheyakir when the sun is `11°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn misheyakir_11_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MISHEYAKIR_11_DEGREES)
}

#[doc = "Misheyakir when the sun is `11.5°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn misheyakir_11_point_5_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MISHEYAKIR_11_POINT_5_DEGREES)
}

#[doc = "Misheyakir when the sun is `7.65°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn misheyakir_7_point_65_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MISHEYAKIR_7_POINT_65_DEGREES)
}

#[doc = "Misheyakir when the sun is `9.5°` below the geometric horizon (degrees-below-horizon dawn)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn misheyakir_9_point_5_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MISHEYAKIR_9_POINT_5_DEGREES)
}

#[doc = "Plag hamincha: `10.75` shaos after sunrise (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_sunrise_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_SUNRISE_SUNSET)
}

#[doc = "Plag hamincha (Ahavat Shalom): `1.25` shaos zmaniyos before tzais `3.8°` (day = alos16.1° → tzais3.8°)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_ahavat_shalom(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_AHAVAT_SHALOM)
}

#[doc = "Plag hamincha: `10.75` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais7.083°)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_16_point_1_to_tzais_geonim_7_point_083(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083)
}

#[doc = "Plag hamincha: `10.75` shaos zmaniyos after alos `16.1°` (day end = sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_alos_to_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_ALOS_TO_SUNSET)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `60` minutes (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_60_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_60_MINUTES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `72` minutes (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_72_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_72_MINUTES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `72 zmaniyos` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_72_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_72_ZMANIS)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `90` minutes (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_90_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_90_MINUTES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `90 zmaniyos` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_90_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_90_ZMANIS)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `96` minutes (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_96_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_96_MINUTES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `96 zmaniyos` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_96_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_96_ZMANIS)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `120` minutes (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_120_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_120_MINUTES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `120 zmaniyos` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_120_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_120_ZMANIS)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `16.1°` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_16_POINT_1_DEGREES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `18°` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_18_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_18_DEGREES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `19.8°` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_19_point_8_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_19_POINT_8_DEGREES)
}

#[doc = "Plag hamincha: `10.75` shaos after alos `26°` (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_26_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_26_DEGREES)
}

#[doc = "Plag hamincha: `10.75` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_ateret_torah(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_ATERET_TORAH)
}

#[doc = "Plag hamincha: `10.75` shaos after Baal HaTanya day start (or `4.75` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_BAAL_HATANYA)
}

#[doc = "Plag hamincha: `4.75` shaos zmaniyos after fixed local chatzos, using fixed-local-chatzos → sunset half-day."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn plag_hamincha_gra_fixed_local_chatzos_to_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::PLAG_HAMINCHA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET)
}

#[doc = "Samuch le-mincha ketana: `9` shaos after sunrise (or `3` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn samuch_le_mincha_ketana_gra(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SAMUCH_LE_MINCHA_KETANA_GRA)
}

#[doc = "Samuch le-mincha ketana: `9` shaos after alos `16.1°` (or `3` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn samuch_le_mincha_ketana_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SAMUCH_LE_MINCHA_KETANA_16_POINT_1_DEGREES)
}

#[doc = "Samuch le-mincha ketana: `9` shaos after alos `72` minutes (or `3` shaos after chatzos if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn samuch_le_mincha_ketana_72_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SAMUCH_LE_MINCHA_KETANA_72_MINUTES)
}

#[doc = "Sof zman achilas chametz: `4` shaos after sunrise (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_achilas_chametz_gra(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_ACHILAS_CHAMETZ_GRA)
}

#[doc = "Sof zman achilas chametz: `4` shaos after alos `72` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_achilas_chametz_mga_72_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_ACHILAS_CHAMETZ_MGA_72_MINUTES)
}

#[doc = "Sof zman achilas chametz: `4` shaos after alos `16.1°` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_achilas_chametz_mga_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_ACHILAS_CHAMETZ_MGA_16_POINT_1_DEGREES)
}

#[doc = "Sof zman achilas chametz: `4` shaos after Baal HaTanya day start (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_achilas_chametz_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_ACHILAS_CHAMETZ_BAAL_HATANYA)
}

#[doc = "Sof zman biur chametz: `5` shaos zmaniyos after sunrise (day = sunrise → sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_biur_chametz_gra(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_BIUR_CHAMETZ_GRA)
}

#[doc = "Sof zman biur chametz: `5` shaos zmaniyos after alos `72` minutes (day = alos72 → tzais72)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_biur_chametz_mga_72_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_BIUR_CHAMETZ_MGA_72_MINUTES)
}

#[doc = "Sof zman biur chametz: `5` shaos zmaniyos after alos `16.1°` (day = alos16.1° → tzais16.1°)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_biur_chametz_mga_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_BIUR_CHAMETZ_MGA_16_POINT_1_DEGREES)
}

#[doc = "Sof zman biur chametz: `5` shaos zmaniyos after Baal HaTanya day start (day = Baal HaTanya sunrise → sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_biur_chametz_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_BIUR_CHAMETZ_BAAL_HATANYA)
}

#[doc = "Sof zman shma: `3` shaos after sunrise (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_gra(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_GRA)
}

#[doc = "Sof zman shma: `3` shaos after alos `72` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA)
}

#[doc = "Sof zman shma: `3` shaos after alos `19.8°` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_19_point_8_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_19_POINT_8_DEGREES)
}

#[doc = "Sof zman shma: `3` shaos after alos `16.1°` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES)
}

#[doc = "Sof zman shma: `3` shaos after alos `18°` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_18_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_18_DEGREES)
}

#[doc = "Sof zman shma: `3` shaos after alos `72` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_72_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_72_MINUTES)
}

#[doc = "Sof zman shma: `3` shaos after alos `72 zmaniyos` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_72_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_72_ZMANIS)
}

#[doc = "Sof zman shma: `3` shaos after alos `90` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_90_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_90_MINUTES)
}

#[doc = "Sof zman shma: `3` shaos after alos `90 zmaniyos` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_90_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_90_ZMANIS)
}

#[doc = "Sof zman shma: `3` shaos after alos `96` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_96_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_96_MINUTES)
}

#[doc = "Sof zman shma: `3` shaos after alos `96 zmaniyos` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_96_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_96_ZMANIS)
}

#[doc = "Sof zman shma: `3` hours before solar transit."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_hours_3_before_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_HOURS_3_BEFORE_CHATZOS)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `120` minutes (day = alos120 → tzais120)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_120_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_120_MINUTES)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_alos_16_point_1_to_sunset(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_SUNSET)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = tzais7.083°)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_alos_16_point_1_to_tzais_geonim_7_point_083(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after sunrise (day end = fixed local chatzos)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_kol_eliyahu(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_KOL_ELIYAHU)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_ateret_torah(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_ATERET_TORAH)
}

#[doc = "Sof zman shma: `3` shaos after Baal HaTanya day start (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_BAAL_HATANYA)
}

#[doc = "Sof zman shma: `3` hours before fixed local chatzos (12:00 local mean time)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_fixed_local(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_FIXED_LOCAL)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after sunrise (day end = fixed local chatzos)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_gra_sunrise_to_fixed_local_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `18°` (day end = fixed local chatzos)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_18_degrees_to_fixed_local_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_18_DEGREES_TO_FIXED_LOCAL_CHATZOS)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `16.1°` (day end = fixed local chatzos)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_16_point_1_degrees_to_fixed_local_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES_TO_FIXED_LOCAL_CHATZOS)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `90` minutes (day end = fixed local chatzos)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_90_minutes_to_fixed_local_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_90_MINUTES_TO_FIXED_LOCAL_CHATZOS)
}

#[doc = "Sof zman shma: `3` shaos zmaniyos after alos `72` minutes (day end = fixed local chatzos)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_shma_mga_72_minutes_to_fixed_local_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_SHMA_MGA_72_MINUTES_TO_FIXED_LOCAL_CHATZOS)
}

#[doc = "Sof zman tfila: `4` shaos after sunrise (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_gra(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_GRA)
}

#[doc = "Sof zman tfila: `4` shaos after alos `72` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA)
}

#[doc = "Sof zman tfila: `4` shaos after alos `19.8°` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_19_point_8_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_19_POINT_8_DEGREES)
}

#[doc = "Sof zman tfila: `4` shaos after alos `16.1°` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_16_POINT_1_DEGREES)
}

#[doc = "Sof zman tfila: `4` shaos after alos `18°` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_18_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_18_DEGREES)
}

#[doc = "Sof zman tfila: `4` shaos after alos `72` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_72_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_72_MINUTES)
}

#[doc = "Sof zman tfila: `4` shaos after alos `72 zmaniyos` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_72_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_72_ZMANIS)
}

#[doc = "Sof zman tfila: `4` shaos after alos `90` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_90_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_90_MINUTES)
}

#[doc = "Sof zman tfila: `4` shaos after alos `90 zmaniyos` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_90_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_90_ZMANIS)
}

#[doc = "Sof zman tfila: `4` shaos after alos `96` minutes (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_96_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_96_MINUTES)
}

#[doc = "Sof zman tfila: `4` shaos after alos `96 zmaniyos` (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_96_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_96_ZMANIS)
}

#[doc = "Sof zman tfila: `2` hours before solar transit."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_hours_2_before_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_HOURS_2_BEFORE_CHATZOS)
}

#[doc = "Sof zman tfila: `4` shaos zmaniyos after alos `120` minutes (day = alos120 → tzais120)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_mga_120_minutes(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_MGA_120_MINUTES)
}

#[doc = "Sof zman tfila: `4` shaos zmaniyos after alos `72 zmaniyos` (day end = Ateret Torah tzais)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_ateret_torah(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_ATERET_TORAH)
}

#[doc = "Sof zman tfila: `4` shaos after Baal HaTanya day start (or half-day based if configured)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_BAAL_HATANYA)
}

#[doc = "Sof zman tfila: `2` hours before fixed local chatzos (12:00 local mean time)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_fixed_local(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_FIXED_LOCAL)
}

#[doc = "Sof zman tfila: `4` shaos zmaniyos after sunrise (day end = fixed local chatzos)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_tfila_gra_sunrise_to_fixed_local_chatzos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_TFILA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS)
}

#[doc = "Tzais when the sun is `8.5°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_degrees_8_point_5(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_DEGREES_8_POINT_5)
}

#[doc = "Tzais: `50` minutes after sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_minutes_50(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_MINUTES_50)
}

#[doc = "Tzais: `60` minutes after sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_minutes_60(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_MINUTES_60)
}

#[doc = "Tzais: `72` minutes after sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_minutes_72(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_MINUTES_72)
}

#[doc = "Tzais: `72 zmaniyos` minutes after sunset (1.2 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_72_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_72_ZMANIS)
}

#[doc = "Tzais: `90` minutes after sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_minutes_90(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_MINUTES_90)
}

#[doc = "Tzais: `90 zmaniyos` minutes after sunset (1.5 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_90_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_90_ZMANIS)
}

#[doc = "Tzais: `96` minutes after sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_minutes_96(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_MINUTES_96)
}

#[doc = "Tzais: `96 zmaniyos` minutes after sunset (1.6 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_96_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_96_ZMANIS)
}

#[doc = "Tzais: `120` minutes after sunset."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_minutes_120(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_MINUTES_120)
}

#[doc = "Tzais: `120 zmaniyos` minutes after sunset (2.0 *shaos zmaniyos*)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_120_zmanis(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_120_ZMANIS)
}

#[doc = "Tzais when the sun is `16.1°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_16_point_1_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_16_POINT_1_DEGREES)
}

#[doc = "Tzais when the sun is `18°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_18_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_18_DEGREES)
}

#[doc = "Tzais when the sun is `19.8°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_19_point_8_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_19_POINT_8_DEGREES)
}

#[doc = "Tzais when the sun is `26°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_26_degrees(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_26_DEGREES)
}

#[doc = "Tzais (Ateret Torah): (elevation-adjusted) sunset plus [`crate::CalculatorConfig::ateret_torah_sunset_offset`]."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_ateret_torah(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_ATERET_TORAH)
}

#[doc = "Tzais (Baal HaTanya): when the sun is `6°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_baal_hatanya(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_BAAL_HATANYA)
}

#[doc = "Tzais (Geonim): when the sun is `3.7°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_3_point_7(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_3_POINT_7)
}

#[doc = "Tzais (Geonim): when the sun is `3.8°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_3_point_8(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_3_POINT_8)
}

#[doc = "Tzais (Geonim): when the sun is `5.95°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_5_point_95(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_5_POINT_95)
}

#[doc = "Tzais (Geonim): when the sun is `3.65°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_3_point_65(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_3_POINT_65)
}

#[doc = "Tzais (Geonim): when the sun is `3.676°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_3_point_676(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_3_POINT_676)
}

#[doc = "Tzais (Geonim): when the sun is `4.61°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_4_point_61(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_4_POINT_61)
}

#[doc = "Tzais (Geonim): when the sun is `4.37°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_4_point_37(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_4_POINT_37)
}

#[doc = "Tzais (Geonim): when the sun is `5.88°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_5_point_88(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_5_POINT_88)
}

#[doc = "Tzais (Geonim): when the sun is `4.8°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_4_point_8(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_4_POINT_8)
}

#[doc = "Tzais (Geonim): when the sun is `6.45°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_6_point_45(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_6_POINT_45)
}

#[doc = "Tzais (Geonim): when the sun is `7.083°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_7_point_083(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_7_POINT_083)
}

#[doc = "Tzais (Geonim): when the sun is `7.67°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_7_point_67(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_7_POINT_67)
}

#[doc = "Tzais (Geonim): when the sun is `8.5°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_8_point_5(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_8_POINT_5)
}

#[doc = "Tzais (Geonim): when the sun is `9.3°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_9_point_3(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_9_POINT_3)
}

#[doc = "Tzais (Geonim): when the sun is `9.75°` below the geometric horizon (after sunset)."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tzais_geonim_degrees_9_point_75(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TZAIS_GEONIM_DEGREES_9_POINT_75)
}

#[doc = "Returns the latest time of _Kiddush Levana_ calculated as 15 days after the molad."]
#[doc = ""]
#[doc = "Will return None if the zman will not occur on this day. If the location does not contain"]
#[doc = "a timezone, this will always return None."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_kidush_levana_15_days(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_KIDUSH_LEVANA_15_DAYS)
}

#[doc = "The latest time of _Kiddush Levana_ according to the"]
#[doc = "[Maharil](https://en.wikipedia.org/wiki/Yaakov_ben_Moshe_Levi_Moelin)'s opinion that it"]
#[doc = "is calculated as halfway between molad and molad."]
#[doc = ""]
#[doc = "Will return None if the zman will not occur on this day. If the location does not contain"]
#[doc = "a timezone, this will always return None."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn sof_zman_kidush_levana_between_moldos(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::SOF_ZMAN_KIDUSH_LEVANA_BETWEEN_MOLDOS)
}

#[doc = "The earliest time of _Kiddush Levana_ according to [Rabbeinu Yonah](https://en.wikipedia.org/wiki/Yonah_Gerondi)'s opinion that it can be said 3 days after the molad."]
#[doc = ""]
#[doc = "Will return None if the zman will not occur on this day. If the location does not contain"]
#[doc = "a timezone, this will always return None."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tchilas_zman_kidush_levana_3_days(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TCHILAS_ZMAN_KIDUSH_LEVANA_3_DAYS)
}

#[doc = "The earliest time of _Kiddush Levana_ according to the opinions that it should"]
#[doc = "not be said until 7 days after the molad."]
#[doc = ""]
#[doc = "Will return None if the zman will not occur on this day. If the location does not contain"]
#[doc = "a timezone, this will always return None."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn tchilas_zman_kidush_levana_7_days(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::TCHILAS_ZMAN_KIDUSH_LEVANA_7_DAYS)
}

#[doc = "The time of the molad (new moon) for the current date's Hebrew month."]
#[no_mangle]
#[ffi_function]
pub extern "C" fn molad(calculator: &ZmanimCalculator) -> FFIOption<i64> {
    calculate_preset_timestamp(calculator, crate::presets::MOLAD)
}

/// Returns the Interoptopus inventory used to generate foreign bindings.
pub fn my_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(function!(new_location))
        .register(function!(new_calculator))
        .register(function!(sunrise))
        .register(function!(sea_level_sunrise))
        .register(function!(sunset))
        .register(function!(sea_level_sunset))
        .register(function!(alos_60_minutes))
        .register(function!(alos_72_minutes))
        .register(function!(alos_72_zmanis))
        .register(function!(alos_90_minutes))
        .register(function!(alos_90_zmanis))
        .register(function!(alos_96_minutes))
        .register(function!(alos_96_zmanis))
        .register(function!(alos_120_minutes))
        .register(function!(alos_120_zmanis))
        .register(function!(alos_16_point_1_degrees))
        .register(function!(alos_18_degrees))
        .register(function!(alos_19_degrees))
        .register(function!(alos_19_point_8_degrees))
        .register(function!(alos_26_degrees))
        .register(function!(alos_baal_hatanya))
        .register(function!(bain_hashmashos_rt_13_point_24_degrees))
        .register(function!(bain_hashmashos_rt_58_point_5_minutes))
        .register(function!(bain_hashmashos_rt_13_point_5_minutes_before_7_point_083_degrees))
        .register(function!(bain_hashmashos_rt_2_stars))
        .register(function!(bain_hashmashos_yereim_18_minutes))
        .register(function!(bain_hashmashos_yereim_16_point_875_minutes))
        .register(function!(bain_hashmashos_yereim_13_point_5_minutes))
        .register(function!(candle_lighting))
        .register(function!(chatzos_astronomical))
        .register(function!(chatzos_half_day))
        .register(function!(chatzos_fixed_local))
        .register(function!(mincha_gedola_sunrise_sunset))
        .register(function!(mincha_gedola_16_point_1_degrees))
        .register(function!(mincha_gedola_minutes_30))
        .register(function!(mincha_gedola_minutes_72))
        .register(function!(mincha_gedola_ahavat_shalom))
        .register(function!(mincha_gedola_ateret_torah))
        .register(function!(mincha_gedola_baal_hatanya))
        .register(function!(mincha_gedola_baal_hatanya_greater_than_30))
        .register(function!(mincha_gedola_gra_fixed_local_chatzos_30_minutes))
        .register(function!(mincha_gedola_greater_than_30))
        .register(function!(mincha_ketana_sunrise_sunset))
        .register(function!(mincha_ketana_16_point_1_degrees))
        .register(function!(mincha_ketana_minutes_72))
        .register(function!(mincha_ketana_ahavat_shalom))
        .register(function!(mincha_ketana_ateret_torah))
        .register(function!(mincha_ketana_baal_hatanya))
        .register(function!(mincha_ketana_gra_fixed_local_chatzos_to_sunset))
        .register(function!(misheyakir_10_point_2_degrees))
        .register(function!(misheyakir_11_degrees))
        .register(function!(misheyakir_11_point_5_degrees))
        .register(function!(misheyakir_7_point_65_degrees))
        .register(function!(misheyakir_9_point_5_degrees))
        .register(function!(plag_hamincha_sunrise_sunset))
        .register(function!(plag_hamincha_ahavat_shalom))
        .register(function!(plag_hamincha_16_point_1_to_tzais_geonim_7_point_083))
        .register(function!(plag_hamincha_alos_to_sunset))
        .register(function!(plag_hamincha_60_minutes))
        .register(function!(plag_hamincha_72_minutes))
        .register(function!(plag_hamincha_72_zmanis))
        .register(function!(plag_hamincha_90_minutes))
        .register(function!(plag_hamincha_90_zmanis))
        .register(function!(plag_hamincha_96_minutes))
        .register(function!(plag_hamincha_96_zmanis))
        .register(function!(plag_hamincha_120_minutes))
        .register(function!(plag_hamincha_120_zmanis))
        .register(function!(plag_hamincha_16_point_1_degrees))
        .register(function!(plag_hamincha_18_degrees))
        .register(function!(plag_hamincha_19_point_8_degrees))
        .register(function!(plag_hamincha_26_degrees))
        .register(function!(plag_hamincha_ateret_torah))
        .register(function!(plag_hamincha_baal_hatanya))
        .register(function!(plag_hamincha_gra_fixed_local_chatzos_to_sunset))
        .register(function!(samuch_le_mincha_ketana_gra))
        .register(function!(samuch_le_mincha_ketana_16_point_1_degrees))
        .register(function!(samuch_le_mincha_ketana_72_minutes))
        .register(function!(sof_zman_achilas_chametz_gra))
        .register(function!(sof_zman_achilas_chametz_mga_72_minutes))
        .register(function!(sof_zman_achilas_chametz_mga_16_point_1_degrees))
        .register(function!(sof_zman_achilas_chametz_baal_hatanya))
        .register(function!(sof_zman_biur_chametz_gra))
        .register(function!(sof_zman_biur_chametz_mga_72_minutes))
        .register(function!(sof_zman_biur_chametz_mga_16_point_1_degrees))
        .register(function!(sof_zman_biur_chametz_baal_hatanya))
        .register(function!(sof_zman_shma_gra))
        .register(function!(sof_zman_shma_mga))
        .register(function!(sof_zman_shma_mga_19_point_8_degrees))
        .register(function!(sof_zman_shma_mga_16_point_1_degrees))
        .register(function!(sof_zman_shma_mga_18_degrees))
        .register(function!(sof_zman_shma_mga_72_minutes))
        .register(function!(sof_zman_shma_mga_72_zmanis))
        .register(function!(sof_zman_shma_mga_90_minutes))
        .register(function!(sof_zman_shma_mga_90_zmanis))
        .register(function!(sof_zman_shma_mga_96_minutes))
        .register(function!(sof_zman_shma_mga_96_zmanis))
        .register(function!(sof_zman_shma_hours_3_before_chatzos))
        .register(function!(sof_zman_shma_mga_120_minutes))
        .register(function!(sof_zman_shma_alos_16_point_1_to_sunset))
        .register(function!(sof_zman_shma_alos_16_point_1_to_tzais_geonim_7_point_083))
        .register(function!(sof_zman_shma_kol_eliyahu))
        .register(function!(sof_zman_shma_ateret_torah))
        .register(function!(sof_zman_shma_baal_hatanya))
        .register(function!(sof_zman_shma_fixed_local))
        .register(function!(sof_zman_shma_gra_sunrise_to_fixed_local_chatzos))
        .register(function!(sof_zman_shma_mga_18_degrees_to_fixed_local_chatzos))
        .register(function!(sof_zman_shma_mga_16_point_1_degrees_to_fixed_local_chatzos))
        .register(function!(sof_zman_shma_mga_90_minutes_to_fixed_local_chatzos))
        .register(function!(sof_zman_shma_mga_72_minutes_to_fixed_local_chatzos))
        .register(function!(sof_zman_tfila_gra))
        .register(function!(sof_zman_tfila_mga))
        .register(function!(sof_zman_tfila_mga_19_point_8_degrees))
        .register(function!(sof_zman_tfila_mga_16_point_1_degrees))
        .register(function!(sof_zman_tfila_mga_18_degrees))
        .register(function!(sof_zman_tfila_mga_72_minutes))
        .register(function!(sof_zman_tfila_mga_72_zmanis))
        .register(function!(sof_zman_tfila_mga_90_minutes))
        .register(function!(sof_zman_tfila_mga_90_zmanis))
        .register(function!(sof_zman_tfila_mga_96_minutes))
        .register(function!(sof_zman_tfila_mga_96_zmanis))
        .register(function!(sof_zman_tfila_hours_2_before_chatzos))
        .register(function!(sof_zman_tfila_mga_120_minutes))
        .register(function!(sof_zman_tfila_ateret_torah))
        .register(function!(sof_zman_tfila_baal_hatanya))
        .register(function!(sof_zman_tfila_fixed_local))
        .register(function!(sof_zman_tfila_gra_sunrise_to_fixed_local_chatzos))
        .register(function!(tzais_degrees_8_point_5))
        .register(function!(tzais_minutes_50))
        .register(function!(tzais_minutes_60))
        .register(function!(tzais_minutes_72))
        .register(function!(tzais_72_zmanis))
        .register(function!(tzais_minutes_90))
        .register(function!(tzais_90_zmanis))
        .register(function!(tzais_minutes_96))
        .register(function!(tzais_96_zmanis))
        .register(function!(tzais_minutes_120))
        .register(function!(tzais_120_zmanis))
        .register(function!(tzais_16_point_1_degrees))
        .register(function!(tzais_18_degrees))
        .register(function!(tzais_19_point_8_degrees))
        .register(function!(tzais_26_degrees))
        .register(function!(tzais_ateret_torah))
        .register(function!(tzais_baal_hatanya))
        .register(function!(tzais_geonim_degrees_3_point_7))
        .register(function!(tzais_geonim_degrees_3_point_8))
        .register(function!(tzais_geonim_degrees_5_point_95))
        .register(function!(tzais_geonim_3_point_65))
        .register(function!(tzais_geonim_3_point_676))
        .register(function!(tzais_geonim_degrees_4_point_61))
        .register(function!(tzais_geonim_degrees_4_point_37))
        .register(function!(tzais_geonim_degrees_5_point_88))
        .register(function!(tzais_geonim_degrees_4_point_8))
        .register(function!(tzais_geonim_degrees_6_point_45))
        .register(function!(tzais_geonim_degrees_7_point_083))
        .register(function!(tzais_geonim_degrees_7_point_67))
        .register(function!(tzais_geonim_degrees_8_point_5))
        .register(function!(tzais_geonim_degrees_9_point_3))
        .register(function!(tzais_geonim_degrees_9_point_75))
        .register(function!(sof_zman_kidush_levana_15_days))
        .register(function!(sof_zman_kidush_levana_between_moldos))
        .register(function!(tchilas_zman_kidush_levana_3_days))
        .register(function!(tchilas_zman_kidush_levana_7_days))
        .register(function!(molad))
        .inventory()
}
