"""Martensite screen-overlay fixture family retains strict native pixel/depth gates."""
RUBBER_BAND_MODES = (
    "rubber-rectangle", "rubber-open", "rubber-filled", "rubber-transparent",
    "rubber-concave", "rubber-dash", "rubber-wide", "rubber-updated",
)


def rubber_band_pixels(width: int, height: int, rgba: bytes, mode: str) -> dict[str, object]:
    """Reject blank overlays and missing filling; rubber_band_pixels(1, 1, pixel, 'rubber-filled')."""
    if width <= 0 or height <= 0 or len(rgba) != width * height * 4 or mode not in RUBBER_BAND_MODES:
        raise ValueError(f"received {(width, height, len(rgba), mode)}; expected positive dimensions, width*height*4 bytes and rubber-band mode")
    pixels = [rgba[index:index + 4] for index in range(0, len(rgba), 4)]
    visible = sum(pixel != bytes([0, 0, 255, 255]) for pixel in pixels)
    red_fill = sum(pixel[0] > pixel[1] and pixel[1] == 0 for pixel in pixels)
    requires_fill = mode in ("rubber-filled", "rubber-transparent", "rubber-concave")
    return {"status": "passed" if visible > 0 and (not requires_fill or red_fill > 0) else "failed", "visible_pixels": visible, "red_fill_pixels": red_fill}
