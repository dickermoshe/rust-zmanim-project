from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

from dsl import (
    DOCS,
    ZMAN,
    HalfDayBasedOffset,
    LocalMeanTime,
    MinchaGedola,
    MinchaKetana,
    Offset,
    PlagHamincha,
    SamuchLeMinchaKetana,
    Shema,
    SofZmanAchilasChametz,
    SofZmanBiurChametz,
    SunriseOffsetByDegrees,
    SunsetOffsetByDegrees,
    Tefila,
    Zman,
    ZmanisOffset,
)

SCRIPT_DIR = Path(__file__).parent
OUTPUT = SCRIPT_DIR.parent / "src" / "zmanim" / "presets.rs"

NAMES = {
    "getSunrise": "ELEVATION_ADJUSTED_SUNRISE",
    "getSunset": "ELEVATION_ADJUSTED_SUNSET",
}


def method_to_const(method_name: str) -> str:
    if not method_name.startswith("get"):
        raise ValueError(f"Expected Java getter name, got {method_name!r}")

    name = method_name[3:]
    with_underscores = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    with_underscores = re.sub(r"([a-zA-Z])(\d)", r"\1_\2", with_underscores)
    with_underscores = re.sub(r"(\d)([a-zA-Z])", r"\1_\2", with_underscores)
    return with_underscores.upper()


def validate_docs(docs: object) -> dict[str, str]:
    if not isinstance(docs, dict):
        raise ValueError("dsl.DOCS must be a dictionary")
    if not docs:
        raise ValueError("No docs found in dsl.DOCS")

    parsed: dict[str, str] = {}
    for method_name, text in docs.items():
        if not isinstance(method_name, str) or not isinstance(text, str):
            raise ValueError("dsl.DOCS keys and values must be strings")
        if not text.strip():
            raise ValueError(f"{method_name} has empty docs")
        parsed[method_name] = text
    return parsed


def snake_to_pascal(value: str) -> str:
    return "".join(part.capitalize() for part in value.split("_"))


def rust_float(value: float) -> str:
    return repr(float(value))


def rust_bool(value: bool) -> str:
    return "true" if value else "false"


def rust_duration(seconds: float) -> str:
    millis = round(seconds * 1000)
    if abs(seconds * 1000 - millis) > 1e-9:
        raise ValueError(
            f"Duration must be representable in whole milliseconds: {seconds}"
        )
    if millis % 60_000 == 0:
        return f"Duration::from_mins({millis // 60_000})"
    return f"Duration::from_millis({millis})"


def rust_ref(primitive: object) -> str:
    return f"&{rust_primitive(primitive)}"


def rust_primitive(primitive: object) -> str:
    type_name = getattr(primitive, "type_", None)
    if not isinstance(type_name, str):
        raise TypeError(f"Expected DSL primitive with type_, got {primitive!r}")

    if isinstance(primitive, (SunriseOffsetByDegrees, SunsetOffsetByDegrees)):
        return (
            f"ZmanPrimitive::{snake_to_pascal(type_name)}"
            f"({rust_float(primitive.degrees)})"
        )
    if isinstance(primitive, LocalMeanTime):
        return f"ZmanPrimitive::LocalMeanTime({rust_float(primitive.hour)})"
    if isinstance(primitive, Offset):
        return (
            "ZmanPrimitive::Offset("
            f"{rust_ref(primitive.base)}, {rust_duration(primitive.duration_secs)}"
            ")"
        )
    if isinstance(primitive, ZmanisOffset):
        return (
            "ZmanPrimitive::ZmanisOffset("
            f"{rust_ref(primitive.base)}, {rust_float(primitive.hours)}"
            ")"
        )
    if isinstance(primitive, HalfDayBasedOffset):
        return (
            "ZmanPrimitive::HalfDayBasedOffset("
            f"{rust_ref(primitive.start)}, {rust_ref(primitive.end)}, "
            f"{rust_float(primitive.fraction)}"
            ")"
        )
    if isinstance(
        primitive,
        (
            Shema,
            MinchaGedola,
            SamuchLeMinchaKetana,
            MinchaKetana,
            Tefila,
            PlagHamincha,
            SofZmanAchilasChametz,
            SofZmanBiurChametz,
        ),
    ):
        return (
            f"ZmanPrimitive::{snake_to_pascal(type_name)}("
            f"{rust_ref(primitive.start)}, {rust_ref(primitive.end)}, "
            f"{rust_bool(primitive.synchronous)}"
            ")"
        )
    return f"ZmanPrimitive::{snake_to_pascal(type_name)}"


def rust_doc_comment(text: str) -> str:
    """Format user-facing text as Rust line doc comments."""
    lines = text.strip().splitlines() or ["Generated zman preset."]
    return "\n".join(f"/// {line}" if line else "///" for line in lines)


def rust_format_template(text: str) -> str:
    escaped = text.replace("{", "{{").replace("}", "}}")
    for placeholder in ("candel_lighting_offset", "ateret_torah_offset"):
        escaped = escaped.replace(f"{{{{{placeholder}}}}}", f"{{{placeholder}}}")
    return json.dumps(escaped, ensure_ascii=False)


