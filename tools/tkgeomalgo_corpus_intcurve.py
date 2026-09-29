"""Deterministic requests for IntRes2d, IntImpParGen and IntCurve
(tools/oracle/tkgeomalgo_probe_intres2d.cxx, _intimppargen.cxx and _intcurve.cxx).

A conic is `kind x y dx dy sense r1 r2` (kind 0 line, 1 circle, 2 ellipse, 3 parabola,
4 hyperbola, 9 the default IntCurve_IConicTool); a domain `kind p1 t1 p2 t2 closed z zp`
(kind 0 infinite, 1 bounded, 2 first bound only, 3 last bound only). `intcurve_conics`
intersects every ordered pair of conics accepted by IntCurve_IntConicConic: crossing,
tangent, coincident, disjoint and nearly tangent pairs, direct and indirect frames, open,
closed, semi-infinite and wrapped domains, both tolerances and the reversed flag, then
seeded random pairs. The other operations probe the tools, the transition and domain
helpers, and the result merge of IntRes2d.
"""
from __future__ import annotations
import math
import random

TWO_PI = 2.0 * math.pi


def fmt(values) -> str:
    """The numbers of a request, exactly (repr round-trips doubles)."""
    return ' '.join(repr(float(v)) for v in values)


def line(x, y, dx, dy):
    return [0, x, y, dx, dy, 1, 0.0, 0.0]


def circle(x, y, r, dx=1.0, dy=0.0, sense=1):
    return [1, x, y, dx, dy, sense, r, 0.0]


def ellipse(x, y, a, b, dx=1.0, dy=0.0, sense=1):
    return [2, x, y, dx, dy, sense, a, b]


def parabola(x, y, f, dx=1.0, dy=0.0, sense=1):
    return [3, x, y, dx, dy, sense, f, 0.0]


def hyperbola(x, y, a, b, dx=1.0, dy=0.0, sense=1):
    return [4, x, y, dx, dy, sense, a, b]


INFINITE = [0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, 0.0]


def bounded(p1, p2, tol=1e-9, closed=False, z=None, zp=None):
    z = p1 if z is None else z
    zp = z + TWO_PI if zp is None else zp
    return [1, p1, tol, p2, tol, 1 if closed else 0, z, zp]


def first_only(p1, tol=1e-9):
    return [2, p1, tol, 0.0, 0.0, 0, 0.0, 0.0]


def last_only(p2, tol=1e-9):
    return [3, 0.0, 0.0, p2, tol, 0, 0.0, 0.0]


FULL = bounded(0.0, TWO_PI, closed=True, z=0.0, zp=TWO_PI)
FULL_OPEN = bounded(0.0, TWO_PI)
LINE_DOM = bounded(-10.0, 10.0)
CONIC_DOM = bounded(-5.0, 5.0)


def conics(a, da, b, db, tol_conf=1e-9, tol=1e-9, reversed_flag=0) -> str:
    return 'intcurve_conics ' + fmt([*a, *da, *b, *db, tol_conf, tol, reversed_flag])


def natural_domains(kind):
    """Domains of a conic kind: closed full turns, partial and wrapped arcs, bounds."""
    if kind in (1, 2):
        return [FULL, FULL_OPEN, bounded(0.5, 2.5), bounded(-1.0, 1.0, closed=True),
                bounded(4.0, 7.5), bounded(1.0, 1.0 + 1e-12)]
    if kind == 0:
        return [INFINITE, LINE_DOM, first_only(-1.0), last_only(1.0), bounded(0.0, 1e-12)]
    return [INFINITE, CONIC_DOM, bounded(-1.0, 0.5), first_only(0.0), last_only(0.0)]


