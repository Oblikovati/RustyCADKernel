"""Deterministic requests for the HLRAlgo polyhedral classes
(tools/oracle/tkhlr_probe_hlralgo_poly.cxx).

- `ha_polydata`: faces on integer grids (segments through vertices, along edges, in the
  plane) and on random coordinates, with every edge flag combination, then
  HideByPolyData of segments in front of, behind and across the face planes, with and
  without connexity to the face, and packed boxes that pass or fail the box test;
- `ha_polyalgo`: shells of triangulated patches and tetrahedra with hiding and
  non-hiding faces, shells without segments or without hiding faces, then Update and
  the hide and show walks;
- `ha_internal`: grid triangulations with UpdateLinks, then AddNode and the split
  UpdateLinks on interior and boundary segments (growing every array), a segment that
  is not there, the flag setters and Dump;
- `ha_grow`: the Inc members with the same or different pointers until the arrays
  double, the Dec members, and an internal data of no node (Standard_OutOfRange).

Every array entry that a request prints is written first; the hiding arrays have
exactly one entry per hiding triangle, as HLRBRep sizes them (docs/tkhlr-progress.md).
"""
from __future__ import annotations
import math
import random

HIDING = 256


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def plane_of(nodes: list[tuple], tri: tuple) -> list[float]:
    """The unit normal and constant of a triangle (zero for a degenerate one)."""
    p1, p2, p3 = (nodes[i - 1] for i in tri)
    d1 = [p2[k] - p1[k] for k in range(3)]
    d2 = [p3[k] - p2[k] for k in range(3)]
    n = [d1[1] * d2[2] - d2[1] * d1[2], d1[2] * d2[0] - d2[2] * d1[0], d1[0] * d2[1] - d2[0] * d1[1]]
    length = math.sqrt(sum(c * c for c in n))
    n = [c / length for c in n] if length > 0 else [0.0, 0.0, 0.0]
    return n + [sum(n[k] * p1[k] for k in range(3))]


def face_fields(nodes: list[tuple], tris: list[tuple], flags: list[int], hiding: bool,
                index: int, box: tuple[int, int], boxes: list[tuple[int, int]]) -> list[float]:
    """One face for hl_read_face; the hiding entries cover the flagged triangles."""
    values: list[float] = [len(nodes)] + [c for p in nodes for c in p]
    values += [len(tris)] + [v for t, f in zip(tris, flags) for v in (*t, f)]
    if not hiding:
        return values + [-1, index, *box]
    hidden = [i for i, f in enumerate(flags) if f & HIDING]
    values.append(len(hidden))
    for k, i in enumerate(hidden):
        values += [i + 1, *boxes[k % len(boxes)], *plane_of(nodes, tris[i])]
    return values + [index, *box]


def grid_face(rng: random.Random, size: int, integer: bool) -> tuple[list[tuple], list[tuple]]:
    """A (size+1)^2 grid of nodes, lifted in z, triangulated two per cell."""
    def coordinate(value: float) -> float:
        return float(value) if integer else value + rng.uniform(-0.3, 0.3)
    nodes = [(coordinate(i), coordinate(j), coordinate(rng.randint(0, 2)) if integer else rng.uniform(0, 2))
             for j in range(size + 1) for i in range(size + 1)]
    tris = []
    for j in range(size):
        for i in range(size):
            a = j * (size + 1) + i + 1
            b, c, d = a + 1, a + size + 2, a + size + 1
            tris += [(a, b, c), (a, c, d)]
    return nodes, tris


def segment_fields(rng: random.Random, integer: bool, index: int, conex: bool) -> list[float]:
    """P1 P2 indices[10] hidingShell V1 V2 V3 param tolParam tolAng tolerance."""
    def point() -> list[float]:
        if integer:
            return [float(rng.randint(-1, 3)), float(rng.randint(-1, 3)), float(rng.randint(-2, 4))]
        return [rng.uniform(-1, 3), rng.uniform(-1, 3), rng.uniform(-2, 4)]
    min_seg, max_seg = rng.choice(((0, 0), (0, 0), (5, 0), (0, -7)))
    indices = [index, index if conex else 99, rng.randint(1, 9), rng.randint(1, 9),
               99, rng.randint(1, 9), rng.randint(1, 9), min_seg, max_seg, 0]
    tolerance = rng.choice((1e-7, 0.1, 0.0))
    return (point() + point() + indices + [rng.randint(0, 1)] + [rng.uniform(-1, 1) for _ in range(6)]
            + [0.0, 1e-8, 1e-4, tolerance])


def polydata_requests(rng: random.Random) -> list[str]:
    """ha_polydata on integer and random faces."""
    out = []
    for case in range(120):
        integer = case % 2 == 0
        nodes, tris = grid_face(rng, rng.randint(1, 2), integer)
        flags = [rng.choice((HIDING, HIDING, 0)) | rng.randint(0, 63) for _ in tris]
        box = rng.choice(((0, 0), (0, 0), (0x200, 0), (-5, 3)))
        boxes = [rng.choice(((0, 0), (0, 0), (3, 0)))]
        face = face_fields(nodes, tris, flags, True, case + 1, box, boxes)
        segments = [segment_fields(rng, integer, case + 1, k % 2 == 0) for k in range(rng.randint(1, 5))]
        out.append('ha_polydata ' + numbers(face + [len(segments)] + [v for s in segments for v in s]))
    nodes, tris = grid_face(rng, 1, True)
    no_hiding = face_fields(nodes, tris, [0, 0], False, 1, (0, 0), [(0, 0)])
    out.append('ha_polydata ' + numbers(no_hiding + [0]))
    return out


