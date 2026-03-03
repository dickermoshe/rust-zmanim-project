# Limudim

A Rust library for calculating various Jewish learning schedules (limudim), including Daf Yomi, Mishna Yomis, Pirkei Avos, and more.

> [!NOTE]
> This crate focuses exclusively on Jewish learning schedules calculations. For related functionality, see:
>
> - [hebrew_holiday_calendar](https://github.com/dickermoshe/hebrew_holiday_calendar) -- Hebrew holidays
> - [astronomical-calculator](https://github.com/dickermoshe/astronomical-calculator) -- General astronomical events
> - [zmanim-calculator](https://github.com/dickermoshe/zmanim-calculator) -- Zmanim calculations

## Features

This library provides calculators for the following learning schedules:

- **Daf Yomi Bavli** - Babylonian Talmud daily page (7.5 year cycle)
- **Daf Yomi Yerushalmi** - Jerusalem Talmud daily page (Vilna edition, ~4 year cycle)
- **Amud Yomi Bavli Dirshu** - Babylonian Talmud daily column (Dirshu schedule)
- **Daf Hashavua Bavli** - Babylonian Talmud weekly page
- **Mishna Yomis** - Daily Mishna (2 mishnayos per day)
- **Pirkei Avos** - Ethics of the Fathers (seasonal, Shabbos afternoons between Pesach and Rosh Hashanah)
- **Tehillim Monthly** - Complete book of Psalms monthly

## Installation

```bash
cargo add limudim- icu_calendar
```

Or manually add to your `Cargo.toml`:

```toml
[dependencies]
limudim-calendar = "0.1.0"
icu_calendar = "1.5"
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
```

### Other Calculators

```rust
use limudim_calendar::*;

// Mishna Yomis
let mishnas = date.limud(MishnaYomis::default());

// Pirkei Avos (Israel schedule)
let pirkei_avos = date.limud(PirkeiAvos::new(true));

// Tehillim Monthly
let tehillim = date.limud(TehillimMonthly::default());
```

## License

Licensed under the LGPL-2.1 license. See [LICENSE](LICENSE) for details.
