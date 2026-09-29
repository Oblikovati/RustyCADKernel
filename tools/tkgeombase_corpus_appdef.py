"""Deterministic requests for AppDef (tools/oracle/tkgeombase_probe_appdef.cxx and
tkgeombase_probe_appdef_variational.cxx).

AppDef_MultiPointConstraint runs every constructor on 3D, 2D, mixed and empty point
sets (arrays from 1 and from 3, mismatched vector counts), the tangent and curvature
setters and getters inside and outside their ranges (a failing setter still
allocates its array), the Dump text and the sharing of copies. AppDef_MultiLine runs
every constructor (negative and zero sizes too), its members and the
AppDef_MyLineTool members with arrays of every size and lower bound. AppDef_Compute
runs its four constructors and a second Perform. AppDef_LinearCriteria runs every
member on curves of one to three elements, C0 to C2, matching and mismatching the
line dimension, parameters and point weights with shifted bounds, and every
SetCurve path. The paths that dereference a null array natively (the vectors of a
point that has none, a default line, a gradient before any basis cache, SetCurve of
a null curve) and point ranges with LastPoint < FirstPoint - 1 (a failed allocation of
wrapped size, then a crash) are left out.
AppDef_Variational is in tkgeombase_corpus_appdef_variational.py.
"""
from __future__ import annotations
import math
import random
from tkgeombase_corpus_appparcurves import line_points, uvec
from tkgeombase_corpus_gce import add

NO, PASS, TANGENT, CURVATURE = 0, 1, 2, 3


def normal_part(s, d):
    """The component of `s` normal to `d` (a curvature constraint must be normal)."""
    dd = sum(x * x for x in d)
    k = sum(a * b for a, b in zip(s, d)) / dd if dd else 0.0
    return [a - k * b for a, b in zip(s, d)]


def appdef_line(rng, nb3d, nb2d, n, tangents=(), curvatures=(), noise=0.01, shape=0, scale=1.0):
    """A gb_apc_read_line spec: every flagged point carries tangents (and curvatures
    normal to them) for each of its 3D and 2D points."""
    spec = [nb3d, nb2d, n]
    for i, (t, p3, d3, s3, p2, d2, s2) in enumerate(line_points(rng, n, noise, shape), start=1):
        tan, curv = i in tangents or i in curvatures, i in curvatures
        spec += [1 if tan else 0, 1 if curv else 0]
        for k in range(nb3d):
            spec += [scale * p3[0] + k, scale * p3[1], scale * p3[2] - k]
        for k in range(nb2d):
            spec += [scale * p2[0], scale * p2[1] + k]
        if tan:
            spec += d3 * nb3d + d2 * nb2d
        if curv:
            spec += normal_part(s3, d3) * nb3d + normal_part(s2 + [0.0], d2 + [0.0])[:2] * nb2d
    return spec


def vec3(rng):
    return [rng.uniform(-2, 2) for _ in range(3)]


def vec2(rng):
    return [rng.uniform(-2, 2) for _ in range(2)]


# --- AppDef_MultiPointConstraint ---

ALLOCATES = {5: 'tTcC', 6: 'tT', 7: 'tc', 8: 't', 9: 'T', 10: 'TC'}
COUNTS = {0: (0, 0), 1: (1, 1), 2: (1, 0), 3: (0, 1), 4: (1, 1), 5: (1, 1), 6: (1, 1), 7: (1, 0), 8: (1, 0), 9: (0, 1), 10: (0, 1)}


