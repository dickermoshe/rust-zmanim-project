//! Adapter tests for the diplomat bridge (Rust-side, no generated bindings required).

use interop::common::{civil_date_to_jiff, gregorian_to_hebrew, hebrew_to_gregorian, tractate_to_code};
use kosher_rust::{calendar::HebrewHolidayCalendar, limudim::prelude::*, zmanim::prelude::*};

#[test]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn gregorian_hebrew_round_trip() {
    let civil = interop::common::CivilDate {
        year: 2024,
        month: 1,
        day: 20,
    };
    let hebrew = gregorian_to_hebrew(civil).expect("valid conversion");
    assert_eq!(hebrew.year, 5784);
    assert_eq!(hebrew.month, 5);
    assert_eq!(hebrew.day, 10);

    let back = hebrew_to_gregorian(hebrew).expect("valid conversion");
    assert_eq!(back, civil);
}

#[test]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn yom_kippur_is_assur_bemelacha() {
    let date = interop::common::CivilDate {
        year: 2025,
        month: 10,
        day: 2,
    };
    let jiff = civil_date_to_jiff(date).expect("valid date");
    assert!(jiff.is_assur_bemelacha(false));
}

#[test]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn daf_yomi_bavli_matches_core_api() {
    let date = interop::common::CivilDate {
        year: 2017,
        month: 12,
        day: 28,
    };
    let jiff = civil_date_to_jiff(date).expect("valid date");
    let daf = jiff.limud(DafYomiBavli::default()).expect("daf scheduled");
    assert_eq!(daf.tractate, Tractate::Shevuos);
    assert_eq!(daf.page, 30);
    assert!(matches!(
        tractate_to_code(daf.tractate),
        interop::common::TractateCode::Shevuos
    ));
}

#[test]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn elevation_adjusted_sunrise_via_preset_dispatch() {
    use interop::generated::preset_dispatch::{PRESET_METADATA, preset_by_index};
    use jiff::{civil::Date, tz::TimeZone};

    let location =
        Location::new(31.78, 35.22, 0.0, Some(TimeZone::get("Asia/Jerusalem").unwrap())).expect("valid location");
    let calc = ZmanimCalculator::new(location, Date::new(2017, 10, 17).unwrap(), CalculatorConfig::default());
    let direct = calc
        .calculate(&presets::ELEVATION_ADJUSTED_SUNRISE)
        .expect("sunrise available");

    let index = PRESET_METADATA
        .iter()
        .position(|meta| meta.method_name == "getSunrise")
        .expect("sunrise preset in dispatch table");
    let via_dispatch = calc
        .calculate(preset_by_index(index).expect("preset lookup"))
        .expect("sunrise via dispatch");
    assert_eq!(direct, via_dispatch);
}

#[test]
#[allow(clippy::unwrap_used, clippy::expect_used)]
fn preset_dispatch_count_matches_dsl() {
    use interop::generated::preset_dispatch::{ZMAN_PRESET_COUNT, preset_by_index};

    assert_eq!(ZMAN_PRESET_COUNT, 167);
    assert!(preset_by_index(0).is_some());
    assert!(preset_by_index(ZMAN_PRESET_COUNT - 1).is_some());
    assert!(preset_by_index(ZMAN_PRESET_COUNT).is_none());
}
