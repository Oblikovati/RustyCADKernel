"""Deterministic requests for Graphic3d_Aspects (tools/oracle/tkservice_probe_aspects.cxx).

`sv_aspects_dump` dumps every aspects class at depths 0, 1 and -1, with and without a text
font, and with alpha cutoffs and text angles whose float values print with every digit.
"""
from __future__ import annotations


def append_aspects_requests(requests: list[str]) -> None:
    """Append the aspects dump requests."""
    for kind in range(5):
        for depth in (-1, 0, 1, 2):
            for has_font in (0, 1):
                for cutoff, angle in [(0.5, 0.0), (0.285, 33.3), (0.1, -45.125)]:
                    fields = [kind, depth, has_font, cutoff, angle]
                    requests.append('sv_aspects_dump ' + ' '.join(repr(float(v)) for v in fields))
