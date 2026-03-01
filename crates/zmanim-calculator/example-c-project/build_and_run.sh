#!/usr/bin/env bash
# Linux only.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo run --features c --bin build_c_headers
cargo zigbuild --features c --release

source_file="$repo_root/example-c-project/main.c"
header_dir="$repo_root/bindings/c"
static_lib="$repo_root/target/release/libzmanim_calculator.a"
output="$repo_root/example-c-project/example_c_project"

zig cc -std=c11 -I "$header_dir" "$source_file" "$static_lib" -lpthread -o "$output"

"$output"
