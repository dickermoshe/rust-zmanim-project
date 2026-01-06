# Hebrew Holiday Calendar

A comprehensive Rust library for working with the Hebrew calendar, including holidays, Torah readings (parshiyot), and calendar calculations. This library provides `no_std` support and can be used in embedded environments.

## Features

- 📅 **Hebrew Calendar Operations**: Convert between Gregorian and Hebrew dates
- 🎉 **Holiday Tracking**: Determine Jewish holidays with customizable rules
- 📖 **Torah Readings**: Find weekly parshiyot for any date
- ⭐ **Special Shabbatot**: Identify Shekalim, Zachor, Parah, Hachodesh, and more
- 🌍 **Israel & Diaspora Support**: Respects location-specific customs
- 🕯️ **Candle Lighting**: Determine days requiring candle lighting
- ⚡ **Fast & Efficient**: Optimized calendar calculations
- 🔧 **No Std Compatible**: Works in embedded environments

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hebrew_holiday_calendar = "0.1.0"
```

For `no_std` environments:

```toml
[dependencies]
hebrew_holiday_calendar = { version = "0.1.0", default-features = false }
```

## Usage Examples

### Basic Date Creation and Conversion

```rust
use hebrew_holiday_calendar::*;
use icu_calendar::{Date, cal::Hebrew};

// Create a Hebrew date: 1 Tishrei 5784
let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 1).unwrap();

// Convert to Gregorian
let gregorian = date.gregorian_date();
println!("Gregorian: {}", gregorian); // 2023-09-16

// Get day of week
let day_of_week = date.chrono_day_of_week();
println!("Day: {:?}", day_of_week);
```

### Finding Holidays

```rust
use hebrew_holiday_calendar::*;
use icu_calendar::{Date, cal::Hebrew};

let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 1).unwrap();

// Get all holidays on this date
for holiday in date.holidays(false, false) {
    println!("Holiday: {}", holiday); // "Rosh Hashana"
}

// Check if work is forbidden
if date.is_assur_bemelacha(false) {
    println!("Work is forbidden today");
}

// Check if candle lighting is required
if date.has_candle_lighting(false) {
    println!("Light candles tonight");
}
```

### Torah Readings (Parshiyot)

```rust
use hebrew_holiday_calendar::*;
use icu_calendar::{Date, cal::Hebrew};

let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Tishrei, 21).unwrap();

// Get the weekly parsha (only on Shabbat)
if let Some(parsha) = date.parsha(false) {
    println!("Parsha: {}", parsha.en()); // English name
    println!("פרשה: {}", parsha.he());   // Hebrew name
}

// Get special parsha designation
if let Some(special) = date.special_parsha(false) {
    println!("Special Shabbat: {}", special);
}

// Get the upcoming parsha (even if today isn't Shabbat)
let upcoming = date.upcoming(false);
println!("Upcoming parsha: {}", upcoming);
```

### Calendar Calculations

```rust
use hebrew_holiday_calendar::*;
use icu_calendar::{Date, cal::Hebrew};

// Check if a year is a leap year
let is_leap = Date::<Hebrew>::is_hebrew_leap_year(5784);
println!("Is leap year: {}", is_leap);

// Get number of days in a year
let days = Date::<Hebrew>::days_in_hebrew_year(5784);
println!("Days in year: {}", days);

// Get number of days in a month
let days_in_month = Date::<Hebrew>::days_in_hebrew_month(5784, HebrewMonth::Tishrei);
println!("Days in Tishrei: {}", days_in_month);

// Check year type (Chaserim/Kesidran/Shelaimim)
let kviah = Date::<Hebrew>::cheshvan_kislev_kviah(5784);
println!("Year type: {:?}", kviah);

// Check if Cheshvan is long (30 days)
let is_long = Date::<Hebrew>::is_cheshvan_long(5784);

// Check if Kislev is short (29 days)
let is_short = Date::<Hebrew>::is_kislev_short(5784);
```

### Israel vs. Diaspora

```rust
use hebrew_holiday_calendar::*;
use icu_calendar::{Date, cal::Hebrew};