def pair_cases():
    """(conic A, conic B) cases for every ordered pair: crossing, tangent, coincident,
    disjoint and nearly tangent configurations."""
    cases = {
        (0, 0): [(line(0, 0, 1, 0), line(1, -1, 0, 1)), (line(0, 0, 1, 0), line(0, 1e-12, 1, 0)),
                 (line(0, 0, 1, 0), line(3, 0, -1, 0)), (line(0, 0, 1, 0), line(0, 1, 1, 0)),
                 (line(0, 0, 1, 0), line(0, 0, 1, 1e-10)), (line(0, 0, 0.6, 0.8), line(2, 0, -0.8, 0.6))],
        (0, 1): [(line(-3, 0, 1, 0), circle(0, 0, 1)), (line(-3, 1, 1, 0), circle(0, 0, 1)),
                 (line(-3, 1 + 1e-10, 1, 0), circle(0, 0, 1)), (line(-3, 2, 1, 0), circle(0, 0, 1)),
                 (line(0, -3, 0, 1), circle(0, 0, 1, sense=0)), (line(-3, 0.5, 1, 0), circle(0, 0, 1, 0.6, 0.8))],
        (0, 2): [(line(-5, 0, 1, 0), ellipse(0, 0, 2, 1)), (line(-5, 1, 1, 0), ellipse(0, 0, 2, 1)),
                 (line(2, -3, 0, 1), ellipse(0, 0, 2, 1)), (line(-5, 3, 1, 0), ellipse(0, 0, 2, 1)),
                 (line(-5, 0.5, 1, 0.1), ellipse(1, 1, 3, 1, 0.6, 0.8, 0)), (line(0, -5, 0, 1), ellipse(0, 0, 2, 1e-6))],
        (0, 3): [(line(-5, 1, 1, 0), parabola(0, 0, 1)), (line(0, -5, 0, 1), parabola(0, 0, 1, 0, 1)),
                 (line(-5, -1, 1, 0), parabola(0, 0, 0.5)), (line(-1, -5, 0, 1), parabola(0, 0, 1))],
        (0, 4): [(line(3, -5, 0, 1), hyperbola(0, 0, 2, 1)), (line(-5, 0, 1, 0), hyperbola(0, 0, 2, 1)),
                 (line(0, -5, 0, 1), hyperbola(0, 0, 2, 1)), (line(-5, 0.5, 1, 0.5), hyperbola(0, 0, 1, 1))],
        (1, 1): [(circle(0, 0, 1), circle(1, 0, 1)), (circle(0, 0, 1), circle(2, 0, 1)),
                 (circle(0, 0, 1), circle(0, 0, 1)), (circle(0, 0, 1), circle(0.5, 0, 0.5)),
                 (circle(0, 0, 1), circle(3, 0, 1)), (circle(0, 0, 1), circle(1, 0, 1, sense=0)),
                 (circle(0, 0, 1), circle(0, 0, 1, sense=0)), (circle(0, 0, 2), circle(0, 0, 1))],
        (1, 2): [(circle(0, 0, 1.5), ellipse(0, 0, 2, 1)), (circle(0, 0, 1), ellipse(0, 0, 2, 1)),
                 (circle(3, 0, 1), ellipse(0, 0, 2, 1)), (circle(0, 0, 3), ellipse(0, 0, 2, 1))],
        (1, 3): [(circle(0, 1, 1), parabola(0, 0, 1)), (circle(0, 0, 1), parabola(0, 0, 0.5)),
                 (circle(5, 5, 1), parabola(0, 0, 1)), (circle(1, 0, 2), parabola(0, 0, 1, 0, 1, 0))],
        (1, 4): [(circle(0, 0, 3), hyperbola(0, 0, 2, 1)), (circle(0, 0, 2), hyperbola(0, 0, 2, 1)),
                 (circle(0, 0, 1), hyperbola(0, 0, 2, 1)), (circle(3, 0, 1.5), hyperbola(0, 0, 2, 1))],
        (2, 2): [(ellipse(0, 0, 2, 1), ellipse(0, 0, 1.5, 1.2)), (ellipse(0, 0, 2, 1), ellipse(0, 0, 2, 1)),
                 (ellipse(0, 0, 2, 1), ellipse(0, 0, 2, 1, 0, 1)), (ellipse(0, 0, 2, 1), ellipse(5, 0, 2, 1))],
        (2, 3): [(ellipse(0, 0, 2, 1), parabola(0, 0, 1)), (ellipse(0, 0, 2, 1), parabola(-3, 0, 1)),
                 (ellipse(0, 0, 2, 1), parabola(5, 0, 1))],
        (2, 4): [(ellipse(0, 0, 3, 1), hyperbola(0, 0, 2, 1)), (ellipse(0, 0, 2, 1), hyperbola(0, 0, 2, 1)),
                 (ellipse(0, 0, 1, 0.5), hyperbola(0, 0, 2, 1))],
        (3, 3): [(parabola(0, 0, 1), parabola(0, 0, 1, -1, 0)), (parabola(0, 0, 1), parabola(1, 0, 1)),
                 (parabola(0, 0, 1), parabola(0, 0, 2))],
        (3, 4): [(parabola(0, 0, 1), hyperbola(0, 0, 2, 1)), (parabola(-5, 0, 1), hyperbola(0, 0, 2, 1))],
        (4, 4): [(hyperbola(0, 0, 2, 1), hyperbola(0, 0, 1, 1, 0, 1)), (hyperbola(0, 0, 2, 1), hyperbola(0, 0, 2, 1)),
                 (hyperbola(0, 0, 2, 1), hyperbola(5, 0, 2, 1))],
    }
    return cases


