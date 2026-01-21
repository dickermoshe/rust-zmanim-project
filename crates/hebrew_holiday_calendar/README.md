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

See the [docs](https://docs.rs/hebrew_holiday_calendar/) for more info.  

## Testing

This library is extensively tested against [KosherJava](https://github.com/KosherJava/zmanim), the widely-used Java library for Jewish calendar calculations. Our test suite includes:

The extensive testing ensures compatibility and accuracy with established Jewish calendar standards.

## no_std Support

This library works in `no_std` environments. All core functionality is available without the standard library.

```toml
[dependencies]
hebrew_holiday_calendar = { version = "0.1.0", default-features = false }
```

### Optional Features

- `defmt`: Enables formatting support for embedded logging

## License

The library is released under the LGPL 2.1 license.

## Acknowledgments

- [icu_calendar](https://github.com/unicode-org/icu4x) for core calendar operations
- [KosherJava](https://github.com/KosherJava/zmanim) for validation and reference implementation

