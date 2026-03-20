> This crate is part of the [Rust Zmanim Project](https://github.com/dickermoshe/rust-zmanim-project).

# Astronomical Calculator

A high-precision Rust library for calculating solar position, sunrise/sunset times, and related astronomical phenomena. Based on VSOP87 theory (ported from [freespa](https://github.com/IEK-5/freespa)), it accounts for atmospheric refraction, parallax, nutation, and aberration. The library is `no_std` compatible and validated against both the original C implementation and [Skyfield](https://rhodesmill.org/skyfield/).

[![Crates.io](https://img.shields.io/crates/v/astronomical-calculator.svg)](https://crates.io/crates/astronomical-calculator)
[![Documentation](https://docs.rs/astronomical-calculator/badge.svg)](https://docs.rs/astronomical-calculator)
[![codecov](https://codecov.io/gh/dickermoshe/rust-zmanim-project/graph/badge.svg?flag=astronomical-calculator)](https://codecov.io/gh/dickermoshe/rust-zmanim-project)

## Installation

```bash
cargo add astronomical-calculator chrono
```

Or manually add to your `Cargo.toml`:

```toml
[dependencies]
astronomical-calculator = "0.4"
chrono = { version = "0.4", default-features = false }
```

The `chrono` dependency is required for datetime handling. In a `no_std` environment, ensure `chrono` is configured without the `std` feature.

## Usage

```rust
use astronomical_calculator::{AstronomicalCalculator, Refraction};
use chrono::NaiveDateTime;

// Create a datetime (UTC)
let dt = NaiveDateTime::parse_from_str("2024-06-21 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

// Create calculator for London (51.5°N, 0°E)
let mut calc = AstronomicalCalculator::new(
    dt,
    None,                     // Calculate ΔT automatically
    0.0,                      // ΔUT1 (use 0.0 if unknown)
    0.0,                      // longitude in degrees (0° = Greenwich)
    51.5,                     // latitude in degrees (positive = North)
    0.0,                      // elevation in meters
    20.0,                     // temperature in Celsius
    1013.25,                  // pressure in millibars
    None,                     // geometric dip (None = standard horizon)
    Refraction::ApSolposBennetNA, // refraction model
).unwrap();

// Get current solar position
let position = calc.get_solar_position();
println!("Zenith: {:.2}°", position.zenith.to_degrees());
println!("Azimuth: {:.2}°", position.azimuth.to_degrees());

// Get sunrise and sunset times
use astronomical_calculator::SolarEventResult;
match calc.get_sunrise().unwrap() {
    SolarEventResult::Occurs(timestamp) => {
        println!("Sunrise at Unix timestamp: {}", timestamp);
    }
    SolarEventResult::AllDay => println!("Sun never sets (midnight sun)"),
    SolarEventResult::AllNight => println!("Sun never rises (polar night)"),
}
```

When calculating solar events (sunrise, sunset, twilight), the input time should be close to local noon for the given location. This ensures events are calculated for the correct solar day.

## License

Licensed under MIT. See [LICENSE](LICENSE) for details.
