> This crate is part of the [Rust Zmanim Project](TODO).

# Embedded Tz

[![codecov](https://codecov.io/gh/dickermoshe/rust-zmanim-project/graph/badge.svg?flag=embedded-tz)](https://codecov.io/gh/dickermoshe/rust-zmanim-project)

A `no_std` `chrono::TimeZone` implementation for TZif data, aimed at embedded deployments. Forked from the `tzfile` crate with added support for embedded environments and over-the-wire timezone updates.

## When to Use This Crate

Use this crate when all of the following are true:

- You run in an embedded / `no_std` environment.
- You need to deliver time zone database updates over the wire.
- You cannot (or do not want to) rely on full firmware OTA flows from your chip vendor or platform provider.

Most users should not use this crate directly. If you can do normal application or firmware updates, it is usually simpler to use `chrono-tz`, rebuild with the latest tzdb, and ship via your standard deployment path.

## Installation

Add the dependency and enable `bundled-tzdb` if you want a compile-time copy of the IANA tz database included in your binary:

```toml
[dependencies]
embedded-tz = { git = "https://github.com/dickermoshe/embedded-tz", features = ["bundled-tzdb"] }
```

## Usage

`Tz` does not implement `TimeZone` directly because it can be large and expensive to clone. Instead, use one of:

- `&Tz` — zero-cost to clone, bounded by a lifetime
- `RcTz` — reference-counted via `Rc`, not thread-safe
- `ArcTz` — atomically reference-counted via `Arc`, thread-safe

### Parse a time zone

From the bundled database (requires `bundled-tzdb` feature):

```rust
use embedded_tz::Tz;

let tz = Tz::named("America/New_York")?;
```

From raw TZif bytes received over the wire:

```rust
use embedded_tz::Tz;

let tz = Tz::parse("America/New_York", &raw_tzif_bytes)?;
```

From a fixed offset or UTC:

```rust
use embedded_tz::Tz;

let tz = Tz::from(chrono::Utc);
let tz = Tz::from(chrono::FixedOffset::east_opt(2 * 3600).unwrap());
```

### Convert times

Use `&Tz`, `RcTz`, or `ArcTz` as a `chrono::TimeZone`:

```rust
use chrono::{TimeZone, Utc};
use embedded_tz::Tz;

let tz = Tz::named("America/New_York")?;
let utc = Utc.with_ymd_and_hms(2026, 3, 8, 12, 0, 0).unwrap();
let local = utc.with_timezone(&&tz);
```

## Updating Time Zones Over the Wire

The main reason to use this crate instead of `chrono-tz` is to push tz database updates to a device without a full firmware update. A typical flow:

1. **Server side** — Build or download a tzdb release. For each zone you need, extract the compiled TZif file (the binary blobs in `/usr/share/zoneinfo`).
2. **Validate** — Parse every payload with the same `embedded-tz` version running on the device.
3. **Transmit** — Send the validated TZif bytes and their IANA identifiers (e.g. `"America/New_York"`) to the device.
4. **Device side** — Call `Tz::parse(name, &bytes)` to load each zone.

Use `bundled::version()` to check which tzdb version is compiled into the binary.

## Limitations

- **No far-future POSIX tail rules** — Unlike `chrono-tz`, this crate does not expand the POSIX TZ string appended to TZif files. Offsets past the last explicit transition repeat the final transition's offset indefinitely. For most practical use (within ~10 years of the tzdb release) this is equivalent.

## Publishing Policy

This fork is not intended to be published to crates.io. If you want to use it in production, fork the repository and publish under your own ownership.

## License

Licensed under MIT. See [LICENSE](LICENSE.txt) for details.
