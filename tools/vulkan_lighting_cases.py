"""Native shader/material/light cases share all projection, size and translation variants."""

LIGHTING_MODES = (
    "lit-phong",
    "lit-facet",
    "lit-gouraud",
    "lit-default",
    "lit-no-normals",
    "lit-ambient",
    "lit-point",
    "lit-range",
    "lit-spot",
    "lit-headlight",
    "lit-intensity",
    "lit-back",
    "lit-distinct-back",
    "lit-colored",
    "lit-front-color-only",
    "lit-headlight-rotated",
    "lit-clipped",
    "lit-normal-updated",
    "lit-material-updated",
    "lit-light-updated",
    "lit-light-off",
)

LIT_LINE_MODES = tuple(
    f"lit-line-{suffix}"
    for suffix in (
        "phong",
        "gouraud",
        "facet",
        "no-normals",
        "colored",
        "patterned",
        "clipped",
        "normal-updated",
        "material-updated",
        "light-updated",
    )
)
