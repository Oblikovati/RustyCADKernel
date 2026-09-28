"""Deterministic GC surface and transformation requests (tools/oracle/tkgeombase_probe_gc_surfaces.cxx).

Planes from every overload (null equations included), cones and cylinders from frames,
points and radii (bad angles at both resolution bounds, parallel radii, confused
points, collinear points, projections outside the Extrema_ExtPElC bounds), trimmed
cones and cylinders, and every transformation maker with null and negative factors.
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import axis, frame, point
from tkgeombase_corpus_gce import AXES, RES, TINY, add, axis_frames, near, normalized, point2d, unit2d


def plane_requests(rng, requests):
    for f in [frame(rng) for _ in range(3)] + axis_frames(rng)[:3]:
        add(requests, 'gc_plane', 0, *f)
        add(requests, 'gc_plane', 1, *f[0:6])
        add(requests, 'gc_plane', 6, *f[0:6])
        add(requests, 'gc_plane', 5, *f, *point(rng))
        for dist in [2.0, 0.0, -1.5]:
            add(requests, 'gc_plane', 4, *f, dist)
    for coefficients in [(1.0, 2.0, 3.0, 4.0), (0.0, 0.0, 1.0, 0.0), (0.0, 0.0, 0.0, 1.0), (1e-160, 0.0, 0.0, 1.0),
                         (1e-150, 1e-150, 1e-150, 0.0), (RES, 0.0, 0.0, 0.0), (-2.0, 0.0, 0.0, 3.0), (1e200, 0.0, 0.0, 0.0)]:
        add(requests, 'gc_plane', 2, *coefficients)
    for _ in range(6):
        add(requests, 'gc_plane', 3, *point(rng), *point(rng), *point(rng))
    a, b = point(rng), point(rng)
    add(requests, 'gc_plane', 3, *a, *b, *[(x + y) / 2 for x, y in zip(a, b)])
    add(requests, 'gc_plane', 3, *a, *a, *b)


def cone_requests(rng, requests):
    frames = [frame(rng) for _ in range(2)] + axis_frames(rng)[:3]
    for f in frames:
        for angle, radius in [(0.5, 2.0), (0.5, 0.0), (0.5, -1.0), (0.5 * RES, 2.0), (RES, 1.0), (-0.3, 1.0),
                              (math.pi / 2, 1.0), (math.pi / 2 - 1e-300, 1.0), (math.pi / 2 - 1e-15, 1.0), (1.5, 1.0)]:
            add(requests, 'gc_conical', 0, *f, angle, radius)
        add(requests, 'gc_conical', 1, *f, 0.5, 3.0)
        add(requests, 'gc_conical', 1, *f, -0.7, 0.0)
    for _ in range(6):
        pts = [point(rng) for _ in range(4)]
        add(requests, 'gc_conical', 2, *[c for p in pts for c in p])
        add(requests, 'gc_trimmed_cone', 0, *[c for p in pts for c in p])
    o, z = [0.0, 0.0, 0.0], [0.0, 0.0, 10.0]
    for p3, p4 in [([4, 0, 0], [2, 0, 5]), ([3, 0, 0], [3, 0, 5]), ([3, 0, 2], [5, 0, 2]), ([0, 0, 0], [2, 0, 5]),
                   ([2, 0, -3], [5, 0, 4]), ([1, 0, 1e150], [2, 0, 3e150])]:
        pts = [*o, *z, *[float(v) for v in p3], *[float(v) for v in p4]]
        add(requests, 'gc_conical', 2, *pts)
        add(requests, 'gc_trimmed_cone', 0, *pts)
    for _ in range(4):
        a, b = point(rng), point(rng)
        for r1, r2 in [(1.0, 2.0), (2.0, 1.0), (1.0, 1.0), (0.0, 1.0), (-1.0, 1.0)]:
            add(requests, 'gc_conical', 3, *a, *b, r1, r2)
            add(requests, 'gc_trimmed_cone', 1, *a, *b, r1, r2)
        add(requests, 'gc_conical', 3, *a, *a, 1.0, 2.0)
        add(requests, 'gc_trimmed_cone', 1, *a, *a, 1.0, 2.0)
    for d in AXES:
        add(requests, 'gc_trimmed_cone', 1, 0.0, 0.0, 0.0, *normalized(d), 1.0, 2.0)


def cylinder_requests(rng, requests):
    for f in [frame(rng) for _ in range(2)] + axis_frames(rng):
        for radius in [2.0, 0.0, -1.0]:
            add(requests, 'gc_cylindrical', 1, *f, radius)
            add(requests, 'gc_cylindrical', 2, *f[0:6], radius)
            add(requests, 'gc_cylindrical', 5, *f, 3.0, radius)
            add(requests, 'gc_trimmed_cylinder', 2, *f[0:6], radius, 4.0)
        add(requests, 'gc_cylindrical', 0, *f, 1.5)
        add(requests, 'gc_cylindrical', 3, *f, 1.5)
        add(requests, 'gc_cylindrical', 3, *f, 0.0)
        add(requests, 'gc_cylindrical', 5, *f, 3.0, 5.0)
        add(requests, 'gc_cylindrical', 6, *f, 3.0, *point(rng))
        for height in [2.0, 0.0, -3.0]:
            add(requests, 'gc_trimmed_cylinder', 1, *f, 1.5, height)
    for _ in range(6):
        pts = [*point(rng), *point(rng), *point(rng)]
        add(requests, 'gc_cylindrical', 4, *pts)
        add(requests, 'gc_trimmed_cylinder', 0, *pts)
    for delta in TINY:
        p = point(rng)
        add(requests, 'gc_cylindrical', 4, *p, *near(p, delta), *point(rng))
        add(requests, 'gc_trimmed_cylinder', 0, *p, *near(p, delta), *point(rng))
    add(requests, 'gc_trimmed_cylinder', 0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5)


def transformation_requests(rng, requests):
    for _ in range(3):
        a, f = axis(rng), frame(rng)
        add(requests, 'gc_mirror', 0, *point(rng))
        for kind in (1, 2, 3):
            add(requests, 'gc_mirror', kind, *a)
        add(requests, 'gc_mirror', 4, *f)
        add(requests, 'gc_mirror', 5, *f)
        a2 = [*point2d(rng), *unit2d(rng)]
        add(requests, 'gc_mirror2d', 0, *a2[0:2])
        for kind in (1, 2, 3):
            add(requests, 'gc_mirror2d', kind, *a2)
        for angle in [0.7, 0.0, -2.0 * math.pi, 1e10]:
            for kind in range(3):
                add(requests, 'gc_rotation', kind, *a, angle)
            add(requests, 'gc_rotation2d', *a2[0:2], angle)
        for scale in [2.0, -1.0, 1.0, 0.0, 1e-300, -0.5]:
            add(requests, 'gc_scale', *point(rng), scale)
            add(requests, 'gc_scale2d', *point2d(rng), scale)
        add(requests, 'gc_translation', 0, *point(rng))
        add(requests, 'gc_translation', 1, *point(rng), *point(rng))
        add(requests, 'gc_translation2d', 0, *point2d(rng))
        add(requests, 'gc_translation2d', 1, *point2d(rng), *point2d(rng))


def append_gc_surface_requests(requests: list[str]) -> None:
    """Append the GC surface and transformation requests in a fixed order."""
    rng = random.Random(0x6C5)
    plane_requests(rng, requests)
    cone_requests(rng, requests)
    cylinder_requests(rng, requests)
    transformation_requests(rng, requests)
