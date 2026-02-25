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

## Getting started

Add the dependency (git or path) and enable `bundled-tzdb` if you want a
compile-time copy of the IANA tz database included in your binary:

```toml
[dependencies]
embedded-tz = { git = "https://github.com/dickermoshe/embedded-tz", features = ["bundled-tzdb"] }
```

## Usage

Every parsed time zone is reference-counted via a `heapless` arc pool.
You must set up that pool **once** at startup, before calling any parsing
function. The pool size determines how many `Tz` values you can hold
simultaneously.

### 1. Initialize the pool

```rust
use core::ptr::addr_of_mut;
use heapless::pool::arc::ArcBlock;
use embedded_tz::{TzData, TzDataArcPool};

const TZ_POOL_SIZE: usize = 16;
const TZ_DATA_BLOCK: ArcBlock<TzData> = ArcBlock::new();
static mut TZ_DATA_BLOCKS: [ArcBlock<TzData>; TZ_POOL_SIZE] =
    [TZ_DATA_BLOCK; TZ_POOL_SIZE];

fn init_tz_pool() {
    let blocks = unsafe { addr_of_mut!(TZ_DATA_BLOCKS).as_mut().unwrap() };
    for block in blocks {
        TzDataArcPool.manage(block);
    }
}
```

### 2. Parse a time zone

From the bundled database (requires `bundled-tzdb` feature):

```rust
use embedded_tz::bundled;

let tz = bundled::parse("America/New_York")?;
```

From raw TZif bytes received over the wire:

```rust
use embedded_tz::Tz;

let tz = Tz::parse("America/New_York", &raw_tzif_bytes)?;
```

Or create a fixed-offset zone directly:

```rust
use chrono::FixedOffset;
use embedded_tz::Tz;

let tz = Tz::from_offset(FixedOffset::east_opt(2 * 3600).unwrap())?;
```

### 3. Convert times

`Tz` implements `chrono::TimeZone`, so it works with the standard chrono API:

```rust
use chrono::{TimeZone, Utc};

let utc = Utc.with_ymd_and_hms(2026, 3, 8, 12, 0, 0).unwrap();
let local = utc.with_timezone(&&tz);
```

## Updating time zones over the wire

The main reason to use this crate instead of `chrono-tz` is to push tz database
updates to a device without a full firmware update. A typical flow:

1. **Server side** -- Build or download a tzdb release. For each zone you need,
   extract the compiled TZif file (the binary blobs in `/usr/share/zoneinfo`).
2. **Validate** -- Parse every payload with the same `embedded-tz` version
   running on the device.
3. **Transmit** -- Send the validated TZif bytes and their IANA identifiers
   (e.g. `"America/New_York"`) to the device.
4. **Device side** -- Call `Tz::parse(name, &bytes)` to load each zone into the
   arc pool.

Use `bundled::version()` to check which tzdb version is compiled into the
binary. This is useful for deciding whether a wire update is needed.

## Limitations

- **Fixed pool capacity.** You can only hold as many `Tz` values as the arc pool
  you allocated at startup. Once the pool is exhausted, parsing returns
  `Error::AllocationFailed`.
- **No far-future POSIX tail rules.** Unlike `chrono-tz`, this crate does not
  expand the POSIX TZ string appended to TZif files. Offsets past the last
  explicit transition in the file repeat the final transition's offset
  indefinitely. For most practical use (within ~10 years of the tzdb release)
  this is equivalent.

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
