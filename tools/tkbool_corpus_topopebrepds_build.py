"""Deterministic requests for TopOpeBRepDS_DataStructure, TopOpeBRepDS_HDataStructure and
TopOpeBRepDS_BuildTool (tools/oracle/tkbool_probe_topopebrepds_build.cxx; the records are
documented there).

`ds_recompute` rebuilds the section edge of the square face on z = 1 and the cylinder
of radius 1 (their circle): walking polylines approximated (APPROX) or kept as they are
(BSPLINE1), and the circle projected on the faces, with the 3d curve and the p-curves
asked or not, including a polyline of more than 100 poles. `ds_same_domain` relates two
cube faces, a face and its reversed copy, and two cylinder faces, by reference first or
second and with configurations; `ds_sort` sorts curve/point interferences.
"""
from __future__ import annotations
import math
from tkbool_corpus_topopebreptool_curve_tool import arc_points, faces, pole_rich_points
from tktopalgo_corpus_brepcheck_shapes import cube, cylinder_face
from tktopalgo_shape_program import REVERSED, ShapeProgram


def recompute_request(walk: int, ty: int, comps, points: list, pars) -> str:
    p = ShapeProgram()
    f = faces(p)
    fields = ' '.join(repr(float(v)) for q in points for v in q)
    return (f"ds_recompute {p.tokens()} {f['plane']} {f['cyl']} {walk} {ty} {comps[0]} {comps[1]} {comps[2]} "
            f"{pars[0]!r} {pars[1]!r} {len(points)} {fields}")


def same_domain_request(a: str, b: str, ref: int, c1: int, c2: int, mode: int) -> str:
    p = ShapeProgram()
    c = cube(p, 2.0)
    cyl = cylinder_face(p, 1.0, 2.0)
    s = {'f0': c.faces[0], 'f1': c.faces[1], 'f0r': p.oriented(c.faces[0], REVERSED), 'cyl': cyl['face'],
         'cylr': p.oriented(cyl['face'], REVERSED), 'e01': c.edges[(0, 1)], 'e12': c.edges[(1, 2)]}
    return f'ds_same_domain {p.tokens()} {s[a]} {s[b]} {ref} {c1} {c2} {mode}'


def append_recompute_requests(requests: list[str]) -> None:
    pts = arc_points(12)
    for ty in (1, 0):
        for comps in ((1, 1, 1), (1, 1, 0), (1, 0, 1), (0, 1, 1), (1, 0, 0)):
            requests.append(recompute_request(1, ty, comps, pts, (1.0, 12.0)))
            requests.append(recompute_request(0, ty, comps, pts[:3], (0.3, 2.5)))
    requests.append(recompute_request(1, 1, (1, 1, 1), pts, (3.0, 9.0)))
    requests.append(recompute_request(0, 1, (1, 1, 1), pts[:3], (2.5, 0.3)))
    requests.append(recompute_request(0, 1, (1, 1, 1), pts[:3], (5.0, 7.0)))
    rich = pole_rich_points()
    requests.append(recompute_request(1, 1, (1, 1, 1), rich, (1.0, float(len(rich)))))


def append_same_domain_requests(requests: list[str]) -> None:
    for a, b in (('f0', 'f1'), ('f0', 'f0r'), ('cyl', 'cylr'), ('f1', 'cyl'), ('e01', 'e12')):
        for ref in (1, 0):
            requests.append(same_domain_request(a, b, ref, 1, 1, 0))
        for c1, c2 in ((0, 0), (0, 1), (2, 0)):
            requests.append(same_domain_request(a, b, 1, c1, c2, 1))


def append_sort_requests(requests: list[str]) -> None:
    lists = [
        [(0.5, 0), (0.2, 1), (0.9, 0), (0.1, 2)],
        [(0.3, 1), (0.3, 0), (0.1, 1)],
        [(1.0, 3)],
        [],
        [(2.0, 1), (math.pi, 1), (-1.0, 0), (0.0, 0), (5.0, 2)],
    ]
    for l in lists:
        fields = ' '.join(f'{par!r} {o}' for par, o in l)
        requests.append(f'ds_sort {len(l)} {fields}'.rstrip())


def append_topopebrepds_build_requests(requests: list[str]) -> None:
    """The section edge rebuilds, same domain relations and sorts."""
    append_recompute_requests(requests)
    append_same_domain_requests(requests)
    append_sort_requests(requests)
