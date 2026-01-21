use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::Tz;
use j4rs::{ClasspathEntry, Instance, InvocationArg, Jvm, JvmBuilder, Null};
use rand::Rng;
use std::env;
use std::str::FromStr;
use std::sync::Once;

use super::*;

/// Default number of iterations for randomized tests.
pub fn get_test_iterations() -> i64 {
    env::var("TEST_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000)
}

// /// Default Epsilon for floating point comparisons.
static JVM_INIT: Once = Once::new();

/// Initializes or attaches to the shared JVM instance for testing against KosherJava.
///
/// The JVM is created once on first call, then subsequent calls attach the current thread.
/// This allows multi-threaded tests to share a single JVM instance.
pub fn init_jvm() -> Jvm {
    JVM_INIT.call_once(|| {
        let _ = JvmBuilder::new()
            .classpath_entry(ClasspathEntry::new(
                "./kosher-java/target/zmanim-2.6.0-SNAPSHOT.jar",
            ))
            .classpath_entry(ClasspathEntry::new(
                "./kosher-java/target/dependency/icu4j-78.1.jar",
            ))
            .build()
            .unwrap();
    });

    // Attach the current thread to the existing shared JVM (returns a local handle).
    // This works on any thread; JNI allows re-attach on the same thread.
    Jvm::attach_thread().unwrap()
}

/// Default number of years to test.
static DEFAULT_TEST_YEARS: i64 = 100;

/// Default number of milliseconds in the given number of years.
static DEFAULT_TEST_YEARS_IN_MILLISECONDS: i64 = 1000 * 3600 * 24 * 365 * DEFAULT_TEST_YEARS;

const TZ_CHOICES: [&str; 8] = [
    "UTC",
    "Etc/UTC",
    "Etc/GMT",
    "Etc/GMT+1",
    "Etc/GMT-1",
    "Etc/GMT+2",
    "Etc/GMT-2",
    "Etc/GMT-14",
];

/// Generates a random DateTime in the range 1870-2070 with the given timezone.
fn random_date_time(rng: &mut impl Rng) -> DateTime<Utc> {
    let milliseconds_since_epoch: i64 = rng.gen_range(
        -DEFAULT_TEST_YEARS_IN_MILLISECONDS..=DEFAULT_TEST_YEARS_IN_MILLISECONDS, // 1870 to 2070
    );
    Utc.timestamp_millis_opt(milliseconds_since_epoch).unwrap()
}

fn random_tz(rng: &mut impl Rng) -> Tz {
    let idx = rng.gen_range(0..TZ_CHOICES.len());
    Tz::from_str(TZ_CHOICES[idx]).unwrap()
}
/// Generates a random Hebrew date in the range 1870-2070.
fn random_hebrew_date(rng: &mut impl Rng) -> (i32, HebrewMonth, u8) {
    let dt = random_date_time(rng);
    let year = dt.year() + 3760; // 3760 is the difference between the Gregorian and Hebrew years

    let month = match rng.gen_range(1..=13) {
        1 => HebrewMonth::Nissan,
        2 => HebrewMonth::Iyar,
        3 => HebrewMonth::Sivan,
        4 => HebrewMonth::Tammuz,
        5 => HebrewMonth::Av,
        6 => HebrewMonth::Elul,
        7 => HebrewMonth::Tishrei,
        8 => HebrewMonth::Cheshvan,
        9 => HebrewMonth::Kislev,
        10 => HebrewMonth::Teves,
        11 => HebrewMonth::Shevat,
        12 => HebrewMonth::Adar,
        13 => HebrewMonth::AdarII,
        _ => unreachable!(),
    };
    let day = rng.gen_range(1..=30);
    (year, month, day as u8)
}
pub struct JavaJewishCalendar<'a> {
    pub jvm: &'a Jvm,
    pub instance: Instance,
    #[allow(dead_code)]
    pub in_israel: bool,
    #[allow(dead_code)]
    pub is_mukaf_choma: bool,
    #[allow(dead_code)]
    pub use_modern_holidays: bool,
}

