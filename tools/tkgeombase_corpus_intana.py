"""Deterministic requests for the IntAna package (tools/oracle/tkgeombase_probe_intana.cxx).

Every intersection pair gets random inputs plus the configurations that reach each
branch: coaxial, parallel, perpendicular, intersecting and skew axes; tangent,
secant, identical, nested and missing surfaces; apexes on the other surface; conics
in, parallel to and across quadrics and planes; near-degenerate angles and
distances around the inner tolerances (1e-14, 1e-13, Precision::Angular/Confusion).
"""
from __future__ import annotations
import math
import random
from tkg3d_corpus import axis, frame, point, unit
from tkgeombase_corpus_gce import add

Z = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0]


def ax3(p, n, x):
    return [*p, *n, *x]


def pln(f):
    return [0, *f]


def cyl(f, r):
    return [1, *f, r]


def sph(f, r):
    return [2, *f, r]


def cone(f, a, r):
    return [3, *f, a, r]


def tor(f, big, small):
    return [4, *f, big, small]


def at(x, y, z, n=(0.0, 0.0, 1.0), xd=(1.0, 0.0, 0.0)):
    return ax3([x, y, z], list(n), list(xd))


def random_surface(rng, kind):
    f = frame(rng)
    return [pln(f), cyl(f, rng.uniform(0.5, 3.0)), sph(f, rng.uniform(0.5, 4.0)),
            cone(f, rng.uniform(0.2, 1.2), rng.uniform(0.0, 2.0)), tor(f, 4.0, rng.uniform(0.5, 2.0))][kind]


def random_conic(rng, kind):
    if kind == 0:
        return [0, *axis(rng)]
    f = frame(rng)
    return [[1, *f, rng.uniform(0.5, 3.0)], [2, *f, 3.0, rng.uniform(0.5, 2.5)], [3, *f, rng.uniform(0.25, 2.0)],
            [4, *f, rng.uniform(0.5, 2.0), rng.uniform(0.5, 2.0)]][kind - 1]


def quadric_requests(rng, requests):
    for kind in range(4):
        for _ in range(3):
            add(requests, 'ia_quadric', *random_surface(rng, kind), *frame(rng))
    add(requests, 'ia_quadric', *cone(Z, 0.5, 0.0), *Z)
    add(requests, 'ia_quadric', *sph(Z, 2.0), *at(1, 2, 3, (0, 1, 0), (0, 0, 1)))


def conic_quad_requests(rng, requests):
    for _ in range(3):
        for c in range(5):
            for q in range(4):
                add(requests, 'ia_conic_quad', *random_conic(rng, c), *random_surface(rng, q))
    designed = [
        ([0, -5, 0, 0, 1, 0, 0], sph(Z, 2.0)), ([0, -5, 2, 0, 1, 0, 0], sph(Z, 2.0)),
        ([0, -5, 3, 0, 1, 0, 0], sph(Z, 2.0)), ([0, 1, 0, 0, 0, 0, 1], cyl(Z, 1.0)),
        ([0, 2, 0, 0, 0, 0, 1], cyl(Z, 1.0)), ([0, 0, 0, 0, 0, 0, 1], cyl(Z, 1.0)),
        ([0, 0, 0, 5, 0, 0, -1], cone(Z, 0.5, 1.0)), ([0, 0, 0, 1, 1, 0, 0], pln(Z)),
        ([0, 0, 0, 0, 1, 0, 0], pln(Z)), ([1, *Z, 2.0], sph(Z, 2.0)), ([1, *Z, 1.0], cyl(Z, 1.0)),
        ([1, *at(0, 0, 1), 2.0], sph(Z, 2.0)), ([1, *at(0, 0, 0, (1, 0, 0), (0, 1, 0)), 1.0], cyl(Z, 1.0)),
        ([2, *Z, 3.0, 1.0], cyl(Z, 1.0)), ([2, *Z, 3.0, 1.0], pln(at(0, 0, 0, (1, 0, 0), (0, 1, 0)))),
        ([3, *Z, 1.0], pln(at(0, 0, 0, (0, 1, 0), (0, 0, 1)))), ([3, *Z, 1.0], sph(Z, 3.0)),
        ([4, *Z, 1.0, 1.0], pln(at(2, 0, 0, (1, 0, 0), (0, 1, 0)))), ([4, *Z, 1.0, 1.0], sph(Z, 1.0)),
        ([4, *Z, 1.0, 1.0], cyl(at(0, 0, 0, (1, 0, 0), (0, 1, 0)), 0.5)),
    ]
    for c, q in designed:
        add(requests, 'ia_conic_quad', *c, *q)


