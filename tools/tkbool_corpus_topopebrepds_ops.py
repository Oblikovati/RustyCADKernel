"""Deterministic requests for the TopOpeBRepDS interference processing: TKI, the
ProcessInterferencesTool selectors, EXPORT, repvg, redu, EIR, FIR, Filter, TOOL, connex,
Check and Dumper (tools/oracle/tkbool_probe_topopebrepds_ops.cxx; the records are
documented there).

Each scenario builds the data structure of two solids (overlapping boxes, a cylinder
through a box, boxes sharing a face) with points on edges, same domain pairs, section
edges (among them the edges holding interferences) and pseudo-random interferences of the native types on edges (edge/vertex and
curve/point ones on face or edge transitions) and faces (face/edge ones); every
operation then runs on several target shapes.
"""
from __future__ import annotations
import random

SCENARIOS = [
    ((0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0), (0, 2.0, 2.0, 2.0, 0.0, 1.0, 1.0, 1.0)),
    ((0, 2.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0), (1, 0.6, 4.0, 0.0, 0.0, 0.8, 0.9, -1.0)),
    ((0, 2.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0), (0, 2.0, 2.0, 2.0, 0.0, 2.0, 0.0, 0.0)),
]
# kinds of the references: 0 solid, 1 face, 2 edge, 3 vertex, 4 point
SOLID, FACE, EDGE, VERTEX, POINT = range(5)
# TopOpeBRepDS_Kind and TopAbs_ShapeEnum values
K_POINT, K_VERTEX, K_EDGE, K_FACE = 0, 3, 4, 6
S_FACE, S_EDGE = 4, 6
OPS = 29


def ref(rng: random.Random, kind: int) -> int:
    """A reference to a shape (or point) of the kind."""
    return kind * 1000 + rng.randrange(40)


def states(rng: random.Random) -> tuple[int, int]:
    """Transition states: IN 0, OUT 1, ON 2, UNKNOWN 3 (UNKNOWN pairs sometimes)."""
    if rng.random() < 0.1:
        return 3, 3
    return rng.choice((0, 1, 1, 0, 2)), rng.choice((0, 1, 0, 1, 2))


def transition(rng: random.Random) -> list:
    """`sb sa shb sha ib ia` on faces or edges (often the same index)."""
    sb, sa = states(rng)
    on_edge = rng.random() < 0.3
    sh, kind = (S_EDGE, EDGE) if on_edge else (S_FACE, FACE)
    ib = ref(rng, kind)
    ia = ib if rng.random() < 0.8 else ref(rng, kind)
    return [sb, sa, sh, sh, ib, ia]


def edge_interference(rng: random.Random, target: int) -> list:
    """An edge/vertex or curve/point interference on an edge (on an EDGE support for
    EDGE transitions)."""
    tr = transition(rng)
    on_edge = tr[2] == S_EDGE or rng.random() < 0.5
    sk, s = (K_EDGE, ref(rng, EDGE)) if on_edge else (K_FACE, ref(rng, FACE))
    if rng.random() < 0.6:
        ty, gk, g = 1, K_VERTEX, ref(rng, VERTEX)
    else:
        ty, gk, g = 0, K_POINT, ref(rng, POINT)
    t = rng.choice((0.0, 1.0, 0.25, 0.5, 0.75, rng.random()))
    return [target, ty, *tr, sk, s, gk, g, round(t, 6), rng.randrange(2), rng.randrange(3)]


def face_interference(rng: random.Random, target: int) -> list:
    """A face/edge interference on a face."""
    tr = transition(rng)
    tr[2] = tr[3] = S_FACE
    tr[4] = ref(rng, FACE)
    tr[5] = tr[4] if rng.random() < 0.8 else ref(rng, FACE)
    return [target, 2, *tr, K_FACE, ref(rng, FACE), K_EDGE, ref(rng, EDGE), 0.0,
            rng.randrange(2), rng.randrange(3)]


def structure(rng: random.Random, solids) -> tuple[list, list, list]:
    """The structure tokens and the targets (edges, faces) holding interferences."""
    out = [*solids[0], *solids[1]]
    pts = [[ref(rng, EDGE), round(rng.random(), 6), 1e-7] for _ in range(4)]
    out += [len(pts)] + [v for p in pts for v in p]
    sdm = [[ref(rng, k), ref(rng, k)] for k in (FACE, EDGE, VERTEX, EDGE) if rng.random() < 0.8]
    out += [len(sdm)] + [v for p in sdm for v in p]
    edges = [ref(rng, EDGE) for _ in range(3)]
    se = [ref(rng, EDGE) for _ in range(rng.randrange(1, 4))] + edges[:2]
    out += [len(se)] + se
    faces = [ref(rng, FACE) for _ in range(2)]
    ints = []
    for e in edges:
        ints += [edge_interference(rng, e) for _ in range(rng.randrange(2, 7))]
    for f in faces:
        ints += [face_interference(rng, f) for _ in range(rng.randrange(2, 6))]
    out += [len(ints)] + [v for i in ints for v in i]
    return out, edges, faces


def append_topopebrepds_ops_requests(requests: list[str]) -> None:
    """Append the `dsx` requests."""
    rng = random.Random(20261015)
    for solids in SCENARIOS:
        for _ in range(4):
            tokens, edges, faces = structure(rng, solids)
            head = ' '.join(repr(v) if isinstance(v, float) else str(v) for v in tokens)
            for op in range(OPS):
                targets = faces if op in (6, 7, 9) else edges
                for target in targets[:2]:
                    arg = rng.randrange(1, 30) if op != 11 else ref(rng, rng.choice((EDGE, FACE)))
                    requests.append(f'dsx {head} {op} {target} {arg}')
