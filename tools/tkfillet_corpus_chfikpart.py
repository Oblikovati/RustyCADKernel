"""Deterministic requests for the ChFiKPart probes (tools/oracle/tkfillet_probe_chfikpart.cxx).

Periodic parameters, straight pcurves, curves projected on elementary surfaces, and the
guide circles of spherical corners between a plane and a plane or a cylinder.
"""
from __future__ import annotations
import math

from tkfillet_corpus_blendfunc import numbers
from tkfillet_corpus_builder0 import CIRCLE2, LINE_Y1, SLANT, SURFACES, surface
from tkfillet_corpus_chfi3d import guide

PI = math.pi


def append_chfikpart_requests(requests: list[str]) -> None:
    """Append the ChFiKPart request families."""
    plane, cyl, sph, cone, tor = SURFACES[0], SURFACES[2], SURFACES[3], SURFACES[4], SURFACES[5]
    for (u, f, l), (g, gf, gl, s) in zip(((7, 0, 5), (-3, 0, 2 * PI), (2 * PI, 0, 2 * PI), (0.5, 0, 1), (-1e-12, 0, 1), (12, -1, 1)),
                                         ((LINE_Y1, 0, 3, plane), (CIRCLE2, 0, PI, cyl), (SLANT, -1, 1, plane), (CIRCLE2, 0, 1, sph),
                                          (CIRCLE2, 0.5, 2.0, cone), (guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 5.0), 0, 1, tor))):
        for uv1, uv2, pd, pf in (((0, 0), (1, 2), 0, 1), ((1, 1), (1, 3), 2, 4)):
            requests.append('fl_kpart_fcts ' + numbers([u, f, l, 1e-9, *uv1, *uv2, pd, pf, *g, gf, gl, *s]))
    plane_x = surface(0, (0, 0, 0), (1, 0, 0), (0, 1, 0))
    for s2, p1s1, p1s2, p2s2 in ((plane_x, (1, 1), (1, 1), (2, 3)), (cyl, (0.5, 0.2), (0.1, 0.2), (0.6, 0.9)),
                                 (surface(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), a=3.0), (2, 0.5), (0.2, 0.0), (0.4, 1.0))):
        for rad in (0.5, 2.0):
            requests.append('fl_corner_spine ' + numbers([*plane, *s2, *p1s1, *p1s2, *p2s2, rad]))
    append_kpart_make_requests(requests)
    append_kpart_corner_requests(requests)


ORIENTATION_PAIRS = ((0, 0), (1, 0), (0, 1), (1, 1))


def kpart_make(op, s1, s2, ors, ofpl, spine, first, a, b, plandab=0, dison=0):
    """`fl_kpart_make`: op 0 fillet, 1..3 chamfer of mode op-1, 4 asymmetric chamfer."""
    return 'fl_kpart_make ' + numbers([op, *s1, *s2, *ors, ofpl, *spine, first, a, b, 0, 2 * PI, plandab, dison])


def append_kpart_make_requests(requests: list[str]) -> None:
    """The closed-form fillets and chamfers between a plane and a plane, a cylinder along
    a line or a circle, and a cone along a circle, with every orientation."""
    plane_z = surface(0, (0, 0, 0), (0, 0, 1), (1, 0, 0))
    root5 = math.sqrt(5.0)
    cases = (
        (surface(0, (0, 0, 0), (1, 0, 0), (0, 1, 0)), guide(0, (0, 0, 0), (0, 1, 0), (1, 0, 0), 0), ((0.5, 0.8), (2.0, 1.0))),
        (surface(0, (0, 0, 0), (1, 0, 1), (0, 1, 0)), guide(0, (0, 0, 0), (0, 1, 0), (1, 0, 0), 0), ((0.5, 0.6),)),
        (surface(1, (0, 0, 2), (1, 0, 0), (0, 1, 0), a=3.0), guide(0, (0, root5, 0), (1, 0, 0), (0, 1, 0), 0), ((0.5, 0.7), (1.0, 0.4))),
        (surface(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), a=3.0), guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 3.0), ((0.5, 0.8), (1.0, 0.5))),
        (surface(3, (0, 0, 0), (0, 0, 1), (1, 0, 0), a=0.5, b=3.0), guide(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), 3.0), ((0.5, 0.6), (1.0, 0.4))),
    )
    for s2, spine, dims in cases:
        for op in range(5):
            for ors in ORIENTATION_PAIRS:
                for ofpl in (0, 1):
                    for a, b in dims:
                        for plandab, dison in ((1, 1), (0, 0)):
                            requests.append(kpart_make(op, plane_z, s2, ors, ofpl, spine, 0.0, a, b, plandab, dison))


def kpart_corner(kind, s1, s2, s3, ors, r1, r2, pts):
    """`fl_kpart_corner`: kind 0 torus, 1 sphere, 2 torus joint of three planes."""
    return 'fl_kpart_corner ' + numbers([kind, *s1, *s2, *s3, *ors, r1, r2, *(c for p in pts for c in p)])


def append_kpart_corner_requests(requests: list[str]) -> None:
    """The corners: the torus and the sphere between a plane and a plane or a cylinder,
    and the torus joint of three planes."""
    plane_z = surface(0, (0, 0, 0), (0, 0, 1), (1, 0, 0))
    plane_x = surface(0, (0, 0, 0), (1, 0, 0), (0, 1, 0))
    plane_y = surface(0, (0, 0, 0), (0, 1, 0), (0, 0, 1))
    cyl = surface(1, (0, 0, 0), (0, 0, 1), (1, 0, 0), a=3.0)
    for ors in ((0, 0, 0, 0, 0), (1, 0, 1, 0, 1), (0, 1, 0, 1, 1), (1, 1, 1, 1, 0)):
        for s2, pts in ((plane_x, ((1, 1), (2, 3), (1, 1), (2, 3))), (cyl, ((3, 0), (0, 3), (0, 0), (PI / 2, 0))),
                        (cyl, ((2.5, 0.5), (0.5, 2.5), (0.2, 0.1), (1.2, 0.1)))):
            for r1, r2 in ((0.5, 1.0), (1.0, 2.0)):
                requests.append(kpart_corner(0, plane_z, s2, plane_y, ors, r1, r2, pts))
                requests.append(kpart_corner(1, plane_z, s2, plane_y, ors, r1, r2, pts))
        for r1 in (0.5, 1.5):
            requests.append(kpart_corner(2, plane_z, plane_x, plane_y, ors, r1, 0.0, ((0, 0),) * 4))