impl<'a> JavaJewishCalendar<'a> {
    pub fn from_gregorian_date(
        jvm: &'a Jvm,
        year: i32,
        month: i32,
        day: i32,
        in_israel: bool,
        is_mukaf_choma: bool,
        use_modern_holidays: bool,
    ) -> Option<Self> {
        let year_arg = InvocationArg::try_from(year)
            .unwrap()
            .into_primitive()
            .unwrap();
        let month_arg = InvocationArg::try_from(month)
            .unwrap()
            .into_primitive()
            .unwrap();
        let day_arg = InvocationArg::try_from(day)
            .unwrap()
            .into_primitive()
            .unwrap();
        let local_date = jvm
            .invoke_static("java.time.LocalDate", "of", &[year_arg, month_arg, day_arg])
            .unwrap();
        let instance = jvm
            .create_instance(
                "com.kosherjava.zmanim.hebrewcalendar.JewishCalendar",
                &[InvocationArg::from(local_date)],
            )
            .ok();
        instance.as_ref()?;
        let instance = instance.unwrap();
        let self_ = Self {
            jvm,
            instance,
            in_israel,
            is_mukaf_choma,
            use_modern_holidays,
        };
        self_.set_in_israel(in_israel);
        self_.set_is_mukaf_choma(is_mukaf_choma);
        self_.set_use_modern_holidays(use_modern_holidays);
        Some(self_)
    }
    pub fn from_jewish_date(
        jvm: &'a Jvm,
        year: i32,
        month: HebrewMonth,
        day: i32,
        in_israel: bool,
        is_mukaf_choma: bool,
        use_modern_holidays: bool,
    ) -> Option<Self> {
        let year_arg = InvocationArg::try_from(year)
            .unwrap()
            .into_primitive()
            .unwrap();
        let month_arg = InvocationArg::try_from(month as i32)
            .unwrap()
            .into_primitive()
            .unwrap();
        let day_arg = InvocationArg::try_from(day)
            .unwrap()
            .into_primitive()
            .unwrap();
        let instance = jvm
            .create_instance(
                "com.kosherjava.zmanim.hebrewcalendar.JewishCalendar",
                &[year_arg, month_arg, day_arg],
            )
            .ok();
        instance.as_ref()?;
        let instance = instance.unwrap();
        let self_ = Self {
            jvm,
            instance,
            in_israel,
            is_mukaf_choma,
            use_modern_holidays,
        };
        self_.set_in_israel(in_israel);
        self_.set_is_mukaf_choma(is_mukaf_choma);
        self_.set_use_modern_holidays(use_modern_holidays);
        Some(self_)
    }
    fn set_is_mukaf_choma(&self, is_mukaf_choma: bool) {
        self.jvm
            .invoke(
                &self.instance,
                "setIsMukafChoma",
                &[InvocationArg::try_from(is_mukaf_choma)
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .unwrap();
    }

    fn invoke_bool(&self, method: &str) -> bool {
        let java_result = self
            .jvm
            .invoke(&self.instance, method, InvocationArg::empty())
            .unwrap();
        self.jvm.to_rust::<bool>(java_result).unwrap()
    }

    fn invoke_i64(&self, method: &str) -> i64 {
        let java_result = self
            .jvm
            .invoke(&self.instance, method, InvocationArg::empty())
            .unwrap();
        self.jvm.to_rust::<i64>(java_result).unwrap()
    }
    #[allow(unused)]
    pub fn set_in_israel(&self, in_israel: bool) {
        self.jvm
            .invoke(
                &self.instance,
                "setInIsrael",
                &[InvocationArg::try_from(in_israel)
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .unwrap();
    }

    #[allow(unused)]
    pub fn set_mukaf_choma(&self, mukaf_choma: bool) {
        self.jvm
            .invoke(
                &self.instance,
                "setIsMukafChoma",
                &[InvocationArg::try_from(mukaf_choma)
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .unwrap();
    }

    #[allow(unused)]
    pub fn set_use_modern_holidays(&self, use_modern_holidays: bool) {
        self.jvm
            .invoke(
                &self.instance,
                "setUseModernHolidays",
                &[InvocationArg::try_from(use_modern_holidays)
                    .unwrap()
                    .into_primitive()
                    .unwrap()],
            )
            .unwrap();
    }

    #[allow(dead_code)]
    fn java_date_to_rust_datetime(&self, java_date: &Instance) -> Option<DateTime<Utc>> {
        let is_null = self
            .jvm
            .check_equals(
                java_date,
                InvocationArg::try_from(Null::Of("java.util.Date")).unwrap(),
            )
            .unwrap();
        if is_null {
            return None;
        }

        let millis = self
            .jvm
            .to_rust::<i64>(
                self.jvm
                    .invoke(java_date, "getTime", InvocationArg::empty())
                    .unwrap(),
            )
            .unwrap();

        DateTime::<Utc>::from_timestamp_millis(millis)
    }

    #[allow(dead_code)]
    fn invoke_date(&self, method: &str) -> Option<DateTime<Utc>> {
        let java_result = self
            .jvm
            .invoke(&self.instance, method, InvocationArg::empty())
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    fn parsha_from_java(&self, method: &str) -> Option<Parsha> {
        let java_parsha = self
            .jvm
            .invoke(&self.instance, method, InvocationArg::empty())
            .ok()?;
        let ordinal_instance = self
            .jvm
            .invoke(&java_parsha, "ordinal", InvocationArg::empty())
            .ok()?;
        let ordinal = self.jvm.to_rust::<i32>(ordinal_instance).ok()?;
        if ordinal == 0 {
            None
        } else {
            Parsha::try_from((ordinal - 1) as u8).ok()
        }
    }

    pub fn is_assur_bemelacha(&self) -> bool {
        self.invoke_bool("isAssurBemelacha")
    }

    pub fn has_candle_lighting(&self) -> bool {
        self.invoke_bool("hasCandleLighting")
    }

    pub fn is_aseres_yemei_teshuva(&self) -> bool {
        self.invoke_bool("isAseresYemeiTeshuva")
    }

    pub fn get_parshah(&self) -> Option<Parsha> {
        self.parsha_from_java("getParshah")
    }

    pub fn get_special_shabbos(&self) -> Option<Parsha> {
        self.parsha_from_java("getSpecialShabbos")
    }

    pub fn get_upcoming_parshah(&self) -> Option<Parsha> {
        self.parsha_from_java("getUpcomingParshah")
    }

    pub fn get_day_of_chanukah(&self) -> Option<u8> {
        let result = self.invoke_i64("getDayOfChanukah");
        if result == -1 {
            None
        } else {
            Some(result as u8)
        }
    }

    pub fn get_day_of_omer(&self) -> Option<u8> {
        let result = self.invoke_i64("getDayOfOmer");
        if result == -1 {
            None
        } else {
            Some(result as u8)
        }
    }

    pub fn get_yom_tov_index(&self) -> Option<i32> {
        let java_result = self
            .jvm
            .invoke(&self.instance, "getYomTovIndex", InvocationArg::empty())
            .unwrap();
        let index = self.jvm.to_rust::<i32>(java_result).unwrap();
        if index == -1 {
            None
        } else {
            Some(index)
        }
    }

    pub fn is_taanis_bechoros(&self) -> bool {
        self.invoke_bool("isTaanisBechoros")
    }

    pub fn is_birkas_hachamah(&self) -> bool {
        self.invoke_bool("isBirkasHachamah")
    }

    pub fn is_machar_chodesh(&self) -> bool {
        self.invoke_bool("isMacharChodesh")
    }

    pub fn is_shabbos_mevorchim(&self) -> bool {
        self.invoke_bool("isShabbosMevorchim")
    }

    pub fn is_rosh_chodesh(&self) -> bool {
        self.invoke_bool("isRoshChodesh")
    }

    pub fn is_yom_kippur_katan(&self) -> bool {
        self.invoke_bool("isYomKippurKatan")
    }

    pub fn is_behab(&self) -> bool {
        self.invoke_bool("isBeHaB")
    }

    pub fn get_molad_as_date(&self) -> Option<DateTime<Utc>> {
        self.invoke_date("getMoladAsDate")
    }
}

pub struct JavaComplexZmanimCalendar<'a> {
    pub jvm: &'a Jvm,
    pub instance: Instance,
}

impl<'a> JavaComplexZmanimCalendar<'a> {
    pub fn from_gregorian_date(
        jvm: &'a Jvm,
        year: i32,
        month: i32,
        day: i32,
        tz: &Tz,
    ) -> Option<Self> {
        let instance = jvm
            .create_instance(
                "com.kosherjava.zmanim.ComplexZmanimCalendar",
                InvocationArg::empty(),
            )
            .ok()?;

        let tz_arg = InvocationArg::try_from(tz.name()).unwrap();
        let java_tz = jvm
            .invoke_static("com.ibm.icu.util.TimeZone", "getTimeZone", &[tz_arg])
            .ok()?;
        let calendar = jvm
            .invoke(&instance, "getCalendar", InvocationArg::empty())
            .ok()?;
        jvm.invoke(&calendar, "setTimeZone", &[InvocationArg::from(java_tz)])
            .ok()?;
        let year_arg = InvocationArg::try_from(year)
            .unwrap()
            .into_primitive()
            .unwrap();
        let month_arg = InvocationArg::try_from(month)
            .unwrap()
            .into_primitive()
            .unwrap();
        let day_arg = InvocationArg::try_from(day)
            .unwrap()
            .into_primitive()
            .unwrap();
        let local_date = jvm
            .invoke_static("java.time.LocalDate", "of", &[year_arg, month_arg, day_arg])
            .ok()?;
        let sql_date = jvm
            .invoke_static(
                "java.sql.Date",
                "valueOf",
                &[InvocationArg::from(local_date)],
            )
            .ok()?;
        jvm.invoke(&calendar, "setTime", &[InvocationArg::from(sql_date)])
            .ok()?;

        Some(Self { jvm, instance })
    }

    fn java_date_to_rust_datetime(&self, java_date: &Instance) -> Option<DateTime<Utc>> {
        let is_null = self
            .jvm
            .check_equals(
                java_date,
                InvocationArg::try_from(Null::Of("java.util.Date")).unwrap(),
            )
            .unwrap();
        if is_null {
            return None;
        }

        let millis = self
            .jvm
            .to_rust::<i64>(
                self.jvm
                    .invoke(java_date, "getTime", InvocationArg::empty())
                    .unwrap(),
            )
            .unwrap();

        DateTime::<Utc>::from_timestamp_millis(millis)
    }

    fn invoke_date(&self, method: &str) -> Option<DateTime<Utc>> {
        let java_result = self
            .jvm
            .invoke(&self.instance, method, InvocationArg::empty())
            .ok()?;
        self.java_date_to_rust_datetime(&java_result)
    }

    pub fn get_sof_zman_kidush_levana_between_moldos(&self) -> Option<DateTime<Utc>> {
        self.invoke_date("getSofZmanKidushLevanaBetweenMoldos")
    }

    pub fn get_sof_zman_kidush_levana_15_days(&self) -> Option<DateTime<Utc>> {
        self.invoke_date("getSofZmanKidushLevana15Days")
    }

    pub fn get_tchilas_zman_kidush_levana_3_days(&self) -> Option<DateTime<Utc>> {
        self.invoke_date("getTchilasZmanKidushLevana3Days")
    }

    pub fn get_tchilas_zman_kidush_levana_7_days(&self) -> Option<DateTime<Utc>> {
        self.invoke_date("getTchilasZmanKidushLevana7Days")
    }

    pub fn get_zman_molad(&self) -> Option<DateTime<Utc>> {
        self.invoke_date("getZmanMolad")
    }
}

/// Maps Java's getYomTovIndex() integer values to Rust Holiday enum values.
/// Based on the KosherJava JewishCalendar class constants.
fn java_holiday_index_to_rust(index: i32) -> Option<Holiday> {
    match index {
        0 => Some(Holiday::ErevPesach),
        1 => Some(Holiday::Pesach),
        2 => Some(Holiday::CholHamoed), // Chol Hamoed Pesach
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
        16 => Some(Holiday::CholHamoed), // Chol Hamoed Succos
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

pub fn create_jewish_calendars<'a>(
    jvm: &'a Jvm,
    rng: &mut impl Rng,
) -> Option<(
    Date<Hebrew>,
    JavaJewishCalendar<'a>,
    bool,
    bool,
    bool,
    String,
)> {
    let use_gregorian_date = rng.gen_bool(0.5);
    let in_israel = rng.gen_bool(0.5);
    let is_mukaf_choma = rng.gen_bool(0.5);
    let use_modern_holidays = rng.gen_bool(0.5);

    if use_gregorian_date {
        let date_time = random_date_time(rng);
        let message = format!(
            "year: {}, month: {}, day: {}, in_israel: {}, is_mukaf_choma: {}, use_modern_holidays: {}",
            date_time.year(),
            date_time.month(),
            date_time.day(),
            in_israel,
            is_mukaf_choma,
            use_modern_holidays
        );
        let rust_date = Date::try_new_gregorian(
            date_time.year(),
            date_time.month() as u8,
            date_time.day() as u8,
        )
        .map(|date| date.to_calendar(Hebrew))
        .ok();

        let java_calendar = JavaJewishCalendar::from_gregorian_date(
            jvm,
            date_time.year() as i32,
            date_time.month() as i32,
            date_time.day() as i32,
            in_israel,
            is_mukaf_choma,
            use_modern_holidays,
        );

        assert_eq!(rust_date.is_some(), java_calendar.is_some(), "{}", message);
        if rust_date.is_none() || java_calendar.is_none() {
            return None;
        }

        Some((
            rust_date.unwrap(),
            java_calendar.unwrap(),
            in_israel,
            is_mukaf_choma,
            use_modern_holidays,
            message,
        ))
    } else {
        let (year, month, day) = random_hebrew_date(rng);
        let message = format!(
            "year: {}, month: {}, day: {}, in_israel: {}, is_mukaf_choma: {}, use_modern_holidays: {}",
            year, month as i32, day, in_israel, is_mukaf_choma, use_modern_holidays
        );

        let rust_calendar = Date::from_hebrew_date(year, month, day);
        let java_calendar = JavaJewishCalendar::from_jewish_date(
            jvm,
            year,
            month,
            day as i32,
            in_israel,
            is_mukaf_choma,
            use_modern_holidays,
        );

        assert_eq!(
            rust_calendar.is_some(),
            java_calendar.is_some(),
            "{}",
            message
        );
        if rust_calendar.is_none() || java_calendar.is_none() {
            return None;
        }

        let java_calendar = java_calendar.unwrap();

        Some((
            rust_calendar.unwrap(),
            java_calendar,
            in_israel,
            is_mukaf_choma,
            use_modern_holidays,
            message,
        ))
    }
}

#[cfg(test)]
mod holiday_tests {
    use super::*;

    // This test module covers all Rust Holiday enum values against Java's KosherJava library:
    //
    // Main holidays tested via getYomTovIndex():
    // - All holidays returned by Java's getYomTovIndex() (indexes 0-37)
    //   are tested in test_holidays_contains_yom_tov_index()
    //
    // Special holidays tested via specific Java methods:
    // - FastOfTheFirstborn: test_fast_of_firstborn() uses isTaanisBechoros()
    // - BirchasHachamah: test_birkas_hachamah() uses isBirkasHachamah()
    // - MacharHachodesh: test_machar_chodesh() uses isMacharChodesh()
    // - ShabbosMevarchim: test_shabbos_mevorchim() uses isShabbosMevorchim()
    // - RoshChodesh: test_is_rosh_chodesh() uses isRoshChodesh()
    // - YomKippurKatan: test_yom_kippur_katan() uses isYomKippurKatan()
    // - Behab: test_behab() uses isBeHaB()
    // - CountOfTheOmer: covered by test_day_of_the_omer() via getDayOfOmer()

    fn random_zmanim_calendar<'a>(
        jvm: &'a Jvm,
        gregorian: &Date<Gregorian>,
        rng: &mut impl Rng,
    ) -> (Tz, JavaComplexZmanimCalendar<'a>) {
        for _ in 0..TZ_CHOICES.len() {
            let tz = random_tz(rng);
            if let Some(calendar) = JavaComplexZmanimCalendar::from_gregorian_date(
                jvm,
                gregorian.extended_year(),
                gregorian.month().month_number() as i32,
                gregorian.day_of_month().0 as i32,
                &tz,
            ) {
                return (tz, calendar);
            }
        }
        panic!(
            "Failed to create JavaComplexZmanimCalendar for {}-{}-{}",
            gregorian.extended_year(),
            gregorian.month().month_number(),
            gregorian.day_of_month().0
        );
    }

    #[test]
    fn test_is_assur_bemelacha() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.is_assur_bemelacha(in_israel);
            let java_result = java_calendar.is_assur_bemelacha();

            assert_eq!(
                rust_result, java_result,
                "is_assur_bemelacha mismatch: Rust={}, Java={}, {}",
                rust_result, java_result, message
            );
        }
    }

    #[test]
    fn test_has_candle_lighting() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.has_candle_lighting(in_israel);
            let java_result = java_calendar.has_candle_lighting();

            assert_eq!(
                rust_result, java_result,
                "has_candle_lighting mismatch: Rust={}, Java={}, {}",
                rust_result, java_result, message
            );
        }
    }

    #[test]
    fn test_is_aseres_yemei_teshuva() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.is_aseres_yemei_teshuva();
            let java_result = java_calendar.is_aseres_yemei_teshuva();

            assert_eq!(
                rust_result, java_result,
                "is_aseres_yemei_teshuva mismatch: Rust={}, Java={}, {}",
                rust_result, java_result, message
            );
        }
    }

