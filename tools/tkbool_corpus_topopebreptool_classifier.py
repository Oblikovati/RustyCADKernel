"""Deterministic requests for TopOpeBRepTool_SolidClassifier, TopOpeBRepTool_ShapeClassifier,
the FSC_* functions and TOOL::TggeomE (tools/oracle/tkbool_probe_topopebreptool_classifier.cxx;
the records are documented there).

The shapes are those of the TopOpeBRepTool shape requests (the cube [0, 2]^3, the
cylinder face, segments on and off them) with the solid of the cube shell and a segment
inside the cube. Points lie inside, outside, on faces and edges of the cube; shapes are
classified against the solid, the shell and faces, in and out of same domain mode,
avoiding edges; tangents are taken inside, at and beyond the bounds of edges and on a
closed circle.
"""
from __future__ import annotations
import math
from tkbool_corpus_topopebreptool_shapes import scene
from tkprim_corpus_sweep_shapes import segment
from tktopalgo_shape_program import SOLID, ShapeProgram


def classifier_scene(p: ShapeProgram) -> dict:
    """The shapes of the shape requests, the solid of the shell and an inner segment."""
    s = scene(p)
    s['solid'] = p.container(SOLID)
    p.add(s['solid'], s['shell'])
    s['inner'] = segment(p, (0.5, 0.5, 0.5), (1.5, 1.0, 1.2))
    return s


def request(op: str, names: list[str], numbers=()) -> str:
    p = ShapeProgram()
    s = classifier_scene(p)
    fields = ' '.join(str(s[n]) for n in names)
    tail = ' '.join(repr(float(v)) if isinstance(v, float) else str(v) for v in numbers)
    return f'{op} {p.tokens()} {fields} {tail}'.rstrip()


POINTS = [(1.0, 1.0, 1.0), (3.0, 1.0, 1.0), (1.0, 1.0, 0.0), (0.0, 0.0, 1.0), (1.0, 1.0, 2.0 + 1e-8),
          (2.0, 2.0, 2.0), (-1e-3, 1.0, 1.0)]

SHAPES = [('above', 'solid', 0, -1), ('inner', 'solid', 0, -1), ('inner', 'shell', 0, 'e01'),
          ('e01', 'f0', 1, -1), ('e01', 'f0', 0, 'e12'), ('e12', 'f0', 1, 'e01'), ('v0', 'solid', 0, -1),
          ('f1', 'solid', 0, -1), ('f1', 'f0', 1, -1), ('onc', 'cyl', 0, -1), ('onc', 'cyl', 1, 'seam'),
          ('bottom', 'cyl', 1, -1), ('above', 'f1', 0, -1), ('inner', 'f0', 1, -1), ('cyl', 'shell', 0, -1)]

FSC = [('e01', 'f0', 0.5, (1.0, 0.0, 0.0)), ('e12', 'f0', 0.25, (1.0, 1.0, 0.5)), ('onc', 'cyl', 0.5, (1.0, 0.0, 1.0)),
       ('above', 'f1', 0.3, (1.0, 1.0, 2.0)), ('inner', 'f0', 0.7, (5.0, 5.0, 0.0)), ('bottom', 'cyl', 0.1, (0.0, 0.0, 9.0))]

TGGEOM = [('e01', (0.0, 0.5, 1.0, -1.0, 1.0 + 1e-9)), ('seam', (0.0, 1.0, 2.0, 2.5)),
          ('bottom', (0.0, math.pi, 2 * math.pi, 7.0)), ('above', (0.0, 0.3, 2.0))]


def append_topopebreptool_classifier_requests(requests: list[str]) -> None:
    """The solid, shape and FSC classifications and the edge tangents."""
    for tol in (1e-7, 1e-2):
        coords = [c for q in POINTS for c in q]
        requests.append(request('tt_solid_classifier', ['shell', 'solid'], (tol, len(POINTS), *coords)))
    for s, ref, sd, av in SHAPES:
        p = ShapeProgram()
        sc = classifier_scene(p)
        avi = sc[av] if isinstance(av, str) else av
        requests.append(f'tt_shape_classifier {p.tokens()} {sc[s]} {sc[ref]} {sd} {avi}')
    for e, f, t, q in FSC:
        requests.append(request('tt_fsc', [e, f], (t, *q)))
    for e, pars in TGGEOM:
        requests.append(request('tt_tggeom', [e], (len(pars), *pars)))
