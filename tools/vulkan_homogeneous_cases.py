"""Native Vec4 positions across primitive topology, camera and affine transformation."""
HOMOGENEOUS_MODES = tuple(
    f"hom-{primitive}-{weight}"
    for primitive in ("triangle", "line", "point")
    for weight in ("unit", "double", "mixed", "infinite", "negative", "behind", "clipped")
)
