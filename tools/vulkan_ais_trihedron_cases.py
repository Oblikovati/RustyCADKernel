"""Native and Martensite trihedron fixtures retain every strict capture gate."""
AIS_TRIHEDRON_MODES = (
    "tri-wire", "tri-shaded", "tri-wire-noarrows", "tri-shaded-noarrows",
    "tri-labels", "tri-color", "tri-arrow-color", "tri-text-color", "tri-size",
    "tri-moved", "tri-selected", "tri-axis", "tri-origin", "tri-plane",
    "tri-shaded-axis", "tri-shaded-origin",
)


def trihedron_pixels(width: int, height: int, rgba: bytes, mode: str, reference: bytes | None = None) -> dict[str, object]:
    """Reject false empty datum captures; trihedron_pixels(1, 1, pixel, 'tri-wire')."""
    if width <= 0 or height <= 0 or len(rgba) != width * height * 4 or mode not in AIS_TRIHEDRON_MODES:
        raise ValueError(f"received {(width, height, len(rgba), mode)}; expected positive dimensions, width*height*4 bytes and a trihedron mode")
    pixels = [rgba[index:index + 4] for index in range(0, len(rgba), 4)]
    visible = sum(pixel != bytes([0, 0, 255, 255]) for pixel in pixels)
    # Yellow is the unselected label color, not evidence of selection (OCCT 3d097a0).
    selected = sum(pixel[:3] == bytes([204, 204, 204]) for pixel in pixels)
    needs_selection = mode in ("tri-selected", "tri-axis", "tri-origin", "tri-plane", "tri-shaded-axis", "tri-shaded-origin")
    if needs_selection and (reference is None or len(reference) != len(rgba)):
        raise ValueError(f"received reference length {None if reference is None else len(reference)}; expected {len(rgba)} unselected bytes for {mode}")
    changed = sum(rgba[index:index + 4] != reference[index:index + 4] for index in range(0, len(rgba), 4)) if reference is not None else 0
    passed = visible > 0 and (not needs_selection or changed > 0)
    return {"status": "passed" if passed else "failed", "visible_pixels": visible, "selected_pixels": selected, "changed_pixels": changed}