def tetra(rng: random.Random, shift: tuple) -> tuple[list[tuple], list[tuple]]:
    """A tetrahedron around `shift` with outward triangles."""
    base = [(0, 0, 0), (2, 0, 0), (0, 2, 0), (0, 0, 2)]
    nodes = [tuple(b[k] + shift[k] + rng.uniform(-0.2, 0.2) for k in range(3)) for b in base]
    return nodes, [(1, 3, 2), (1, 2, 4), (2, 3, 4), (1, 4, 3)]


def shell_fields(rng: random.Random, shell: int, hiding: bool, with_segments: bool) -> list[float]:
    """One shell: two faces (a tetrahedron split in halves, or a grid patch) and segments."""
    faces: list[list[float]] = []
    all_nodes: list[tuple] = []
    for f in range(2):
        if rng.random() < 0.5:
            nodes, tris = tetra(rng, (shell * 1.5 + f, f * 0.5, -f * 0.5))
        else:
            nodes, tris = grid_face(rng, 1, rng.random() < 0.5)
        flags = [HIDING if hiding and rng.random() < 0.8 else 0 for _ in tris]
        face_hides = hiding and any(flags)
        faces.append(face_fields(nodes, tris, flags, face_hides, f + 1, (0, 0), [(0, 0)]))
        all_nodes += nodes
    values: list[float] = [2] + [v for face in faces for v in face]
    segments = []
    for _ in range(rng.randint(1, 6) if with_segments else 0):
        a, b = rng.sample(all_nodes, 2)
        coords = [*a, *b, *a, *b]
        segments.append(coords + [shell * 10 + len(segments), 1, 1, 2, 2, 1, 2, rng.randint(0, 31)])
    return values + [len(segments)] + [v for s in segments for v in s]


def polyalgo_requests(rng: random.Random) -> list[str]:
    """ha_polyalgo on zero to three shells."""
    out = ['ha_polyalgo 0.0']
    for _ in range(80):
        count = rng.randint(1, 3)
        shells = [shell_fields(rng, s, rng.random() < 0.8, rng.random() < 0.9) for s in range(count)]
        out.append('ha_polyalgo ' + numbers([count] + [v for s in shells for v in s]))
    return out


def internal_nodes(rng: random.Random, size: int) -> list[float]:
    """Node data of a grid: point, normal, uv, scal."""
    values: list[float] = []
    for j in range(size + 1):
        for i in range(size + 1):
            values += [i, j, rng.uniform(0, 1), rng.uniform(-1, 1), rng.uniform(-1, 1), 1.0,
                       i / size, j / size, rng.uniform(-1, 1)]
    return values


def split_ops(rng: random.Random, size: int, count: int) -> list[list[float]]:
    """AddNode at the middle of a grid segment, then the split UpdateLinks there."""
    ops: list[list[float]] = []
    nb_nodes = (size + 1) ** 2
    edges = []
    for j in range(size + 1):
        for i in range(size + 1):
            a = j * (size + 1) + i + 1
            if i < size:
                edges.append((a, a + 1))
            if j < size:
                edges.append((a, a + size + 1))
            if i < size and j < size:
                edges.append((a, a + size + 2))
    for _ in range(count):
        a, b = rng.choice(edges)
        edges.remove((a, b))
        if rng.random() < 0.5:
            a, b = b, a
        nb_nodes += 1
        ops.append([0, a, b, 0.5, rng.uniform(0, 3), rng.uniform(0, 3), rng.uniform(0, 1)])
        ops.append([1, a, b, nb_nodes])
    return ops


def internal_requests(rng: random.Random) -> list[str]:
    """ha_internal on grids of 1 to 3 cells."""
    out = []
    for case in range(48):
        size = case % 3 + 1
        _, tris = grid_face(rng, size, True)
        values = [(size + 1) ** 2, len(tris)] + internal_nodes(rng, size)
        values += [v for t in tris for v in (*t, rng.randint(0, 7))]
        ops = split_ops(rng, size, rng.randint(1, 3))
        ops += [[2, rng.randint(0, 1)], [3, rng.randint(0, 1)], [4]]
        if size >= 2 and case % 2 == 0:
            # Nodes 1 and (size+1)^2 are opposite grid corners: no segment joins them.
            ops.append([1, 1, (size + 1) ** 2, 2])
        out.append('ha_internal ' + numbers(values + [len(ops)] + [v for op in ops for v in op]))
    out.append('ha_internal ' + numbers([0, 0, 0]))
    return out


def grow_requests(rng: random.Random) -> list[str]:
    """ha_grow: the Inc members until the arrays double, with both pointer modes."""
    out = []
    for nn, nt in ((1, 1), (2, 0), (3, 2), (1, 0), (0, 1)):
        steps = [(rng.randint(0, 2), rng.randint(0, 1)) for _ in range(12)]
        out.append('ha_grow ' + numbers([nn, nt, len(steps)] + [v for s in steps for v in s]))
    return out


def append_hlralgo_poly_requests(requests: list[str]) -> None:
    """Append the HLRAlgo polyhedral requests in a fixed order."""
    rng = random.Random(20261006)
    requests.extend(polydata_requests(rng))
    requests.extend(polyalgo_requests(rng))
    requests.extend(internal_requests(rng))
    requests.extend(grow_requests(rng))
