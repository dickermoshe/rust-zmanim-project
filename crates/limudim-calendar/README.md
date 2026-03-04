> This crate is part of the [Rust Zmanim Project](https://github.com/dickermoshe/rust-zmanim-project).

# Limudim Calendar

A Rust library for calculating Jewish learning schedules (limudim), including Daf Yomi, Mishna Yomis, Pirkei Avos, and more. Supports `no_std` environments.

[![Crates.io](https://img.shields.io/crates/v/limudim-calendar.svg)](https://crates.io/crates/limudim-calendar)
[![Documentation](https://docs.rs/limudim-calendar/badge.svg)](https://docs.rs/limudim-calendar)
[![codecov](https://codecov.io/gh/dickermoshe/rust-zmanim-project/graph/badge.svg?flag=limudim-calendar)](https://codecov.io/gh/dickermoshe/rust-zmanim-project)

## Supported Schedules

- Daf Yomi Bavli
- Daf Yomi Yerushalmi (Vilna edition)
- Amud Yomi Bavli (Dirshu)
- Daf Hashavua Bavli
- Mishna Yomis
- Pirkei Avos
- Tehillim Monthly

## Installation

```bash
cargo add limudim-calendar icu_calendar
```

Or manually add to your `Cargo.toml`:

```toml
[dependencies]
limudim-calendar = "0.1"
icu_calendar = "2"
```

## Usage

```rust
use limudim_calendar::{DafYomiBavli, LimudCalendar};
use icu_calendar::{cal::Hebrew, Date};

// Create a Hebrew date
let date = Date::try_new_iso(2020, 1, 5).unwrap().to_calendar(Hebrew);

// Calculate what should be learned
let daf = date.limud(DafYomiBavli::default()).unwrap();
println!("Today's Daf: {:?} page {}", daf.tractate, daf.page);

// Mishna Yomis
let mishnas = date.limud(limudim_calendar::MishnaYomis::default());

// Pirkei Avos (Israel schedule)
let pirkei_avos = date.limud(limudim_calendar::PirkeiAvos::new(true));

// Tehillim Monthly
let tehillim = date.limud(limudim_calendar::TehillimMonthly::default());
```

## License

Licensed under LGPL-2.1. See [LICENSE](LICENSE) for details.
