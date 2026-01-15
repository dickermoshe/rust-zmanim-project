#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use chrono::DateTime;
use j4rs::{ClasspathEntry, Jvm, JvmBuilder};
mod java_bindings;
mod java_rnd;
use std::env;
use std::sync::Once;

use crate::Zman;

/// Default number of iterations for randomized tests.
pub fn get_test_iterations() -> i64 {
    env::var("TEST_ITERATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
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

fn uses_elevation(zman: &Zman) -> bool {
    matches!(
        zman,
        Zman::Sunrise
            | Zman::Sunset
            | Zman::SofZmanShmaGRA
            | Zman::SofZmanShmaMGA
            | Zman::SofZmanTfilaGRA
            | Zman::SofZmanTfilaMGA
            | Zman::Alos72
            | Zman::Tzais72
            | Zman::MinchaKetana
            | Zman::MinchaGedola
            | Zman::PlagHamincha
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Location, Zman};
    use chrono::TimeZone;
    use rand::{Rng, SeedableRng};

    /// Maximum allowed difference in seconds between Rust and Java implementations
    const MAX_DIFF_SECONDS: i64 = 30;

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
            (None, None) => {
                // Both None is acceptable
            }
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
    fn test_zman_vs_java(zman: Zman, seed: u64, max_diff_override: Option<i64>) {
        println!(
            "Testing {:?} with seed: {} (set TEST_SEED={} to reproduce)",
            zman, seed, seed
        );
        let jvm = init_jvm();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let iterations = get_test_iterations();

        for i in 0..iterations {
            let Some((rust_calculator, location, date_time, java_calendar)) =
                java_rnd::random_zmanim_calendars(&jvm, &mut rng)
            else {
                continue;
            };

            // Randomly test with or without timezone in location
            let remove_tz = rng.gen_bool(0.5);
            let location = if remove_tz {
                Location::new(
                    location.latitude(),
                    location.longitude(),
                    location.elevation(),
                    None,
                )
            } else {
                Some(location)
            };
            if location.is_none() {
                continue;
            }
            let location = location.unwrap();

            let rust_result = rust_calculator.calculate(date_time.date_naive(), &location, zman);
            let java_result = java_calendar.get_zman(&zman);

            // Convert from Utc to the local timezone for comparison
            let rust_result_tz =
                rust_result.map(|dt| date_time.timezone().from_utc_datetime(&dt.naive_utc()));

            let mut max_diff = max_diff_override.unwrap_or(MAX_DIFF_SECONDS);

            // If this zman uses elevation, enlarge the max diff based on the elevation
            if uses_elevation(&zman) {
                max_diff += (location.elevation() * 0.1) as i64;
                assert!(
                    max_diff > 0 && max_diff < 100,
                    "Max diff is out of range for {:?}: {} meters",
                    zman,
                    location.elevation()
                );
            }

            assert_times_close(
                rust_result_tz,
                java_result,
                max_diff,
                &format!(
                    "Iteration {}: {:?} at {:?}, Location: ({}, {}), Elevation: {}",
                    i,
                    zman,
                    date_time,
                    location.latitude(),
                    location.longitude(),
                    location.elevation(),
                ),
            );
        }
    }

    // ============================================================================
    // Sunrise and Sunset Tests
    // ============================================================================

    #[test]
    fn test_sunrise_vs_java() {
        test_zman_vs_java(Zman::Sunrise, get_test_seed(), None);
    }
    #[test]
    fn test_sunset_vs_java() {
        test_zman_vs_java(Zman::Sunset, get_test_seed(), None);
    }

    #[test]
    fn test_sea_level_sunrise_vs_java() {
        test_zman_vs_java(Zman::SeaLevelSunrise, get_test_seed(), None);
    }

    #[test]
    fn test_sea_level_sunset_vs_java() {
        test_zman_vs_java(Zman::SeaLevelSunset, get_test_seed(), None);
    }

    #[test]
    fn test_chatzos_vs_java() {
        test_zman_vs_java(Zman::Chatzos, get_test_seed(), None);
    }

    #[test]
    fn test_alos_hashachar_vs_java() {
        test_zman_vs_java(Zman::AlosHashachar, get_test_seed(), None);
    }
    #[test]
    fn test_tzais_vs_java() {
        test_zman_vs_java(Zman::Tzais, get_test_seed(), None);
    }
    #[test]
    fn test_sof_zman_shma_gra_vs_java() {
        test_zman_vs_java(Zman::SofZmanShmaGRA, get_test_seed(), None);
    }
    #[test]
    fn test_sof_zman_shma_mga_vs_java() {
        test_zman_vs_java(Zman::SofZmanShmaMGA, get_test_seed(), None);
    }
    #[test]
    fn test_sof_zman_tfila_gra_vs_java() {
        test_zman_vs_java(Zman::SofZmanTfilaGRA, get_test_seed(), None);
    }
    #[test]
    fn test_sof_zman_tfila_mga_vs_java() {
        test_zman_vs_java(Zman::SofZmanTfilaMGA, get_test_seed(), None);
    }
    #[test]
    fn test_plag_hamincha_vs_java() {
        test_zman_vs_java(Zman::PlagHamincha, get_test_seed(), None);
    }
    #[test]
    fn test_mincha_gedola_vs_java() {
        test_zman_vs_java(Zman::MinchaGedola, get_test_seed(), None);
    }
    #[test]
    fn test_mincha_ketana_vs_java() {
        test_zman_vs_java(Zman::MinchaKetana, get_test_seed(), None);
    }
    #[test]
    fn test_candle_lighting_vs_java() {
        test_zman_vs_java(Zman::CandleLighting, get_test_seed(), None);
    }

    // ============================================================================
    // Regression Tests
    // ============================================================================

    /// Regression test for CandleLighting calculation issue
    /// Seed: 6793576168821758564, DateTime: 1877-02-20T08:24:11.832-06
    /// Location: (2.7157419038702884, -92.4056796520771), Elevation: 31.014520302567668
    /// Difference was 50 seconds, max allowed was 33 seconds
    #[test]
    fn test_candle_lighting_regression_6793576168821758564() {
        test_zman_vs_java(Zman::CandleLighting, 6793576168821758564, None);
    }

    // ============================================================================
    // Additional Zmanim Tests (add more as needed)
    // ============================================================================
    // To add a new test, simply call: test_zman_vs_java(Zman::YourZman, seed, tolerance_override)
    // Example:
    // #[test]
    // fn test_your_zman_vs_java() {
    //     test_zman_vs_java(Zman::YourZman, 12350, None);
    // }
}
