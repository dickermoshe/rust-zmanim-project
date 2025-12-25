# Astronomical Calculator

A high-precision Rust library for calculating solar position, sunrise/sunset times, and related astronomical phenomena.

[![Crates.io](https://img.shields.io/crates/v/astronomical-calculator.svg)](https://crates.io/crates/astronomical-calculator)
[![Documentation](https://docs.rs/astronomical-calculator/badge.svg)](https://docs.rs/astronomical-calculator)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

This library calculates the position of the Sun with high precision for any given time and location on Earth. It determines solar zenith and azimuth angles, sunrise and sunset times, and solar transit (solar noon).

This is a Rust port of the [freespa](https://github.com/IEK-5/freespa) library. The library is `no_std` compatible and extensively tested using property-based testing with thousands of randomized inputs.

## Documentation

For usage examples and API documentation, see the [documentation on docs.rs](https://docs.rs/astronomical-calculator).

## Installation

```bash
cargo add astronomical-calculator chrono
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Submit issues or pull requests on [GitHub](https://github.com/dickermoshe/astronomical-calculator).
