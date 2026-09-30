"""Record builders of the BRepLib probe requests (tools/oracle/tktopalgo_probe_breplib_common.cxx).

Each function returns the list of numbers of one record, in the layout that the native
reader documents: vertices, gp curves, Geom curves, gp 2d curves, Geom2d curves and the
surfaces of the face reader (tools/oracle/tktopalgo_probe_face.cxx).
"""
from __future__ import annotations

INF = 2e100  # Precision::Infinite(): an infinite parameter inside the range of a line


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def vertices(points: list[tuple[float, float, float, float]]) -> list[float]:
    """`n (x y z tol)...`."""
    return [len(points)] + [c for p in points for c in p]


# gp curves without their kind (the constructors 10 + 4k + j imply it).
def gp_lin(p=(0., 0., 0.), d=(1., 0., 0.)) -> list[float]:
    return [*p, *d]


def gp_circ(p=(0., 0., 0.), n=(0., 0., 1.), radius=1.) -> list[float]:
    return [*p, *n, radius]


def gp_elips(p=(0., 0., 0.), n=(0., 0., 1.), major=2., minor=1.) -> list[float]:
    return [*p, *n, major, minor]


def gp_hypr(p=(0., 0., 0.), n=(0., 0., 1.), major=2., minor=1.) -> list[float]:
    return [*p, *n, major, minor]


def gp_parab(p=(0., 0., 0.), n=(0., 0., 1.), focal=1.) -> list[float]:
    return [*p, *n, focal]


GP_BUILDERS = [gp_lin, gp_circ, gp_elips, gp_hypr, gp_parab]


def geom_gp(kind: int, **args) -> list[float]:
    """A Geom curve of the gp kind 0..4."""
    return [kind] + GP_BUILDERS[kind](**args)


def geom_bezier(poles: list[tuple[float, float, float]]) -> list[float]:
    return [5, len(poles)] + [c for p in poles for c in p]


def geom_bspline(degree: int, periodic: bool, poles, knots) -> list[float]:
    """`knots` is a list of (value, multiplicity)."""
    return [6, degree, int(periodic), len(poles)] + [c for p in poles for c in p] + \
        [len(knots)] + [c for k in knots for c in k]


def geom_trimmed(u1: float, u2: float, basis: list[float], sense: bool = True) -> list[float]:
    return [7, u1, u2, int(sense)] + basis


def geom_offset(value: float, d, basis: list[float]) -> list[float]:
    return [8, value, *d] + basis


# gp 2d curves without their kind.
def gp_lin2d(p=(0., 0.), d=(1., 0.)) -> list[float]:
    return [*p, *d]


def gp_circ2d(p=(0., 0.), x=(1., 0.), radius=1., sense=True) -> list[float]:
    return [*p, *x, radius, int(sense)]


def gp_elips2d(p=(0., 0.), x=(1., 0.), major=2., minor=1., sense=True) -> list[float]:
    return [*p, *x, major, minor, int(sense)]


def gp_hypr2d(p=(0., 0.), x=(1., 0.), major=2., minor=1., sense=True) -> list[float]:
    return [*p, *x, major, minor, int(sense)]


def gp_parab2d(p=(0., 0.), x=(1., 0.), focal=1., sense=True) -> list[float]:
    return [*p, *x, focal, int(sense)]


GP2D_BUILDERS = [gp_lin2d, gp_circ2d, gp_elips2d, gp_hypr2d, gp_parab2d]


def geom2d_gp(kind: int, **args) -> list[float]:
    """A Geom2d curve of the gp 2d kind 0..4."""
    return [kind] + GP2D_BUILDERS[kind](**args)


def geom2d_bezier(poles: list[tuple[float, float]]) -> list[float]:
    return [5, len(poles)] + [c for p in poles for c in p]


def geom2d_bspline(degree: int, periodic: bool, poles, knots) -> list[float]:
    return [6, degree, int(periodic), len(poles)] + [c for p in poles for c in p] + \
        [len(knots)] + [c for k in knots for c in k]


def geom2d_trimmed(u1: float, u2: float, basis: list[float], sense: bool = True) -> list[float]:
    return [7, u1, u2, int(sense)] + basis


# Surfaces of the face reader (`tp_surface`).
def plane(p=(0., 0., 0.), n=(0., 0., 1.)) -> list[float]:
    return [0, *p, *n]


def cylinder(p=(0., 0., 0.), n=(0., 0., 1.), radius=1.) -> list[float]:
    return [1, *p, *n, radius]


def sphere(p=(0., 0., 0.), n=(0., 0., 1.), radius=1.) -> list[float]:
    return [2, *p, *n, radius]


def trimmed_plane(u1, u2, v1, v2, p=(0., 0., 0.), n=(0., 0., 1.)) -> list[float]:
    return [3, *p, *n, u1, u2, v1, v2]


def cone(p=(0., 0., 0.), n=(0., 0., 1.), radius=1., angle=0.5) -> list[float]:
    return [4, *p, *n, radius, angle]


def torus(p=(0., 0., 0.), n=(0., 0., 1.), major=3., minor=1.) -> list[float]:
    return [5, *p, *n, major, minor]


def bezier_surface(poles: list[list[tuple[float, float, float]]]) -> list[float]:
    return [6, 0., 0., 0., 0., 0., 1., len(poles), len(poles[0])] + \
        [c for row in poles for p in row for c in p]


def bspline_surface(udeg, vdeg, poles, uknots, vknots) -> list[float]:
    return [7, 0., 0., 0., 0., 0., 1., udeg, vdeg, len(poles), len(poles[0])] + \
        [c for row in poles for p in row for c in p] + \
        [len(uknots)] + [c for k in uknots for c in k] + [len(vknots)] + [c for k in vknots for c in k]
