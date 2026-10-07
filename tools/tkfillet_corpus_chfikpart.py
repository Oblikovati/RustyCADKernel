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