def append_conics_requests(requests: list[str]) -> None:
    """The fixed pairs over their natural domains and both tolerances, then random ones."""
    for (ka, kb), pairs in pair_cases().items():
        for index, (a, b) in enumerate(pairs):
            for da in natural_domains(ka)[:3]:
                for db in natural_domains(kb)[:3]:
                    requests.append(conics(a, da, b, db))
            for tol in (1e-7, 1e-3):
                requests.append(conics(a, natural_domains(ka)[1], b, natural_domains(kb)[1], tol, tol, index % 2))
            for da, db in zip(natural_domains(ka)[3:], natural_domains(kb)[3:]):
                requests.append(conics(a, da, b, db))
    append_random_conics(requests, random.Random(20260930), 150)


def random_conic(rng: random.Random, kind: int):
    angle = rng.uniform(0.0, TWO_PI)
    x, y = rng.uniform(-3, 3), rng.uniform(-3, 3)
    dx, dy, sense = math.cos(angle), math.sin(angle), rng.randint(0, 1)
    if kind == 0:
        return line(x, y, dx, dy)
    r1 = rng.uniform(0.2, 4.0)
    r2 = rng.uniform(0.1, r1) if kind in (2, 4) else 0.0
    return [kind, x, y, dx, dy, sense, r1, r2]


def random_domain(rng: random.Random, kind: int):
    if kind in (1, 2):
        start = rng.uniform(-TWO_PI, TWO_PI)
        span = rng.uniform(0.1, TWO_PI)
        return bounded(start, start + span, closed=rng.random() < 0.5)
    if kind == 0:
        return rng.choice([INFINITE, bounded(rng.uniform(-10, 0), rng.uniform(0, 10))])
    return rng.choice([INFINITE, bounded(rng.uniform(-6, 0), rng.uniform(0, 6))])


PAIRS = [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (1, 1), (1, 2), (1, 3), (1, 4), (2, 2), (2, 3),
         (2, 4), (3, 3), (3, 4), (4, 4)]


def append_random_conics(requests: list[str], rng: random.Random, count: int) -> None:
    for i in range(count):
        ka, kb = PAIRS[i % len(PAIRS)]
        a, b = random_conic(rng, ka), random_conic(rng, kb)
        tol = rng.choice([1e-9, 1e-7, 1e-5])
        requests.append(conics(a, random_domain(rng, ka), b, random_domain(rng, kb), tol, tol, rng.randint(0, 1)))


