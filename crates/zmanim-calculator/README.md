> This crate is part of the [Rust Zmanim Project](https://github.com/dickermoshe/rust-zmanim-project).

# Zmanim Calculator

A Rust library for calculating halachic zmanim (times), following KosherJava naming and behavior. Supports `no_std` environments.

[![Crates.io](https://img.shields.io/crates/v/zmanim-calculator.svg)](https://crates.io/crates/zmanim-calculator)
[![Documentation](https://docs.rs/zmanim-calculator/badge.svg)](https://docs.rs/zmanim-calculator)

## Installation

```bash
cargo add zmanim-calculator chrono
```

## Usage

```rust
use chrono::{NaiveDate, Utc};
use zmanim_calculator::{
    prelude::{CalculatorConfig, Location, ZmanimCalculator},
    presets::{SUNRISE, TZAIS_MINUTES_72},
};

fn main() {
    let location = Location::new(40.7128, -74.0060, 10.0, Some(Utc)).expect("valid location");
    let date = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
    let mut calc =
        ZmanimCalculator::new(location, date, CalculatorConfig::default()).expect("calculator");

    let sunrise = calc.calculate(SUNRISE).expect("sunrise");
    let tzais = calc.calculate(TZAIS_MINUTES_72).expect("tzais");

    println!("Sunrise (UTC): {sunrise}");
    println!("Tzais 72 (UTC): {tzais}");
}
```

If you omit a timezone, calculations near the anti-meridian (`|longitude| > 150`) will fail. Kiddush Levana and Molad calculations require a timezone as well.

## Feature Flags

- **`defmt`** — Enables `defmt` formatting/logging for embedded targets

## Compatibility

The API aims to follow KosherJava naming and behavior where possible. For background and broader algorithm documentation, see the [KosherJava documentation](https://kosherjava.com/zmanim-project/how-to-use-the-zmanim-api/).

## License

Licensed under LGPL-2.1. See [LICENSE](LICENSE) for details.
