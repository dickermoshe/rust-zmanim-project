use crate::*;
use chrono::prelude::*;
use serde::Deserialize;

extern crate std;
/// Converts a timestamp in seconds to a `DateTime` in the given time zone.
fn tz_ts_to_dt<T: TimeZone>(ts: f64, tz: T) -> DateTime<T> {
    tz.timestamp_millis_opt((ts * 1000.0) as i64).unwrap()
}

/// Assert that a list of Option<DateTime> values are in chronological order.
/// Only compares consecutive pairs where both values are Some.
///
/// # Arguments
/// * `events` - Slice of Option values to check
/// * `names` - Slice of names for error messages (should match events length)
/// * `context` - Additional context string for error messages
/// * `row` - Record data for error messages
/// * `index` - Index for error messages
fn assert_events_in_order<T: PartialOrd + std::fmt::Display, C: std::fmt::Debug>(
    events: &[Option<T>],
    names: &[&str],
    context: &str,
    row: &C,
    index: usize,
) {
    assert_eq!(
        events.len(),
        names.len(),
        "Events and names slices must have the same length"
    );

    for i in 0..events.len().saturating_sub(1) {
        if let (Some(prev), Some(next)) = (&events[i], &events[i + 1]) {
            assert!(
                prev < next,
                "{}: {} should be before {}. {}: {}, {}: {}. Record: {:?} Index: {}",
                context,
                names[i],
                names[i + 1],
                names[i],
                prev,
                names[i + 1],
                next,
                row,
                index
            );
        }
    }
}

#[test]
fn test_geonames_csv_transit() {
    use chrono::TimeZone;
    use chrono_tz::Tz;
    use csv::ReaderBuilder;
    use rand::Rng;
    use std::fs::File;
    use std::io::BufReader;
    #[allow(unused)]
    #[derive(Debug, Deserialize)]
    struct GeonamesRow {
        geoname_id: std::string::String,
        name: std::string::String,
        ascii_name: std::string::String,
        #[serde(default)]
        alternate_names: std::string::String,
        feature_class: std::string::String,
        feature_code: std::string::String,
        country_code: std::string::String,
        cou_name_en: std::string::String,
        #[serde(default)]
        country_code_2: std::string::String,
        #[serde(default)]
        admin1_code: std::string::String,
        #[serde(default)]
        admin2_code: std::string::String,
        #[serde(default)]
        admin3_code: std::string::String,
        #[serde(default)]
        admin4_code: std::string::String,
        population: std::string::String,
        #[serde(default)]
        elevation: std::string::String,
        #[serde(default)]
        dem: std::string::String,
        timezone: std::string::String,
        #[serde(default)]
        modification_date: std::string::String,
        #[serde(default)]
        label_en: std::string::String,
        coordinates: std::string::String,
    }

    // Open and read the CSV file
    let file = File::open("test_data/cities.csv").expect("Failed to open CSV file");
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(BufReader::new(file));

    let mut rng = rand::thread_rng();

    // Process each row
    for (index, result) in rdr.deserialize().enumerate() {
        let row: GeonamesRow = result.expect("Failed to parse CSV row");

        // Parse coordinates (format: "lat,lon")
        let coords: std::vec::Vec<&str> = row.coordinates.split(',').collect();
        if coords.len() != 2 {
            continue;
        }

        let lat: f64 = coords[0].trim().parse().unwrap();
        let lon: f64 = coords[1].trim().parse().unwrap();

        // Skip extreme latitudes that might cause issues
        if lat.abs() > 75.0 {
            continue;
        }

        // Parse timezone using chrono-tz
        let tz: Tz = row.timezone.parse().unwrap();

        // Generate a random input time between 1900 and 2100
        let year = rng.gen_range(1900..=2100);
        let month = rng.gen_range(1..=12);
        let day = rng.gen_range(1..=28); // Use 28 to avoid month-end issues

        // Create a naive datetime at midday
        let dt = tz.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap();

        // Create calculator and get transit
        let mut calc = AstronomicalCalculator::new(
            dt.to_utc(),
            None,
            0.0,
            lon,
            lat,
            0.0,
            20.0,
            1013.25,
            None,
            Refraction::ApSolposBennet,
        )
        .unwrap();

        let transit = tz_ts_to_dt(calc.get_solar_transit().unwrap() as f64, tz);

        let diff = transit - dt;
        let midday_to_transit_diff = diff.num_seconds().abs();
        assert!(
            midday_to_transit_diff <= 60 * 60 * 8,
            "Midday to transit difference too large: {} seconds, input: {}, transit: {}. Record: {:?} Index: {}",
            midday_to_transit_diff,
            dt,
            transit,
            row,
            index
        );
        // Sunrise is always before transit and sunset is always after transit
        let sunrise = calc
            .get_sunrise()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let sunset = calc
            .get_sunset()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let astronomical_dawn = calc
            .get_astronomical_dawn()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let nautical_dawn = calc
            .get_nautical_dawn()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let civil_dawn = calc
            .get_civil_dawn()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let civil_dusk = calc
            .get_civil_dusk()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let nautical_dusk = calc
            .get_nautical_dusk()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();
        let astronomical_dusk = calc
            .get_astronomical_dusk()
            .map(|result| result.timestamp().map(|ts| tz_ts_to_dt(ts as f64, tz)))
            .ok()
            .flatten();

        assert_events_in_order(
            &[
                astronomical_dawn,
                nautical_dawn,
                civil_dawn,
                sunrise,
                Some(transit),
                sunset,
                civil_dusk,
                nautical_dusk,
                astronomical_dusk,
            ],
            &[
                "Astronomical dawn",
                "Nautical dawn",
                "Civil dawn",
                "Sunrise",
                "Transit",
                "Sunset",
                "Civil dusk",
                "Nautical dusk",
                "Astronomical dusk",
            ],
            "Solar events",
            &row,
            index,
        );
    }
}
