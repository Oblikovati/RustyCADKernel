"""Deterministic requests for AppDef_Variational
(tools/oracle/tkgeombase_probe_appdef_variational.cxx).

The smoothing runs on 3D, 2D and mixed lines (smooth, noisy, collinear, with a
repeated point, scaled), without constraints and with pass, tangency and curvature
constraints (ends and inner points, cut and uncut), every continuity, degrees and
segment counts from over-constrained to generous, size and max-error tolerances
(with Adjusting), one to several iterations and every criterion weight; every
query before and after Approximate, Distance, Dump and every setter. The degenerate
inputs are a single point, two points, coincident points, FirstPoint above 1 (the
last tangent estimation reads at LastPoint, out of range), constraints before the
last point (the constraint search reads past the table), missing or non-normal
constraint vectors, a zero tangent and bad arguments.

Left out, as the native result is undefined there: NoConstraint couples whose
uninitialized table entries the searches reach, InitCutting runs whose back scan
reads a knot not yet set (simulated below), negative tolerances with the
maximum-error mode (Adjusting then reads an unset worst point), and C0 smoothings
that reduce an element to degree 1 (the native FEmTool_Curve::D2 then evaluates a
polynomial of degree -1, reading outside the element's coefficients).
"""
from __future__ import annotations
import random
from tkgeombase_corpus_appdef import appdef_line, NO, PASS, TANGENT, CURVATURE
from tkgeombase_corpus_appparcurves import uvec
from tkgeombase_corpus_gce import add

C0, C1, C2, G1, C3 = 0, 1, 2, 3, 4
NIV = {C0: 0, C1: 1, C2: 2}
KIND = {PASS: 0, TANGENT: 1, CURVATURE: 2}