def append_imppar_requests(requests: list[str]) -> None:
    """IntImpConicParConic directly: every implicit kind against every parametric kind,
    open, closed and unbounded (ConstructionError) parametric domains."""
    samples = [line(-3, 0.5, 1, 0), circle(0, 0, 1), ellipse(0, 0, 2, 1), parabola(0, -1, 1), hyperbola(0, 0, 1, 1)]
    domains = {0: LINE_DOM, 1: FULL, 2: FULL, 3: CONIC_DOM, 4: bounded(-2.0, 2.0)}
    for a in samples:
        for b in samples:
            for accuracy, epsx in ((20, 1e-8), (5, 1e-12), (60, 1e-3)):
                requests.append('intcurve_imppar ' + fmt([*a, *domains[a[0]], *b, *domains[b[0]], 1e-9, 1e-9, 0, accuracy, epsx]))
            requests.append('intcurve_imppar ' + fmt([*a, *INFINITE, *b, *INFINITE, 1e-9, 1e-9, 1, 20, 1e-8]))
            requests.append('intcurve_imppar ' + fmt([*a, *domains[a[0]], *b, *first_only(0.0), 1e-9, 1e-9, 0, 20, 1e-8]))


def append_tool_requests(requests: list[str]) -> None:
    """IConicTool, PConicTool, ProjectOnPConicTool and the root function on every kind,
    including the centre of a circle or an ellipse (gp exceptions) and the default tool."""
    samples = [line(1, 2, 0.6, 0.8), circle(1, -1, 2, 0, 1), circle(0, 0, 1, 1, 0, 0), ellipse(1, 1, 3, 1, 0.6, -0.8),
               ellipse(0, 0, 2, 2), parabola(0, 0, 0.5, 1, 0, 0), hyperbola(-1, 2, 2, 1, 0, 1), [9, 0, 0, 1, 0, 1, 0, 0]]
    points = [(0.3, 0.7), (1.0, -1.0), (0.0, 0.0), (1.0, 1.0), (-4.0, 5.0), (3.0, 0.0)]
    for c in samples:
        for u in (-1.5, 0.0, 0.7, 4.0):
            for x, y in points:
                requests.append('intcurve_itool ' + fmt([*c, u, x, y]))
                if c[0] != 9:
                    requests.append('intcurve_ptool ' + fmt([*c, u, x, y, 0.5, -0.5, 7, 1e-6]))
                    requests.append('intcurve_ptool ' + fmt([*c, u, x, y, 1.0, 1.0, 20, 1e-8]))
    for a in samples[:7]:
        for b in samples[:7]:
            for u in (-0.5, 0.25, 2.0):
                requests.append('intcurve_function ' + fmt([*a, *b, u]))


def append_interval_requests(requests: list[str]) -> None:
    """Interval and PeriodicInterval on domains and intervals around 0 and 2PI."""
    domains = [INFINITE, bounded(0.5, 2.0), bounded(-1.0, 1.0), bounded(5.0, 8.0), first_only(1.0),
               last_only(3.0), bounded(0.0, TWO_PI), bounded(-3.0, 9.0), bounded(2.0, 1.0)]
    intervals = [(0.2, 1.0, 1, 1), (1.0, 0.2, 1, 0), (-2.0, 7.0, 0, 1), (6.0, 6.5, 0, 0), (3.0, 3.0, 1, 1),
                 (-0.5, 0.5, 1, 1), (4.0, 10.0, 1, 1)]
    for d in domains:
        for a, b, ha, hb in intervals:
            requests.append('intcurve_interval ' + fmt([*d, a, b, ha, hb]))


def append_transition_requests(requests: list[str]) -> None:
    """The DetermineTransition variants on crossing, tangent (both curvature sides),
    null-tangent and degenerate vectors, and DeterminePosition/NormalizeOnDomain."""
    vectors = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 1e-9), (1e-7, 0.0), (0.0, 0.0), (2.0, 3.0), (1e-13, 1e-13)]
    normals = [(0.0, 1.0), (0.0, -1.0), (0.0, 0.0), (0.5, 0.5)]
    rng = random.Random(7)
    for t1 in vectors:
        for t2 in vectors:
            n1, n2 = rng.choice(normals), rng.choice(normals)
            requests.append('imppar_transition ' + fmt([rng.randint(0, 2), *t1, *n1, rng.randint(0, 2), *t2, *n2]))
    for n1 in normals:
        for n2 in normals:
            requests.append('imppar_transition ' + fmt([0, 1.0, 0.0, *n1, 2, -1.0, 0.0, *n2]))
            requests.append('imppar_transition ' + fmt([1, 1.0, 0.0, *n1, 1, 1.0, 0.0, *n2]))
    domains = [INFINITE, bounded(0.0, 1.0, 1e-3), bounded(0.0, 1.0, 1e-3, closed=True, z=0.0, zp=1.0),
               first_only(0.0, 1e-3), last_only(1.0, 1e-3), bounded(0.0, 1e-4, 1e-3),
               bounded(1.0, 4.0, 1e-3, closed=True, z=0.0, zp=TWO_PI), bounded(0.0, TWO_PI, 1e-9, closed=True, z=0.0, zp=TWO_PI)]
    for d in domains:
        for x, param in ((0.0, 0.0), (1.0, 1.0), (5e-5, 5e-5), (0.5, -3.0), (2.0, 9.0), (0.9995, 0.9995), (3.0, 20.0)):
            requests.append('imppar_domain ' + fmt([*d, x, 0.0, param]))


