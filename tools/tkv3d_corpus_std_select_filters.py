"""StdSelect owner filters: every filter type on every shape kind of
tools/oracle/tkv3d_probe_std_select_filters.cxx (edges, faces, a vertex, a plain owner).

A request is `v3_stdsel_filter Family Type Kind`. The null shape (kind 10) dereferences a null
TShape in the edge and face filters natively, so it reaches only the shape type filter.
"""
from __future__ import annotations


def append_std_select_filter_requests(requests: list[str]) -> None:
    """Edge filter types 0..2, face filter types 0..6, shape type filter types 0..8."""
    kinds = [kind for kind in range(12) if kind != 10]
    for family, types, shape_kinds in ((0, 3, kinds), (1, 7, kinds), (2, 9, list(range(12)))):
        for type_ in range(types):
            for kind in shape_kinds:
                requests.append(f'v3_stdsel_filter {family} {type_} {kind}')
