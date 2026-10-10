"""Circular/arc datums, live geometry, appearance and filled/boundary picks."""

CIRCLE_MODES = tuple(
    "circle-" + name
    for name in (
        "full",
        "arc",
        "reverse-full",
        "filled",
        "arc-filled",
        "color",
        "width",
        "unset-color",
        "unset-width",
        "replaced",
        "radius",
        "moved",
        "erased",
        "redisplayed",
        "selected",
        "detected",
        "filled-detected",
        "trim",
        "absolute",
    )
)