    #[test]
    fn test_todays_parsha() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.todays_parsha(in_israel);
            let java_result = java_calendar.get_parshah();

            assert_eq!(
                rust_result, java_result,
                "todays_parsha mismatch: Rust={:?}, Java={:?}, {}",
                rust_result, java_result, message
            );
        }
    }

    #[test]
    fn test_special_parsha() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.special_parsha(in_israel);
            let java_result = java_calendar.get_special_shabbos();

            assert_eq!(
                rust_result, java_result,
                "special_parsha mismatch: Rust={:?}, Java={:?}, {}",
                rust_result, java_result, message
            );
        }
    }

    #[test]
    fn test_upcoming_parsha() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.upcoming_parsha(in_israel);
            let java_result = java_calendar.get_upcoming_parshah();

            // Java returns Option, but Rust always returns a value
            if let Some(java_parsha) = java_result {
                assert_eq!(
                    rust_result, java_parsha,
                    "upcoming_parsha mismatch: Rust={:?}, Java={:?}, {}",
                    rust_result, java_parsha, message
                );
            }
        }
    }

    #[test]
    fn test_day_of_chanukah() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.day_of_chanukah();
            let java_result = java_calendar.get_day_of_chanukah();

            assert_eq!(
                rust_result, java_result,
                "day_of_chanukah mismatch: Rust={:?}, Java={:?}, {}",
                rust_result, java_result, message
            );
        }
    }

    #[test]
    fn test_day_of_the_omer() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let rust_result = rust_date.day_of_the_omer();
            let java_result = java_calendar.get_day_of_omer();

            assert_eq!(
                rust_result, java_result,
                "day_of_the_omer mismatch: Rust={:?}, Java={:?}, {}",
                rust_result, java_result, message
            );
        }
    }

    #[test]
    fn test_holidays_contains_yom_tov_index() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_yom_tov_index = java_calendar.get_yom_tov_index();

            // If Java returns a holiday index, convert it to a Rust Holiday
            if let Some(index) = java_yom_tov_index {
                if let Some(expected_holiday) = java_holiday_index_to_rust(index) {
                    let rust_holidays: Vec<Holiday> = rust_date
                        .holidays(in_israel, use_modern_holidays)
                        .copied()
                        .collect();

                    assert!(
                        rust_holidays.contains(&expected_holiday),
                        "Holiday from Java's getYomTovIndex (index={}, {:?}) not found in Rust holidays {:?}, {}",
                        index, expected_holiday, rust_holidays, message
                    );
                }
            }
        }
    }

    #[test]
    fn test_fast_of_firstborn() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_result = java_calendar.is_taanis_bechoros();
            let rust_holidays: Vec<Holiday> = rust_date
                .holidays(in_israel, use_modern_holidays)
                .copied()
                .collect();
            let rust_result = rust_holidays.contains(&Holiday::FastOfTheFirstborn);

            assert_eq!(
                rust_result, java_result,
                "FastOfTheFirstborn mismatch: Rust={}, Java={}, Rust holidays={:?}, {}",
                rust_result, java_result, rust_holidays, message
            );
        }
    }

    #[test]
    fn test_birkas_hachamah() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_result = java_calendar.is_birkas_hachamah();
            let rust_holidays: Vec<Holiday> = rust_date
                .holidays(in_israel, use_modern_holidays)
                .copied()
                .collect();
            let rust_result = rust_holidays.contains(&Holiday::BirchasHachamah);

            assert_eq!(
                rust_result, java_result,
                "BirchasHachamah mismatch: Rust={}, Java={}, Rust holidays={:?}, {}",
                rust_result, java_result, rust_holidays, message
            );
        }
    }

    #[test]
    fn test_machar_chodesh() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_result = java_calendar.is_machar_chodesh();
            let rust_holidays: Vec<Holiday> = rust_date
                .holidays(in_israel, use_modern_holidays)
                .copied()
                .collect();
            let rust_result = rust_holidays.contains(&Holiday::MacharHachodesh);

            assert_eq!(
                rust_result, java_result,
                "MacharHachodesh mismatch: Rust={}, Java={}, Rust holidays={:?}, {}",
                rust_result, java_result, rust_holidays, message
            );
        }
    }

    #[test]
    fn test_shabbos_mevorchim() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_result = java_calendar.is_shabbos_mevorchim();
            let rust_holidays: Vec<Holiday> = rust_date
                .holidays(in_israel, use_modern_holidays)
                .copied()
                .collect();
            let rust_result = rust_holidays.contains(&Holiday::ShabbosMevarchim);

            assert_eq!(
                rust_result, java_result,
                "ShabbosMevarchim mismatch: Rust={}, Java={}, Rust holidays={:?}, {}",
                rust_result, java_result, rust_holidays, message
            );
        }
    }

    #[test]
    fn test_is_rosh_chodesh() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_result = java_calendar.is_rosh_chodesh();
            let rust_holidays: Vec<Holiday> = rust_date
                .holidays(in_israel, use_modern_holidays)
                .copied()
                .collect();
            let rust_result = rust_holidays.contains(&Holiday::RoshChodesh);

            assert_eq!(
                rust_result, java_result,
                "RoshChodesh mismatch: Rust={}, Java={}, Rust holidays={:?}, {}",
                rust_result, java_result, rust_holidays, message
            );
        }
    }

    #[test]
    fn test_yom_kippur_katan() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_result = java_calendar.is_yom_kippur_katan();
            let rust_holidays: Vec<Holiday> = rust_date
                .holidays(in_israel, use_modern_holidays)
                .copied()
                .collect();
            let rust_result = rust_holidays.contains(&Holiday::YomKippurKatan);

            assert_eq!(
                rust_result, java_result,
                "YomKippurKatan mismatch: Rust={}, Java={}, Rust holidays={:?}, {}",
                rust_result, java_result, rust_holidays, message
            );
        }
    }

    #[test]
    fn test_behab() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                in_israel,
                _is_mukaf_choma,
                use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_result = java_calendar.is_behab();
            let rust_holidays: Vec<Holiday> = rust_date
                .holidays(in_israel, use_modern_holidays)
                .copied()
                .collect();
            let rust_result = rust_holidays.contains(&Holiday::Behab);

            assert_eq!(
                rust_result, java_result,
                "Behab mismatch: Rust={}, Java={}, Rust holidays={:?}, {}",
                rust_result, java_result, rust_holidays, message
            );
        }
    }

    #[test]
    fn test_molad() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();

            let java_molad = java_calendar.get_molad_as_date();
            let rust_molad = months_molad(&rust_date);

            // Both should be Some or both None
            if java_molad.is_none() && rust_molad.is_none() {
                continue;
            }

            assert_eq!(
                java_molad.is_some(),
                rust_molad.is_some(),
                "Molad existence mismatch: Rust={:?}, Java={:?}, {}",
                rust_molad.is_some(),
                java_molad.is_some(),
                message
            );

            if let (Some(java), Some(rust)) = (java_molad, rust_molad) {
                // Calculate the difference in milliseconds
                let diff_millis = (java.timestamp_millis() - rust.timestamp_millis()).abs();

                // Allow a tolerance of 1 second (1000 ms) due to potential rounding differences
                assert!(
                    diff_millis <= 1000,
                    "Molad time mismatch: Rust={}, Java={}, diff={}ms, {}",
                    rust,
                    java,
                    diff_millis,
                    message
                );
            }
        }
    }

    fn assert_java_rust_zman<Tz: TimeZone>(
        java_date: Option<DateTime<Utc>>,
        rust_date: Option<(DateTime<Tz>, HebrewMonth)>,
        label: &str,
        message: &str,
    ) {
        if java_date.is_none() && rust_date.is_none() {
            return;
        }

        assert_eq!(
            java_date.is_some(),
            rust_date.is_some(),
            "{} existence mismatch: Rust={:?}, Java={:?}, JavaDate={:?}, RustDate={:?}, {}",
            label,
            rust_date.is_some(),
            java_date.is_some(),
            java_date,
            rust_date,
            message
        );

        let (java, (rust, rust_month)) = match (java_date, rust_date) {
            (Some(java), Some(rust)) => (java, rust),
            _ => return,
        };

        let rust_utc = rust.with_timezone(&Utc);
        let diff_millis = (java.timestamp_millis() - rust_utc.timestamp_millis()).abs();
        assert!(
            diff_millis <= 1000,
            "{} time mismatch: Rust={}, Java={}, diff={}ms, {}",
            label,
            rust_utc,
            java,
            diff_millis,
            message
        );

        let _ = rust_month;
        let _ = java;
    }

    #[test]
    fn test_sof_zman_kidush_levana_between_moldos() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                _java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();
            let rust_date = rust_date.to_calendar(Gregorian);

            let gregorian = rust_date;

            let (tz, zmanim_calendar) = random_zmanim_calendar(&jvm, &gregorian, &mut rng);

            let java_date = zmanim_calendar.get_sof_zman_kidush_levana_between_moldos();
            let rust_date = rust_date.sof_zman_kidush_levana_between_moldos(&tz);

            assert_java_rust_zman(
                java_date,
                rust_date,
                "sof_zman_kidush_levana_between_moldos",
                &message,
            );
        }
    }

    #[test]
    fn test_sof_zman_kidush_levana_15_days() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                _java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();
            let rust_date = rust_date.to_calendar(Gregorian);

            let gregorian = rust_date;

            let (tz, zmanim_calendar) = random_zmanim_calendar(&jvm, &gregorian, &mut rng);

            let java_date = zmanim_calendar.get_sof_zman_kidush_levana_15_days();
            let rust_date = rust_date.sof_zman_kidush_levana_15_days(&tz);

            assert_java_rust_zman(
                java_date,
                rust_date,
                "sof_zman_kidush_levana_15_days",
                &message,
            );
        }
    }

    #[test]
    fn test_tchilas_zman_kidush_levana_3_days() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                _java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();
            let rust_date = rust_date.to_calendar(Gregorian);

            let gregorian = rust_date;
            let (tz, zmanim_calendar) = random_zmanim_calendar(&jvm, &gregorian, &mut rng);

            let java_date = zmanim_calendar.get_tchilas_zman_kidush_levana_3_days();
            let rust_date = rust_date.tchilas_zman_kidush_levana_3_days(&tz);

            assert_java_rust_zman(
                java_date,
                rust_date,
                "tchilas_zman_kidush_levana_3_days",
                &message,
            );
        }
    }

    #[test]
    fn test_tchilas_zman_kidush_levana_7_days() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_date,
                _java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();
            let rust_date = rust_date.to_calendar(Gregorian);

            let gregorian = rust_date;
            let (tz, zmanim_calendar) = random_zmanim_calendar(&jvm, &gregorian, &mut rng);

            let java_date = zmanim_calendar.get_tchilas_zman_kidush_levana_7_days();
            let rust_date = rust_date.tchilas_zman_kidush_levana_7_days(&tz);

            assert_java_rust_zman(
                java_date,
                rust_date,
                "tchilas_zman_kidush_levana_7_days",
                &message,
            );
        }
    }

    #[test]
    fn test_zman_molad() {
        let jvm = init_jvm();
        let mut rng = rand::thread_rng();
        let iterations = get_test_iterations();

        for _ in 0..iterations {
            let result = create_jewish_calendars(&jvm, &mut rng);
            if result.is_none() {
                continue;
            }

            let (
                rust_hdate,
                _java_calendar,
                _in_israel,
                _is_mukaf_choma,
                _use_modern_holidays,
                message,
            ) = result.unwrap();
            let rust_date = rust_hdate.to_calendar(Gregorian);

            let gregorian = rust_date;
            let (tz, zmanim_calendar) = random_zmanim_calendar(&jvm, &gregorian, &mut rng);

            let java_date = zmanim_calendar.get_zman_molad();
            let rust_date = rust_date.molad(&tz);

            assert_java_rust_zman(java_date, rust_date, "zman_molad", &message);
        }
    }
}
