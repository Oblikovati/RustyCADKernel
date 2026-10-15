"""Deterministic requests for TopOpeBRepBuild_Tools (tools/oracle/tkbool_probe_topopebrepbuild_tools.cxx;
the records are documented there).

The shapes are those of the TopOpeBRepTool classifier requests (the cube [0, 2]^3 with
its shell and solid, the cylinder face with its seam and circles) with a second cube
[1, 3] x [0.5, 2.5] x [0.5, 2.5] and its solid. The geometric tools run on plane and
cylinder faces at their edges (the seam included); the states are found through the
vertices of the shell, a face and a wire against the solids (one vertex avoided), and
propagated from a face IN; the tolerances are corrected on the shells and the cylinder.
"""
from __future__ import annotations
from tkbool_corpus_topopebreptool_classifier import classifier_scene
from tktopalgo_corpus_brepcheck_shapes import CUBE_FACES, CUBE_POINTS, PlanarShapes, solid_of
from tktopalgo_shape_program import ShapeProgram


def tools_scene(p: ShapeProgram) -> dict:
    """The classifier scene with the shifted cube, its shell and solid."""
    s = classifier_scene(p)
    other = PlanarShapes(p, [(1.0 + 2 * x, 0.5 + 2 * y, 0.5 + 2 * z) for x, y, z in CUBE_POINTS])
    for loop in CUBE_FACES:
        other.face([loop])
    other.finish()
    s['shell2'] = other.shell()
    s['solid2'] = solid_of(p, s['shell2'])
    s['g0'] = other.faces[0]
    return s


def request(op: str, names: list, numbers=()) -> str:
    p = ShapeProgram()
    s = tools_scene(p)
    fields = ' '.join(str(s[n]) if isinstance(n, str) else str(n) for n in names)
    tail = ' '.join(repr(float(v)) for v in numbers)
    return f'{op} {p.tokens()} {fields} {tail}'.rstrip()


GEOM = [('f0', 'e01', 'e01'), ('f0', 'e01', 'e12'), ('f0', 'e12', 'e01'), ('cyl', 'seam', 'seam'),
        ('cyl', 'bottom', 'bottom'), ('cylr', 'bottom', 'seam'), ('f1', 'e01', 'e01'), ('f0r', 'e12', 'e12')]

STATES = [('shell', 'solid2', -1), ('shell', 'solid2', 'v0'), ('shell2', 'solid', -1), ('shell', 'solid', -1),
          ('f0', 'solid2', -1), ('g0', 'solid', 'v1'), ('shell2', 'f0', -1)]

TOLERANCES = [('shell', 1.0), ('shell2', 1e-9), ('cyl', 1.0), ('solid', 0.5)]


def append_topopebrepbuild_tools_requests(requests: list[str]) -> None:
    """The geometric tools, the state propagation and the tolerance corrections."""
    for names in GEOM:
        requests.append(request('tb_tools_geom', list(names)))
    for s, ref, av in STATES:
        requests.append(request('tb_tools_states', [s, ref, av]))
    for s, tol in TOLERANCES:
        requests.append(request('tb_tools_tolerances', [s], (tol,)))
