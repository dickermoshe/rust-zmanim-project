# Astronomical Calculator

A high-precision Rust library for calculating solar position, sunrise/sunset times, and related astronomical phenomena.

[![Crates.io](https://img.shields.io/crates/v/astronomical-calculator.svg)](https://crates.io/crates/astronomical-calculator)
[![Documentation](https://docs.rs/astronomical-calculator/badge.svg)](https://docs.rs/astronomical-calculator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

This library calculates the position of the Sun with high precision for any given time and location on Earth. It determines solar zenith and azimuth angles, sunrise and sunset times, and solar transit (solar noon). The implementation accounts for atmospheric refraction, parallax, nutation, aberration, and other astronomical phenomena that affect solar position calculations.

The library is `no_std` compatible and extensively tested using property-based testing with thousands of randomized inputs. All calculations have been validated against NREL's Solar Position Algorithm (SPA) with accuracy within 0.0000001° for all output parameters.

## Installation

```bash
cargo add astronomical-calculator chrono
```

## Implementation

This is an independent implementation written from scratch using algorithms from Jean Meeus's "Astronomical Algorithms" (2nd Edition), USNO publications, and VSOP87 theory for planetary positions. While the implementation was validated against NREL's Solar Position Algorithm (SPA) for accuracy testing, no code was copied from the NREL SPA library.

## Parameter Ranges

All input parameters are validated at construction time. The library will return an error if any parameter falls outside its valid range:

| Parameter     | Range                     | Notes                           |
| ------------- | ------------------------- | ------------------------------- |
| Year          | -2000 to 6000             | Extracted from datetime         |
| Longitude     | -180.0 to 180.0°          | Positive East, negative West    |
| Latitude      | -90.0 to 90.0°            | Positive North, negative South  |
| Elevation     | -6,500,000 to 6,500,000 m | Observer height above sea level |
| Pressure      | 0.0 to 5000.0 mb          | Affects atmospheric refraction  |
| Temperature   | -273.0 to 6000.0°C        | Affects atmospheric refraction  |
| Delta UT1     | -1.0 to 1.0 s             | Earth rotation correction       |
| Delta T       | -8000.0 to 8000.0 s       | TT-UT time scale correction     |
| Atmos Refract | -5.0 to 5.0°              | Refraction at horizon           |
| Slope         | -360.0 to 360.0°          | Surface slope angle             |
| Azm Rotation  | -360.0 to 360.0°          | Surface azimuth orientation     |
| Timezone      | -18.0 to 18.0 hours       | From datetime offset            |

## Delta T Parameter

The `delta_t` parameter represents the difference between Terrestrial Time (TT) and Universal Time (UT). This value changes over time due to variations in Earth's rotation rate. As of 2024, delta_t is approximately 69 seconds. It was approximately 64 seconds in 2000 and 57 seconds in 1990.

For most applications within a few years of the present, using a constant value is sufficient. For high-precision historical or future calculations, consult [IERS Bulletin A](https://www.iers.org/IERS/EN/Publications/Bulletins/bulletins.html) for current values or use published approximation formulas.

## Performance

The library is optimized for accuracy over speed. Each calculation performs multiple trigonometric operations, iterative refinement for rise/transit/set times, and applies nutation and aberration corrections. Typical performance on modern hardware is approximately 50-100 microseconds per calculation.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Submit issues or pull requests on [GitHub](https://github.com/dickermoshe/astronomical-calculator).
