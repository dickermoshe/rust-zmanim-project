> This crate is part of the [Rust Zmanim Project](TODO).

# Hebrew Holiday Calendar

A Rust library for working with the Hebrew calendar, including holidays, Torah readings (parshiyot), molad, and calendar calculations. Supports `no_std` environments.

[![Crates.io](https://img.shields.io/crates/v/hebrew_holiday_calendar.svg)](https://crates.io/crates/hebrew_holiday_calendar)
[![Documentation](https://docs.rs/hebrew_holiday_calendar/badge.svg)](https://docs.rs/hebrew_holiday_calendar)
[![codecov](https://codecov.io/gh/dickermoshe/rust-zmanim-project/graph/badge.svg?flag=hebrew_holiday_calendar)](https://codecov.io/gh/dickermoshe/rust-zmanim-project)

## Installation

```bash
cargo add hebrew_holiday_calendar icu_calendar
```

Or manually add to your `Cargo.toml`:

```toml
[dependencies]
hebrew_holiday_calendar = "0.2"
icu_calendar = "2"
```

For `no_std` environments, disable default features:

```toml
[dependencies]
hebrew_holiday_calendar = { version = "0.2", default-features = false }
```

## Usage

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

## Feature Flags

- **`defmt`** — Enables formatting support for embedded logging

## License

Licensed under LGPL-2.1. See [LICENSE](LICENSE) for details.
