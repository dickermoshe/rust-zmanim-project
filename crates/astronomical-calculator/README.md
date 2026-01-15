# Astronomical Calculator

A high-precision Rust library for calculating solar position, sunrise/sunset times, and related astronomical phenomena.

[![Crates.io](https://img.shields.io/crates/v/astronomical-calculator.svg)](https://crates.io/crates/astronomical-calculator)
[![Documentation](https://docs.rs/astronomical-calculator/badge.svg)](https://docs.rs/astronomical-calculator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

This library calculates the position of the Sun with high precision for any given time and location on Earth. It provides accurate solar position calculations (zenith and azimuth angles), sunrise and sunset times, solar transit (solar noon), solar midnight, and various twilight events (civil, nautical, and astronomical dawn/dusk).

The library accounts for atmospheric refraction, parallax, nutation, aberration, and other astronomical phenomena that affect solar position calculations. It is based on VSOP87 theory and uses the same algorithms as the [freespa](https://github.com/IEK-5/freespa) library, which this is a Rust port of.

The library is `no_std` compatible, making it suitable for embedded systems and other constrained environments. It has been extensively tested using property-based testing with thousands of randomized inputs, and validated against both the original C implementation (freespa) and the [Skyfield](https://rhodesmill.org/skyfield/) Python library to ensure accuracy and reliability.

## Features

- **Solar Position Calculations**: Compute the Sun's zenith and azimuth angles for any time and location
- **Sunrise and Sunset Times**: Calculate precise sunrise and sunset times, handling polar day/night conditions
- **Solar Transit**: Determine solar noon (when the Sun reaches its highest point) and solar midnight
- **Twilight Calculations**: Calculate civil, nautical, and astronomical dawn and dusk times
- **Sea-Level Calculations**: Compute sunrise/sunset at sea level (important for twilight calculations)
- **Atmospheric Refraction Models**: Support for multiple refraction models (Bennett with and without atmospheric correction)
- **Elevation Support**: Account for observer elevation above sea level
- **High Precision**: Based on VSOP87 theory with sub-arcsecond accuracy
- **Wide Date Range**: Valid for dates from 2000 BC to 6000 CE
- **`no_std` Compatible**: Works in embedded and constrained environments
- **Comprehensive Testing**: Property-based testing ensures accuracy across diverse inputs

## Quick Start

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

## Input Time Considerations

**Important:** When calculating solar events (sunrise, sunset, twilight), the input time should be close to local noon for the given location. This ensures that events are calculated for the correct solar day. Using times far from noon may result in events being calculated for the wrong day.

## Installation

Add the library to your `Cargo.toml`:

```bash
cargo add astronomical-calculator chrono
```

Or add it manually:

```toml
[dependencies]
astronomical-calculator = "0.1.0"
chrono = { version = "0.4.42", default-features = false }
```

Note: The `chrono` dependency is required for datetime handling. If you're using this in a `no_std` environment, ensure `chrono` is configured without the `std` feature.

## Accuracy

The library provides high-precision calculations based on:

- **VSOP87 Theory**: Uses VSOP87 (Variations Séculaires des Orbites Planétaires) for planetary ephemeris calculations
- **Atmospheric Refraction**: Accounts for how the atmosphere bends light, making the Sun appear higher than its geometric position
- **Parallax**: Corrects for the observer's position on Earth's surface
- **Nutation**: Accounts for small periodic variations in Earth's rotation axis
- **Aberration**: Corrects for the finite speed of light and Earth's motion

The library is valid for dates from 2000 BC to 6000 CE. For dates within the historical range (1657-2024), ΔT values are interpolated from historical data. For dates outside this range, polynomial approximations are used.

## Testing

This library is extensively tested using property-based testing with thousands of randomized inputs. The test suite validates results against:

- The original C implementation ([freespa](https://github.com/IEK-5/freespa)) to ensure algorithmic correctness
- The [Skyfield](https://rhodesmill.org/skyfield/) Python library for astronomical accuracy validation

Testing covers diverse scenarios including:

- Various dates and times across the valid date range
- Different geographic locations (including polar regions)
- Different elevations and atmospheric conditions
- Edge cases (polar day/night, extreme dates)

## Documentation

For detailed API documentation, usage examples, and advanced topics, see the [documentation on docs.rs](https://docs.rs/astronomical-calculator).

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests on [GitHub](https://github.com/dickermoshe/astronomical-calculator).

## Changelog

- 0.1.0: Initial Release
- 0.2.0: Remove licensed code
- 0.3.0 :`get_sunrise_offset_by_degrees` and `get_sunset_offset_by_degrees` do not use the provided geometric dip, instead they use the geometric horizon.