def mpc_spec(rng, kind, lower, n3, n2, mismatch=0):
    """The point of `kind` and the arrays it allocates (t, T, c, C: 3D and 2D tangents
    and curvatures); a mismatch shortens the first vector array (a construction error)."""
    use3, use2 = COUNTS[kind]
    n3, n2 = (n3 if use3 else 0), (n2 if use2 else 0)
    alloc = ALLOCATES.get(kind, '')
    nv3 = n3 if 't' in alloc else 0
    nv2 = n2 if 'T' in alloc else 0
    nc3 = n3 if 'c' in alloc else 0
    nc2 = n2 if 'C' in alloc else 0
    if mismatch:
        if 't' in alloc:
            nv3 += 1
        elif 'T' in alloc:
            nv2 += 1
    values = [kind, lower, n3, n2, nv3, nv2, nc3, nc2]
    for _ in range(n3):
        values += [rng.uniform(-5, 5) for _ in range(3)]
    for _ in range(n2):
        values += [rng.uniform(-5, 5) for _ in range(2)]
    for count, vec in [(nv3, vec3), (nv2, vec2), (nc3, vec3), (nc2, vec2)]:
        for _ in range(count):
            values += vec(rng)
    if kind == 1:
        values[2:4] = [n3, n2]
    return values, set(alloc) if not mismatch else set(), (n3, n2)


def mpc_commands(rng, alloc, n3, n2):
    """Setters and getters across the index ranges, never reading an unallocated array
    at a valid index (a null dereference natively)."""
    c, count = [], 0
    for setter, flag, low, high, vec in [(1, 't', 1, n3, vec3), (3, 'T', n3 + 1, n3 + n2, vec2),
                                         (5, 'c', 1, n3, vec3), (7, 'C', n3 + 1, n3 + n2, vec2)]:
        for i in [low - 1, low, high, high + 1]:
            getter = setter + 1
            if low <= i <= high and flag not in alloc:
                c += [9]
            else:
                c += [getter, i]
            count += 1
        for i in sorted({low - 1, high, high + 1}):
            c += [setter, i, *vec(rng)]
            alloc = alloc | {flag}
            count += 1
            if low <= i <= high:
                c += [getter, i]
                count += 1
    c += [9, 10, 12]
    count += 3
    for i in [1, n3 + 1]:
        valid = 1 <= i <= n3
        if valid and 't' not in alloc:
            continue
        c += [11, i, *vec3(rng)]
        count += 1
    return count, c


def mpc_requests(rng, requests):
    for kind in range(11):
        for n3, n2 in [(1, 0), (0, 1), (2, 2), (1, 3), (0, 0)]:
            for lower in [1, 3]:
                for mismatch in ([0, 1] if kind >= 5 else [0]):
                    spec, alloc, (m3, m2) = mpc_spec(rng, kind, lower, n3, n2, mismatch)
                    count, c = mpc_commands(rng, alloc, m3, m2) if not mismatch else (0, [])
                    add(requests, 'ad_mpc', *spec, count, *c)
    # A copy shares an allocated array: its SetTang changes the original.
    spec, _, _ = mpc_spec(rng, 8, 1, 2, 0)
    add(requests, 'ad_mpc', *spec, 3, 11, 1, 1.0, 2.0, 3.0, 11, 2, 0.5, 0.5, 0.5, 2, 1)


# --- AppDef_MultiLine and AppDef_MyLineTool ---

def tool_commands(n, nb3d, nb2d):
    c = [1, 2, 5, 6, 7]
    for i in [0, 1, n, n + 1]:
        c += [4, i]
        for code in [8, 9, 10]:
            for overload in [0, 1, 2]:
                c += [code, i, overload, 1, nb3d, nb2d]
    if n:
        c += [8, 1, 2, 3, nb3d, nb2d, 8, 1, 0, 1, max(nb3d - 1, 0), nb2d, 8, n, 2, -2, nb3d + 1, nb2d + 1]
        c += [9, n, 2, 0, nb3d, nb2d, 10, n, 2, 5, nb3d, nb2d]
    c += [11, 1, n, 12, 1, n, 7, 13, 1, n, 2]
    return c


def one_point_line(rng, nb3d, nb2d):
    return appdef_line(rng, nb3d, nb2d, 2, tangents=(1,), curvatures=(1,))


def line_command_count(c):
    sizes = {1: 0, 2: 0, 4: 1, 5: 0, 6: 0, 7: 0, 8: 5, 9: 5, 10: 5, 11: 2, 12: 3, 13: 3}
    count, i = 0, 0
    while i < len(c):
        code = c[i]
        if code in (3, 14):
            nb3d, nb2d, n = c[i + 2], c[i + 3], c[i + 4]
            i += 2 + 3 + n * (2 + 3 * nb3d + 2 * nb2d) + 2 * (3 * nb3d + 2 * nb2d)
        else:
            i += 1 + sizes[code]
        count += 1
    return count


