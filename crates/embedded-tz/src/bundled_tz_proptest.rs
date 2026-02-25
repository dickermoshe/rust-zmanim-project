extern crate std;
use chrono::{TimeZone, Utc};

use proptest::prelude::*;
use std::vec::Vec;

fn bundled_entries_supported_by_chrono_tz() -> Vec<&'static str> {
    super::bundled::all()
        .iter()
        .filter(|(name, _)| name.parse::<chrono_tz::Tz>().is_ok())
        .map(|(name, _)| *name)
        .collect()
}

fn bundled_entry_strategy() -> impl Strategy<Value = &'static str> {
    let entries = bundled_entries_supported_by_chrono_tz();
    assert!(
        !entries.is_empty(),
        "no bundled entries are parseable by chrono-tz"
    );
    proptest::sample::select(entries)
}

proptest! {
    #[test]
    fn bundled_matches_chrono_tz_with_proptest(
        tz_name in bundled_entry_strategy(),
        ts in -2_208_988_800_i64..4_102_444_800_i64
    ) {
        let our_tz = super::bundled::parse(tz_name).map_err(|e| {
            TestCaseError::fail(std::format!("tzfile parse failed for {tz_name}: {e:?}"))
        })?;
        let chrono_tz = tz_name.parse::<chrono_tz::Tz>().map_err(|e| {
            TestCaseError::fail(std::format!("chrono_tz parse failed for {tz_name}: {e}"))
        })?;
        let utc = Utc.timestamp_opt(ts, 0).single().ok_or_else(|| {
            TestCaseError::fail(std::format!("timestamp out of range for chrono: {ts}"))
        })?;

        let ours = utc.with_timezone(&&our_tz);
        let theirs = utc.with_timezone(&chrono_tz);

        prop_assert_eq!(ours.naive_local(), theirs.naive_local());

    }
}
