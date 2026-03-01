#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

find_bin() {
  local name="$1"
  if command -v "$name" >/dev/null 2>&1; then
    command -v "$name"
    return 0
  fi
  if command -v "${name}.exe" >/dev/null 2>&1; then
    command -v "${name}.exe"
    return 0
  fi
  if [ -x "/c/Users/${USERNAME}/.cargo/bin/${name}.exe" ]; then
    echo "/c/Users/${USERNAME}/.cargo/bin/${name}.exe"
    return 0
  fi
  return 1
}

CARGO_BIN="$(find_bin cargo)"
ZIG_BIN="$(find_bin zig)"

"$CARGO_BIN" run --features c --bin build_c_headers
"$CARGO_BIN" zigbuild --target x86_64-pc-windows-gnu --features c --release

source_file="$repo_root/example-c-project/main.c"
header_dir="$repo_root/bindings/c"
static_lib="$repo_root/target/x86_64-pc-windows-gnu/release/libzmanim_calculator.a"
output="$repo_root/example-c-project/example_c_project.exe"

"$ZIG_BIN" cc -target x86_64-windows-gnu -std=c11 -I "$header_dir" "$source_file" "$static_lib" \
  -lws2_32 -luserenv -lkernel32 -ladvapi32 -lntdll -luser32 \
  -lgcc -lgcc_eh -lmingw32 -lmingwex -lpthread \
  -o "$output"

"$output"