def conic_plane_requests(rng, requests):
    for _ in range(3):
        for c in range(5):
            add(requests, 'ia_conic_plane', *random_conic(rng, c), *frame(rng), 1e-12, 1e-7, rng.choice([0.0, 10.0]))
    xz = at(0, 0, 0, (0, 1, 0), (0, 0, 1))
    for eps in [0.0, 1e-13, 1e-10, 1e-6]:
        d = [1.0, eps, 0.0]
        add(requests, 'ia_conic_plane', 0, 0.0, 0.0, 0.0, *d, *xz, 1e-9, 1e-7, 0.0)
        add(requests, 'ia_conic_plane', 0, 0.0, 1.0, 0.0, *d, *xz, 1e-9, 1e-7, 100.0)
        add(requests, 'ia_conic_plane', 0, 0.0, 1e-8, 0.0, *d, *xz, 1e-9, 1e-7, 1e3)
    for h in [0.0, 0.5, 1.0, 1.0 + 1e-9, 2.0]:
        add(requests, 'ia_conic_plane', 1, *Z, 1.0, *at(0, h, 0, (0, 1, 0), (0, 0, 1)), 1e-12, 1e-7, 0.0)
        add(requests, 'ia_conic_plane', 1, *Z, 1.0, *at(0, 0, h, (0, 0, 1), (1, 0, 0)), 1e-12, 1e-7, 0.0)
    add(requests, 'ia_conic_plane', 1, *Z, 1.0, *Z, 1e-12, 1e-7, 0.0)
    add(requests, 'ia_conic_plane', 1, *Z, 1.0, *at(0, 0, 1e-9), 1e-12, 1e-7, 0.0)
    add(requests, 'ia_conic_plane', 1, *Z, 1.0, *at(0, 0, 0, (0, math.sin(1e-13), math.cos(1e-13))), 1e-12, 1e-7, 0.0)
    for c in [[2, *Z, 3.0, 1.0], [3, *Z, 1.0], [4, *Z, 1.0, 1.0]]:
        for plane in [Z, at(0, 0, 0, (1, 0, 0), (0, 1, 0)), at(2, 0, 0, (1, 0, 0), (0, 1, 0)), at(0, 1, 0, (0, 1, 0), (0, 0, 1))]:
            add(requests, 'ia_conic_plane', *c, *plane, 1e-12, 1e-7, 0.0)


def lin_torus_requests(rng, requests):
    for _ in range(10):
        add(requests, 'ia_lin_torus', *axis(rng, 3.0), *frame(rng, 1.0), 4.0, rng.uniform(0.5, 2.0))
    for p, d in [((0, 0, -9), (0, 0, 1)), ((-9, 0, 0), (1, 0, 0)), ((-9, 0, 1), (1, 0, 0)), ((-9, 0, 1.5), (1, 0, 0)),
                 ((-9, 3, 0), (1, 0, 0)), ((-9, 5, 0), (1, 0, 0)), ((-9, 6, 0), (1, 0, 0)), ((4, 0, -9), (0, 0, 1)),
                 ((5, 0, -9), (0, 0, 1)), ((3, 0, -9), (0, 0, 1)), ((-9, 0, 0), (0.6, 0.8, 0))]:
        add(requests, 'ia_lin_torus', *p, *d, *Z, 4.0, 1.0)


def append_intana_requests(requests: list[str]) -> None:
    """Append every IntAna request family in a fixed order."""
    rng = random.Random(0x1A0A)
    quadric_requests(rng, requests)
    conic_quad_requests(rng, requests)
    conic_plane_requests(rng, requests)
    lin_torus_requests(rng, requests)
    __import__('tkgeombase_corpus_intana_geo').append_intana_geo_requests(rng, requests)
