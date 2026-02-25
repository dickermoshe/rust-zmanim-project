use super::{Error, Tz};

include!(concat!(env!("OUT_DIR"), "/bundled_tzdb.rs"));

pub fn all() -> &'static [(&'static str, &'static [u8])] {
    BUNDLED_TZDB
}

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
    use chrono::{Offset, TimeZone, Utc};

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

    #[test]
    fn parse_all() {
        init();
        for o in all() {
            let _ = parse(o.0).unwrap();
        }
    }

    proptest! {
        #[test]
        fn bundled_matches_chrono_tz_with_proptest(
            tz_name in bundled_entry_strategy(),
            // Keep this sanity check in the Unix-era window where chrono-tz and
            // bundled zoneinfo sources align best across historical backzone
            // differences and far-future POSIX tail handling differences.
            ts in 0_i64..2_147_483_648_i64
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