def ipoint(u1, u2, x, y, trans1=(1, 0, 0, 2, 0), trans2=(1, 1, 0, 2, 0)):
    return [u1, u2, x, y, *trans1, *trans2]


def append_intres2d_requests(requests: list[str]) -> None:
    """Domains (clamped infinite values, exceptions), Insert order and duplicates, and
    the composite merge of Append."""
    for kind in range(4):
        for p1, p2 in ((0.0, 1.0), (-1e300, 1e300), (2.0, -3.0)):
            for closed in (0, 1):
                requests.append('intres2d_domain ' + fmt([kind, p1, 1e-7, p2, 2e-7, closed, 0.0, TWO_PI]))
    touch = (0, 2, 1, 0, 1)
    points = [ipoint(2.0, 0.0, 2, 0), ipoint(1.0, 0.0, 1, 0), ipoint(1.0 + 1e-9, 1e-9, 1, 0),
              ipoint(1.0, 0.5, 1, 0), ipoint(3.0, 0.0, 3, 0, touch, touch), ipoint(3.0, 0.0, 3, 0, touch, touch),
              ipoint(3.0, 0.0, 3, 0, (0, 2, 1, 1, 1), touch), ipoint(0.5, 0.0, 0, 0, (2, 3, 1, 2, 0), (0, 3, 1, 2, 0))]
    requests.append('intres2d_insert ' + fmt([len(points), *[v for p in points for v in p]]))
    requests.append('intres2d_insert ' + fmt([3, *points[4], *points[5], *points[6]]))
    requests.append('intres2d_insert ' + fmt([2, *points[1], *points[2]]))
    head, end = (0, 0, 0, 2, 0), (2, 1, 0, 2, 0)
    seg1 = [0, *ipoint(0.0, 0.0, 0, 0, head, head), *ipoint(1.0, 1.0, 1, 1, end, end)]
    seg2 = [0, *ipoint(1.0, 1.0, 1, 1, head, head), *ipoint(2.0, 2.0, 2, 2, end, end)]
    seg3 = [0, *ipoint(3.0, 3.0, 3, 3, head, head), *ipoint(2.0, 2.0, 2, 2, end, end)]
    seg4 = [1, *ipoint(1.0, 1.0, 1, 1, head, head), *ipoint(2.0, 2.0, 2, 2, end, end)]
    inner = ipoint(1.5, 1.5, 1.5, 1.5, head, end)
    for second in (seg2, seg3, seg4):
        requests.append('intres2d_append ' + fmt([0.0, 3.0, 0.0, 3.0, 1, *inner, 1, *seg1, 1, *ipoint(0.5, 2.5, 0, 0, head, end), 1, *second]))
    requests.append('intres2d_append ' + fmt([0.0, 1.0, 0.0, 1.0, 0, 1, *seg1, 2, *inner, *ipoint(1.0, 1.0, 1, 1, end, end), 1, *seg3]))


def append_intcurve_requests(requests: list[str]) -> None:
    """Append the IntRes2d, IntImpParGen and IntCurve requests in a fixed order."""
    append_intres2d_requests(requests)
    append_transition_requests(requests)
    append_interval_requests(requests)
    append_tool_requests(requests)
    append_imppar_requests(requests)
    append_conics_requests(requests)
