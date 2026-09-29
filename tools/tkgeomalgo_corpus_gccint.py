"""Deterministic requests for the GccInt package (tools/oracle/tkgeomalgo_probe_gccint.cxx).

`gccint_bisec` builds each bisector kind and calls every accessor: the base-class accessors
raise Standard_NotImplemented except the one of the bisector's kind.
"""
from __future__ import annotations
from tkgeomalgo_corpus_gccent import line


def append_gccint_requests(requests: list[str]) -> None:
    """Append the GccInt requests: every bisector kind on a few frames."""
    frames = [(0.0, 0.0, 1.0, 0.0), (1.5, -2.0, 0.6, 0.8), (-1e5, 3e5, -0.28, 0.96)]
    radii = [(3.0, 2.0), (0.5, 0.25), (7.0, 7.0)]
    for kind in range(5):
        for frame, (a, b) in zip(frames, radii):
            requests.append(line('gccint_bisec', kind, *frame, a, b))
