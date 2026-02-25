//! Access to the bundled time zone database.
//!
//! This module is available when the `bundled-tzdb` feature is enabled.
use super::{Error, Tz};

include!(concat!(env!("OUT_DIR"), "/bundled_tzdb.rs"));

/// Returns the IANA tzdb version used to build this bundled database.
pub fn version() -> &'static str {
    TZ_DB_VERSION
}

/// Returns all bundled tzdb entries as `(name, raw_tzif_bytes)` pairs.
pub fn all() -> &'static [(&'static str, &'static [u8])] {
    BUNDLED_TZDB
}
/// Parses a named time zone from the bundled tz database.
///
/// Returns [`Error::InvalidTimeZoneFileName`] when `name` is not found in the
/// bundled database.
pub fn parse(name: &str) -> Result<Tz, Error> {
    let bytes = BUNDLED_TZDB
        .iter()
        .find(|(tz_name, _)| *tz_name == name)
        .map(|(_, bytes)| *bytes)
        .ok_or(Error::InvalidTimeZoneFileName)?;
    Tz::parse(name, bytes)
}
#[cfg(test)]
mod tests {
    extern crate std;
    use chrono::{Datelike, Duration, NaiveDate, Offset, TimeZone, Utc};

    use proptest::prelude::*;
    use std::vec::Vec;

    use crate::{
        bundled::{all, parse},
        test_utils::init,
    };

    fn bundled_entry_strategy() -> impl Strategy<Value = &'static str> {
        let entries = all()
            .iter()
            .filter_map(|(name, _)| {
                if name.parse::<chrono_tz::Tz>().is_ok() {
                    Some(*name)
                } else {
                    None
                }
            })
            .collect::<Vec<&'static str>>();
        assert!(
            !entries.is_empty(),
            "no bundled entries are parseable by chrono-tz"
        );
        proptest::sample::select(entries)
    }

    fn max_test_timestamp() -> i64 {
        let now = Utc::now();
        now.with_year(now.year() + 10)
            .unwrap_or_else(|| now + Duration::days(3653))
            .timestamp()
    }

    #[test]
    fn parse_all() {
        init();
        for o in all() {
            let _ = parse(o.0).unwrap();
        }
    }

    #[test]
    fn bundled_matches_chrono_tz_second_offsets_amsterdam() {
        init();
        let our_tz = parse("Europe/Amsterdam").expect("bundled Europe/Amsterdam should parse");
        let chrono_tz = "Europe/Amsterdam"
            .parse::<chrono_tz::Tz>()
            .expect("chrono-tz Europe/Amsterdam should parse");
        let utc_naive = NaiveDate::from_ymd_opt(1914, 1, 1)
            .expect("valid date")
            .and_hms_opt(13, 40, 28)
            .expect("valid time");
        let utc = Utc.from_utc_datetime(&utc_naive);

        let ours = utc.with_timezone(&&our_tz);
        let theirs = utc.with_timezone(&chrono_tz);

        assert_eq!(ours.naive_local(), theirs.naive_local());
        assert_eq!(
            ours.offset().fix().local_minus_utc(),
            theirs.offset().fix().local_minus_utc()
        );
        assert_eq!(
            std::format!("{}", ours.format("%Z")),
            std::format!("{}", theirs.format("%Z"))
        );
    }

    proptest! {
        #[test]
        fn bundled_matches_chrono_tz_with_proptest(
            tz_name in bundled_entry_strategy(),
            // Keep this sanity check in the Unix-era window where chrono-tz and
            // bundled zoneinfo sources align best across historical backzone
            // differences and far-future POSIX tail handling differences.
            // Limit the upper bound to 10 years from test execution time.
            ts in chrono::DateTime::<Utc>::MIN_UTC.timestamp()..max_test_timestamp()
        ) {
            init();
            let our_tz = parse(tz_name).map_err(|e| {
                TestCaseError::fail(std::format!("tzfile parse failed for {tz_name}: {e:?}"))
            })?;
            let utc = Utc.timestamp_opt(ts, 0).single().ok_or_else(|| {
                TestCaseError::fail(std::format!("timestamp out of range for chrono: {ts}"))
            })?;

            let ours = (&our_tz).offset_from_utc_datetime(&utc.naive_utc()).fix().local_minus_utc();

            let chrono_tz = tz_name.parse::<chrono_tz::Tz>().map_err(|e| {
                TestCaseError::fail(std::format!("chrono_tz parse failed for {tz_name}: {e}"))
            })?;
            let theirs = chrono_tz.offset_from_utc_datetime(&utc.naive_utc()).fix().local_minus_utc();

            prop_assert_eq!(ours, theirs);
        }
    }
}
