# Zmanim Calculator

[![CI](https://github.com/dickermoshe/zmanim-calculator/actions/workflows/ci.yml/badge.svg)](https://github.com/dickermoshe/zmanim-calculator/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/dickermoshe/zmanim-calculator/branch/main/graph/badge.svg)](https://codecov.io/gh/dickermoshe/zmanim-calculator)

## Install

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
zmanim-calculator = "0.1.0"
```

## Usage

```rust
use chrono::{NaiveDate, Utc};
use zmanim_calculator::{
    CalculatorConfig, Location, ZmanimCalculator, zman::SUNRISE, zman::TZAIS_MINUTES_72,
};

let location = Location::new(40.7128, -74.0060, 10.0, Some(Utc))
    .expect("valid location");
let date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid date");
let config = CalculatorConfig::default();
let mut calc = ZmanimCalculator::new(location, date, config).expect("calculator");

let sunrise = calc.calculate(SUNRISE).expect("sunrise");
let tzais = calc.calculate(TZAIS_MINUTES_72).expect("tzais");

println!("Sunrise (UTC): {sunrise}");
println!("Tzais (UTC): {tzais}");
```

Notes:

- `calculate` returns `DateTime<Utc>`; convert to local time as needed.
- If you omit a timezone, locations near the anti-meridian require one.

## Zmanim Calculator

We strive to follow KosherJava's APIs as closely as possible to maintain compatibility and familiarity for developers who have used the Java library. However, our documentation is not as comprehensive as KosherJava's. For detailed API documentation and usage examples, please refer to the [KosherJava documentation](https://kosherjava.com/zmanim-project/how-to-use-the-zmanim-api/).

## Technical Details

The library is designed with modularity and flexibility in mind. It supports both `std` and `no_std` environments, making it suitable for embedded systems, web assembly, and traditional applications. The core implementation is written in Rust, providing memory safety, performance, and cross-platform compatibility.

- `std` (optional): Enables standard library support.
- `defmt` (optional): Enables `defmt` logging for embedded targets.

- **KosherJava**: The original Java implementation by Eliyahu Hershfeld and contributors, which provides the core algorithms and calculations that this library is based upon. KosherJava has been a trusted resource in the Jewish programming community for many years, and we are grateful for their excellent work and the open-source license that makes projects like this possible.

## License

This project is based on KosherJava, which is released under the GNU Lesser General Public License version 2.1 (LGPL 2.1). This license allows the library to be used in both free and proprietary software while ensuring that modifications to the library itself remain open source.

For the full license text, please refer to the LICENSE file in the kosher-java subdirectory.
