from __future__ import annotations

import re
import subprocess
from pathlib import Path

from dsl import ZMAN, Zman, ZmanType
from typing import get_args


SCRIPT_DIR = Path(__file__).parent
OUTPUT_DIR = SCRIPT_DIR.parent / "interop" / "src" / "generated"

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


def snake_to_pascal(value: str) -> str:
    return "".join(part.capitalize() for part in value.split("_"))


def ordered_zman_types(presets: list[tuple[str, str, str, bool]]) -> list[str]:
    """DSL order, restricted to types that appear on at least one preset."""
    used = {zman_type for _, _, zman_type, _ in presets}
    return [zman_type for zman_type in get_args(ZmanType) if zman_type in used]


def collect_presets() -> list[tuple[str, str, str, bool]]:
    """Return sorted (const_name, method_name, zman_type, deprecated) tuples."""
    presets: list[tuple[str, str, str, bool]] = []
    seen_consts: set[str] = set()

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
        presets.append((const_name, method_name, zman.type_, zman.deprecated))

    presets.sort(key=lambda preset: preset[0])
    return presets


def generate_zman_preset_bridge(presets: list[tuple[str, str, str, bool]]) -> str:
    variants = "\n        ".join(
        f"{snake_to_pascal(const_name)}," for const_name, _, _, _ in presets
    )
    zman_types = ordered_zman_types(presets)
    type_variants = "\n        ".join(
        f"{snake_to_pascal(zman_type)}," for zman_type in zman_types
    )
    type_match_arms = "\n            ".join(
        f"kosher_rust::zmanim::ZmanType::{snake_to_pascal(zman_type)} => "
        f"ZmanType::{snake_to_pascal(zman_type)},"
        for zman_type in zman_types
    )
    return f"""//! Generated zman preset diplomat bridge.
#![allow(missing_docs, dead_code)]

#[diplomat::bridge]
mod ffi {{
    use core::fmt::Write;

    use diplomat_runtime::DiplomatWrite;

    use crate::generated::preset_dispatch::{{PRESET_METADATA, ZMAN_PRESET_COUNT}};

    /// Stable identifier for a generated zman preset.
    pub enum ZmanPresetId {{
        {variants}
    }}

    /// Broad category of a zman preset.
    pub enum ZmanType {{
        {type_variants}
    }}

    fn to_ffi_zman_type(zman_type: kosher_rust::zmanim::ZmanType) -> ZmanType {{
        match zman_type {{
            {type_match_arms}
        }}
    }}

    /// Returns the number of available presets.
    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn preset_count() -> u32 {{
        u32::try_from(ZMAN_PRESET_COUNT).unwrap_or(u32::MAX)
    }}

    /// Writes the display name for a preset. Returns false when the preset is unknown.
    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn preset_name(preset: ZmanPresetId, write: &mut DiplomatWrite) -> bool {{
        PRESET_METADATA
            .get(preset as usize)
            .map(|meta| write!(write, "{{}}", meta.name).is_ok())
            .unwrap_or(false)
    }}

    /// Returns whether a preset is deprecated.
    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn preset_deprecated(preset: ZmanPresetId) -> bool {{
        PRESET_METADATA
            .get(preset as usize)
            .is_some_and(|meta| meta.deprecated)
    }}

    /// Writes the KosherJava-style method name for a preset. Returns false when unknown.
    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn preset_method_name(preset: ZmanPresetId, write: &mut DiplomatWrite) -> bool {{
        PRESET_METADATA
            .get(preset as usize)
            .map(|meta| write!(write, "{{}}", meta.method_name).is_ok())
            .unwrap_or(false)
    }}

    /// Returns the broad category for a preset.
    #[diplomat::attr(not(supports = free_functions), disable)]
    pub fn preset_zman_type(preset: ZmanPresetId) -> ZmanType {{
        to_ffi_zman_type(PRESET_METADATA[preset as usize].zman_type)
    }}

    /// Dart entry point: construct once and call instance methods.
    #[diplomat::cfg(not(supports = free_functions))]
    #[diplomat::opaque]
    pub struct ZmanPresets(u8);

    impl ZmanPresets {{
        #[diplomat::attr(*, constructor)]
        pub fn new() -> Box<ZmanPresets> {{
            Box::new(ZmanPresets(0))
        }}

        pub fn preset_count(&self) -> u32 {{
            u32::try_from(ZMAN_PRESET_COUNT).unwrap_or(u32::MAX)
        }}

        pub fn preset_name(&self, preset: ZmanPresetId, write: &mut DiplomatWrite) -> bool {{
            PRESET_METADATA
                .get(preset as usize)
                .map(|meta| write!(write, "{{}}", meta.name).is_ok())
                .unwrap_or(false)
        }}

        pub fn preset_deprecated(&self, preset: ZmanPresetId) -> bool {{
            PRESET_METADATA
                .get(preset as usize)
                .is_some_and(|meta| meta.deprecated)
        }}

        pub fn preset_method_name(&self, preset: ZmanPresetId, write: &mut DiplomatWrite) -> bool {{
            PRESET_METADATA
                .get(preset as usize)
                .map(|meta| write!(write, "{{}}", meta.method_name).is_ok())
                .unwrap_or(false)
        }}

        pub fn preset_zman_type(&self, preset: ZmanPresetId) -> ZmanType {{
            to_ffi_zman_type(PRESET_METADATA[preset as usize].zman_type)
        }}
    }}
}}
"""


