"""AIS filters and AIS_GraphicTool: the type, signature, attribute and exclusion filters on a
point, a circle, a colored shape and an owner without selectable (with and without own color
and width), the bad edge and C0 regularity filters on a cylinder, a sphere and an edge, and
every graphic tool attribute of a default drawer.

A request is `v3_ais_attr_filter Op Object Color Width Values...` (see
tools/oracle/tkv3d_probe_ais_attribute_filters.cxx).
"""
from __future__ import annotations

OBJECTS = ((0, -1, 0.), (1, 21, 2.), (2, 21, 0.), (2, -1, 3.), (3, -1, 0.))
EXCLUSION_EDITS = (
    '0',
    '2 0 2 0 1 1 5',
    '3 1 1 1 1 1 7 3 1 1',
    '3 0 3 0 2 2 0 5 0 0',
    '2 4 0 0 0 1 0',
    '3 1 2 0 1 2 4 1 2 0',
)


def append_ais_attribute_filter_requests(requests: list[str]) -> None:
    """Every filter on every object, then the graphic tool attributes."""
    for obj, color, width in OBJECTS:
        head = f'{obj} {color} {width}'
        for kind in range(7):
            requests.append(f'v3_ais_attr_filter 0 {head} {kind}')
            for sig in (0, 1, 6):
                requests.append(f'v3_ais_attr_filter 1 {head} {kind} {sig}')
        for mode, col, wid in ((0, 0, 0.), (1, 21, 0.), (1, 5, 0.), (2, 0, 2.), (2, 0, 3.)):
            for edits in ('0', '1 2', '1 3', '2 0 21 1 3.', '2 0 5 1 2.'):
                requests.append(f'v3_ais_attr_filter 2 {head} {mode} {col} {wid} {edits}')
        for ctor in range(3):
            for kind, sig in ((1, 1), (2, 0), (1, 6)):
                for flag in (0, 1):
                    for edits in EXCLUSION_EDITS:
                        requests.append(f'v3_ais_attr_filter 3 {head} {ctor} {kind} {sig} {flag} {edits}')
    for attribute in range(16):
        requests.append(f'v3_ais_attr_filter 4 3 -1 0 {attribute}')
    for shape in range(3):
        for edits in ('0', '1 0 1 0', '2 1 0 1 0 1 0', '3 1 0 1 1 2 1 0 1 0', '4 1 0 2 0 2 0 2 2 0 0 0 0'):
            requests.append(f'v3_ais_attr_filter 5 2 -1 0 {shape} {edits}')
        requests.append(f'v3_ais_attr_filter 6 2 -1 0 {shape}')
