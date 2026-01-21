# Hebrew Holiday Calendar

A comprehensive Rust library for working with the Hebrew calendar, including holidays, Torah readings (parshiyot), and calendar calculations. This library provides `no_std` support and can be used in embedded environments.

## Features

- 📅 **Hebrew Calendar Operations**: Convert between Gregorian and Hebrew dates
- 🎉 **Holiday Tracking**: Determine Jewish holidays with customizable rules
- 📖 **Torah Readings**: Find weekly parshiyot for any date
- 🌙 **Molad Calculations**: Calculate molad and Kiddush Levana times
- ⭐ **Special Shabbatot**: Identify Shekalim, Zachor, Parah, Hachodesh, and more
- 🌍 **Israel & Diaspora Support**: Respects location-specific customs
- 🕯️ **Candle Lighting**: Determine days requiring candle lighting
- ⚡ **Fast & Efficient**: Optimized calendar calculations
- 🔧 **No Std Compatible**: Works in embedded environments

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hebrew_holiday_calendar = "0.1.0"
```

For `no_std` environments:

```toml
[dependencies]
hebrew_holiday_calendar = { version = "0.1.0", default-features = false }
```

## Quick Start

```rust
use hebrew_holiday_calendar::*;
use icu_calendar::{Date, cal::Hebrew};

// Create a Hebrew date: 1 Tishrei 5784 (Rosh Hashana)
let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 1).unwrap();

// Get holidays
for holiday in date.holidays(false, false) {
    println!("Holiday: {}", holiday.he()); // "ראש השנה"
}

// Check if work is forbidden
if date.is_assur_bemelacha(false) {
    println!("Work is forbidden today");
}

// Get the weekly parsha (on Shabbat)
if let Some(parsha) = date.todays_parsha(false) {
    println!("Parsha: {}", parsha.en());
}
```

## Holiday Types

### Major Holidays

- Rosh Hashana, Yom Kippur
- Succos, Shemini Atzeres, Simchas Torah
- Chanukah, Purim
- Pesach, Shavuos

### Fast Days

- Fast of Gedalyah, Tenth of Teves, Fast of Esther
- Seventeenth of Tammuz, Tisha B'Av

### Modern Israeli Holidays

- Yom HaShoah, Yom Hazikaron, Yom Ha'atzmaut, Yom Yerushalayim

### Special Days

- Rosh Chodesh, Lag BaOmer, Tu B'Shvat, Tu B'Av
- Isru Chag, Yom Kippur Katan, BaHaB

## Torah Readings

- **Weekly Parshiyot**: All 54 Torah portions
- **Combined Readings**: For non-leap years (e.g., Vayakhel-Pekudei)
- **Special Shabbatot**: Shekalim, Zachor, Parah, Hachodesh
- **Holiday-Adjacent**: Shabbat Hagadol, Shuva, Chazon, Nachamu
- **Israel vs. Diaspora**: Different reading schedules when holidays differ

## Testing

This library is extensively tested against [KosherJava](https://github.com/KosherJava/zmanim), the widely-used Java library for Jewish calendar calculations. Our test suite includes:

- **16 comprehensive integration tests** comparing Rust implementations against KosherJava
- **Randomized testing** across a wide range of dates (1870-2070)
- **All 41 holiday types** validated for correctness
- **Parsha calculations** verified for both Israel and Diaspora
- **Special observances** including day of Chanukah, day of Omer, fast days, and more

The extensive testing ensures compatibility and accuracy with established Jewish calendar standards.

## no_std Support

This library works in `no_std` environments. All core functionality is available without the standard library.

```toml
[dependencies]
hebrew_holiday_calendar = { version = "0.1.0", default-features = false }
```

### Optional Features

- `defmt`: Enables formatting support for embedded logging

## Implementation

The library uses the ICU calendar system (`icu_calendar`) for accurate Hebrew date calculations and conversions. All Hebrew calendar rules are implemented according to traditional Jewish law:

- Molad calculations
- Dechiyot (postponement rules)
- Leap year cycles (19-year Metonic cycle)
- Variable month lengths (Cheshvan and Kislev)

### Molad and Kiddush Levana APIs

Molad and Kiddush Levana calculations are exposed on `Date<Gregorian>` via the `MoladCalendar` trait. These methods only return `Some` when the molad-based time falls on that Gregorian date in the provided timezone.

## License

The library is released under the LGPL 2.1 license.

## Acknowledgments

- [icu_calendar](https://github.com/unicode-org/icu4x) for core calendar operations
- [KosherJava](https://github.com/KosherJava/zmanim) for validation and reference implementation