def cutting_reads_unset_knot(kinds, maxdeg, nivcont, maxseg):
    """True when AppDef_Variational::InitCutting would compare a parameter with the
    knot it is about to set (an uninitialized FEmTool_Curve knot natively)."""
    weights = [k + 1 for k in kinds]
    nb_constr, n_cont = len(weights), sum(weights)
    if not weights or max(weights) > maxdeg - nivcont:
        return False
    n_libre = max(maxdeg - nivcont - (maxdeg + 1) // 4, nivcont + 1)
    count = lambda nl: n_cont // nl + (0 if n_cont % nl == 0 else 1)
    nb_elem = count(n_libre)
    while nb_elem > maxseg and n_libre < maxdeg - nivcont:
        n_libre += 1
        nb_elem = count(n_libre)
    if nb_elem > maxseg:
        return False
    n_cnt = (n_cont - 1) // nb_elem + 1
    n_plus = nb_elem - (n_cnt * nb_elem - n_cont)
    i_deb, i_fin, n_deb, n_fin, nb_el = 0, nb_constr + 1, 0, 0, 0
    while nb_elem - nb_el > 1:
        nb_el += 1
        if n_plus == 0:
            n_cnt -= 1
        while n_deb < n_cnt and i_deb < i_fin:
            i_deb += 1
            if i_deb > nb_constr:
                return False  # out of the table: a Standard_OutOfRange first
            n_deb += weights[i_deb - 1]
        n_deb = 0 if n_deb == n_cnt else n_deb - n_cnt
        n_plus -= 1
        if n_plus == 0:
            n_cnt -= 1
        if nb_elem - nb_el == 1:
            break
        nb_el += 1
        while n_fin < n_cnt and i_deb < i_fin:
            i_fin -= 1
            n_fin += weights[i_fin - 1]
        if n_fin == n_cnt:
            n_fin = 0
        elif i_fin >= 1:
            return True
        else:
            return False
    return False


class State:
    """The fields InitSmoothCriterion and the over-constraint tests depend on."""

    def __init__(self, couples, maxdeg, maxseg, cont, cutting):
        self.couples, self.maxdeg, self.maxseg, self.cont, self.cutting = couples, maxdeg, maxseg, cont, cutting

    def kinds(self):
        return [KIND[c] for _, c in self.couples if c != NO]

    def over(self, maxdeg, niv, maxseg):
        kinds = self.kinds()
        return (maxdeg - niv) * maxseg - kinds.count(0) - 2 * kinds.count(1) - 3 * kinds.count(2) < 0

    def rebuild_safe(self):
        """InitSmoothCriterion never reads an unset knot in InitCutting."""
        if self.cont not in NIV or not self.cutting or not self.kinds():
            return True
        return not cutting_reads_unset_knot(self.kinds(), self.maxdeg, NIV[self.cont], self.maxseg)

    def init_safe(self):
        """Init (the constructor, SetConstraints) rebuilds unless over-constrained."""
        if self.cont not in NIV:
            return True
        max_seg = self.maxseg if self.cutting else 1
        return self.over(self.maxdeg, NIV[self.cont], max_seg) or self.rebuild_safe()

    def command_safe(self, command):
        """Applies a setter to the state; False when its rebuild would read an unset knot."""
        code, args = command[0], command[1:]
        niv = NIV.get(self.cont, 0)
        if code == 6:
            n = args[0]
            self.couples = [(args[1 + 2 * i], args[2 + 2 * i]) for i in range(n)]
            return self.init_safe()
        if code == 9:
            if self.over(args[0], niv, self.maxseg):
                return True
            self.maxdeg = args[0]
        elif code == 10:
            if not (self.cutting and self.over(self.maxdeg, niv, args[0])):
                self.maxseg = args[0]
            return True
        elif code == 11:
            if args[0] not in NIV or self.over(self.maxdeg, NIV[args[0]], self.maxseg):
                return True
            self.cont = args[0]
        elif code == 13:
            if args[0] and self.over(self.maxdeg, niv, self.maxseg):
                return True
            self.cutting = args[0]
        elif code not in (12, 14, 15, 16):
            return True
        return self.rebuild_safe()


def couples_of(pairs):
    values = [len(pairs)]
    for index, kind in pairs:
        values += [index, kind]
    return values


def smooth(requests, line, first, last, pairs, options, commands):
    """One request of `commands` (each a list); dropped when a rebuild it runs would
    read an unset knot."""
    maxdeg, maxseg, cont, minmax, cutting, tol, iterations = options
    state = State(pairs, min(max(maxdeg, 1), 30), maxseg, cont, cutting)
    if not state.init_safe() or not all(state.command_safe(c) for c in commands):
        return
    flat = [value for command in commands for value in command]
    add(requests, 'ad_var', *line, first, last, *couples_of(pairs), maxdeg, maxseg, cont, minmax, cutting, tol,
        iterations, len(commands), *flat)


def distance_rows(nb3d, nb2d):
    return [4, 1, 1, nb3d + nb2d]


def basic_commands(nb3d, nb2d):
    return [[2], [3], [5], [1], [2], [3], distance_rows(nb3d, nb2d), [5], [1], [3]]


def lines(rng):
    result = []
    for nb3d, nb2d in [(1, 0), (0, 1), (1, 1), (2, 0)]:
        for n, noise, shape in [(6, 0.0, 0), (12, 0.01, 0), (25, 0.02, 0), (10, 0.0, 1), (9, 0.005, 2)]:
            ends = (1, n)
            spec = appdef_line(rng, nb3d, nb2d, n, tangents=ends, curvatures=(n // 2,), noise=noise, shape=shape)
            result.append((spec, n, nb3d, nb2d, shape))
    return result


def basic_requests(rng, requests):
    for spec, n, nb3d, nb2d, shape in lines(rng):
        cmds = basic_commands(nb3d, nb2d)
        smooth(requests, spec, 1, n, [], [14, 100, C2, 0, 1, 1.0, 2], cmds)
        smooth(requests, spec, 1, n, [(1, PASS), (n, PASS)], [14, 100, C2, 0, 1, 1e-3, 2], cmds)
        smooth(requests, spec, 1, n, [(1, TANGENT), (n, TANGENT)], [10, 20, C1, 0, 1, 1e-2, 3], cmds)
        smooth(requests, spec, 1, n, [(1, PASS), (n // 2, CURVATURE), (n, TANGENT)], [12, 30, C2, 0, 1, 1e-3, 2], cmds)
        if shape != 1:
            # C0 elements of straight points reduce to degree 1, whose second
            # derivative the native D2 reads outside the element (left out).
            smooth(requests, spec, 1, n, [(1, PASS), (n, PASS)], [8, 4, C0, 1, 1, 1e-4, 2], cmds)
        smooth(requests, spec, 1, n, [(1, TANGENT), (n, PASS)], [6, 1, C2, 0, 0, 1e-3, 1], cmds)


def dense_constraints(n, kinds):
    return [(i, kinds[(i - 1) % len(kinds)]) for i in range(1, n + 1)]


def constraint_requests(rng, requests):
    for nb3d, nb2d in [(1, 0), (0, 1), (1, 1)]:
        n = 16
        spec = appdef_line(rng, nb3d, nb2d, n, tangents=range(1, n + 1), curvatures=(4, 8, 12, 16), noise=0.01)
        cmds = [[1], [3], distance_rows(nb3d, nb2d)]
        for kinds in [[PASS], [PASS, TANGENT], [TANGENT], [PASS, PASS, TANGENT, CURVATURE]]:
            pairs = dense_constraints(n, kinds)
            for maxdeg, maxseg, cont, cutting in [(14, 100, C2, 1), (8, 10, C1, 1), (6, 3, C0, 1), (14, 2, C2, 0), (5, 1, C2, 0)]:
                smooth(requests, spec, 1, n, pairs, [maxdeg, maxseg, cont, 0, cutting, 1e-3, 2], cmds)
        # Every fourth point, ends included.
        pairs = [(i, PASS if i % 8 else TANGENT) for i in range(1, n + 1) if i % 4 == 1 or i == n]
        for cont in [C0, C1, C2]:
            if cont != C0:  # the C0 size-error smoothing reduces an element to degree 1
                smooth(requests, spec, 1, n, pairs, [9, 50, cont, 0, 1, 1e-3, 3], cmds)
            smooth(requests, spec, 1, n, pairs, [9, 50, cont, 1, 1, 1e-5, 3], cmds)


def weight_requests(rng, requests):
    spec = appdef_line(rng, 1, 0, 20, tangents=(1, 20), noise=0.02)
    base = [(1, PASS), (20, PASS)]
    for weights in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.3, 0.3, 0.4], [5.0, 1.0, 0.1]]:
        cmds = [[14, *weights], [2], [1], [3]]
        smooth(requests, spec, 1, 20, base, [14, 100, C2, 0, 1, 1e-3, 2], cmds)
    for order, percent in [(1, 0.0), (2, 3.0), (3, 0.5), (0, 1.0), (4, 1.0), (2, -1.0)]:
        smooth(requests, spec, 1, 20, base, [14, 100, C2, 0, 1, 1e-3, 2], [[15, order, percent], [2], [1], [3]])
    smooth(requests, spec, 1, 20, base, [14, 100, C2, 0, 1, 1e-3, 2], [[14, -1.0, 1.0, 1.0], [2]])
    for tol, minmax, iterations in [(0.0, 0, 2), (0.0, 1, 2), (1e-6, 1, 4), (0.05, 1, 1), (1e-3, 0, 1), (1e-3, 0, 5)]:
        smooth(requests, spec, 1, 20, base, [14, 100, C2, minmax, 1, tol, iterations], [[1], [3], [5]])


def setter_requests(rng, requests):
    n = 14
    spec = appdef_line(rng, 1, 1, n, tangents=(1, 7, n), curvatures=(7,), noise=0.01)
    pairs = [(1, PASS), (7, TANGENT), (n, PASS)]
    opts = [10, 10, C2, 0, 1, 1e-3, 2]
    ts = [i / (n - 1) for i in range(n)]
    setters = [
        [[6, *couples_of([(1, TANGENT), (n, TANGENT)])], [2], [1], [3]],
        # A SetConstraints that raises inside Init leaves partly written, uninitialized
        # tables natively: the smoothing is not run after it.
        [[6, *couples_of([(i, CURVATURE) for i in range(1, n + 1)])], [2]],
        [[1], [7, *uvec(1, ts)], [3], [1], [3]],
        [[1], [8, *uvec(1, [0.0, 0.5, 1.0])], [3]],
        [[9, 12], [2], [1], [3], [9, 1], [9, 40]],
        [[10, 3], [2], [10, 0], [1], [3]],
        [[11, C0], [11, C1], [2], [1], [3], [11, G1], [11, C3]],
        [[12, 1], [2], [1], [3], [12, 0]],
        [[13, 0], [2], [1], [3], [13, 1], [1]],
        [[16, 1e-5], [16, 0.0], [2], [1], [3]],
        [[17, 1], [1], [3], [17, 6], [1], [3]],
        [[1], [5], [16, 1e-2], [5], [3]],
    ]
    for cmds in setters:
        smooth(requests, spec, 1, n, pairs, opts, cmds)
    # An over-constrained start, then relaxed setters.
    tight = [(i, TANGENT) for i in range(1, n + 1)]
    smooth(requests, spec, 1, n, tight, [4, 2, C2, 0, 1, 1e-3, 2],
           [[2], [1], [5], [10, 20], [9, 14], [2], [1], [3], [6, *couples_of(pairs)], [2], [5]])


def degenerate_requests(rng, requests):
    one = appdef_line(rng, 1, 0, 2)
    smooth(requests, one, 1, 1, [], [14, 100, C2, 0, 1, 1.0, 2], [[2]])
    smooth(requests, one, 1, 2, [], [14, 100, C2, 0, 1, 1.0, 2], [[2]])
    same = [1, 0, 4] + [0, 0, 1.0, 2.0, 3.0] * 4
    smooth(requests, same, 1, 4, [], [14, 100, C2, 0, 1, 1.0, 2], [[2]])
    line = appdef_line(rng, 1, 0, 12, tangents=(1, 12), curvatures=(6,))
    for first, last in [(2, 12), (1, 11), (3, 2), (0, 12), (1, 13)]:
        smooth(requests, line, first, last, [], [14, 100, C2, 0, 1, 1.0, 2], [[1], [3]])
    bad = [
        ([(1, PASS), (6, PASS)], [14, 100, C2, 0, 1, 1e-3, 2]),
        ([(1, TANGENT), (2, TANGENT), (12, PASS)], [14, 100, C2, 0, 1, 1e-3, 2]),
        ([(1, CURVATURE), (12, PASS)], [14, 100, C2, 0, 1, 1e-3, 2]),
        ([(1, PASS), (12, PASS)], [0, 100, C2, 0, 1, 1e-3, 2]),
        ([(1, PASS), (12, PASS)], [14, 0, C2, 0, 1, 1e-3, 2]),
        ([(1, PASS), (12, PASS)], [14, 100, C3, 0, 1, 1e-3, 2]),
        ([(1, PASS), (12, PASS)], [14, 100, G1, 0, 1, 1e-3, 2]),
        ([(1, PASS), (12, PASS)], [40, 100, C2, 0, 1, 1e-3, 2]),
        ([(1, PASS), (12, PASS)], [14, 100, C2, 0, 1, -1e-3, 2]),
        ([(1, PASS), (13, PASS)], [14, 100, C2, 0, 1, 1e-3, 2]),
        ([(1, NO), (12, NO)], [14, 100, C2, 0, 1, 1e-3, 2]),
        ([(1, PASS), (6, NO), (12, PASS)], [14, 100, C2, 0, 0, 1e-3, 2]),
    ]
    for pairs, opts in bad:
        smooth(requests, line, 1, 12, pairs, opts, [[2], [1], [3]])
    # Collinear points with a zero tangent at a tangency point, and a non-normal curvature.
    zero = [1, 0, 3, 1, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0, 1.0, 0.0, 0.0, 0, 0, 2.0, 0.0, 0.0]
    smooth(requests, zero, 1, 3, [(1, TANGENT), (3, PASS)], [14, 100, C2, 0, 1, 1e-3, 2], [[2]])
    skew = [1, 0, 3, 1, 1, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0, 0, 1.0, 0.0, 0.0, 0, 0, 2.0, 0.0, 0.0]
    smooth(requests, skew, 1, 3, [(1, CURVATURE), (3, PASS)], [14, 100, C2, 0, 1, 1e-3, 2], [[2]])
    skew2d = [0, 1, 3, 1, 1, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0, 0, 1.0, 0.0, 0, 0, 2.0, 0.0]
    smooth(requests, skew2d, 1, 3, [(1, CURVATURE), (3, PASS)], [14, 100, C2, 0, 1, 1e-3, 2], [[2]])
    # A line without points (no dimension: a null chord length).
    smooth(requests, [0, 0, 3, 0, 0, 0, 0, 0, 0], 1, 3, [(1, TANGENT), (3, PASS)], [14, 100, C2, 0, 1, 1e-3, 2], [[2]])
    # A curvature constraint needs more free coefficients than MaxDegree - Continuity.
    smooth(requests, line, 1, 12, [(1, PASS), (6, CURVATURE), (12, PASS)], [4, 5, C2, 0, 1, 1e-3, 2], [[2], [1]])
    # Straight points: the curve may interpolate them exactly.
    straight = [1, 0, 5] + sum(([0, 0, float(i), 0.0, 0.0] for i in range(5)), [])
    smooth(requests, straight, 1, 5, [(1, PASS), (5, PASS)], [6, 4, C2, 0, 1, 1e-3, 2], [[1], [3], [5], [4, 1, 1, 1]])
    scaled = appdef_line(rng, 1, 1, 10, tangents=(1, 10), noise=0.001, scale=1e3)
    smooth(requests, scaled, 1, 10, [(1, TANGENT), (10, TANGENT)], [14, 100, C2, 1, 1, 1e-2, 2], [[1], [3], [5]])
    tiny = appdef_line(rng, 0, 1, 10, noise=0.0, scale=1e-4)
    smooth(requests, tiny, 1, 10, [], [14, 100, C2, 0, 1, 1e-8, 2], [[1], [3]])


def append_variational_requests(requests: list[str]) -> None:
    """Append the AppDef_Variational requests."""
    rng = random.Random(0xAD0EF + 1)
    basic_requests(rng, requests)
    constraint_requests(rng, requests)
    weight_requests(rng, requests)
    setter_requests(rng, requests)
    degenerate_requests(rng, requests)
