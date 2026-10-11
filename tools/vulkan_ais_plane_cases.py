"""AIS plane and legacy quad controls retain strict default image comparisons."""

AIS_PLANE_MODES = (
    "aisplane-wire",
    "aisplane-current",
    "aisplane-manual",
    "aisplane-xy",
    "aisplane-xz",
    "aisplane-yz",
    "aisplane-shaded",
    "aisplane-shaded-alpha",
    "aisplane-shaded-opaque",
    "aisplane-shaded-xyz",
    "aisplane-minimum",
    "aisplane-sized",
    "aisplane-selected",
    "aisplane-detected",
    "aisplane-erased",
    "aisplane-redisplayed",
    "aisplane-unset-color",
)
QUAD_MODES = (
    "quad-gradient",
    "quad-indexed",
    "quad-strips",
    "quad-bounds",
    "quad-transparent",
    "quad-material-only",
    "quad-layers",
    "quad-clear-depth",
    "quad-occlusion",
    "quad-overlap",
    "quad-alpha-opaque",
)


def layer_quad_pixels(
    width: int, height: int, rgba: bytes, mode: str
) -> dict[str, object]:
    """Verify the added layer is visible; layer_quad_pixels(1,1,bytes([187,187,0,255]),'quad-layers')."""
    if width <= 0 or height <= 0 or len(rgba) != width * height * 4:
        raise ValueError(
            f"received {width=} {height=} RGBA length {len(rgba)}; expected positive dimensions and four bytes per pixel"
        )
    at = ((height // 2) * width + width // 2) * 4
    color = list(rgba[at : at + 4])
    expected = [255, 0, 0, 255] if mode == "quad-clear-depth" else [187, 187, 0, 255]
    visible = color[2:] == expected[2:] and (
        color[:2] == expected[:2]
        if mode == "quad-clear-depth"
        else all(180 <= c <= 190 for c in color[:2])
    )
    return {
        "status": "passed" if visible else "failed",
        "center_rgba": color,
        "expected_center_rgba": expected,
    }
