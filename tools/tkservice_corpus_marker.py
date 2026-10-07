"""Deterministic requests for Graphic3d_MarkerImage (tools/oracle/tkservice_probe_marker.cxx).

`sv_marker` builds the standard marker of every type (EMPTY and USERDEFINED give none) over the
scale range of the bitmap tables, including scales that round between table sizes, and balls
of several colors; ring scales stay at or above 1, where the native code has an image.
"""
from __future__ import annotations

SCALES = [1.0, 1.24, 1.25, 2.0, 2.5, 3.0, 3.6, 4.0, 5.5, 7.0, 8.0]
COLORS = [[1.0, 1.0, 0.0], [0.2, 0.4, 0.6], [0.0, 0.0, 0.0]]


def append_marker_requests(requests: list[str]) -> None:
    """Append the standard marker requests: every type and scale, then ball colors."""
    for kind in range(-1, 14):
        for scale in SCALES:
            requests.append('sv_marker ' + ' '.join(repr(float(v)) for v in [kind, scale, *COLORS[0]]))
    for color in COLORS[1:]:
        for scale in [1.0, 3.0, 6.5]:
            requests.append('sv_marker ' + ' '.join(repr(float(v)) for v in [12, scale, *color]))
    requests.append('sv_marker ' + ' '.join(repr(float(v)) for v in [1, 0.5, 1.0, 1.0, 1.0]))
