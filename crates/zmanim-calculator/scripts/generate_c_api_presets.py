#!/usr/bin/env python3
"""Generate Interoptopus preset wrapper functions for src/c_api.rs.

Usage:
    uv run python scripts/generate_c_api_presets.py
    uv run python scripts/generate_c_api_presets.py --apply
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


WORKSPACE = Path(__file__).resolve().parent.parent
PRESETS_RS = WORKSPACE / "src" / "presets.rs"
C_API_RS = WORKSPACE / "src" / "c_api.rs"
GEN_HEADER_PREFIX = "// Generated from "
FIRST_INVOCATION = "preset_time_fn!(sunrise, SUNRISE);"
FIRST_FUNCTION = 'pub extern "C" fn sunrise('


def escape_doc_line(doc_line: str) -> str:
    return doc_line.replace("\\", "\\\\").replace('"', '\\"')


def extract_presets(text: str) -> list[tuple[str, list[str]]]:
    pattern = re.compile(
        r"(?P<docs>(?:(?:^///[^\n]*\n)+)?)^pub const (?P<name>[A-Z0-9_]+):",
        flags=re.MULTILINE,
    )
    presets: list[tuple[str, list[str]]] = []
    for match in pattern.finditer(text):
        name = match.group("name")
        docs_blob = match.group("docs") or ""
        docs = [
            line.removeprefix("///").lstrip()
            for line in docs_blob.splitlines()
            if line.startswith("///")
        ]
        presets.append((name, docs))
    return presets


def render_functions(presets: list[tuple[str, list[str]]]) -> str:
    lines = [f"// Generated from {PRESETS_RS.as_posix()} ({len(presets)} presets)"]
    lines.append(
        "fn calculate_preset_timestamp<T>(calculator: &ZmanimCalculator, preset: T) -> FFIOption<i64>"
    )
    lines.append("where")
    lines.append("    T: crate::calculator::ZmanLike<FixedOffset>,")
    lines.append("{")
    lines.append(
        "    let Some(mut rust_calculator) = calculator.into_rust_calculator() else {"
    )
    lines.append("        return FFIOption::none();")
    lines.append("    };")
    lines.append("")
    lines.append("    rust_calculator")
    lines.append("        .calculate(preset)")
    lines.append("        .map(|datetime| datetime.timestamp())")
    lines.append("        .ok()")
    lines.append("        .into()")
    lines.append("}")
    lines.append("")

    for name, docs in presets:
        if docs:
            for doc in docs:
                lines.append(f'#[doc = "{escape_doc_line(doc)}"]')
        else:
            lines.append(f'#[doc = "Calculates preset `{name}`."]')
        lines.append("#[no_mangle]")
        lines.append("#[ffi_function]")
        lines.append(
            f'pub extern "C" fn {name.lower()}(calculator: &ZmanimCalculator) -> FFIOption<i64> {{'
        )
        lines.append(
            f"    calculate_preset_timestamp(calculator, crate::presets::{name})"
        )
        lines.append("}")
        lines.append("")

    function_names = ["new_location", "new_calculator"]
    function_names.extend(name.lower() for name, _ in presets)

    lines.append("/// Returns the Interoptopus inventory used to generate foreign bindings.")
    lines.append("pub fn my_inventory() -> Inventory {")
    lines.append("    InventoryBuilder::new()")
    for fn_name in function_names:
        lines.append(f"        .register(function!({fn_name}))")
    lines.append("        .inventory()")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)


def apply_to_c_api(generated_block: str) -> None:
    c_api = C_API_RS.read_text(encoding="utf-8")
    marker_index = c_api.find(GEN_HEADER_PREFIX)
    if marker_index == -1:
        marker_index = c_api.find("macro_rules! preset_time_fn {")
    if marker_index == -1:
        marker_index = c_api.find(FIRST_INVOCATION)
    if marker_index == -1:
        marker_index = c_api.find(FIRST_FUNCTION)
    if marker_index == -1:
        marker_match = re.search(
            r"(?m)^#\[doc = .*?\]\n(?:preset_time_fn!\(\n\s*sunrise,\n\s*SUNRISE\n\);)",
            c_api,
        )
        if marker_match is not None:
            marker_index = marker_match.start()
    if marker_index == -1:
        raise RuntimeError(
            f"Could not find generated preset block marker in {C_API_RS}"
        )
    updated = c_api[:marker_index] + generated_block
    C_API_RS.write_text(updated, encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Apply generated function block directly to src/c_api.rs",
    )
    return parser.parse_args()


def main() -> None:
    sys.stdout.reconfigure(encoding="utf-8")  # type: ignore
    args = parse_args()
    presets = extract_presets(PRESETS_RS.read_text(encoding="utf-8"))
    generated_block = render_functions(presets)
    if args.apply:
        apply_to_c_api(generated_block)
        return
    print(generated_block, end="")


if __name__ == "__main__":
    main()
