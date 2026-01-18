#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::panic)]

use chrono::DateTime;
use j4rs::{ClasspathEntry, Jvm, JvmBuilder};
mod java_bindings;
mod java_rnd;
extern crate std;

use std::env;
use std::sync::Once;

use crate::types::zman::ZmanLike;
use crate::*;

/// Default number of iterations for randomized tests.
pub fn get_test_iterations() -> i64 {
    env::var("TEST_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

// /// Default Epsilon for floating point comparisons.
// pub static DEFAULT_F64_TEST_EPSILON: f64 = 1e-4;
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rand::{Rng, SeedableRng};

    /// Maximum allowed difference in seconds between Rust and Java implementations
    const MAX_DIFF_SECONDS: i64 = 40;

    /// Gets a seed for testing. Uses a random seed by default, but can be overridden
    /// with the TEST_SEED environment variable for reproducibility.
    fn get_test_seed() -> u64 {
        env::var("TEST_SEED")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                use rand::RngCore;
                let mut rng = rand::thread_rng();
                rng.next_u64()
            })
    }

    /// Helper function to compare two optional DateTimes
    fn assert_times_close(
        rust_time: Option<DateTime<chrono_tz::Tz>>,
        java_time: Option<DateTime<chrono_tz::Tz>>,
        max_diff_seconds: i64,
        context: &str,
    ) {
        match (rust_time, java_time) {
            (Some(rust), Some(java)) => {
                let diff = (rust.timestamp() - java.timestamp()).abs();
                assert!(
                    diff <= max_diff_seconds,
                    "{}: Difference too large: {} seconds. Max allowed: {} seconds\nRust: {:?}\nJava: {:?}",
                    context,
                    diff,
                    max_diff_seconds,
                    rust,
                    java
                );
            }
            (None, None) => {}

            (rust, java) => {
                panic!("{}: Mismatch - Rust: {:?}, Java: {:?}", context, rust, java);
            }
        }
    }

    /// Generic helper to test any Zman against Java implementation
    ///
    /// # Arguments
    /// * `zman` - The Zman to test
    /// * `seed` - Random seed for reproducibility
    /// * `max_diff_override` - Optional override for max difference (if None, uses MAX_DIFF_SECONDS)
    fn test_zman_vs_java<Z: ZmanLike + Copy>(
        zman: Z,
        seed: u64,
        max_diff_override: Option<i64>,
        max_lat: Option<f64>,
    ) {
        std::println!(
            "Testing {:?} with seed: {} (set TEST_SEED={} to reproduce)",
            zman.java_function_name(),
            seed,
            seed
        );
        let jvm = init_jvm();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let iterations = get_test_iterations();

        for i in 0..iterations {
            let Some((mut rust_calculator, java_calendar, tz)) =
                java_rnd::random_zmanim_calendars(&jvm, &mut rng, max_lat)
            else {
                continue;
            };
            if !Location::<chrono_tz::Tz>::near_anti_meridian(rust_calculator.location.longitude)
                && rng.gen_bool(0.5)
            {
                rust_calculator.location.timezone = None;
            }

            let rust_result = rust_calculator.calculate(zman);
            let java_result = java_calendar.get_zman(zman);

            // Convert from Utc to the local timezone for comparison
            let rust_result_tz = rust_result.map(|dt| tz.from_utc_datetime(&dt.naive_utc()));

            let mut max_diff = max_diff_override.unwrap_or(MAX_DIFF_SECONDS);

            // If this zman uses elevation, enlarge the max diff based on the elevation
            if zman.uses_elevation() {
                max_diff += (rust_calculator.location.elevation * 0.1) as i64;
                assert!(
                    max_diff > 0 && max_diff < 100,
                    "Max diff is out of range for {:?}: {} meters",
                    zman.java_function_name(),
                    rust_calculator.location.elevation
                );
            }

            assert_times_close(
                rust_result_tz,
                java_result,
                max_diff,
                &std::format!(
                    "Iteration {}: {:?} at {:?}, Location: ({}, {}), Elevation: {}, Timezone: {:?}",
                    i,
                    &zman.java_function_name(),
                    &rust_calculator.date,
                    &rust_calculator.location.latitude,
                    &rust_calculator.location.longitude,
                    &rust_calculator.location.elevation,
                    &rust_calculator.location.timezone.map(|tz| tz.name()),
                ),
            );
            // for fixed chatzos, make sure that the date is the same at the naive data
            if let Some(java_result) = java_result {
                if zman.java_function_name() == "getFixedLocalChatzos" {
                    assert_eq!(rust_calculator.date, java_result.naive_local().date());
                }
            }
        }
    }

    fn test_zman_iteration<Z: ZmanLike + Copy>(
        zman: Z,
        seed: u64,
        iteration: i64,
        max_diff_override: Option<i64>,
        max_lat: Option<f64>,
    ) {
        let jvm = init_jvm();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut sample = None;

        for i in 0..=iteration {
            let Some((mut rust_calculator, java_calendar, tz)) =
                java_rnd::random_zmanim_calendars(&jvm, &mut rng, max_lat)
            else {
                if i == iteration {
                    panic!("Iteration {} did not produce a test case", iteration);
                }
                continue;
            };
            if !Location::<chrono_tz::Tz>::near_anti_meridian(rust_calculator.location.longitude)
                && rng.gen_bool(0.5)
            {
                rust_calculator.location.timezone = None;
            }

            if i == iteration {
                sample = Some((rust_calculator, java_calendar, tz));
                break;
            }
        }

        let (mut rust_calculator, java_calendar, tz) =
            sample.expect("Expected to find regression sample");
        let rust_result = rust_calculator.calculate(zman);
        let java_result = java_calendar.get_zman(zman);
        let rust_result_tz = rust_result.map(|dt| tz.from_utc_datetime(&dt.naive_utc()));

        let mut max_diff = max_diff_override.unwrap_or(MAX_DIFF_SECONDS);
        if zman.uses_elevation() {
            max_diff += (rust_calculator.location.elevation * 0.1) as i64;
            assert!(
                max_diff > 0 && max_diff < 100,
                "Max diff is out of range for {:?}: {} meters",
                zman.java_function_name(),
                rust_calculator.location.elevation
            );
        }

        assert_times_close(
            rust_result_tz,
            java_result,
            max_diff,
            &std::format!(
                "Regression {:?} seed {} iteration {}: {:?}, Location: ({}, {}), Elevation: {}, Timezone: {:?}",
                &zman.java_function_name(),
                seed,
                iteration,
                &rust_calculator.date,
                &rust_calculator.location.latitude,
                &rust_calculator.location.longitude,
                &rust_calculator.location.elevation,
                &rust_calculator.location.timezone.map(|tz| tz.name()),
            ),
        );
    }

    macro_rules! zman_test {
        ($name:ident, $zman:expr) => {
            #[test]
            fn $name() {
                test_zman_vs_java($zman, get_test_seed(), None, None);
            }
        };
        ($name:ident, $zman:expr, $max_lat:expr) => {
            #[test]
            fn $name() {
                test_zman_vs_java($zman, get_test_seed(), None, $max_lat);
            }
        };
    }

    zman_test!(test_neitz, NeitzZman);
    zman_test!(test_shkia, ShkiaZman);
    zman_test!(test_sea_level_neitz, SeaLevelNeitzZman);
    zman_test!(test_sea_level_shkia, SeaLevelShkiaZman);

    zman_test!(test_alos_minutes_120, AlosZman::Minutes120);
    zman_test!(test_alos_minutes_120_zmanis, AlosZman::Minutes120Zmanis);
    zman_test!(test_alos_degrees_16_point_1, AlosZman::Degrees16Point1);
    zman_test!(test_alos_degrees_18, AlosZman::Degrees18);
    zman_test!(test_alos_degrees_19, AlosZman::Degrees19);
    zman_test!(test_alos_degrees_19_point_8, AlosZman::Degrees19Point8);
    zman_test!(test_alos_degrees_26, AlosZman::Degrees26, Some(40.0));
    zman_test!(test_alos_minutes_60, AlosZman::Minutes60);
    zman_test!(test_alos_minutes_72, AlosZman::Minutes72);
    zman_test!(test_alos_minutes_72_zmanis, AlosZman::Minutes72Zmanis);
    zman_test!(test_alos_minutes_90, AlosZman::Minutes90);
    zman_test!(test_alos_minutes_90_zmanis, AlosZman::Minutes90Zmanis);
    zman_test!(test_alos_minutes_96, AlosZman::Minutes96);
    zman_test!(test_alos_minutes_96_zmanis, AlosZman::Minutes96Zmanis);
    zman_test!(test_alos_baal_hatanya, AlosZman::BaalHatanya);

    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_13_point_24_degrees,
        BainHashmashosZman::RabbeinuTam13Point24Degrees
    );
    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_13_point_5_minutes_before_7_point_083_degrees,
        BainHashmashosZman::RabbeinuTam13Point5MinutesBefore7Point083Degrees
    );
    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_2_stars,
        BainHashmashosZman::RabbeinuTam2Stars
    );
    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_58_point_5_minutes,
        BainHashmashosZman::RabbeinuTam58Point5Minutes
    );
    zman_test!(
        test_bain_hashmashos_yereim_13_point_5_minutes,
        BainHashmashosZman::Yereim13Point5Minutes
    );
    zman_test!(
        test_bain_hashmashos_yereim_16_point_875_minutes,
        BainHashmashosZman::Yereim16Point875Minutes
    );
    zman_test!(
        test_bain_hashmashos_yereim_18_minutes,
        BainHashmashosZman::Yereim18Minutes
    );
    // zman_test!(
    //     test_bain_hashmashos_yereim_2_point_1_degrees,
    //     BainHashmashosZman::Yereim2Point1Degrees
    // );
    // zman_test!(
    //     test_bain_hashmashos_yereim_2_point_8_degrees,
    //     BainHashmashosZman::Yereim2Point8Degrees
    // );
    // zman_test!(
    //     test_bain_hashmashos_yereim_3_point_05_degrees,
    //     BainHashmashosZman::Yereim3Point05Degrees
    // );

    zman_test!(test_candle_lighting, CandleLightingZman);

    zman_test!(test_chatzos_astronomical, ChatzosZman::Astronomical);
    zman_test!(test_chatzos_half_day, ChatzosZman::HalfDay);
    zman_test!(test_chatzos_fixed_local, ChatzosZman::FixedLocal);

    zman_test!(
        test_mincha_gedola_sunrise_sunset,
        MinchaGedolaZman::SunriseSunset
    );
    zman_test!(
        test_mincha_gedola_degrees_16_point_1,
        MinchaGedolaZman::Degrees16Point1
    );
    zman_test!(test_mincha_gedola_minutes_30, MinchaGedolaZman::Minutes30);
    zman_test!(test_mincha_gedola_minutes_72, MinchaGedolaZman::Minutes72);
    zman_test!(
        test_mincha_gedola_ahavat_shalom,
        MinchaGedolaZman::AhavatShalom
    );
    zman_test!(
        test_mincha_gedola_ateret_torah,
        MinchaGedolaZman::AteretTorah
    );
    zman_test!(
        test_mincha_gedola_baal_hatanya,
        MinchaGedolaZman::BaalHatanya
    );
    zman_test!(
        test_mincha_gedola_baal_hatanya_greater_than_30,
        MinchaGedolaZman::BaalHatanyaGreaterThan30
    );
    zman_test!(
        test_mincha_gedola_gra_fixed_local_chatzos_30_minutes,
        MinchaGedolaZman::GRAFixedLocalChatzos30Minutes
    );
    zman_test!(
        test_mincha_gedola_greater_than_30,
        MinchaGedolaZman::GreaterThan30
    );

    zman_test!(
        test_mincha_ketana_sunrise_sunset,
        MinchaKetanaZman::SunriseSunset
    );
    zman_test!(
        test_mincha_ketana_degrees_16_point_1,
        MinchaKetanaZman::Degrees16Point1
    );
    zman_test!(test_mincha_ketana_minutes_72, MinchaKetanaZman::Minutes72);
    zman_test!(
        test_mincha_ketana_ahavat_shalom,
        MinchaKetanaZman::AhavatShalom
    );
    zman_test!(
        test_mincha_ketana_ateret_torah,
        MinchaKetanaZman::AteretTorah
    );
    zman_test!(
        test_mincha_ketana_baal_hatanya,
        MinchaKetanaZman::BaalHatanya
    );
    zman_test!(
        test_mincha_ketana_gra_fixed_local_chatzos_to_sunset,
        MinchaKetanaZman::GRAFixedLocalChatzosToSunset
    );

    zman_test!(
        test_misheyakir_degrees_10_point_2,
        MisheyakirZman::Degrees10Point2
    );
    zman_test!(test_misheyakir_degrees_11, MisheyakirZman::Degrees11);
    zman_test!(
        test_misheyakir_degrees_11_point_5,
        MisheyakirZman::Degrees11Point5
    );
    zman_test!(
        test_misheyakir_degrees_7_point_65,
        MisheyakirZman::Degrees7Point65
    );
    zman_test!(
        test_misheyakir_degrees_9_point_5,
        MisheyakirZman::Degrees9Point5
    );

    zman_test!(
        test_plag_hamincha_ahavat_shalom,
        PlagHaminchaZman::AhavatShalom
    );
    zman_test!(
        test_plag_hamincha_degrees_16_point_1_to_tzais_geonim_7_point_083,
        PlagHaminchaZman::Degrees16Point1ToTzaisGeonim7Point083
    );
    zman_test!(
        test_plag_hamincha_alos_to_sunset,
        PlagHaminchaZman::AlosToSunset
    );
    zman_test!(
        test_plag_hamincha_sunrise_sunset,
        PlagHaminchaZman::SunriseSunset
    );
    zman_test!(test_plag_hamincha_minutes_120, PlagHaminchaZman::Minutes120);
    zman_test!(
        test_plag_hamincha_minutes_120_zmanis,
        PlagHaminchaZman::Minutes120Zmanis
    );
    zman_test!(
        test_plag_hamincha_degrees_16_point_1,
        PlagHaminchaZman::Degrees16Point1
    );
    zman_test!(test_plag_hamincha_degrees_18, PlagHaminchaZman::Degrees18);
    zman_test!(
        test_plag_hamincha_degrees_19_point_8,
        PlagHaminchaZman::Degrees19Point8
    );
    zman_test!(
        test_plag_hamincha_degrees_26,
        PlagHaminchaZman::Degrees26,
        Some(40.0)
    );
    zman_test!(test_plag_hamincha_minutes_60, PlagHaminchaZman::Minutes60);
    zman_test!(test_plag_hamincha_minutes_72, PlagHaminchaZman::Minutes72);
    zman_test!(
        test_plag_hamincha_minutes_72_zmanis,
        PlagHaminchaZman::Minutes72Zmanis
    );
    zman_test!(test_plag_hamincha_minutes_90, PlagHaminchaZman::Minutes90);
    zman_test!(
        test_plag_hamincha_minutes_90_zmanis,
        PlagHaminchaZman::Minutes90Zmanis
    );
    zman_test!(test_plag_hamincha_minutes_96, PlagHaminchaZman::Minutes96);
    zman_test!(
        test_plag_hamincha_minutes_96_zmanis,
        PlagHaminchaZman::Minutes96Zmanis
    );
    zman_test!(
        test_plag_hamincha_ateret_torah,
        PlagHaminchaZman::AteretTorah
    );
    zman_test!(
        test_plag_hamincha_baal_hatanya,
        PlagHaminchaZman::BaalHatanya
    );
    zman_test!(
        test_plag_hamincha_gra_fixed_local_chatzos_to_sunset,
        PlagHaminchaZman::GRAFixedLocalChatzosToSunset
    );

    zman_test!(
        test_samuch_le_mincha_ketana_degrees_16_point_1,
        SamuchLeMinchaKetanaZman::Degrees16Point1
    );
    zman_test!(
        test_samuch_le_mincha_ketana_minutes_72,
        SamuchLeMinchaKetanaZman::Minutes72
    );
    zman_test!(
        test_samuch_le_mincha_ketana_gra,
        SamuchLeMinchaKetanaZman::GRA
    );

    zman_test!(
        test_sof_zman_achilas_chametz_baal_hatanya,
        SofZmanAchilasChametzZman::BaalHatanya
    );
    zman_test!(
        test_sof_zman_achilas_chametz_gra,
        SofZmanAchilasChametzZman::GRA
    );
    zman_test!(
        test_sof_zman_achilas_chametz_mga_16_point_1_degrees,
        SofZmanAchilasChametzZman::MGA16Point1Degrees
    );
    zman_test!(
        test_sof_zman_achilas_chametz_mga_72_minutes,
        SofZmanAchilasChametzZman::MGA72Minutes
    );

    zman_test!(
        test_sof_zman_biur_chametz_baal_hatanya,
        SofZmanBiurChametzZman::BaalHatanya
    );
    zman_test!(test_sof_zman_biur_chametz_gra, SofZmanBiurChametzZman::GRA);
    zman_test!(
        test_sof_zman_biur_chametz_mga_16_point_1_degrees,
        SofZmanBiurChametzZman::MGA16Point1Degrees
    );
    zman_test!(
        test_sof_zman_biur_chametz_mga_72_minutes,
        SofZmanBiurChametzZman::MGA72Minutes
    );

    zman_test!(
        test_sof_zman_shma_hours_3_before_chatzos,
        SofZmanShmaZman::Hours3BeforeChatzos
    );
    zman_test!(
        test_sof_zman_shma_alos_16_point_1_to_sunset,
        SofZmanShmaZman::Alos16Point1ToSunset
    );
    zman_test!(
        test_sof_zman_shma_alos_16_point_1_to_tzais_geonim_7_point_083_degrees,
        SofZmanShmaZman::Alos16Point1ToTzaisGeonim7Point083Degrees
    );
    zman_test!(
        test_sof_zman_shma_ateret_torah,
        SofZmanShmaZman::AteretTorah
    );
    zman_test!(
        test_sof_zman_shma_baal_hatanya,
        SofZmanShmaZman::BaalHatanya
    );
    zman_test!(test_sof_zman_shma_fixed_local, SofZmanShmaZman::FixedLocal);
    zman_test!(test_sof_zman_shma_gra, SofZmanShmaZman::GRA);
    zman_test!(
        test_sof_zman_shma_gra_sunrise_to_fixed_local_chatzos,
        SofZmanShmaZman::GRASunriseToFixedLocalChatzos
    );
    zman_test!(test_sof_zman_shma_kol_eliyahu, SofZmanShmaZman::KolEliyahu);
    zman_test!(test_sof_zman_shma_mga, SofZmanShmaZman::MGA);
    zman_test!(
        test_sof_zman_shma_mga_120_minutes,
        SofZmanShmaZman::MGA120Minutes
    );
    zman_test!(
        test_sof_zman_shma_mga_16_point_1_degrees,
        SofZmanShmaZman::MGA16Point1Degrees
    );
    zman_test!(
        test_sof_zman_shma_mga_16_point_1_degrees_to_fixed_local_chatzos,
        SofZmanShmaZman::MGA16Point1DegreesToFixedLocalChatzos
    );
    zman_test!(
        test_sof_zman_shma_mga_18_degrees,
        SofZmanShmaZman::MGA18Degrees
    );
    zman_test!(
        test_sof_zman_shma_mga_18_degrees_to_fixed_local_chatzos,
        SofZmanShmaZman::MGA18DegreesToFixedLocalChatzos
    );
    zman_test!(
        test_sof_zman_shma_mga_19_point_8_degrees,
        SofZmanShmaZman::MGA19Point8Degrees
    );
    zman_test!(
        test_sof_zman_shma_mga_72_minutes,
        SofZmanShmaZman::MGA72Minutes
    );
    zman_test!(
        test_sof_zman_shma_mga_72_minutes_to_fixed_local_chatzos,
        SofZmanShmaZman::MGA72MinutesToFixedLocalChatzos
    );
    zman_test!(
        test_sof_zman_shma_mga_72_minutes_zmanis,
        SofZmanShmaZman::MGA72MinutesZmanis
    );
    zman_test!(
        test_sof_zman_shma_mga_90_minutes,
        SofZmanShmaZman::MGA90Minutes
    );
    zman_test!(
        test_sof_zman_shma_mga_90_minutes_to_fixed_local_chatzos,
        SofZmanShmaZman::MGA90MinutesToFixedLocalChatzos
    );
    zman_test!(
        test_sof_zman_shma_mga_90_minutes_zmanis,
        SofZmanShmaZman::MGA90MinutesZmanis
    );
    zman_test!(
        test_sof_zman_shma_mga_96_minutes,
        SofZmanShmaZman::MGA96Minutes
    );
    zman_test!(
        test_sof_zman_shma_mga_96_minutes_zmanis,
        SofZmanShmaZman::MGA96MinutesZmanis
    );

    zman_test!(
        test_sof_zman_tfila_hours_2_before_chatzos,
        SofZmanTfilaZman::Hours2BeforeChatzos
    );
    zman_test!(
        test_sof_zman_tfila_ateret_torah,
        SofZmanTfilaZman::AteretTorah
    );
    zman_test!(
        test_sof_zman_tfila_baal_hatanya,
        SofZmanTfilaZman::BaalHatanya
    );
    zman_test!(
        test_sof_zman_tfila_fixed_local,
        SofZmanTfilaZman::FixedLocal
    );
    zman_test!(test_sof_zman_tfila_gra, SofZmanTfilaZman::GRA);
    zman_test!(
        test_sof_zman_tfila_gra_sunrise_to_fixed_local_chatzos,
        SofZmanTfilaZman::GRASunriseToFixedLocalChatzos
    );
    zman_test!(test_sof_zman_tfila_mga, SofZmanTfilaZman::MGA);
    zman_test!(
        test_sof_zman_tfila_mga_120_minutes,
        SofZmanTfilaZman::MGA120Minutes
    );
    zman_test!(
        test_sof_zman_tfila_mga_16_point_1_degrees,
        SofZmanTfilaZman::MGA16Point1Degrees
    );
    zman_test!(
        test_sof_zman_tfila_mga_18_degrees,
        SofZmanTfilaZman::MGA18Degrees
    );
    zman_test!(
        test_sof_zman_tfila_mga_19_point_8_degrees,
        SofZmanTfilaZman::MGA19Point8Degrees
    );
    zman_test!(
        test_sof_zman_tfila_mga_72_minutes,
        SofZmanTfilaZman::MGA72Minutes
    );
    zman_test!(
        test_sof_zman_tfila_mga_72_minutes_zmanis,
        SofZmanTfilaZman::MGA72MinutesZmanis
    );
    zman_test!(
        test_sof_zman_tfila_mga_90_minutes,
        SofZmanTfilaZman::MGA90Minutes
    );
    zman_test!(
        test_sof_zman_tfila_mga_90_minutes_zmanis,
        SofZmanTfilaZman::MGA90MinutesZmanis
    );
    zman_test!(
        test_sof_zman_tfila_mga_96_minutes,
        SofZmanTfilaZman::MGA96Minutes
    );
    zman_test!(
        test_sof_zman_tfila_mga_96_minutes_zmanis,
        SofZmanTfilaZman::MGA96MinutesZmanis
    );

    zman_test!(test_tzais_degrees_8_point_5, TzaisZman::Degrees8Point5);
    zman_test!(test_tzais_minutes_120, TzaisZman::Minutes120);
    zman_test!(test_tzais_minutes_120_zmanis, TzaisZman::Minutes120Zmanis);
    zman_test!(test_tzais_degrees_16_point_1, TzaisZman::Degrees16Point1);
    zman_test!(test_tzais_degrees_18, TzaisZman::Degrees18);
    zman_test!(test_tzais_degrees_19_point_8, TzaisZman::Degrees19Point8);
    zman_test!(test_tzais_degrees_26, TzaisZman::Degrees26, Some(40.0));
    zman_test!(test_tzais_minutes_50, TzaisZman::Minutes50);
    zman_test!(test_tzais_minutes_60, TzaisZman::Minutes60);
    zman_test!(test_tzais_minutes_72, TzaisZman::Minutes72);
    zman_test!(test_tzais_minutes_72_zmanis, TzaisZman::Minutes72Zmanis);
    zman_test!(test_tzais_minutes_90, TzaisZman::Minutes90);
    zman_test!(test_tzais_minutes_90_zmanis, TzaisZman::Minutes90Zmanis);
    zman_test!(test_tzais_minutes_96, TzaisZman::Minutes96);
    zman_test!(test_tzais_minutes_96_zmanis, TzaisZman::Minutes96Zmanis);
    zman_test!(test_tzais_ateret_torah, TzaisZman::AteretTorah);
    zman_test!(test_tzais_baal_hatanya, TzaisZman::BaalHatanya);
    zman_test!(test_tzais_geonim_3_point_65, TzaisZman::Geonim3Point65);
    zman_test!(test_tzais_geonim_3_point_676, TzaisZman::Geonim3Point676);
    zman_test!(
        test_tzais_geonim_degrees_3_point_7,
        TzaisZman::GeonimDegrees3Point7
    );
    zman_test!(
        test_tzais_geonim_degrees_3_point_8,
        TzaisZman::GeonimDegrees3Point8
    );
    zman_test!(
        test_tzais_geonim_degrees_4_point_37,
        TzaisZman::GeonimDegrees4Point37
    );
    zman_test!(
        test_tzais_geonim_degrees_4_point_61,
        TzaisZman::GeonimDegrees4Point61
    );
    zman_test!(
        test_tzais_geonim_degrees_4_point_8,
        TzaisZman::GeonimDegrees4Point8
    );
    zman_test!(
        test_tzais_geonim_degrees_5_point_88,
        TzaisZman::GeonimDegrees5Point88
    );
    zman_test!(
        test_tzais_geonim_degrees_5_point_95,
        TzaisZman::GeonimDegrees5Point95
    );
    zman_test!(
        test_tzais_geonim_degrees_6_point_45,
        TzaisZman::GeonimDegrees6Point45
    );
    zman_test!(
        test_tzais_geonim_degrees_7_point_083,
        TzaisZman::GeonimDegrees7Point083
    );
    zman_test!(
        test_tzais_geonim_degrees_7_point_67,
        TzaisZman::GeonimDegrees7Point67
    );
    zman_test!(
        test_tzais_geonim_degrees_8_point_5,
        TzaisZman::GeonimDegrees8Point5
    );
    zman_test!(
        test_tzais_geonim_degrees_9_point_3,
        TzaisZman::GeonimDegrees9Point3
    );
    zman_test!(
        test_tzais_geonim_degrees_9_point_75,
        TzaisZman::GeonimDegrees9Point75
    );

    #[test]
    fn regression_mincha_gedola_gra_fixed_local_chatzos_30_minutes() {
        test_zman_iteration(
            MinchaGedolaZman::GRAFixedLocalChatzos30Minutes,
            8218711474067301417,
            2485,
            None,
            None,
        );
    }

    #[test]
    fn regression_fixed_local_chatzos() {
        test_zman_iteration(
            ChatzosZman::FixedLocal,
            8218711474067301417,
            2485,
            None,
            None,
        );
    }

    #[test]
    fn regression_plag_hamincha_ateret_torah() {
        test_zman_iteration(
            PlagHaminchaZman::AteretTorah,
            18375159325404615489,
            1546,
            None,
            None,
        );
    }

    #[test]
    fn regression_sof_zman_shma_mga_90_minutes_to_fixed_local_chatzos() {
        test_zman_iteration(
            SofZmanShmaZman::MGA90MinutesToFixedLocalChatzos,
            3472850580173038015,
            8672,
            None,
            None,
        );
    }
}
