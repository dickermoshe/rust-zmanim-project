#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo run --features c --bin build_c_headers
cargo build --release --features c

zig cc \
    example-c-project/main.c \
    -Ltarget/release \
    -lzmanim_calculator \
    -lm -lpthread -ldl \
    -o example-c-project/example

./example-c-project/example
