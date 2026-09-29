"""Deterministic requests for the GccEnt package (tools/oracle/tkgeomalgo_probe_gccent.cxx).

`gccent_position` feeds every position to PositionToString and Print with every name of the
probe's name table (case variants, the empty string, an unknown name, a trailing blank).
`gccent_qualified` builds qualified circles and lines with every position and the seven
GccEnt makers.
"""
from __future__ import annotations


def line(op: str, *values: float) -> str:
    """One request line of `op` over `values`."""
    return op + ' ' + ' '.join(repr(float(v)) for v in values)


def append_gccent_requests(requests: list[str]) -> None:
    """Append the GccEnt requests: positions by names, then qualified entities."""
    for position in range(5):
        for name in range(8):
            requests.append(line('gccent_position', position, name))
    shapes = [((0.0, 0.0, 1.0, 0.0, 2.0), (1.0, -1.0, 0.6, 0.8)),
              ((-3.5, 7.25, -0.6, 0.8, 1e-9), (0.0, 0.0, 0.0, -1.0)),
              ((1e8, -1e8, 0.0, 1.0, 1e6), (2.0, 3.0, -1.0, 0.0))]
    for position in range(5):
        for circle, lin in shapes:
            requests.append(line('gccent_qualified', position, *circle, *lin))

