"""Deterministic requests for TopOpeBRepTool_TOOL, TOPOLOGY, makeTransition, mkTondgE,
CLASSI, REGUW (Regularize), CORRISO (CorrectONUVISO, PurgeClosingEdges, MakeFaces),
FuseEdges and PurgeInternalEdges (tools/oracle/tkbool_probe_topopebreptool_tool.cxx and
tkbool_probe_topopebreptool_regu.cxx; the records are documented there).

The TOOL and TOPOLOGY requests visit every edge of a box, a cylinder, a cone with an
apex, a sphere, a torus and an L-shaped prism, on each of its two faces, at an inner
parameter and on the first bound. The transitions cross a translated cylinder through
the faces of a box (3d crossings) and lay box edges on their own faces (2d crossings),
with and without restriction edges; the degenerated edges of the cone apex and the
sphere poles are crossed by the planes of boxes through them. The classification sorts
squares into nested, disjoint, overlapping and equal wires; the regularization splits
walks with multiple vertices (figure-eight wires, a loop touching the outer wire); the
UV corrector moves p-curves of periodic faces by a period; the edge fusion runs on fused
boxes sharing faces; the purge removes an INTERNAL diagonal.
"""
from __future__ import annotations
import math

BOX = (0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0)
CYLINDER = (1, 1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0)
CONE = (2, 1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0)
SPHERE = (3, 1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
TORUS = (4, 3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0)
LPRISM = (5, 4.0, 3.0, 2.0, 1.0, 0.0, 0.0, 0.0)
# (solid, edge count of the edge/face map)
SOLIDS = [(BOX, 12), (CYLINDER, 3), (CONE, 3), (SPHERE, 3), (TORUS, 2), (LPRISM, 18)]


def fmt(values) -> str:
    """The tokens of numbers."""
    return ' '.join(repr(float(v)) for v in values)


def edge_face_requests(op: str) -> list[str]:
    """`op solid ie k t` on every edge, both faces, inside and on the first bound."""
    out = []
    for solid, n in SOLIDS:
        for ie in range(1, n + 1):
            for k in (1, 2):
                for t in (0.3, 0.0):
                    out.append(f'{op} {fmt(solid)} {fmt((ie, k, t))}')
    return out


def moved(solid, t) -> tuple:
    """A solid record translated."""
    return solid[:5] + tuple(t)


TALL_CYLINDER = moved((1, 0.6, 4.0, 0.0, 0.0), (0.8, 0.9, -1.0))
UNIT_BOX = (0, 2.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0)


def transition_requests() -> list[str]:
    """3d crossings of the tall cylinder through the box, 2d crossings of box edges."""
    out = []
    for ie in (1, 2, 3):
        for fi, pe in ((5, 1.0), (6, 3.0), (1, 2.0)):
            for ies, pes in ((0, 0.0), (1, 0.5), (5, 1.0)):
                for fac in (0.2, 0.05):
                    out.append(f'tt_make_transition {fmt(TALL_CYLINDER)} {fmt(UNIT_BOX)} {fmt((ie, fi, pe, fac, ies, pes))}')
    for ie, fi in ((1, 1), (2, 1), (5, 3), (9, 5), (12, 6)):
        for ies in (0, 2, 4):
            out.append(f'tt_make_transition {fmt(UNIT_BOX)} {fmt(UNIT_BOX)} {fmt((ie, fi, 1.0, 0.2, ies, 0.5))}')
    return out


def mktondge_requests() -> list[str]:
    """The apex of the cone and the poles of the sphere crossed by box planes."""
    out = []
    # Boxes whose top face plane goes through the cone apex (z = 2) or a sphere pole.
    cases = [(CONE, (0, 2.0, 2.0, 2.0, 0.0, -1.0, -1.0, 0.0), 3),
             (SPHERE, (0, 4.0, 4.0, 1.5, 0.0, -2.0, -2.0, 0.0), 3),
             (CONE, (0, 2.0, 2.0, 1.0, 0.0, -1.0, -1.0, 1.0), 3)]
    for solid, box, n in cases:
        for ie in range(1, n + 1):
            for fi in (1, 5, 6):
                out.append(f'tt_mktondge {fmt(solid)} {fmt(box)} {fmt((ie, fi))}')
    return out


SQUARES = {'outer': (0, 0, 10, 10, 1), 'hole': (2, 2, 4, 4, 0), 'hole2': (6, 6, 8, 8, 0),
           'away': (12, 0, 14, 2, 1), 'island': (2.5, 2.5, 3.5, 3.5, 1), 'cross': (5, 5, 12, 12, 1),
           'outer_cw': (0, 0, 10, 10, 0), 'hole_ccw': (2, 2, 4, 4, 1)}
CLASSI_SETS = [['outer', 'hole'], ['hole', 'outer'], ['outer', 'hole', 'hole2'], ['outer', 'away'],
               ['outer', 'hole', 'island'], ['outer', 'cross'], ['outer', 'outer_cw'],
               ['hole2', 'away', 'outer', 'island', 'hole'], ['outer_cw', 'hole_ccw']]


def classi_requests() -> list[str]:
    out = []
    for names in CLASSI_SETS:
        tokens = ' '.join(fmt(SQUARES[n]) for n in names)
        out.append(f'tt_classi {len(names)} {tokens}')
    return out


REGU_WALKS = [
    ([(0, 0), (2, -1), (2, 1), (-2, 1), (-2, -1)], [0, 1, 2, 0, 3, 4]),
    ([(0, 0), (2, -1), (2, 1), (-2, 1), (-2, -1)], [0, 2, 1, 0, 4, 3]),
    ([(0, 0), (4, 0), (4, 4), (0, 4)], [0, 1, 2, 3]),
    ([(0, 0), (4, 0), (4, 4), (0, 4), (1, 2), (2, 1)], [0, 1, 2, 3, 0, 4, 5]),
    ([(0, 0), (2, -1), (2, 1), (-2, 1), (-2, -1), (1, 3), (-1, 3)], [0, 1, 2, 0, 3, 4, 0, 5, 6]),
]


def regularize_requests() -> list[str]:
    out = []
    for points, ids in REGU_WALKS:
        pts = ' '.join(fmt(p) for p in points)
        out.append(f'tt_regularize {len(points)} {pts} {len(ids)} {fmt(ids)}')
    return out


def corriso_requests() -> list[str]:
    out = []
    two_pi = 2 * math.pi
    # The sphere is left out: CORRISO::Init makes the p-curve of its degenerated edges from a
    # null 3d curve, a null dereference (segmentation fault) natively, Standard_NullObject here.
    for solid, n in ((CYLINDER, 4), (TORUS, 4), ((2, 1.0, 0.5, 2.0, 0.0, 0.0, 0.0, 0.0), 4)):
        for ie in range(1, n + 1):
            for t in ((two_pi, 0.0), (-two_pi, 0.0), (0.0, 0.0), (0.0, 1.0)):
                out.append(f'tt_corriso {fmt(solid)} {fmt((ie,) + t)}')
    return out


def fuse_requests() -> list[str]:
    a = (0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0)
    cases = [(a, moved(a, (1.0, 0.0, 0.0))), ((0, 2.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0), moved(a, (0.0, 1.0, 0.0))),
             (a, moved(a, (3.0, 0.0, 0.0))), (a, moved((0, 1.0, 1.0, 2.0, 0.0), (1.0, 0.0, 0.0)))]
    return [f'tt_fuse_edges {fmt(s1)} {fmt(s2)}' for s1, s2 in cases]


def append_topopebreptool_tool_requests(requests: list[str]) -> None:
    """Every TopOpeBRepTool TOOL/TOPOLOGY/transition/regularization request."""
    requests.extend(edge_face_requests('tt_tool'))
    requests.extend(edge_face_requests('tt_topology'))
    requests.extend(transition_requests())
    requests.extend(mktondge_requests())
    requests.extend(classi_requests())
    requests.extend(regularize_requests())
    requests.extend(corriso_requests())
    requests.extend(fuse_requests())
    requests.extend(f'tt_purge_internal {fmt((a,))}' for a in (1.0, 2.5))