def description_callback(description: str) -> str:
    placeholders = {
        "candel_lighting_offset": "{candel_lighting_offset}" in description,
        "ateret_torah_offset": "{ateret_torah_offset}" in description,
    }
    if not any(placeholders.values()):
        return f"|_| {json.dumps(description, ensure_ascii=False)}.to_string()"

    args: list[str] = []
    if placeholders["candel_lighting_offset"]:
        args.append(
            "candel_lighting_offset = "
            "format_minutes(calculator.config.candle_lighting_offset)"
        )
    if placeholders["ateret_torah_offset"]:
        args.append(
            "ateret_torah_offset = "
            "format_minutes(calculator.config.ateret_torah_sunset_offset)"
        )

    return (
        f"|calculator| format!(\n"
        f"        {rust_format_template(description)},\n"
        f"        {',\n        '.join(args)},\n"
        f"    )"
    )


def preset_block(
    const_name: str,
    method_name: str,
    event: str,
    zman_type: str,
    preset_name: str,
    description: str,
    deprecated: bool,
) -> str:
    name_literal = json.dumps(preset_name, ensure_ascii=False)
    doc = rust_doc_comment(description)
    description_expr = description_callback(description)
    return f"""{doc}
pub static {const_name}: ZmanPreset = ZmanPreset {{
    event: {event},
    zman_type: ZmanType::{snake_to_pascal(zman_type)},
    method_name: {json.dumps(method_name)},
    name: {name_literal},
    #[cfg(feature = "alloc")]
    description: {description_expr},
    deprecated: {rust_bool(deprecated)},
}};
"""


def all_presets_array(presets: list[tuple[str, bool]]) -> str:
    entries = ",\n".join(f"    &{const_name}" for const_name, _ in presets)
    return f"""/// Every generated zman preset.
pub static ALL_ZMANIM: &[&ZmanPreset] = &[
{entries},
];
"""


def for_all_zman_presets_macro(presets: list[tuple[str, bool]]) -> str:
    invocations = "\n        ".join(f"$callback!({const_name});" for const_name, _ in presets)
    return f"""/// Invokes `$callback!(PRESET_IDENT)` once per generated preset static.
///
/// Use this from test crates to expand one `#[test]` (or test module) per preset so
/// `cargo test` can run them in parallel. Iterating [`ALL_ZMANIM`] at runtime only
/// produces a single test function.
#[macro_export]
macro_rules! for_all_zman_presets {{
    ($callback:ident) => {{
        {invocations}
    }};
}}
"""


def generate(docs: dict[str, str]) -> str:
    presets: list[tuple[str, bool, str]] = []
    seen_consts: set[str] = set()
    zman_by_id = {zman.id: zman for zman in ZMAN}

    if len(zman_by_id) != len(ZMAN):
        raise ValueError("ZMAN contains duplicate ids")

    missing_docs = [zman.id for zman in ZMAN if zman.id not in docs]
    if missing_docs:
        raise ValueError(f"Missing docs for DSL methods: {missing_docs}")

    unknown_docs = [
        method_name for method_name in docs if method_name not in zman_by_id
    ]
    if unknown_docs:
        raise ValueError(f"Docs exist for methods missing from DSL: {unknown_docs}")

    for zman in ZMAN:
        method_name = zman.id
        if not isinstance(zman, Zman):
            raise TypeError(f"{method_name} is not a DSL Zman")
        if zman.zman is None:
            raise ValueError(f"{method_name} has no DSL primitive")

        if method_name in NAMES:
            const_name = NAMES[method_name]
        else:
            const_name = method_to_const(method_name)
        if const_name in seen_consts:
            raise ValueError(f"Duplicate preset constant {const_name}")
        seen_consts.add(const_name)

        presets.append(
            (
                const_name,
                zman.deprecated,
                preset_block(
                    const_name,
                    method_name,
                    rust_primitive(zman.zman),
                    zman.type_,
                    zman.name,
                    docs[method_name],
                    zman.deprecated,
                ),
            )
        )

    if not presets:
        raise RuntimeError("No presets were generated; add methods to the DSL.")

    presets.sort(key=lambda preset: preset[0])
    const_names = [(const_name, deprecated) for const_name, deprecated, _ in presets]
    blocks = [block for _, _, block in presets]

    header = """//! Generated by tools/generate-rust.py. Do not edit by hand.

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::format;
#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

use crate::zmanim::{ZmanPreset, ZmanType};
use crate::zmanim::primitives::ZmanPrimitive;
use jiff::SignedDuration as Duration;

#[cfg(feature = "alloc")]
fn format_minutes(duration: Duration) -> String {
    let mins = duration.as_mins().unsigned_abs();
    format!("{mins} minutes")
}

"""
    return (
        header
        + "\n".join(blocks)
        + "\n"
        + all_presets_array(const_names)
        + "\n"
        + for_all_zman_presets_macro(const_names)
        + "\n"
    )


def main() -> None:
    docs = validate_docs(DOCS)
    OUTPUT.write_text(generate(docs), encoding="utf-8", newline="\n")
    print(f"Wrote presets to {OUTPUT}.")
    subprocess.run(["cargo", "fmt"], cwd=SCRIPT_DIR.parent, check=True)


if __name__ == "__main__":
    main()
