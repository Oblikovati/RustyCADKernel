"""Deterministic requests for the ChFiDS probes (tools/oracle/tkfillet_probe_chfids.cxx).

`fl_spine` evaluates guidelines of segments: open and closed polylines, reversed edges,
tangent extensions, approximation references, abscissae at the junctions, before the
start and after the end. `fl_arc_spine` evaluates one circular arc (forward and reversed)
and its Circle(). `fl_fil_law` builds the radius laws of fillet guidelines: constant,
variable by pairs, mixed constant and variable edges, extended and periodic elementary
spines, and missing radii (Standard_DomainError). Every guideline has at least one edge
and no degenerate segment, so no request dereferences a null handle natively.
"""
from __future__ import annotations
import random


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def spine_request(points: list[tuple[float, float, float]], reversed_bits: int, closed: bool,
                  tangents: tuple[bool, bool], reference: tuple[int, float], ws: list[float]) -> str:
    """One `fl_spine` request."""
    fields = [len(points)] + [c for p in points for c in p] + [reversed_bits, int(closed)]
    fields += [int(tangents[0]), int(tangents[1]), reference[0], reference[1], len(ws)] + ws
    return 'fl_spine ' + numbers(fields)


def samples(length: float, junctions: list[float]) -> list[float]:
    """Abscissae over a guideline of `length`: ends, junctions, interior and outside."""
    ws = [0.0, -0.0, length, -0.5, length + 0.75, length * 0.5, length * 0.25, length * 0.9]
    for j in junctions:
        ws += [j, j - 1e-9, j + 1e-9, j - 1e-3]
    return ws


def polyline_lengths(points: list[tuple[float, float, float]]) -> list[float]:
    """The cumulated segment lengths of `points`."""
    total, out = 0.0, []
    for a, b in zip(points, points[1:]):
        total += sum((x - y) ** 2 for x, y in zip(a, b)) ** 0.5
        out.append(total)
    return out


FIXED_POLYLINES = [
    [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)],
    [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 2.0, 0.0)],
    [(0.0, 0.0, 0.0), (3.0, 0.0, 0.0), (3.0, 4.0, 0.0), (3.0, 4.0, 12.0)],
    [(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 0.0)],
    [(-5.0, 2.0, 1.0), (0.5, 0.25, -3.0), (7.0, 7.0, 7.0)],
    [(0.0, 0.0, 0.0), (1e-3, 0.0, 0.0), (1e-3, 1e-3, 0.0)],
]


def append_spine_requests(requests: list[str]) -> None:
    """Fixed polylines under every reversal, closure, extension and reference mode."""
    for points in FIXED_POLYLINES:
        lengths = polyline_lengths(points)
        ws = samples(lengths[-1], lengths[:-1])
        closed = points[0] == points[-1]
        for bits in range(1 << min(len(points) - 1, 3)):
            requests.append(spine_request(points, bits, closed, (False, False), (0, 0.0), ws))
        for tangents in [(True, False), (False, True), (True, True)]:
            requests.append(spine_request(points, 0, False, tangents, (0, 0.0), ws))
        for reference in [(1, lengths[0]), (2, 1.0), (1, -1.0), (2, float(len(lengths)))]:
            requests.append(spine_request(points, 1, closed, (True, True), reference, ws))
    rng = random.Random(20261007)
    for _ in range(24):
        count = rng.randint(2, 6)
        points = [tuple(rng.uniform(-10.0, 10.0) for _ in range(3)) for _ in range(count)]
        lengths = polyline_lengths(points)
        ws = samples(lengths[-1], lengths[:-1]) + [rng.uniform(-1.0, lengths[-1] + 1.0) for _ in range(6)]
        flags = (rng.random() < 0.5, rng.random() < 0.5)
        reference = (rng.randint(0, 2), float(rng.randint(1, count - 1)))
        requests.append(spine_request(points, rng.randint(0, 63), False, flags, reference, ws))


def append_arc_requests(requests: list[str]) -> None:
    """Arcs of several radii and spans, forward and reversed."""
    arcs = [((0.0, 0.0, 0.0), 1.0, 0.0, 1.5707963267948966),
            ((1.0, -2.0, 3.0), 2.5, 0.5, 4.0),
            ((0.0, 0.0, -1.0), 1e-2, -1.0, 1.0),
            ((10.0, 10.0, 0.0), 100.0, 3.0, 6.0)]
    for center, radius, a0, a1 in arcs:
        length = radius * (a1 - a0)
        ws = samples(length, []) + [length * k / 7.0 for k in range(1, 7)]
        for reversed_edge in (0, 1):
            fields = list(center) + [radius, a0, a1, reversed_edge, len(ws)] + ws
            requests.append('fl_arc_spine ' + numbers(fields))


def law_request(edges: int, closed: bool, pairs: list[tuple[float, float]],
                edge_radii: list[tuple[int, float]], span: tuple[float, float, bool], xs: list[float]) -> str:
    """One `fl_fil_law` request."""
    fields: list[float] = [edges, int(closed), len(pairs)] + [v for p in pairs for v in p]
    fields += [len(edge_radii)] + [v for e in edge_radii for v in e]
    fields += [span[0], span[1], int(span[2]), len(xs)] + xs
    return 'fl_fil_law ' + numbers(fields)


def append_law_requests(requests: list[str]) -> None:
    """Constant, variable, mixed, extended and periodic radius laws."""
    cases = [
        (3, False, [(0.0, 2.0), (3.0, 2.0)], [], (0.0, 3.0, False)),
        (3, False, [], [(1, 1.0), (3, 2.0)], (0.0, 3.0, False)),
        (3, False, [], [(2, 1.5)], (0.0, 3.0, False)),
        (3, False, [(0.0, 1.0), (1.5, 3.0), (3.0, 1.0)], [], (0.0, 3.0, False)),
        (3, False, [(0.5, 1.0)], [(3, 2.0)], (-0.5, 3.5, False)),
        (4, False, [(0.0, 1.0)], [(2, 2.0), (4, 0.5)], (0.0, 4.0, False)),
        (2, False, [], [], (0.0, 2.0, False)),
        (4, True, [(0.0, 1.0), (1.5, 2.0)], [], (0.0, 4.0, True)),
        (4, True, [], [(2, 1.0)], (0.0, 4.0, True)),
        (4, True, [(0.25, 1.0)], [(1, 1.0), (3, 2.0)], (0.0, 4.0, False)),
        (4, True, [], [(1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0)], (1.0, 3.0, False)),
        (1, True, [(0.0, 1.5)], [], (0.0, 1.0, True)),
        (3, False, [(1.0, 2.0), (2.0, 4.0)], [(1, 1.0)], (0.0, 3.0, False)),
    ]
    for edges, closed, pairs, edge_radii, span in cases:
        xs = [span[0] + (span[1] - span[0]) * k / 8.0 for k in range(9)] + [-1.0, span[1] + 1.0]
        requests.append(law_request(edges, closed, pairs, edge_radii, span, xs))


def append_chfids_requests(requests: list[str]) -> None:
    """Append the ChFiDS request families."""
    append_spine_requests(requests)
    append_arc_requests(requests)
    append_law_requests(requests)
