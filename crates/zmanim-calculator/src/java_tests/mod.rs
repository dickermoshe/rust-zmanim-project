#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(missing_docs)]

use chrono::DateTime;
use j4rs::{ClasspathEntry, Jvm, JvmBuilder};
mod java_bindings;
mod java_rnd;
extern crate std;

use std::env;
use std::sync::Once;

use crate::prelude::*;

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
    use crate::presets::*;

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
    fn test_zman_vs_java(
        zman: &'static dyn ZmanPresetLike<chrono_tz::Tz>,
        seed: u64,
        max_diff_override: Option<i64>,
        max_lat: Option<f64>,
        always_with_tz: bool,
    ) {
        std::println!(
            "Testing {:?} with seed: {} (set TEST_SEED={} to reproduce)",
            zman.name(),
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

            // Randomly remove the timezone if able to do so
            if !Location::<chrono_tz::Tz>::near_anti_meridian(rust_calculator.location.longitude)
                && rng.gen_bool(0.5)
                && !always_with_tz
            {
                rust_calculator.location.timezone = None;
            }

            let rust_result = zman.calculate(&mut rust_calculator);
            let java_result = java_calendar.get_zman(zman);

            // Convert from Utc to the local timezone for comparison
            let rust_result_tz = rust_result
                .map(|dt| tz.from_utc_datetime(&dt.naive_utc()))
                .ok();

            let mut max_diff = max_diff_override.unwrap_or(MAX_DIFF_SECONDS);

            // If this zman uses elevation, enlarge the max diff based on the elevation
            if zman.uses_elevation(&rust_calculator) && rust_calculator.location.elevation > 100.0 {
                max_diff += (rust_calculator.location.elevation * 0.1) as i64;
                assert!(
                    max_diff > 0 && max_diff < 100,
                    "Max diff is out of range for {:?}: {} meters",
                    zman.name(),
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
                    &zman.name(),
                    &rust_calculator.date,
                    &rust_calculator.location.latitude,
                    &rust_calculator.location.longitude,
                    &rust_calculator.location.elevation,
                    &rust_calculator.location.timezone.map(|tz| tz.name()),
                ),
            );
            // for fixed chatzos, make sure that the date is the same at the naive data
            if let Some(java_result) = java_result {
                if zman.name() == "getFixedLocalChatzos" {
                    assert_eq!(rust_calculator.date, java_result.naive_local().date());
                }
            }
        }
    }

    fn test_zman_iteration(
        zman: &'static dyn ZmanPresetLike<chrono_tz::Tz>,
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
        let rust_result = zman.calculate(&mut rust_calculator);
        let java_result = java_calendar.get_zman(zman);
        let rust_result_tz = rust_result
            .map(|dt| tz.from_utc_datetime(&dt.naive_utc()))
            .ok();

        let mut max_diff = max_diff_override.unwrap_or(MAX_DIFF_SECONDS);
        if zman.uses_elevation(&rust_calculator) {
            max_diff += (rust_calculator.location.elevation * 0.1) as i64;
            assert!(
                max_diff > 0 && max_diff < 100,
                "Max diff is out of range for {:?}: {} meters",
                zman.name(),
                rust_calculator.location.elevation
            );
        }

        assert_times_close(
            rust_result_tz,
            java_result,
            max_diff,
            &std::format!(
                "Regression {:?} seed {} iteration {}: {:?}, Location: ({}, {}), Elevation: {}, Timezone: {:?}",
                &zman.name(),
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
                test_zman_vs_java(&$zman, get_test_seed(), None, None, false);
            }
        };
        ($name:ident, $zman:expr, $max_lat:expr) => {
            #[test]
            fn $name() {
                test_zman_vs_java(&$zman, get_test_seed(), None, $max_lat, false);
            }
        };
        ($name:ident, $zman:expr, $max_lat:expr, $always_with_tz:expr) => {
            #[test]
            fn $name() {
                test_zman_vs_java(&$zman, get_test_seed(), None, $max_lat, $always_with_tz);
            }
        };
    }

    zman_test!(test_neitz, SUNRISE);
    zman_test!(test_shkia, SUNSET);
    zman_test!(test_sea_level_neitz, SEA_LEVEL_SUNRISE);
    zman_test!(test_sea_level_shkia, SEA_LEVEL_SUNSET);

    zman_test!(test_alos_minutes_120, ALOS_120_MINUTES);
    zman_test!(test_alos_minutes_120_zmanis, ALOS_120_ZMANIS);
    zman_test!(test_alos_degrees_16_point_1, ALOS_16_POINT_1_DEGREES);
    zman_test!(test_alos_degrees_18, ALOS_18_DEGREES);
    zman_test!(test_alos_degrees_19, ALOS_19_DEGREES);
    zman_test!(test_alos_degrees_19_point_8, ALOS_19_POINT_8_DEGREES);
    zman_test!(test_alos_degrees_26, ALOS_26_DEGREES, Some(40.0));
    zman_test!(test_alos_minutes_60, ALOS_60_MINUTES);
    zman_test!(test_alos_minutes_72, ALOS_72_MINUTES);
    zman_test!(test_alos_minutes_72_zmanis, ALOS_72_ZMANIS);
    zman_test!(test_alos_minutes_90, ALOS_90_MINUTES);
    zman_test!(test_alos_minutes_90_zmanis, ALOS_90_ZMANIS);
    zman_test!(test_alos_minutes_96, ALOS_96_MINUTES);
    zman_test!(test_alos_minutes_96_zmanis, ALOS_96_ZMANIS);
    zman_test!(test_alos_baal_hatanya, ALOS_BAAL_HATANYA);

    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_13_point_24_degrees,
        BAIN_HASHMASHOS_RT_13_POINT_24_DEGREES
    );
    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_13_point_5_minutes_before_7_point_083_degrees,
        BAIN_HASHMASHOS_RT_13_POINT_5_MINUTES_BEFORE_7_POINT_083_DEGREES
    );
    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_2_stars,
        BAIN_HASHMASHOS_RT_2_STARS
    );
    zman_test!(
        test_bain_hashmashos_rabbeinu_tam_58_point_5_minutes,
        BAIN_HASHMASHOS_RT_58_POINT_5_MINUTES
    );
    zman_test!(
        test_bain_hashmashos_yereim_13_point_5_minutes,
        BAIN_HASHMASHOS_YEREIM_13_POINT_5_MINUTES
    );
    zman_test!(
        test_bain_hashmashos_yereim_16_point_875_minutes,
        BAIN_HASHMASHOS_YEREIM_16_POINT_875_MINUTES
    );
    zman_test!(
        test_bain_hashmashos_yereim_18_minutes,
        BAIN_HASHMASHOS_YEREIM_18_MINUTES
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

    zman_test!(test_candle_lighting, CANDLE_LIGHTING);

    zman_test!(test_chatzos_astronomical, CHATZOS_ASTRONOMICAL);
    zman_test!(test_chatzos_half_day, CHATZOS_HALF_DAY);
    zman_test!(test_chatzos_fixed_local, CHATZOS_FIXED_LOCAL);

    zman_test!(
        test_mincha_gedola_sunrise_sunset,
        MINCHA_GEDOLA_SUNRISE_SUNSET
    );
    zman_test!(
        test_mincha_gedola_degrees_16_point_1,
        MINCHA_GEDOLA_16_POINT_1_DEGREES
    );
    zman_test!(test_mincha_gedola_minutes_30, MINCHA_GEDOLA_MINUTES_30);
    zman_test!(test_mincha_gedola_minutes_72, MINCHA_GEDOLA_MINUTES_72);
    zman_test!(
        test_mincha_gedola_ahavat_shalom,
        MINCHA_GEDOLA_AHAVAT_SHALOM
    );
    zman_test!(test_mincha_gedola_ateret_torah, MINCHA_GEDOLA_ATERET_TORAH);
    zman_test!(test_mincha_gedola_baal_hatanya, MINCHA_GEDOLA_BAAL_HATANYA);
    zman_test!(
        test_mincha_gedola_baal_hatanya_greater_than_30,
        MINCHA_GEDOLA_BAAL_HATANYA_GREATER_THAN_30
    );
    zman_test!(
        test_mincha_gedola_gra_fixed_local_chatzos_30_minutes,
        MINCHA_GEDOLA_GRA_FIXED_LOCAL_CHATZOS_30_MINUTES
    );
    zman_test!(
        test_mincha_gedola_greater_than_30,
        MINCHA_GEDOLA_GREATER_THAN_30
    );

    zman_test!(
        test_mincha_ketana_sunrise_sunset,
        MINCHA_KETANA_SUNRISE_SUNSET
    );
    zman_test!(
        test_mincha_ketana_degrees_16_point_1,
        MINCHA_KETANA_16_POINT_1_DEGREES
    );
    zman_test!(test_mincha_ketana_minutes_72, MINCHA_KETANA_MINUTES_72);
    zman_test!(
        test_mincha_ketana_ahavat_shalom,
        MINCHA_KETANA_AHAVAT_SHALOM
    );
    zman_test!(test_mincha_ketana_ateret_torah, MINCHA_KETANA_ATERET_TORAH);
    zman_test!(test_mincha_ketana_baal_hatanya, MINCHA_KETANA_BAAL_HATANYA);
    zman_test!(
        test_mincha_ketana_gra_fixed_local_chatzos_to_sunset,
        MINCHA_KETANA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET
    );

    zman_test!(
        test_misheyakir_degrees_10_point_2,
        MISHEYAKIR_10_POINT_2_DEGREES
    );
    zman_test!(test_misheyakir_degrees_11, MISHEYAKIR_11_DEGREES);
    zman_test!(
        test_misheyakir_degrees_11_point_5,
        MISHEYAKIR_11_POINT_5_DEGREES
    );
    zman_test!(
        test_misheyakir_degrees_7_point_65,
        MISHEYAKIR_7_POINT_65_DEGREES
    );
    zman_test!(
        test_misheyakir_degrees_9_point_5,
        MISHEYAKIR_9_POINT_5_DEGREES
    );

    zman_test!(
        test_plag_hamincha_ahavat_shalom,
        PLAG_HAMINCHA_AHAVAT_SHALOM
    );
    zman_test!(
        test_plag_hamincha_degrees_16_point_1_to_tzais_geonim_7_point_083,
        PLAG_HAMINCHA_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083
    );
    zman_test!(
        test_plag_hamincha_alos_to_sunset,
        PLAG_HAMINCHA_ALOS_TO_SUNSET
    );
    zman_test!(
        test_plag_hamincha_sunrise_sunset,
        PLAG_HAMINCHA_SUNRISE_SUNSET
    );
    zman_test!(test_plag_hamincha_minutes_120, PLAG_HAMINCHA_120_MINUTES);
    zman_test!(
        test_plag_hamincha_minutes_120_zmanis,
        PLAG_HAMINCHA_120_ZMANIS
    );
    zman_test!(
        test_plag_hamincha_degrees_16_point_1,
        PLAG_HAMINCHA_16_POINT_1_DEGREES
    );
    zman_test!(test_plag_hamincha_degrees_18, PLAG_HAMINCHA_18_DEGREES);
    zman_test!(
        test_plag_hamincha_degrees_19_point_8,
        PLAG_HAMINCHA_19_POINT_8_DEGREES
    );
    zman_test!(
        test_plag_hamincha_degrees_26,
        PLAG_HAMINCHA_26_DEGREES,
        Some(40.0)
    );
    zman_test!(test_plag_hamincha_minutes_60, PLAG_HAMINCHA_60_MINUTES);
    zman_test!(test_plag_hamincha_minutes_72, PLAG_HAMINCHA_72_MINUTES);
    zman_test!(
        test_plag_hamincha_minutes_72_zmanis,
        PLAG_HAMINCHA_72_ZMANIS
    );
    zman_test!(test_plag_hamincha_minutes_90, PLAG_HAMINCHA_90_MINUTES);
    zman_test!(
        test_plag_hamincha_minutes_90_zmanis,
        PLAG_HAMINCHA_90_ZMANIS
    );
    zman_test!(test_plag_hamincha_minutes_96, PLAG_HAMINCHA_96_MINUTES);
    zman_test!(
        test_plag_hamincha_minutes_96_zmanis,
        PLAG_HAMINCHA_96_ZMANIS
    );
    zman_test!(test_plag_hamincha_ateret_torah, PLAG_HAMINCHA_ATERET_TORAH);
    zman_test!(test_plag_hamincha_baal_hatanya, PLAG_HAMINCHA_BAAL_HATANYA);
    zman_test!(
        test_plag_hamincha_gra_fixed_local_chatzos_to_sunset,
        PLAG_HAMINCHA_GRA_FIXED_LOCAL_CHATZOS_TO_SUNSET
    );

    zman_test!(
        test_samuch_le_mincha_ketana_degrees_16_point_1,
        SAMUCH_LE_MINCHA_KETANA_16_POINT_1_DEGREES
    );
    zman_test!(
        test_samuch_le_mincha_ketana_minutes_72,
        SAMUCH_LE_MINCHA_KETANA_72_MINUTES
    );
    zman_test!(
        test_samuch_le_mincha_ketana_gra,
        SAMUCH_LE_MINCHA_KETANA_GRA
    );

    zman_test!(
        test_sof_zman_achilas_chametz_baal_hatanya,
        SOF_ZMAN_ACHILAS_CHAMETZ_BAAL_HATANYA
    );
    zman_test!(
        test_sof_zman_achilas_chametz_gra,
        SOF_ZMAN_ACHILAS_CHAMETZ_GRA
    );
    zman_test!(
        test_sof_zman_achilas_chametz_mga_16_point_1_degrees,
        SOF_ZMAN_ACHILAS_CHAMETZ_MGA_16_POINT_1_DEGREES
    );
    zman_test!(
        test_sof_zman_achilas_chametz_mga_72_minutes,
        SOF_ZMAN_ACHILAS_CHAMETZ_MGA_72_MINUTES
    );

    zman_test!(
        test_sof_zman_biur_chametz_baal_hatanya,
        SOF_ZMAN_BIUR_CHAMETZ_BAAL_HATANYA
    );
    zman_test!(test_sof_zman_biur_chametz_gra, SOF_ZMAN_BIUR_CHAMETZ_GRA);
    zman_test!(
        test_sof_zman_biur_chametz_mga_16_point_1_degrees,
        SOF_ZMAN_BIUR_CHAMETZ_MGA_16_POINT_1_DEGREES
    );
    zman_test!(
        test_sof_zman_biur_chametz_mga_72_minutes,
        SOF_ZMAN_BIUR_CHAMETZ_MGA_72_MINUTES
    );

    zman_test!(
        test_sof_zman_shma_hours_3_before_chatzos,
        SOF_ZMAN_SHMA_HOURS_3_BEFORE_CHATZOS
    );
    zman_test!(
        test_sof_zman_shma_alos_16_point_1_to_sunset,
        SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_SUNSET
    );
    zman_test!(
        test_sof_zman_shma_alos_16_point_1_to_tzais_geonim_7_point_083_degrees,
        SOF_ZMAN_SHMA_ALOS_16_POINT_1_TO_TZAIS_GEONIM_7_POINT_083
    );
    zman_test!(test_sof_zman_shma_ateret_torah, SOF_ZMAN_SHMA_ATERET_TORAH);
    zman_test!(test_sof_zman_shma_baal_hatanya, SOF_ZMAN_SHMA_BAAL_HATANYA);
    zman_test!(test_sof_zman_shma_fixed_local, SOF_ZMAN_SHMA_FIXED_LOCAL);
    zman_test!(test_sof_zman_shma_gra, SOF_ZMAN_SHMA_GRA);
    zman_test!(
        test_sof_zman_shma_gra_sunrise_to_fixed_local_chatzos,
        SOF_ZMAN_SHMA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS
    );
    zman_test!(test_sof_zman_shma_kol_eliyahu, SOF_ZMAN_SHMA_KOL_ELIYAHU);
    zman_test!(test_sof_zman_shma_mga, SOF_ZMAN_SHMA_MGA);
    zman_test!(
        test_sof_zman_shma_mga_120_minutes,
        SOF_ZMAN_SHMA_MGA_120_MINUTES
    );
    zman_test!(
        test_sof_zman_shma_mga_16_point_1_degrees,
        SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES
    );
    zman_test!(
        test_sof_zman_shma_mga_16_point_1_degrees_to_fixed_local_chatzos,
        SOF_ZMAN_SHMA_MGA_16_POINT_1_DEGREES_TO_FIXED_LOCAL_CHATZOS
    );
    zman_test!(
        test_sof_zman_shma_mga_18_degrees,
        SOF_ZMAN_SHMA_MGA_18_DEGREES
    );
    zman_test!(
        test_sof_zman_shma_mga_18_degrees_to_fixed_local_chatzos,
        SOF_ZMAN_SHMA_MGA_18_DEGREES_TO_FIXED_LOCAL_CHATZOS
    );
    zman_test!(
        test_sof_zman_shma_mga_19_point_8_degrees,
        SOF_ZMAN_SHMA_MGA_19_POINT_8_DEGREES
    );
    zman_test!(
        test_sof_zman_shma_mga_72_minutes,
        SOF_ZMAN_SHMA_MGA_72_MINUTES
    );
    zman_test!(
        test_sof_zman_shma_mga_72_minutes_to_fixed_local_chatzos,
        SOF_ZMAN_SHMA_MGA_72_MINUTES_TO_FIXED_LOCAL_CHATZOS
    );
    zman_test!(
        test_sof_zman_shma_mga_72_minutes_zmanis,
        SOF_ZMAN_SHMA_MGA_72_ZMANIS
    );
    zman_test!(
        test_sof_zman_shma_mga_90_minutes,
        SOF_ZMAN_SHMA_MGA_90_MINUTES
    );
    zman_test!(
        test_sof_zman_shma_mga_90_minutes_to_fixed_local_chatzos,
        SOF_ZMAN_SHMA_MGA_90_MINUTES_TO_FIXED_LOCAL_CHATZOS
    );
    zman_test!(
        test_sof_zman_shma_mga_90_minutes_zmanis,
        SOF_ZMAN_SHMA_MGA_90_ZMANIS
    );
    zman_test!(
        test_sof_zman_shma_mga_96_minutes,
        SOF_ZMAN_SHMA_MGA_96_MINUTES
    );
    zman_test!(
        test_sof_zman_shma_mga_96_minutes_zmanis,
        SOF_ZMAN_SHMA_MGA_96_ZMANIS
    );

    zman_test!(
        test_sof_zman_tfila_hours_2_before_chatzos,
        SOF_ZMAN_TFILA_HOURS_2_BEFORE_CHATZOS
    );
    zman_test!(
        test_sof_zman_tfila_ateret_torah,
        SOF_ZMAN_TFILA_ATERET_TORAH
    );
    zman_test!(
        test_sof_zman_tfila_baal_hatanya,
        SOF_ZMAN_TFILA_BAAL_HATANYA
    );
    zman_test!(test_sof_zman_tfila_fixed_local, SOF_ZMAN_TFILA_FIXED_LOCAL);
    zman_test!(test_sof_zman_tfila_gra, SOF_ZMAN_TFILA_GRA);
    zman_test!(
        test_sof_zman_tfila_gra_sunrise_to_fixed_local_chatzos,
        SOF_ZMAN_TFILA_GRA_SUNRISE_TO_FIXED_LOCAL_CHATZOS
    );
    zman_test!(test_sof_zman_tfila_mga, SOF_ZMAN_TFILA_MGA);
    zman_test!(
        test_sof_zman_tfila_mga_120_minutes,
        SOF_ZMAN_TFILA_MGA_120_MINUTES
    );
    zman_test!(
        test_sof_zman_tfila_mga_16_point_1_degrees,
        SOF_ZMAN_TFILA_MGA_16_POINT_1_DEGREES
    );
    zman_test!(
        test_sof_zman_tfila_mga_18_degrees,
        SOF_ZMAN_TFILA_MGA_18_DEGREES
    );
    zman_test!(
        test_sof_zman_tfila_mga_19_point_8_degrees,
        SOF_ZMAN_TFILA_MGA_19_POINT_8_DEGREES
    );
    zman_test!(
        test_sof_zman_tfila_mga_72_minutes,
        SOF_ZMAN_TFILA_MGA_72_MINUTES
    );
    zman_test!(
        test_sof_zman_tfila_mga_72_minutes_zmanis,
        SOF_ZMAN_TFILA_MGA_72_ZMANIS
    );
    zman_test!(
        test_sof_zman_tfila_mga_90_minutes,
        SOF_ZMAN_TFILA_MGA_90_MINUTES
    );
    zman_test!(
        test_sof_zman_tfila_mga_90_minutes_zmanis,
        SOF_ZMAN_TFILA_MGA_90_ZMANIS
    );
    zman_test!(
        test_sof_zman_tfila_mga_96_minutes,
        SOF_ZMAN_TFILA_MGA_96_MINUTES
    );
    zman_test!(
        test_sof_zman_tfila_mga_96_minutes_zmanis,
        SOF_ZMAN_TFILA_MGA_96_ZMANIS
    );

    zman_test!(test_tzais_degrees_8_point_5, TZAIS_DEGREES_8_POINT_5);
    zman_test!(test_tzais_minutes_120, TZAIS_MINUTES_120);
    zman_test!(test_tzais_minutes_120_zmanis, TZAIS_120_ZMANIS);
    zman_test!(test_tzais_degrees_16_point_1, TZAIS_16_POINT_1_DEGREES);
    zman_test!(test_tzais_degrees_18, TZAIS_18_DEGREES);
    zman_test!(test_tzais_degrees_19_point_8, TZAIS_19_POINT_8_DEGREES);
    zman_test!(test_tzais_degrees_26, TZAIS_26_DEGREES, Some(40.0));
    zman_test!(test_tzais_minutes_50, TZAIS_MINUTES_50);
    zman_test!(test_tzais_minutes_60, TZAIS_MINUTES_60);
    zman_test!(test_tzais_minutes_72, TZAIS_MINUTES_72);
    zman_test!(test_tzais_minutes_72_zmanis, TZAIS_72_ZMANIS);
    zman_test!(test_tzais_minutes_90, TZAIS_MINUTES_90);
    zman_test!(test_tzais_minutes_90_zmanis, TZAIS_90_ZMANIS);
    zman_test!(test_tzais_minutes_96, TZAIS_MINUTES_96);
    zman_test!(test_tzais_minutes_96_zmanis, TZAIS_96_ZMANIS);
    zman_test!(test_tzais_ateret_torah, TZAIS_ATERET_TORAH);
    zman_test!(test_tzais_baal_hatanya, TZAIS_BAAL_HATANYA);
    zman_test!(test_tzais_geonim_3_point_65, TZAIS_GEONIM_3_POINT_65);
    zman_test!(test_tzais_geonim_3_point_676, TZAIS_GEONIM_3_POINT_676);
    zman_test!(
        test_tzais_geonim_degrees_3_point_7,
        TZAIS_GEONIM_DEGREES_3_POINT_7
    );
    zman_test!(
        test_tzais_geonim_degrees_3_point_8,
        TZAIS_GEONIM_DEGREES_3_POINT_8
    );
    zman_test!(
        test_tzais_geonim_degrees_4_point_37,
        TZAIS_GEONIM_DEGREES_4_POINT_37
    );
    zman_test!(
        test_tzais_geonim_degrees_4_point_61,
        TZAIS_GEONIM_DEGREES_4_POINT_61
    );
    zman_test!(
        test_tzais_geonim_degrees_4_point_8,
        TZAIS_GEONIM_DEGREES_4_POINT_8
    );
    zman_test!(
        test_tzais_geonim_degrees_5_point_88,
        TZAIS_GEONIM_DEGREES_5_POINT_88
    );
    zman_test!(
        test_tzais_geonim_degrees_5_point_95,
        TZAIS_GEONIM_DEGREES_5_POINT_95
    );
    zman_test!(
        test_tzais_geonim_degrees_6_point_45,
        TZAIS_GEONIM_DEGREES_6_POINT_45
    );
    zman_test!(
        test_tzais_geonim_degrees_7_point_083,
        TZAIS_GEONIM_DEGREES_7_POINT_083
    );
    zman_test!(
        test_tzais_geonim_degrees_7_point_67,
        TZAIS_GEONIM_DEGREES_7_POINT_67
    );
    zman_test!(
        test_tzais_geonim_degrees_8_point_5,
        TZAIS_GEONIM_DEGREES_8_POINT_5
    );
    zman_test!(
        test_tzais_geonim_degrees_9_point_3,
        TZAIS_GEONIM_DEGREES_9_POINT_3
    );
    zman_test!(
        test_tzais_geonim_degrees_9_point_75,
        TZAIS_GEONIM_DEGREES_9_POINT_75
    );

    // zman_test!(test_molad, MOLAD, None, true);
    // zman_test!(
    //     test_tchilas_zman_kidush_levana_3_days,
    //     TCHILAS_ZMAN_KIDUSH_LEVANA_3_DAYS,
    //     None,
    //     true
    // );
    // zman_test!(
    //     test_tchilas_zman_kidush_levana_7_days,
    //     TCHILAS_ZMAN_KIDUSH_LEVANA_7_DAYS,
    //     None,
    //     true
    // );
    // zman_test!(
    //     test_sof_zman_kidush_levana_between_moldos,
    //     SOF_ZMAN_KIDUSH_LEVANA_BETWEEN_MOLDOS,
    //     None,
    //     true
    // );
    // zman_test!(
    //     test_sof_zman_kidush_levana_15_days,
    //     SOF_ZMAN_KIDUSH_LEVANA_15_DAYS,
    //     None,
    //     true
    // );

    #[test]
    fn regression_mincha_gedola_gra_fixed_local_chatzos_30_minutes() {
        test_zman_iteration(
            &MINCHA_GEDOLA_GRA_FIXED_LOCAL_CHATZOS_30_MINUTES,
            8218711474067301417,
            2485,
            None,
            None,
        );
    }

    #[test]
    fn regression_fixed_local_chatzos() {
        test_zman_iteration(&CHATZOS_FIXED_LOCAL, 8218711474067301417, 2485, None, None);
    }

    #[test]
    fn regression_plag_hamincha_ateret_torah() {
        test_zman_iteration(
            &PLAG_HAMINCHA_ATERET_TORAH,
            18375159325404615489,
            1546,
            None,
            None,
        );
    }

    #[test]
    fn regression_sof_zman_shma_mga_90_minutes_to_fixed_local_chatzos() {
        test_zman_iteration(
            &SOF_ZMAN_SHMA_MGA_90_MINUTES_TO_FIXED_LOCAL_CHATZOS,
            3472850580173038015,
            8672,
            None,
            None,
        );
    }
}