def generate_dispatch(presets: list[tuple[str, str, str, bool]]) -> str:
    arms = "\n        ".join(
        f"{index} => Some(&presets::{const_name}),"
        for index, (const_name, _, _, _) in enumerate(presets)
    )
    meta_lines = []
    for const_name, method_name, zman_type, deprecated in presets:
        dep = "true" if deprecated else "false"
        type_variant = snake_to_pascal(zman_type)
        meta_lines.append(
            "    PresetMetadata {\n"
            f'        method_name: "{method_name}",\n'
            f"        name: presets::{const_name}.name,\n"
            f"        zman_type: ZmanType::{type_variant},\n"
            f"        deprecated: {dep},\n"
            "    }"
        )
    metadata_entries = ",\n".join(meta_lines)

    return f"""//! Generated by tools/generate-interop.py. Do not edit by hand.
#![allow(missing_docs, dead_code)]

use kosher_rust::zmanim::{{ZmanPreset, ZmanType, presets}};

/// Stable count of generated zman presets.
pub const ZMAN_PRESET_COUNT: usize = {len(presets)};

/// Metadata for a zman preset (static strings, no closures).
pub struct PresetMetadata {{
    pub method_name: &'static str,
    pub name: &'static str,
    pub zman_type: ZmanType,
    pub deprecated: bool,
}}

/// Static metadata table, indexed by preset discriminant order.
pub static PRESET_METADATA: [PresetMetadata; {len(presets)}] = [
{metadata_entries}
];

/// Looks up a preset static by stable index (matches [`ZmanPresetId`] discriminant order).
pub fn preset_by_index(index: usize) -> Option<&'static ZmanPreset> {{
    match index {{
        {arms}
        _ => None,
    }}
}}
"""


def generate_mod() -> str:
    return """//! Generated module root.

pub mod preset_dispatch;
"""


def main() -> None:
    presets = collect_presets()
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    (OUTPUT_DIR / "mod.rs").write_text(generate_mod(), encoding="utf-8", newline="\n")
    (OUTPUT_DIR.parent / "zman_preset.rs").write_text(
        generate_zman_preset_bridge(presets), encoding="utf-8", newline="\n"
    )
    (OUTPUT_DIR / "preset_dispatch.rs").write_text(
        generate_dispatch(presets), encoding="utf-8", newline="\n"
    )

    print(f"Wrote interop generated files to {OUTPUT_DIR} ({len(presets)} presets).")
    subprocess.run(["cargo", "fmt", "-p", "interop"], cwd=SCRIPT_DIR.parent, check=True)


if __name__ == "__main__":
    main()
