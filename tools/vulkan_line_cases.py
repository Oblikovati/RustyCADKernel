"""Shared line capture matrix, paired with Rust and native OCCT fixtures."""

LINE_MODES = (
    "line-segments",
    "line-segments-indexed",
    "line-polyline",
    "line-polyline-indexed",
    "line-bound-colors",
    "line-vertex-colors",
    "line-wide",
    "line-fractional",
    "line-dash",
    "line-dot",
    "line-dotdash",
    "line-custom",
    "line-empty",
    "line-gradient",
    "line-depth",
)

LINE_ALPHA_MODES = (
    "line-alpha-opaque",
    "line-alpha-auto",
    "line-alpha-blend",
    "line-alpha-mask",
    "line-alpha-forced",
    "line-alpha-zero",
    "line-alpha-vertices",
    "line-alpha-depth",
    "line-alpha-wide",
    "line-alpha-dash",
    "line-alpha-clipped",
)
