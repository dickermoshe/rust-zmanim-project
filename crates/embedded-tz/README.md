# Embedded Tz

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.txt)

`embedded-tz` is a `no_std` `chrono::TimeZone` implementation for TZif data, aimed at
embedded deployments. It is based on the `tzfile` crate, but has been
forked to add support for embedded deployments.

## When to use this crate

Use this crate when all of the following are true:

- You run in an embedded / `no_std` environment.
- You need to deliver time zone database updates over the wire.
- You cannot (or do not want to) rely on full firmware OTA flows from your chip
  vendor or platform provider.

## When not to use this crate

Most users should not use this crate directly.

If you can do normal application or firmware updates, it is usually simpler to:

- use `chrono-tz`,
- rebuild with the latest tzdb,
- ship the update via your standard deployment path (including vendor OTA
  systems such as those commonly available in Espressif/Nordic ecosystems).

## Current model

- No runtime dependency on system zoneinfo files.
- Optional bundled database via the `bundled-tzdb` feature.
- Parse zones from bundled entries with `embedded_tz::bundled::parse`.

## Guardrails

- Far-future transitions are intentionally conservative:
  `embedded-tz` does not apply the POSIX tail generic-rule expansion used by
  `chrono-tz` for very far-future timestamps.
- Capacity is fixed by your arc-pool allocation:
  you can only parse/hold as many `Tz` values as your preallocated pool supports.
- Before sending tzdb payloads over the wire, validate them with the same
  crate version and the same tzdb version (`bundled::version()`) you expect on
  device.

## Arc pool init and parsing example

You must initialize the `heapless` arc pool once at startup before parsing and
holding `Tz` values.

```rust
use chrono::{TimeZone, Utc};
use core::ptr::addr_of_mut;
use heapless::pool::arc::ArcBlock;
use embedded_tz::{bundled, Tz, TzData, TzDataArcPool};

const TZ_POOL_CAPACITY: usize = 16;
const TZ_DATA_BLOCK: ArcBlock<TzData> = ArcBlock::new();
static mut TZ_DATA_BLOCKS: [ArcBlock<TzData>; TZ_POOL_CAPACITY] =
    [TZ_DATA_BLOCK; TZ_POOL_CAPACITY];

fn init_tz_pool_once() {
    // Call exactly once during boot/init.
    let blocks = unsafe { addr_of_mut!(TZ_DATA_BLOCKS).as_mut().unwrap() };
    for block in blocks {
        TzDataArcPool.manage(block);
    }
}

fn parse_zone(name: &str) -> Result<Tz, embedded_tz::Error> {
    bundled::parse(name)
}

fn demo() -> Result<(), embedded_tz::Error> {
    init_tz_pool_once();
    let tzdb_version = bundled::version();
    let tz = parse_zone("America/New_York")?;
    let dt = Utc.with_ymd_and_hms(2026, 2, 25, 12, 0, 0).unwrap();
    let _local = dt.with_timezone(&&tz);
    let _ = tzdb_version;
    Ok(())
}
```

## Suggested wire-update workflow

1. Build/choose a tzdb version on the server side.
2. Validate each payload parses with the same crate + tzdb version you expect on
   device.
3. Transmit only validated TZif bytes and identifiers.
4. On device, initialize the pool once, then parse/store only up to your
   allocated capacity.

## Testing methodology

The test suite validates behavior in several layers:

- Unit tests validate TZif parsing and conversion behavior in `src/lib.rs`.
- Integration-style tests parse many real zone files and verify conversion
  expectations across known transition edge cases.
- Property tests compare this crate against `chrono-tz` for bundled zones over a
  wide timestamp range:
  from `chrono` minimum supported UTC timestamp up to 10 years from test runtime.
- A fixed regression case checks second-level historical offsets
  (`Europe/Amsterdam`) to catch subtle historical-rule drift.
- Proptest regression artifacts are kept in `proptest-regressions/` so failures
  can be reproduced.

Run the main test suite with bundled data:

```sh
cargo test --features bundled-tzdb
```

## Publishing policy

This fork is not intended to be published by this maintainer to crates.io.
If you want to use it in production, fork this repository and publish/manage it
under your own ownership and release process.

[tz database]: https://en.wikipedia.org/wiki/Tz_database
[`chrono`]: https://github.com/chronotope/chrono
