"""Deterministic Convert differential requests for tools/verify_tkmath_numeric.py.

Usage: append_convert_requests(requests) after the BSplSLib requests.
Covers BuildCosAndSin (both public overloads) and BuildPolynomialCosAndSin for every
Convert_ParameterisationType on full, trimmed, tiny, empty, negative and oversized
arcs; circles and ellipses (full and arcs) with every parameterisation; hyperbola and
parabola arcs; spheres, tori, cylinders and cones (whole, patch, band) with direct and
indirect frames and boundary/degenerate ranges; composite Bezier chains; composite
polynomial curves and polynomial surface grids (tkmath_convert_polynomial_corpus.py).
Requests stay inside the native defined domain: no arc makes BuildCosAndSin allocate
a negative array, and no sphere band writes past ComputePoles' stack buffer.
"""
from __future__ import annotations
import math
import random
from tkmath_convert_polynomial_corpus import append_polynomial_requests

PI = math.pi
PARAMETERISATIONS = range(8)


def line(op: str, values: list) -> str:
    """Format one request with round-trip float text; line('cv_polycs', [0, 1])."""
    return op+' '+' '.join(str(value) for value in values)


def append_convert_requests(requests: list[str]) -> None:
    """Append every Convert request with its own seed; append_convert_requests(requests)."""
    rng = random.Random(19911016)
    append_cos_sin_requests(rng, requests)
    append_conic_requests(rng, requests)
    append_surface_requests(rng, requests)
    append_polynomial_requests(rng, requests)


def arcs(rng: random.Random) -> list[tuple[float, float]]:
    """Boundary, degenerate and random arcs; the openings stay above -5 (see module doc)."""
    fixed = [(0.0, 2*PI), (0.0, PI), (0.3, 2.2), (-1.0, 1.0), (1.0, 1.0+1e-9), (0.0, 1e-8),
             (0.0, 0.9999*PI), (0.0, 0.99991*PI), (0.0, 1.9999*PI), (0.0, 1.99991*PI),
             (0.0, 2*PI+1e-3), (5.0, 4.5), (2.0, 2.0), (-7.0, -2.5), (100.0, 101.0),
             (0.0, 4*PI), (-PI, PI), (1e-300, 2e-300), (3.0, 3.0+2e-7), (0.0, 5.9)]
    return fixed+[(u, u+rng.uniform(1e-3, 2*PI)) for u in (rng.uniform(-10, 10) for _ in range(12))]


def append_cos_sin_requests(rng: random.Random, requests: list[str]) -> None:
    """Both BuildCosAndSin overloads for every parameterisation, and the polynomial arc."""
    for param in PARAMETERISATIONS:
        requests.append(line('cv_cossin', [1, param, 0, 0]))
        for u1, u2 in arcs(rng):
            requests.append(line('cv_cossin', [0, param, u1, u2]))
    for u1, u2 in arcs(rng)+[(-20.0, -19.5), (7.0, 12.0), (0.0, 0.0), (13.0, 14.0)]:
        requests.append(line('cv_polycs', [u1, u2]))


def frame2(rng: random.Random, case: int) -> list:
    """Origin, X direction and sense of a 2D frame; the first ones are canonical."""
    if case == 0:
        return [0.0, 0.0, 1.0, 0.0, 1]
    if case == 1:
        return [0.0, 0.0, 1.0, 0.0, 0]
    angle = rng.uniform(-PI, PI)
    return [rng.uniform(-50, 50), rng.uniform(-50, 50), math.cos(angle)*rng.uniform(0.5, 3),
            math.sin(angle)*rng.uniform(0.5, 3), case % 2]


