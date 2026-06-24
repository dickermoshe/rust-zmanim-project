# kosher-rust Diplomat interop

Multi-language FFI bridge for [`kosher-rust`](../README.md), built with [Diplomat](https://rust-diplomat.github.io/diplomat/).

## Conventions

| Concept | FFI representation |
|---------|-------------------|
| Civil date | `{ year: i32, month: u8, day: u8 }` |
| Hebrew date | `{ year: i32, month: u8, day: u8 }` — month codes 1–12 (Tishrei–Elul), 25 = Adar I |
| UTC instant | `i64` epoch **milliseconds** |
| Timezone | IANA string; empty string means none |
| Zman preset | `ZmanPresetId` enum (generated) or `u32` index via `calculate_zman_by_index` |
| Pirkei Avos / Tehillim | flat struct with `kind` discriminant |

## Modules

| Bridge | File | Purpose |
|--------|------|---------|
| Calendar | [`src/calendar.rs`](src/calendar.rs) | Date conversion, holidays, parsha, year helpers |
| Zmanim | [`src/zmanim.rs`](src/zmanim.rs) | Location, calculator, config, zman calculation |
| Zman presets | [`src/zman_preset.rs`](src/zman_preset.rs) | Generated preset enum + metadata (167 presets) |
| Limudim | [`src/limudim.rs`](src/limudim.rs) | Daf Yomi, Mishna Yomis, Pirkei Avos, etc. |

## Regeneration

Preset IDs and dispatch tables are generated from the same DSL as [`src/zmanim/presets.rs`](../src/zmanim/presets.rs):

```bash
cd tools
uv run python generate-interop.py
```

This runs automatically from [`build.rs`](build.rs) on every `cargo build -p interop`.

Verify generated output in CI:

```bash
cd tools && uv run python generate-interop.py
git diff --exit-code interop/src/generated interop/src/zman_preset.rs
```

## Building bindings

1. Build the Rust crate:

```bash
cargo build -p interop
```

2. Install `diplomat-tool` (match crate version 0.15.x):

```bash
cargo install diplomat-tool --version 0.15.0
```

3. Generate bindings from the `interop/` directory:

```bash
cd interop
diplomat-tool gen -l c      --config config.toml
diplomat-tool gen -l cpp    --config config.toml
diplomat-tool gen -l js     --config config.toml
diplomat-tool gen -l kotlin --config config.toml
```

Output is written under [`bindings/`](bindings/) (gitignored until you choose to commit it).

Configuration lives in [`config.toml`](config.toml).

## Testing

Rust-side adapter tests (no generated bindings required):

```bash
cargo test -p interop
```

Existing wasm-bindgen and JNI parity tests remain unchanged; they exercise `kosher-rust` directly.

## Example flow (conceptual)

```text
1. Create FfiLocation(lat, lon, elev, "Asia/Jerusalem")
2. Create FfiZmanimCalculator(location, date, config)
3. Look up preset index from ZmanPresetId or preset_method_name()
4. calculate_zman_by_index(preset_index) -> epoch ms UTC
5. Convert to local time in the host language using the IANA timezone
```

## Design notes

- Bridge modules use only scalars, copy structs, opaque boxes, and plain enums — no traits, generics, or `ZmanPrimitive` at the FFI boundary.
- Custom zman definitions remain Rust-only; hosts use the generated preset enum.
- Molad / Kiddush Levana and preset `description()` text are not yet exposed over FFI.
