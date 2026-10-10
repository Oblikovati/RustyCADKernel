"""Paired retained clipping scenes, using all projection/extent/translation cases."""

CLIPPING_MODES = (
    "clip-global",
    "clip-intersection",
    "clip-union",
    "clip-two-chains",
    "clip-local",
    "clip-override",
    "clip-empty-override",
    "clip-head-off",
    "clip-child-off",
    "clip-equation-updated",
    "clip-cleared",
    "clip-group-transform",
    "clip-lines",
    "clip-points",
    "clip-outside",
    "clip-inside",
    "clip-pers-in",
    "clip-pers-out",
    "clip-pers-local",
    "clip-pers-override",
    "clip-pers-2d",
    "clip-group-pers",
    "clip-nested-pers",
    "clip-text-baseline",
    "clip-text-cut",
    "clip-text-outside",
    "clip-text-no-box",
)


def font_plane_policy(
    baseline: bytes, cut: bytes, no_box: bytes, outside: bytes
) -> dict[str, object]:
    """Check white glyph coverage over red/blue clipping fixtures; font_plane_policy(base, cut, unchecked, culled)."""
    buffers = (baseline, cut, no_box, outside)
    sizes = tuple(map(len, buffers))
    if not sizes[0] or sizes[0] % 4 or len(set(sizes)) != 1:
        raise ValueError(f"received RGBA sizes {sizes}; expected equal nonempty complete pixels")
    # Only the white glyph contributes green in these red-triangle/blue-clear scenes.
    coverage = [pixels[1::4] for pixels in buffers]
    glyph_pixels = sum(channel > 0 for channel in coverage[0])
    unchanged = coverage[0] == coverage[1] == coverage[2]
    culled = not any(coverage[3])
    return {
        "status": "passed" if glyph_pixels and unchanged and culled else "failed",
        "baseline_glyph_pixels": glyph_pixels,
        "fragment_planes_preserve_glyphs": unchanged,
        "whole_structure_cull_removes_glyphs": culled,
    }
