"""Deterministic requests for ShapeExtend_CompositeSurface (tools/oracle/tkshhealing_probe_composite.cxx).

The patches are surfaces of a TKTopAlgo shape program (tools/tktopalgo_shape_program.py):
trimmed planes that tile a square (connected in U, in V, or with a gap), Bezier patches
that share an edge, an infinite plane, a full cylinder and a sphere (closed in U). Each
grid is built with every parametrisation and with explicit joint values (valid, of the
wrong length, not increasing, with another lower bound), then located, mapped and
evaluated at inner points, joints, ends and outside points, re-jointed, shifted, checked,
copied and transformed.
"""
from __future__ import annotations
from tktopalgo_shape_program import ShapeProgram, num

NATURAL, UNIFORM, UNITARY = 0, 1, 2
PLANE, CYLINDER, SPHERE, TRIMMED_PLANE, BEZIER = 0, 1, 2, 3, 6
Z = (0., 0., 0., 0., 0., 1.)


def surfaces() -> tuple[ShapeProgram, dict[str, int]]:
    """The patch surfaces, by name."""
    p = ShapeProgram()
    s = {}
    for name, (u1, u2, v1, v2) in {'p00': (0., 1., 0., 1.), 'p10': (1., 2., 0., 1.), 'p01': (0., 1., 1., 2.),
                                   'p11': (1., 2., 1., 2.), 'gap': (1.5, 2.5, 0., 1.),
                                   'wide': (-3., 3., 0., 0.5)}.items():
        s[name] = p.surface(TRIMMED_PLANE, *Z, u1, u2, v1, v2)
    s['bez0'] = p.surface(BEZIER, *Z, 2, 2, 0., 0., 0., 0., 1., 0.2, 1., 0., 0.1, 1., 1., 0.)
    s['bez1'] = p.surface(BEZIER, *Z, 2, 2, 0., 1., 0.2, 0., 2., 0., 1., 1., 0., 1., 2., 0.5)
    s['plane'] = p.surface(PLANE, *Z)
    s['cylinder'] = p.surface(CYLINDER, *Z, 2.)
    s['sphere'] = p.surface(SPHERE, *Z, 1.)
    return p, s


# (name, rows of patch names).
GRIDS = [
    ('row', [['p00'], ['p10']]),
    ('square', [['p00', 'p01'], ['p10', 'p11']]),
    ('column', [['p00', 'p01']]),
    ('gap', [['p00'], ['gap']]),
    ('bezier', [['bez0', 'bez1']]),
    ('plane', [['plane']]),
    ('cylinder', [['cylinder']]),
    ('sphere', [['sphere']]),
    ('mixed', [['wide', 'p01'], ['p10', 'p11']]),
]
POINTS = [0.25, 0.25, 0.5, 0.5, 1., 1., 0., 0., 2., 2., 1.5, 0.75, -1., 0.5, 3., 3., 0.999, 1.001,
          0.1, 1.9]
TRANSFORMS = [[1, 0.5, -1., 2.], [2, 1., 0., 0., 0., 0., 1., 0.7]]
# Joint records `lower n (value)...`.
GOOD_U = [1, 3, 0., 0.5, 2.]
GOOD_V = [1, 3, -1., 0., 4.]


def ctor_records(nu: int, nv: int) -> list[list[float]]:
    """The constructors of a grid: each parametrisation both ways, and joint values."""
    records: list[list[float]] = [[c, p] for c in (0, 2) for p in (NATURAL, UNIFORM, UNITARY)]
    u_ok = [1, nu+1, *[0.5*i for i in range(nu+1)]]
    v_ok = [0, nv+1, *[2.*i-1. for i in range(nv+1)]]
    for c in (1, 3):
        records += [[c, *u_ok, *v_ok], [c, *u_ok, 1, nv+2, *range(nv+2)],
                    [c, 1, nu+1, *[0.]*(nu+1), *v_ok], [c, 5, nu+1, *[1e-10*i for i in range(nu+1)], *v_ok]]
    return records


def new_joint_records(nu: int, nv: int) -> list[list[float]]:
    """The joint values given to SetUJointValues and SetVJointValues."""
    return [[1, nu+1, *[3.*i for i in range(nu+1)], 1, nv+1, *[1.+i for i in range(nv+1)]],
            [2, nu+2, *range(nu+2), 1, nv+1, *[-float(i) for i in range(nv+1)]]]


def request(p: ShapeProgram, s: dict[str, int], rows: list[list[str]], ctor: list[float],
            trsf: list[float], joints: list[float]) -> str:
    """One `sh_composite` line."""
    grid = [s[name] for row in rows for name in row]
    fields = [len(rows), len(rows[0]), *grid, *ctor, len(POINTS)//2*2, *POINTS, 1e-7, *trsf, *joints]
    return ' '.join(['sh_composite', p.tokens(), *map(num, fields)])


def append_composite_requests(requests: list[str]) -> None:
    """Append the ShapeExtend_CompositeSurface requests."""
    p, s = surfaces()
    for _, rows in GRIDS:
        nu, nv = len(rows), len(rows[0])
        for k, ctor in enumerate(ctor_records(nu, nv)):
            trsf = TRANSFORMS[k % 2]
            joints = new_joint_records(nu, nv)[k % 2]
            requests.append(request(p, s, rows, ctor, trsf, joints))
