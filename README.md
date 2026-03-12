# Zmanim Monorepo

A collection of Rust crates for Jewish calendar computations: astronomical calculations, zmanim (halachic times), Hebrew holidays, and learning schedules. Every core crate is `no_std`-compatible.

## Crates

| Crate                                                       | Version                                                                                                                       | Description                                                                                     |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| [astronomical-calculator](./crates/astronomical-calculator) | [![crates.io](https://img.shields.io/crates/v/astronomical-calculator.svg)](https://crates.io/crates/astronomical-calculator) | High-precision solar position, sunrise/sunset, and twilight calculations based on VSOP87 theory |
| [zmanim-calculator](./crates/zmanim-calculator)             | [![crates.io](https://img.shields.io/crates/v/zmanim-calculator.svg)](https://crates.io/crates/zmanim-calculator)             | Halachic zmanim in the style of KosherJava                                                      |
| [hebrew_holiday_calendar](./crates/hebrew_holiday_calendar) | [![crates.io](https://img.shields.io/crates/v/hebrew_holiday_calendar.svg)](https://crates.io/crates/hebrew_holiday_calendar) | Hebrew calendar, holidays, parshiyot, molad, and Kiddush Levana                                 |
| [limudim-calendar](./crates/limudim-calendar)               | [![crates.io](https://img.shields.io/crates/v/limudim-calendar.svg)](https://crates.io/crates/limudim-calendar)               | Jewish learning schedules: Daf Yomi, Mishna Yomis, Pirkei Avos, and more                        |
| [embedded-tz](./crates/embedded-tz)                         | --                                                                                                                            | `no_std` `chrono::TimeZone` for TZif data, with bundled IANA tzdb and over-the-wire updates     |

## License

See each crate for its respective license.

## Acknowledgments

- **[KosherJava](https://github.com/KosherJava/zmanim)** by Eliyahu Hershfeld — The reference implementation for zmanim and Hebrew calendar calculations
- **[ICU4X](https://github.com/unicode-org/icu4x)** — Provides the underlying Hebrew calendar engine via `icu_calendar`
- **[freespa](https://github.com/IEK-5/freespa)** — The C solar position algorithm that `astronomical-calculator` is ported from
- **[tzfile](https://crates.io/crates/tzfile)** — The basis for the `embedded-tz` TZif parser