// Second day of Pesach
let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Nissan, 16).unwrap();

// In diaspora, this is Yom Tov
let holidays_diaspora: Vec<_> = date.holidays(false, false).collect();
println!("Diaspora: {:?}", holidays_diaspora); // [Pesach]

// In Israel, this is Chol Hamoed
let holidays_israel: Vec<_> = date.holidays(true, false).collect();
println!("Israel: {:?}", holidays_israel); // [CholHamoedPesach]
```

### Modern Israeli Holidays

```rust
use hebrew_holiday_calendar::*;
use icu_calendar::{Date, cal::Hebrew};

let date = Date::<Hebrew>::from_hebrew_date(5784, HebrewMonth::Iyar, 5).unwrap();

// Include modern holidays
for holiday in date.holidays(true, true) {
    println!("Holiday: {}", holiday); // "Yom Ha'atzmaut"
}

// Exclude modern holidays
let count = date.holidays(true, false).count();
println!("Traditional holidays only: {}", count);
```

### Working with Hebrew Months

```rust
use hebrew_holiday_calendar::HebrewMonth;

let month = HebrewMonth::Tishrei;

println!("English: {}", month.en()); // "Tishrei"
println!("Hebrew: {}", month.he());  // "תשרי"
println!("Display: {}", month);      // "Tishrei" (uses Display trait)

// Month arithmetic
let month_num: u8 = HebrewMonth::Nissan.into();
println!("Nissan is month {}", month_num); // 1
```

## Holiday Types

The library includes:

### Major Holidays

- Rosh Hashana, Yom Kippur
- Succos, Shemini Atzeres, Simchas Torah
- Chanukah
- Purim, Shushan Purim
- Pesach, Shavuos

### Fast Days

- Fast of Gedalyah
- Tenth of Teves
- Fast of Esther
- Seventeenth of Tammuz
- Tisha B'Av

### Modern Israeli Holidays

- Yom HaShoah
- Yom Hazikaron
- Yom Ha'atzmaut
- Yom Yerushalayim

### Special Days

- Rosh Chodesh
- Lag BaOmer
- Tu B'Shvat, Tu B'Av
- Isru Chag
- Yom Kippur Katan
- BaHaB (fast days)

## Torah Readings

The library supports:

- **Weekly Parshiyot**: All 54 Torah portions
- **Combined Readings**: For non-leap years (e.g., Vayakhel-Pekudei)
- **Special Shabbatot**: Shekalim, Zachor, Parah, Hachodesh
- **Holiday-Adjacent**: Shabbat Hagadol, Shabbat Shuva, Shabbat Chazon, Shabbat Nachamu
- **Israel vs. Diaspora**: Different reading schedules when holidays differ

## no_std Support

This library can be used in `no_std` environments:

```toml
[dependencies]
hebrew_holiday_calendar = { version = "0.1.0", default-features = false }
```

All core functionality works without the standard library. Some error messages may be less detailed in `no_std` mode.

## Optional Features

- `std`: Enables standard library support (enabled by default)
- `defmt`: Enables formatting support for embedded logging

## Implementation Details

### Calendar System

The library uses the ICU calendar system (`icu_calendar`) for accurate Hebrew date calculations and conversions. All Hebrew calendar rules are implemented according to traditional Jewish law:

- Molad calculations
- Dechiyot (postponement rules)
- Leap year cycles (19-year Metonic cycle)
- Variable month lengths (Cheshvan and Kislev)

### Performance

- Zero-cost abstractions
- Efficient iterator-based holiday filtering
- Compile-time constant parsha tables
- Minimal allocations (no heap usage in no_std mode)

## License

[Add your license here]

## Contributing

Contributions are welcome! Please ensure:

- Code follows Rust idioms
- All tests pass
- New features include documentation
- Breaking changes are clearly marked

## Acknowledgments

This library builds upon:

- [icu_calendar](https://github.com/unicode-org/icu4x) for core calendar operations
- Traditional Jewish calendar algorithms from various sources
