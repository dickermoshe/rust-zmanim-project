$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

cargo run --features c --bin build_c_headers
cargo zigbuild --target x86_64-pc-windows-gnu --features c --release

$source = Join-Path $PSScriptRoot "main.c"
$headerDir = Join-Path $repoRoot "bindings/c"
$staticLib = Join-Path $repoRoot "target/x86_64-pc-windows-gnu/release/libzmanim_calculator.a"
$output = Join-Path $PSScriptRoot "example_c_project.exe"

& zig cc -target x86_64-windows-gnu -std=c11 -I "$headerDir" "$source" "$staticLib" `
    -lws2_32 -luserenv -lkernel32 -ladvapi32 -lntdll -luser32 `
    -lgcc -lgcc_eh -lmingw32 -lmingwex -lpthread `
    -o "$output"

& "$output"