def line_requests(rng, requests):
    add(requests, 'ad_line', 0, 1, 6)
    for nb3d, nb2d in [(1, 0), (0, 1), (1, 1), (2, 1)]:
        for n in [0, 1, 4, 7]:
            source = appdef_line(rng, nb3d, nb2d, max(n, 2), tangents=(1, 2), curvatures=(2,))
            if n == 0:
                source = [nb3d, nb2d, 0]
            c = tool_commands(n, nb3d, nb2d)
            c += [3, 1, *one_point_line(rng, nb3d, nb2d), 3, n + 1, *one_point_line(rng, nb3d, nb2d)]
            c += [14, 1, *one_point_line(rng, nb3d, nb2d), 4, 1, 1, 2]
            for kind, head in [(1, [n]), (2, [1]), (2, [4])]:
                add(requests, 'ad_line', kind, *head, *source, line_command_count(c), *c)
        for n in [0, 3]:
            pts = [rng.uniform(-3, 3) for _ in range(3 * n)]
            c = tool_commands(n, 1, 0)
            add(requests, 'ad_line', 3, 2, n, *pts, line_command_count(c), *c)
            pts2 = [rng.uniform(-3, 3) for _ in range(2 * n)]
            c = tool_commands(n, 0, 1)
            add(requests, 'ad_line', 4, 1, n, *pts2, line_command_count(c), *c)
    # A negative size; a line longer than its source keeps empty multipoints.
    add(requests, 'ad_line', 1, -1, 1, 0, 0, 0)
    source = appdef_line(rng, 1, 0, 3)
    add(requests, 'ad_line', 1, 5, *source, 4, 4, 5, 7, 2, 4, 1)


# --- AppDef_Compute ---

def compute_requests(rng, requests):
    for nb3d, nb2d in [(1, 0), (0, 1), (1, 1)]:
        for n, tangents in [(6, (1, 6)), (15, ()), (24, (1, 24))]:
            line = appdef_line(rng, nb3d, nb2d, n, tangents=tangents)
            ts = [i / (n - 1) for i in range(n)]
            for variant in range(4):
                cons = (TANGENT, TANGENT) if tangents else (PASS, PASS)
                opts = [3, 8, 1e-3, 1e-6, 5, 1, variant % 3, 0]
                add(requests, 'ad_compute', *line, variant, *opts, *cons, *uvec(1, ts), 1 if variant == 0 else 0)


# --- AppDef_LinearCriteria ---

def assembly_size(dim, nb, wd, order):
    nc1 = order + 1
    return dim * ((wd + 1) + (nb - 1) * (wd + 1 - nc1))


def criteria_curve(rng, dim, nb, wd, shape, knots=None):
    knots = knots or [i / nb for i in range(nb + 1)]
    return [dim, nb, wd, shape, *knots, *[rng.uniform(-1, 1) for _ in range(nb * (wd + 1) * dim)]]


def elements_with_points(ts, knots):
    """The elements with a point: t in (u0, u1], or the first knot for element 1."""
    result = set()
    for el in range(1, len(knots)):
        u0, u1 = knots[el - 1], knots[el]
        if any((u0 < t <= u1) or (el == 1 and t == u0) for t in ts):
            result.add(el)
    return result


