# kosher-rust Diplomat interop

Multi-language FFI bridge for [`kosher-rust`](../README.md), built with [Diplomat](https://rust-diplomat.github.io/diplomat/).

## Regeneration

Preset IDs and dispatch tables are generated from the same DSL as [`src/zmanim/presets.rs`](../src/zmanim/presets.rs):

```bash
cd tools
uv run python generate-interop.py
```

This runs automatically from [`build.rs`](build.rs) on every `cargo build -p interop`.

## TypeScript / JavaScript package

Bindings live in [`packages/typescript`](packages/typescript/). Regenerate and test:

```bash
cd interop/packages/typescript
npm install
npm run generate          # diplomat-tool js → lib/
npm run build:wasm        # cargo build -p interop --target wasm32-unknown-unknown
npm test
```

WASM path is configured in [`packages/typescript/diplomat.config.mjs`](packages/typescript/diplomat.config.mjs) (relative to the repo `target/` directory).

## Dart package

See [`packages/dart`](packages/dart/) — run `dart pub get && dart test` after building the native library via the package hook.


