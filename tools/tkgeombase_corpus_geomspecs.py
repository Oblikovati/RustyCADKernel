"""Geometry specs shared by the GeomConvert and GeomLib corpora.

Every spec is the numeric layout read by the GeomTools builders of
tools/oracle/tkgeombase_probe_geomtools_build.cxx (gt_curve, gt_curve2d, gt_surface)
and their Rust twin crates/tkgeombase/examples/gb_probe/geom_tools/build.rs. The shapes
cover every Geom and Geom2d class: conics on direct and tilted frames, lines, Bezier
and B-spline curves and surfaces (rational or not, periodic, C0), trimmed and offset
curves and surfaces, swept surfaces, and B-spline and Bezier forms of canonical shapes.
"""
from __future__ import annotations
import math

R2 = math.sqrt(0.5)
O = [0.0, 0.0, 0.0]
X = [1.0, 0.0, 0.0]
Y = [0.0, 1.0, 0.0]
Z = [0.0, 0.0, 1.0]
AX2 = [*O, *Z, *X]
TILTED = [1.5, -2.25, 0.1, 1.0, 2.0, 3.0, 3.0, 0.0, -1.0]
AX3 = [*O, *Z, *X, 0]
AX3_TILTED = [*TILTED, 0]
AX3_INDIRECT = [0.5, 0.25, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1]


# Curves.
def line(p=O, d=(1.0, 1.0, 0.0)):
    return [1, *p, *d]


def circle(r, ax=AX2):
    return [2, *ax, r]


def ellipse(a, b, ax=AX2):
    return [3, *ax, a, b]


def parabola(f, ax=AX2):
    return [4, *ax, f]


def hyperbola(a, b, ax=AX2):
    return [5, *ax, a, b]


def bezier(poles, weights=None):
    """poles: 3D points; weights make it rational."""
    flat = [c for p in poles for c in p]
    if weights is None:
        return [6, 0, len(poles), *flat]
    return [6, 1, len(poles), *flat, *weights]


def bspline(degree, poles, knots, weights=None, periodic=0):
    """poles: 3D points; knots: (u, mult) pairs; weights make it rational."""
    flat = [c for p in poles for c in p]
    rational = [] if weights is None else list(weights)
    return [7, 0 if weights is None else 1, periodic, degree, len(poles), *flat, *rational,
            len(knots), *[k for k, _ in knots], *[m for _, m in knots]]


def trimmed(u1, u2, basis, sense=1):
    return [8, u1, u2, sense, *basis]


def offset(d, basis, direction=Z):
    return [9, d, *direction, *basis]


# 2D curves.
AX22D = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0]
AX22D_TILTED = [0.5, -1.0, 0.6, 0.8, -0.8, 0.6]
AX22D_INDIRECT = [1.5, -2.0, 0.6, 0.8, 0.8, -0.6]


def line2d(p=(0.0, 0.0), d=(1.0, 0.0)):
    return [1, *p, *d]


def circle2d(r, ax=AX22D):
    return [2, *ax, r]


def ellipse2d(a, b, ax=AX22D):
    return [3, *ax, a, b]


def parabola2d(f, ax=AX22D):
    return [4, *ax, f]


def hyperbola2d(a, b, ax=AX22D):
    return [5, *ax, a, b]


def bezier2d(poles, weights=None):
    flat = [c for p in poles for c in p]
    if weights is None:
        return [6, 0, len(poles), *flat]
    return [6, 1, len(poles), *flat, *weights]


def bspline2d(degree, poles, knots, weights=None, periodic=0):
    flat = [c for p in poles for c in p]
    rational = [] if weights is None else list(weights)
    return [7, 0 if weights is None else 1, periodic, degree, len(poles), *flat, *rational,
            len(knots), *[k for k, _ in knots], *[m for _, m in knots]]


def trimmed2d(u1, u2, basis, sense=1):
    return [8, u1, u2, sense, *basis]


def offset2d(d, basis):
    return [9, d, *basis]


# Surfaces.
def plane(ax=AX3):
    return [1, *ax]


def cylinder(r, ax=AX3):
    return [2, *ax, r]


def cone(angle, r, ax=AX3):
    return [3, *ax, angle, r]


