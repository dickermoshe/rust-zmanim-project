# Zmanim Calculator

[![CI](https://github.com/dickermoshe/zmanim-calculator/actions/workflows/ci.yml/badge.svg)](https://github.com/dickermoshe/zmanim-calculator/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/dickermoshe/zmanim-calculator/branch/main/graph/badge.svg)](https://codecov.io/gh/dickermoshe/zmanim-calculator)

This project has been tested extensively, but the public API is still evolving.
If you are building against it today, pin to a specific commit.

## Install

Recommended while the API is still changing:

```toml
[dependencies]
zmanim-calculator = { git = "https://github.com/dickermoshe/zmanim-calculator", rev = "<commit-sha>" }
```

## Rust Usage

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

Notes:

- `calculate` returns `DateTime<Utc>`.
- If you omit a timezone, calculations near the anti-meridian (`|longitude| > 150`) will fail.

## C API (`c` feature)

The crate can generate and ship a C header and C-callable functions via the `c` feature.

Generate the header:

```bash
cargo run --features c --bin build_c_headers
```

Build the C-callable library:

```bash
cargo build --release --features c
```

This writes the header to `bindings/c/zmanim_calendar.h`.
See `example-c-project/` for a complete usage example (`main.c` and `build_and_run.sh`).

## Feature Flags

- `std`: Enables standard library support.
- `defmt`: Enables `defmt` formatting/logging support for embedded targets.
- `c`: Enables the C FFI surface and header generation tooling (`std` is enabled automatically).

## Compatibility and References

The API aims to follow KosherJava naming and behavior where possible.
For background and broader algorithm documentation, see the [KosherJava documentation](https://kosherjava.com/zmanim-project/how-to-use-the-zmanim-api/).

## License

This project is based on KosherJava, released under LGPL 2.1.
For details, see [LICENSE](./LICENSE).
