"""Deterministic requests for the TopOpeBRepDS value classes
(tools/oracle/tkbool_probe_topopebrepds.cxx; the records are documented there).

`ds_transition` covers every pair of states, faces and edges as shapes, equal and
different indices and every orientation; `ds_kind` prints every kind with a one-digit,
a two-digit and a negative index.
"""
from __future__ import annotations


def append_topopebrepds_requests(requests: list[str]) -> None:
    """The transitions and the kinds."""
    for sb in range(4):
        for sa in range(4):
            for shapes, indices in (((4, 4), (2, 2)), ((6, 4), (1, 3))):
                for o in range(4):
                    requests.append(f'ds_transition {sb} {sa} {shapes[0]} {shapes[1]} {indices[0]} {indices[1]} {o}')
    for k in range(12):
        for i in (3, 12, -1):
            requests.append(f'ds_kind {k} {i}')
