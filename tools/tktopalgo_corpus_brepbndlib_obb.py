"""Deterministic requests for BRepBndLib::AddOBB (tools/oracle/tktopalgo_probe_brepbndlib_obb.cxx).

Every request carries its shape record, so it never relies on an earlier request.
`bbl_obb` runs AddOBB with the eight flag triples (triangulation, optimal, shape
tolerance) on every shape of the BRepBndLib corpus (solids, faces, free edges, free
vertices, an empty solid), on a shell, on curved faces and solids that carry a
triangulation (so the DiTO path reads their nodes), on a located box and on rotated
curved solids (the PCA path in a rotated frame of inertia), into a void box, then on
some of them into an axis-aligned and a rotated initial box.
"""
from __future__ import annotations
import math
from tktopalgo_corpus_brepbndlib import bnd_shapes, free_edges, located, numbers, single
from tktopalgo_corpus_shapes import EDGE, FACE, SHELL, Mesh, ShapeSpec, rotation, translation
from tktopalgo_corpus_solids import box, cylinder, sphere, torus

# (triangulation, optimal, shape tolerance): the eight flag triples.
FLAGS = [(t, o, g) for t in (1, 0) for o in (0, 1) for g in (1, 0)]
COS30, SIN30 = math.cos(math.pi / 6.0), math.sin(math.pi / 6.0)
# Initial boxes: void, an axis-aligned box, a box rotated about z by 30 degrees.
INITIAL_OBBS = [
    [0],
    [1, 1.0, 2.0, 3.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5, 1.0, 1.5, 1],
    [1, 0.0, 0.0, 0.0, COS30, SIN30, 0.0, -SIN30, COS30, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0, 0.5, 0],
]


def meshed_sphere(radius: float) -> ShapeSpec:
    """A sphere solid whose face carries the octahedron of its six poles as mesh."""
    spec = sphere([0.5, -0.25, 1.0], radius)
    c = [0.5, -0.25, 1.0]
    nodes = [[c[0] + radius, c[1], c[2]], [c[0], c[1] + radius, c[2]], [c[0] - radius, c[1], c[2]],
             [c[0], c[1] - radius, c[2]], [c[0], c[1], c[2] + radius], [c[0], c[1], c[2] - radius]]
    triangles = [(1, 2, 5), (2, 3, 5), (3, 4, 5), (4, 1, 5), (2, 1, 6), (3, 2, 6), (4, 3, 6), (1, 4, 6)]
    spec.faces[0].mesh = Mesh(nodes, triangles, 0.3)
    return spec


def meshed_cylinder() -> ShapeSpec:
    """A cylinder solid whose lateral face carries a mesh of twelve nodes (six per rim)."""
    spec = cylinder([1.0, 2.0, 0.0], 1.5, 4.0)
    ring = [[1.0 + 1.5 * math.cos(k * math.pi / 3.0), 2.0 + 1.5 * math.sin(k * math.pi / 3.0)] for k in range(6)]
    nodes = [[x, y, 0.0] for x, y in ring] + [[x, y, 4.0] for x, y in ring]
    triangles = [t for k in range(6) for t in ((k + 1, (k + 1) % 6 + 1, k + 7), ((k + 1) % 6 + 1, (k + 1) % 6 + 7, k + 7))]
    spec.faces[0].mesh = Mesh(nodes, triangles, 0.05)
    return spec


def shell_of(spec: ShapeSpec) -> ShapeSpec:
    """The first shell of `spec`, alone."""
    spec.top = [(SHELL, 0)]
    return spec


def obb_shapes() -> list[ShapeSpec]:
    """Every shape of the AddOBB corpus: the BRepBndLib shapes, then the extra ones."""
    return bnd_shapes() + [
        shell_of(box([0.0, 0.0, 0.0], [2.0, 3.0, 4.0])),
        shell_of(cylinder([0.0, 0.0, 0.0], 1.0, 3.0)),
        meshed_sphere(2.0),
        meshed_cylinder(),
        single(FACE, meshed_cylinder(), 0),
        located(box([0.0, 0.0, 0.0], [3.0, 1.0, 0.5]), rotation([0.0, 0.0, 0.0], [1.0, 2.0, 3.0], 0.9)),
        located(cylinder([0.0, 0.0, 0.0], 0.5, 6.0), rotation([1.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0.6)),
        located(torus([0.0, 0.0, 0.0], 3.0, 0.5), rotation([0.0, 0.0, 0.0], [1.0, 0.0, 1.0], 0.4)),
        located(meshed_cylinder(), translation(-3.0, 1.0, 2.0)),
        single(EDGE, free_edges(), 1),
    ]


def append_brepbndlib_obb_requests(requests: list[str]) -> None:
    """Append every `bbl_obb` request: each shape with each flag triple into a void
    box, then some shapes into the two initial boxes with four flag triples."""
    shapes = obb_shapes()
    for spec in shapes:
        record = spec.numbers()
        for flags in FLAGS:
            requests.append('bbl_obb ' + numbers([*record, *INITIAL_OBBS[0], *flags]))
    chosen = [0, 3, 6, 11, 13, 18, 22, 29, 30, 34, 38]
    for index in chosen:
        record = shapes[index].numbers()
        for initial in INITIAL_OBBS[1:]:
            for flags in FLAGS[::2]:
                requests.append('bbl_obb ' + numbers([*record, *initial, *flags]))