def append_conic_requests(rng: random.Random, requests: list[str]) -> None:
    """Circles, ellipses, hyperbolas and parabolas in canonical and random frames."""
    for case in range(12):
        frame = frame2(rng, case)
        radii = [(5.0, 3.0), (rng.uniform(0.1, 20), rng.uniform(0.01, 0.1)), (2.0, 2.0), (0.0, 0.0)][case % 4]
        for param in PARAMETERISATIONS:
            requests.append(line('cv_conic', [0, 0]+frame+[radii[0], 0, 0, 0, param]))
            requests.append(line('cv_conic', [1, 0]+frame+[radii[0], radii[1], 0, 0, param]))
            for u1, u2 in arcs(rng)[case::4]:
                requests.append(line('cv_conic', [0, 1]+frame+[radii[0], 0, u1, u2, param]))
                requests.append(line('cv_conic', [1, 1]+frame+[radii[0], radii[1], u1, u2, param]))
        for u1, u2 in [(-1.0, 1.0), (1.0, -1.0), (0.0, 0.0), (-5.0, 5.0), (2.0, 3.0),
                       (0.0, 1e-12), (-0.5, 0.25)]:
            requests.append(line('cv_conic', [2, 1]+frame+[radii[0], radii[1], u1, u2, 0]))
            requests.append(line('cv_conic', [3, 1]+frame+[radii[1], 0, u1, u2, 0]))
    append_conic_errors(requests)


def append_conic_errors(requests: list[str]) -> None:
    """Openings at and around the DomainError bounds, and invalid gp conics."""
    base = [0.0, 0.0, 1.0, 0.0, 1]
    for delta in [2*PI, 2*PI+1e-10, 2*PI+1e-8, 0.0, -1e-12, 1e-300]:
        for kind in (0, 1):
            requests.append(line('cv_conic', [kind, 1]+base+[3.0, 2.0, 0.5, 0.5+delta, 0]))
    requests.append(line('cv_conic', [1, 0]+base+[2.0, 3.0, 0, 0, 0]))
    requests.append(line('cv_conic', [0, 0]+base+[-1.0, 0, 0, 0, 0]))
    requests.append(line('cv_conic', [0, 0, 0.0, 0.0, 0.0, 0.0, 1, 1.0, 0, 0, 0, 0]))
    requests.append(line('cv_conic', [3, 1]+base+[0.0, 0, -1.0, 1.0, 0]))
    requests.append(line('cv_conic', [2, 1]+base+[0.0, 1.0, -1.0, 1.0, 0]))


def frame3(rng: random.Random, case: int) -> list:
    """Origin, normal, X direction and the indirect flag of a 3D frame."""
    if case == 0:
        return [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0]
    normal = [rng.uniform(-1, 1) for _ in range(3)]
    x = [rng.uniform(-1, 1) for _ in range(3)]
    return [rng.uniform(-20, 20) for _ in range(3)]+normal+x+[case % 2]


def surface(kind: int, mode: int, frame: list, a: float, b: float, p: list, u_trim: int) -> str:
    """One cv_surface request; surface(0, 1, frame, 2.0, 0.0, [0, 1, -1, 1], 0)."""
    return line('cv_surface', [kind, mode]+frame+[a, b]+p+[u_trim])


def append_surface_requests(rng: random.Random, requests: list[str]) -> None:
    """Whole, patch and band conversions of every quadric."""
    for case in range(10):
        frame = frame3(rng, case)
        append_sphere_requests(rng, requests, frame, [5.0, 0.0, rng.uniform(0.1, 30)][case % 3])
        append_torus_requests(rng, requests, frame, case)
        append_ruled_requests(rng, requests, frame, case)