def sphere(r, ax=AX3):
    return [4, *ax, r]


def torus(a, b, ax=AX3):
    return [5, *ax, a, b]


def extrusion(direction, curve):
    return [6, *direction, *curve]


def revolution(axis, curve):
    return [7, *axis, *curve]


def bezier_surface(grid, weights=None):
    """grid: rows of 3D points (U index outer); weights: rows of weights."""
    nu, nv = len(grid), len(grid[0])
    flat = [c for row in grid for p in row for c in p]
    if weights is None:
        return [8, 0, nu, nv, *flat]
    return [8, 1, nu, nv, *flat, *[w for row in weights for w in row]]


def bspline_surface(degrees, grid, uknots, vknots, weights=None, periodic=(0, 0)):
    nu, nv = len(grid), len(grid[0])
    flat = [c for row in grid for p in row for c in p]
    rational = [] if weights is None else [w for row in weights for w in row]
    return [9, 0 if weights is None else 1, periodic[0], periodic[1], degrees[0], degrees[1], nu, nv,
            *flat, *rational,
            len(uknots), *[k for k, _ in uknots], *[m for _, m in uknots],
            len(vknots), *[k for k, _ in vknots], *[m for _, m in vknots]]


def trimmed_surface(u1, u2, v1, v2, basis):
    return [10, u1, u2, v1, v2, *basis]


def offset_surface(d, basis):
    return [11, d, *basis]


# Shared shapes.
CIRCLE = circle(2.5)
CIRCLE_TILTED = circle(1.5, TILTED)
ELLIPSE = ellipse(3.0, 1.25, TILTED)
PARABOLA = parabola(0.5)
HYPERBOLA = hyperbola(2.0, 1.0, TILTED)
LINE = line()
BEZIER = bezier([(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (3.0, 2.0, 1.0), (4.0, 0.0, -0.5)])
RAT_BEZIER = bezier([(0.0, 0.0, 0.0), (1.0, 2.0, 0.5), (3.0, 2.0, 0.0)], [1.0, 2.0, 0.5])
C2 = bspline(3, [(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, -1.0, 0.5), (3.0, 1.0, 1.0), (4.0, 0.0, 0.0),
                 (5.0, 1.0, -1.0)], [(0.0, 4), (1.0, 1), (2.0, 1), (3.0, 4)])
C0 = bspline(2, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0), (3.0, 1.0, 1.0), (4.0, 0.0, 0.0)],
             [(0.0, 3), (1.0, 2), (3.0, 3)])
RAT = bspline(2, [(0.0, 0.0, 0.0), (1.0, 1.0, 0.5), (2.0, 0.0, 0.0), (3.0, -1.0, 0.5), (4.0, 0.0, 0.0)],
              [(0.0, 3), (1.0, 1), (2.0, 1), (4.0, 3)], [1.0, 2.0, 1.0, 0.5, 1.0])
HEXAGON = [(2.0 * math.cos(k * math.pi / 3), 2.0 * math.sin(k * math.pi / 3), 0.25 * (k % 2)) for k in range(6)]
PERIODIC = bspline(3, HEXAGON, [(float(k), 1) for k in range(7)], periodic=1)
# The exact rational quarter circle of radius 2 in the XY plane.
QUARTER = bspline(2, [(2.0, 0.0, 0.0), (2.0, 2.0, 0.0), (0.0, 2.0, 0.0)], [(0.0, 3), (1.0, 3)], [1.0, R2, 1.0])
# A linear B-spline (collinear poles) and a flat quadratic Bezier.
LINEAR_BSPLINE = bspline(3, [(0.0, 0.0, 0.0), (1.0, 1.0, 1.0), (2.0, 2.0, 2.0), (3.0, 3.0, 3.0)],
                         [(0.0, 4), (1.0, 4)])
LINEAR_BEZIER = bezier([(1.0, 0.0, 0.0), (2.0, 0.5, 0.0), (3.0, 1.0, 0.0)])

