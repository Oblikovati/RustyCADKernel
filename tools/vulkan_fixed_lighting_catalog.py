"""Render source-pinned Martensite fixture catalogs without reading native libraries."""

from collections.abc import Sequence


def render_asset_catalog(modes: Sequence[str], textured: bool) -> str:
    """Return Rust fixture declarations; e.g. render_asset_catalog(["lit-basic"], False)."""
    lines = [
        "//! Generated pinned native asset catalog; source: fixed-lighting/catalog.json.",
        "use super::NativeFixedLightingAsset;",
        "pub const ASSETS: &[NativeFixedLightingAsset] = &[",
    ]
    for mode in modes:
        if not mode or any(
            character not in "abcdefghijklmnopqrstuvwxyz0123456789-"
            for character in mode
        ):
            raise ValueError(
                f"received mode={mode!r}; expected lowercase hyphenated fixture name"
            )
        lines.extend(_render_asset_row(mode, textured))
    return "\n".join([*lines, "];", ""])


def _render_asset_row(mode: str, textured: bool) -> list[str]:
    lines = ["    (", f'        "{mode}",', f"        {str(textured).lower()},"]
    basename = "texture-" + mode if textured else mode
    for profile in ("host", "software"):
        for extension in ("rgba", "depth"):
            lines.append(
                f'        include_bytes!("../fixtures/fixed-lighting/{profile}/{basename}.{extension}"),'
            )
    return [*lines, "    ),"]