def append_sphere_requests(rng: random.Random, requests: list[str], frame: list, r: float) -> None:
    """Sphere patches inside and on the latitude bounds, reversed and invalid ranges."""
    requests.append(surface(0, 0, frame, r, 0, [0, 0, 0, 0], 0))
    patches = [[0, PI, -PI/4, PI/4], [0, 2*PI, -PI/2, PI/2], [1, 1, -1, 1], [0, 2*PI+1e-9, 0, 1],
               [0, -0.1, 0, 1], [0, 1, -PI/2-1e-9, 0], [0, 1, 0, PI/2+1e-9], [0, 1, 1, -1],
               [0, 1, 1.5, -1.5], [0, 1, 3, -3], [0.2, 5.0, -1.2, 1.4], [-3, 3, 0.1, 0.1]]
    patches += [[u, u+rng.uniform(0, 2*PI), v, rng.uniform(v, PI/2)]
                for u, v in ((rng.uniform(-7, 7), rng.uniform(-PI/2, PI/2)) for _ in range(6))]
    for p in patches:
        requests.append(surface(0, 1, frame, r, 0, p, 0))
    for p1, p2 in [(0, PI), (0, 2*PI), (-1, 1), (2, 2), (0, 5.2), (0, -1), (0, 2*PI+1e-9), (-PI/2, PI/2)]:
        for u_trim in (1, 0):
            # A periodic-U band opening by 5 PI / 3 or more (up to 2 PI) makes the native
            # ComputePoles write a 7-pole meridian into its 5-entry std::array.
            if u_trim or not 5*PI/3 <= p2-p1 <= 2*PI:
                requests.append(surface(0, 2, frame, r, 0, [p1, p2, 0, 0], u_trim))


def append_torus_requests(rng: random.Random, requests: list[str], frame: list, case: int) -> None:
    """Tori (ring, horn, spindle radii) whole, as patches and as bands."""
    major, minor = [(5.0, 2.0), (1.0, 3.0), (2.0, 2.0), (rng.uniform(1, 9), rng.uniform(0.1, 1))][case % 4]
    requests.append(surface(1, 0, frame, major, minor, [0, 0, 0, 0], 0))
    patches = [[0, PI, 0, PI], [0, 2*PI, 0, 2*PI], [1, 1, 2, 2], [0, 2*PI+1e-9, 0, 1],
               [0, 1, 0, -1e-9], [0, 1, 2, 2+2*PI], [-2, 4, -3, 0.5]]
    patches += [[u, u+rng.uniform(0, 2*PI), v, v+rng.uniform(0, 2*PI)]
                for u, v in ((rng.uniform(-7, 7), rng.uniform(-7, 7)) for _ in range(5))]
    for p in patches:
        requests.append(surface(1, 1, frame, major, minor, p, 0))
    for p1, p2 in [(0, PI), (0, 2*PI), (-1, 1), (2, 2), (0, -1), (1, 1+2*PI+1e-9), (0.5, 5.9)]:
        for u_trim in (1, 0):
            requests.append(surface(1, 2, frame, major, minor, [p1, p2, 0, 0], u_trim))


def append_ruled_requests(rng: random.Random, requests: list[str], frame: list, case: int) -> None:
    """Cylinders and cones, periodic and patches, with empty and reversed heights."""
    radius = [3.0, 0.0, rng.uniform(0.1, 10)][case % 3]
    angle = [PI/6, -PI/4, 0.1, rng.uniform(-1.5, 1.5)][case % 4]
    heights = [(0.0, 10.0), (-5.0, 5.0), (5.0, -5.0), (1.0, 1.0), (1e20, 1e20+1e5), (0.0, 1e-300)]
    for v1, v2 in heights:
        requests.append(surface(2, 0, frame, radius, 0, [v1, v2, 0, 0], 0))
        requests.append(surface(3, 0, frame, radius, angle, [v1, v2, 0, 0], 0))
    ranges = [(0.0, PI), (0.0, 2*PI), (1.0, 1.0), (0.0, 2*PI+1e-9), (0.5, 0.4), (-3.0, 2.5)]
    for (u1, u2), (v1, v2) in zip(ranges*2, heights*2):
        requests.append(surface(2, 1, frame, radius, 0, [u1, u2, v1, v2], 0))
        requests.append(surface(3, 1, frame, radius, angle, [u1, u2, v1, v2], 0))