FLAT_GRID = [[(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 2.0, 0.0)],
             [(1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (1.0, 2.0, 0.0)],
             [(2.0, 0.0, 0.0), (2.0, 1.0, 0.0), (2.0, 2.0, 0.0)]]
WAVY_GRID = [[(0.0, 0.0, 0.0), (0.0, 1.0, 0.5), (0.0, 2.0, 0.0), (0.0, 3.0, 0.25)],
             [(1.0, 0.0, 1.0), (1.0, 1.0, 2.0), (1.0, 2.0, 1.0), (1.0, 3.0, 0.0)],
             [(2.0, 0.0, 0.0), (2.0, 1.0, -0.5), (2.0, 2.0, 0.5), (2.0, 3.0, 1.0)],
             [(3.0, 0.0, 0.5), (3.0, 1.0, 0.0), (3.0, 2.0, -1.0), (3.0, 3.0, 0.0)]]
BEZIER_SURFACE = bezier_surface(WAVY_GRID[:3])
RAT_BEZIER_SURFACE = bezier_surface(FLAT_GRID, [[1.0, 2.0, 1.0], [0.5, 1.0, 0.5], [1.0, 2.0, 1.0]])
FLAT_BEZIER_SURFACE = bezier_surface(FLAT_GRID)
BSPLINE_SURFACE = bspline_surface((2, 2), WAVY_GRID, [(0.0, 3), (1.0, 1), (2.0, 3)], [(0.0, 3), (0.5, 1), (1.0, 3)])
C0_BSPLINE_SURFACE = bspline_surface((1, 2), WAVY_GRID, [(0.0, 2), (1.0, 1), (2.0, 1), (3.0, 2)],
                                     [(0.0, 3), (0.5, 1), (1.0, 3)])
RAT_BSPLINE_SURFACE = bspline_surface((2, 1), [row[:2] for row in WAVY_GRID], [(0.0, 3), (1.0, 1), (3.0, 3)],
                                      [(0.0, 2), (1.0, 2)], [[1.0, 1.0], [2.0, 0.5], [1.0, 1.0], [0.5, 2.0]])
PERIODIC_BSPLINE_SURFACE = bspline_surface(
    (3, 1), [[(x, y, 0.0), (x, y, 1.0)] for x, y, _ in HEXAGON],
    [(float(k), 1) for k in range(7)], [(0.0, 2), (1.0, 2)], periodic=(1, 0))
# The exact rational quarter cylinder of radius 2 around Z, height 3.
QUARTER_CYLINDER = bspline_surface(
    (2, 1), [[(2.0, 0.0, 0.0), (2.0, 0.0, 3.0)], [(2.0, 2.0, 0.0), (2.0, 2.0, 3.0)], [(0.0, 2.0, 0.0), (0.0, 2.0, 3.0)]],
    [(0.0, 3), (1.0, 3)], [(0.0, 2), (1.0, 2)], [[1.0, 1.0], [R2, R2], [1.0, 1.0]])
FLAT_BSPLINE_SURFACE = bspline_surface((2, 2), FLAT_GRID, [(0.0, 3), (1.0, 3)], [(0.0, 3), (1.0, 3)])

PLANE = plane()
PLANE_TILTED = plane(AX3_TILTED)
CYLINDER = cylinder(2.0)
CONE = cone(0.5, 1.0)
SPHERE = sphere(3.0)
TORUS = torus(5.0, 1.0)
Z_AXIS = [*O, *Z]
# Surfaces of revolution: of a line parallel to the axis (a cylinder), of an inclined
# line (a cone, crossing the axis), of a circle arc (a torus), of a half circle centred
# on the axis (a sphere), and of a B-spline (none).
REV_CYLINDER = revolution(Z_AXIS, line((2.0, 0.0, 0.0), Z))
REV_CONE = revolution(Z_AXIS, line((1.0, 0.0, 0.0), (0.5, 0.0, 1.0)))
REV_TORUS = revolution(Z_AXIS, trimmed(0.0, 6.0, circle(1.0, [5.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 0.0])))
REV_SPHERE = revolution(Z_AXIS, trimmed(-1.5, 1.5, circle(3.0, [0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 0.0])))
REV_BSPLINE = revolution(Z_AXIS, C2)
EXT_CYLINDER = extrusion(Z, CIRCLE)
EXT_PLANE = extrusion(Z, LINE)
EXT_BSPLINE = extrusion([0.0, 0.3, 1.0], BEZIER)