def criteria_script(rng, dim, nb, wd, shape, ts, lower, knots, line_dim):
    order = [0, 1, 2][shape]
    c = [18, 8, 5, 16, 1, lower, len(ts), *ts]
    c += [15, 2.0, 0.5, 1.0, 2.0, 1.0, 16, 4, 1e-2, 1e-3, 2e-3, 5, 6, 3.5, 7]
    c += [2, *criteria_curve(rng, dim, nb, wd, shape, knots), 18, 8, 9, 3]
    c += [10, 1e-8, 1e-6, 1e-6, 10, 1e-8, 1e-6, 1e-6, 11]
    with_points = elements_with_points(ts, knots)
    built = False
    for el in range(1, nb + 1):
        c += [12, el, 1, 1, 0, 0, wd + 1, 12, el, 1, 1, 1, 3, wd + 1]
        if el in with_points:
            built = True
        if built:
            for d in range(1, dim + 1):
                c += [13, el, d, 1 if d % 2 else 0, wd + 1]
    if built:
        c += [13, 1, line_dim + 1, 1, wd + 1, 13, 1, 1, 0, wd]
    c += [12, 1, 1, 2, 0, 0, wd + 1] if dim > 1 else []
    size = assembly_size(dim, nb, wd, order)
    c += [14, 1, size, *[rng.uniform(-1, 1) for _ in range(size)], 10, 1e-8, 1e-6, 1e-6, 11]
    c += [17, lower, len(ts), *[rng.uniform(0.5, 2) for _ in ts], 12, 1, 1, 1, 0, 0, wd + 1]
    c += [15, -1.0, 1.0, 1.0, 1.0, 1.0, 15, 1.0, 1.0, -1.0, 1.0, 1.0, 16]
    return c


CRITERIA_SIZES = {3: 0, 4: 3, 5: 0, 6: 1, 7: 0, 8: 0, 9: 0, 10: 3, 11: 0, 12: 6, 13: 4, 15: 5, 16: 0, 18: 0}


def criteria_count(c):
    count, i = 0, 0
    while i < len(c):
        code = c[i]
        if code in (1, 14, 17):
            i += 3 + c[i + 2]
        elif code == 2:
            dim, nb, wd = c[i + 1], c[i + 2], c[i + 3]
            i += 5 + nb + 1 + nb * (wd + 1) * dim
        else:
            i += 1 + CRITERIA_SIZES[code]
        count += 1
    return count


def criteria_requests(rng, requests):
    for nb3d, nb2d in [(1, 0), (0, 1), (1, 1), (2, 0)]:
        n = 9
        line = appdef_line(rng, nb3d, nb2d, n)
        ts = [(i / (n - 1)) ** 1.3 for i in range(n)]
        dim = 3 * nb3d + 2 * nb2d
        for nb, wd, shape in [(1, 6, 0), (2, 7, 1), (3, 9, 2), (3, 6, 2)]:
            for lower, knots in [(1, None), (1, [0.0, 0.05, 0.1, 1.0][:nb + 1] if nb == 3 else None)]:
                knots = knots or [i / nb for i in range(nb + 1)]
                c = criteria_script(rng, dim, nb, wd, shape, ts, lower, knots, dim)
                add(requests, 'ad_criteria', *line, 1, n, criteria_count(c), *c)
        # A curve of another dimension (ErrorValues and Gradient refuse it), a curve
        # of new degree and order, then of new dimension only.
        c = [1, 1, n, *ts, 2, *criteria_curve(rng, dim + 1, 2, 6, 1), 11, 13, 1, dim + 1, 1, 7]
        c += [2, *criteria_curve(rng, dim, 2, 8, 2), 8, 2, *criteria_curve(rng, dim + 2, 2, 8, 2), 8, 9]
        c += [2, *criteria_curve(rng, dim, 1, 5, 0), 3, 8]
        add(requests, 'ad_criteria', *line, 1, n, criteria_count(c), *c)
    # Before SetCurve every member needing the curve raises; bad point ranges.
    line = appdef_line(rng, 1, 0, 5)
    c = [8, 9, 10, 1e-8, 1e-6, 1e-6, 11, 12, 1, 1, 1, 0, 0, 6, 16, 5, 18, 7]
    add(requests, 'ad_criteria', *line, 1, 5, criteria_count(c), *c)
    add(requests, 'ad_criteria', *line, 2, 1, 1, 18)


def append_appdef_requests(requests: list[str]) -> None:
    """Append every AppDef request family (the containers, the line tool, Compute,
    LinearCriteria, then Variational)."""
    rng = random.Random(0xAD0EF)
    mpc_requests(rng, requests)
    line_requests(rng, requests)
    compute_requests(rng, requests)
    criteria_requests(rng, requests)
    __import__('tkgeombase_corpus_appdef_variational').append_variational_requests(requests)
