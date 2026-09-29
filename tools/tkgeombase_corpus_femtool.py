"""Deterministic requests for FEmTool (tools/oracle/tkgeombase_probe_femtool.cxx and
tkgeombase_probe_femtool_curve.cxx).

Every object runs a command list (see the probe files) so cached state is compared:
the process-wide reference integrals of the linear criteria (the order of the
criterion requests matters and is the same for both probes), the span cache and the
lazy coefficients of FEmTool_Curve, and the Schur-complement profile of
FEmTool_Assembly. The inputs cover every constraint order, the WorkDegree bounds
(0, 14 and a cached order past 14), empty and full profiles, non positive definite
and NaN matrices, entries outside the profile (aliased coefficients), knots changed
between evaluations, parameters outside the knots, every constraint-segment merge of
AddConstraint, and the native exceptions. Paths that are undefined natively
(unchecked out-of-storage accesses, uninitialized reads) are left out.
"""
from __future__ import annotations
import random
from tkgeombase_corpus_gce import add

SHAPES = {'C0': 0, 'C1': 1, 'C2': 2, 'G1': 3, 'C3': 4}


def elements_requests(requests):
    for wd, shape in [(1, 'C0'), (3, 'C0'), (5, 'C1'), (9, 'C2'), (14, 'C1'), (30, 'C2'), (31, 'C0'), (3, 'C2'), (4, 'G1')]:
        for der in [0, 1, 2, 3, 4, -1]:
            for u in [-1.0, -0.5, 0.0, 0.3, 1.0, 1.5]:
                n = (wd + 2) * (wd + 1) // 2
                for flen in ([n, n + 2, n - 1] if u == 0.3 else [n]):
                    add(requests, 'fem_elements', wd, SHAPES[shape], der, u, flen)


def coefficients(rng, rl, cl, rows, cols, scale=3.0):
    return [rl, cl, rows, cols, *[rng.uniform(-scale, scale) for _ in range(rows * cols)]]


def hessian_commands(deg, dims):
    commands = []
    for d in dims:
        commands += [3, d, d, 0, deg, 0, deg]
        commands += [3, d, d, 1, deg + 1, 1, deg + 1]
        commands += [3, d, d, 2, deg + 2, 0, deg]
        commands += [3, d, d, 0, deg + 2, 0, deg + 2]
        if deg >= 2:
            commands += [3, d, d, 0, deg - 2, 0, deg - 2, 4, d, 1, deg - 1]
        commands += [4, d, 1, deg + 1, 4, d, 0, deg, 4, d, 1, deg + 3]
    commands += [3, dims[0], dims[-1] + 1, 0, deg, 0, deg]
    if len(dims) > 1:
        commands += [3, dims[0], dims[1], 0, deg, 0, deg]
    commands += [4, dims[-1] + 1, 1, deg + 1]
    return commands


def criterion_request(requests, kind, wd, shape, coeff, first, last, commands):
    add(requests, 'fem_criterion', kind, wd, SHAPES[shape], 1 if coeff else 0, *(coeff or []),
        first, last, len(command_list(commands)), *commands)


def command_list(commands):
    """The command codes of a flat criterion command list (for the count)."""
    sizes = {1: 0, 2: 0, 3: 6, 4: 3, 5: 2}
    result, i = [], 0
    while i < len(commands):
        result.append(commands[i])
        i += 1 + sizes[commands[i]]
    return result


def criterion_requests(rng, requests):
    # The first criterion of each kind computes the order-2 integrals; a later
    # WorkDegree 15 with the cached order reads past them, a new order refuses it.
    for kind in [0, 1, 2]:
        for wd, shape in [(14, 'C2'), (15, 'C2'), (15, 'C1'), (9, 'C1'), (5, 'C0'), (0, 'C0'), (3, 'C1'),
                          (7, 'C2'), (6, 'G1'), (6, 'C3'), (12, 'C0'), (15, 'C0'), (5, 'C2')]:
            for rows, rl, cl, dims in [(wd + 1, 0, 1, 3), (wd + 1, 1, 1, 1), (max(wd - 1, 1), 5, 1, 2),
                                       (wd + 3, 0, 1, 2), (wd + 1, 0, 0, 2), (wd + 1, 0, 2, 2)]:
                for first, last in [(0.0, 1.0), (-1.0, 2.5), (1.5, 0.25)]:
                    coeff = coefficients(rng, rl, cl, rows, dims)
                    deg = min(rows - 1, wd)
                    commands = [1, 2, *hessian_commands(deg, list(range(cl, cl + dims)))]
                    criterion_request(requests, kind, wd, shape, coeff, first, last, commands)
            criterion_request(requests, kind, wd if wd <= 14 else 5, 'C1', None, 0.0, 1.0,
                              [1, 3, 1, 1, 0, 3, 0, 3])
        # A null element length (inf scale), a Set between reads and a negative J.
        coeff = coefficients(rng, 0, 1, 6, 2)
        criterion_request(requests, kind, 5, 'C1', coeff, 1.0, 1.0, [2, 3, 1, 1, 0, 5, 0, 5])
        criterion_request(requests, kind, 5, 'C1', coeff, 0.0, 1.0, [2, 5, 0.0, 3.0, 2, 4, 2, 1, 6])
        negative = [0, 1, 6, 1, *[(-1.0) ** i * 1e3 for i in range(6)]]
        criterion_request(requests, kind, 5, 'C0', negative, 0.0, 2.0, [2, 5, -3.0, 1.0, 2])


def profile_commands(rng, first, spd=True, nan=False):
    n = len(first)
    commands = [4, 1, n, *[1.0] * n, 3]
    for i in range(1, n + 1):
        for j in range(first[i - 1], i + 1):
            value = (n + 1.0 + rng.random()) if i == j else rng.uniform(-1.0, 1.0)
            if not spd and i == n and j == i:
                value = -1.0
            if nan and i == 2 and j == 2:
                # A huge pivot (the C++ stream reader cannot read nan or inf).
                value = 1e300
            commands += [2, i, j, value]
    commands += [11]
    for i in range(1, n + 1):
        for j in range(1, n + 1):
            commands += [6, i, j]
    commands += [7, 3, 4, 1, n, *[rng.uniform(-2, 2) for _ in range(n)]]
    commands += [5, 1, n, *[rng.uniform(-2, 2) for _ in range(n)], 8, 7]
    commands += [5, 3, n, *[1.0] * n, 9, 10]
    # The sequence ends with Init, which drops the factorization.
    commands += [1, 0.5, 4, 1, n, *[1.0] * n]
    return commands


def profile_count(commands):
    sizes = {1: 1, 2: 3, 3: 0, 6: 2, 7: 0, 8: 0, 9: 0, 10: 0, 11: 0}
    count, i = 0, 0
    while i < len(commands):
        code = commands[i]
        i += 1 + (2 + int(commands[i + 2]) if code in (4, 5) else sizes[code])
        count += 1
    return count


def profile_requests(rng, requests):
    layouts = [[1], [1, 1], [1, 2, 3], [1, 1, 1, 1], [1, 1, 2, 3, 3], [1, 2, 2, 4, 4, 5], [1, 1, 3, 3, 3, 6, 6, 7],
               [1, 1, 2, 2, 5, 5, 7]]
    for first in layouts:
        for spd, nan in [(True, False), (False, False), (True, True)]:
            if nan and len(first) < 2:
                continue
            commands = profile_commands(rng, first, spd, nan)
            add(requests, 'fem_profile', len(first), *first, profile_count(commands), *commands)
    # Entries outside the profile alias stored coefficients (no native check).
    first = [1, 2, 2, 4]
    commands = [2, 3, 1, 5.0, 2, 4, 1, 7.0, 2, 1, 3, 11.0, 7]
    add(requests, 'fem_profile', 4, *first, profile_count(commands), *commands)


def curve_commands(rng, dim, nb, wd, niv, knots):
    commands = []
    for el in range(1, nb + 1):
        rl, cl = rng.choice([(0, 1), (1, 2), (3, 1)])
        commands += [1, el, rl, cl, wd + 1, dim, *[rng.uniform(-2, 2) for _ in range(dim * (wd + 1))]]
    commands += [9, 1, 2, 1, 0, 1, wd + 1, dim, 2, nb, 1, 2, wd + 2, dim + 1]
    commands += [3, 1, nb * (wd + 1) * dim, 3, 0, nb * (wd + 1) * dim + 3]
    params = [knots[0], knots[-1], knots[0] - 0.5, knots[-1] + 0.3, *[rng.uniform(knots[0], knots[-1]) for _ in range(4)],
              *knots[1:-1]]
    for u in params:
        for order in [0, 1, 2]:
            commands += [4, order, u, 1, dim]
    commands += [4, 1, params[5], 0, dim + 2, 4, 2, params[5], 1, dim + 2, 4, 0, params[5], 1, dim + 2]
    for a, b in [(knots[0], knots[-1]), (params[4], params[5]), (params[2], params[3]), (knots[1], knots[1]),
                 (knots[-1], knots[0]), (params[4], params[4])]:
        commands += [5, a, b]
    commands += [5, knots[0], knots[-1]]
    # Knots changed under the span cache, then the same parameter again.
    commands += [4, 0, params[6], 1, dim, 8, 1, knots[0] - 1.0, 4, 0, params[6], 1, dim, 5, knots[0], knots[-1]]
    commands += [6, 1, wd + 1, 6, 1, max(2 * niv + 2, wd - 1), 9, 1, 4, 0, params[4], 1, dim, 5, params[2], params[3]]
    for tol in [1e3, 1e-2, 1e-12]:
        commands += [7, nb, tol, 9, nb, 4, 1, knots[-1], 1, dim, 5, knots[0] - 1.0, knots[-1]]
    commands += [3, 1, nb * (wd + 1) * dim]
    return commands


def curve_count(commands):
    count, i = 0, 0
    while i < len(commands):
        code = commands[i]
        if code == 1:
            i += 6 + int(commands[i + 4]) * int(commands[i + 5])
        else:
            i += 1 + {2: 5, 3: 2, 4: 4, 5: 2, 6: 2, 7: 2, 8: 2, 9: 1}[code]
        count += 1
    return count


def curve_requests(rng, requests):
    for dim, nb, wd, shape in [(1, 1, 3, 'C0'), (2, 2, 5, 'C1'), (3, 4, 7, 'C2'), (2, 3, 9, 'C1'), (1, 2, 6, 'C2'),
                               (3, 1, 14, 'C0'), (2, 4, 4, 'C0')]:
        niv = SHAPES[shape]
        steps = [rng.uniform(0.2, 2.0) for _ in range(nb)]
        knots = [-0.5]
        for step in steps:
            knots.append(knots[-1] + step)
        commands = curve_commands(rng, dim, nb, wd, niv, knots)
        add(requests, 'fem_curve', dim, nb, wd, niv, *knots, curve_count(commands), *commands)
    # A C1 basis below its Hermite degree, and a G1 basis.
    add(requests, 'fem_curve', 1, 1, 2, 1, 0.0, 1.0, 0)
    add(requests, 'fem_curve', 1, 1, 5, 3, 0.0, 1.0, 0)
    # Element indexes out of range and a degree above WorkDegree.
    add(requests, 'fem_curve', 1, 2, 3, 0, 0.0, 1.0, 2.0, 3, 1, 0, 0, 1, 4, 1, 0, 0, 0, 0, 2, 3, 0, 1, 4, 1, 6, 1, 4)


def spd_block(rng, size, scale=1.0):
    """A lower-dominant symmetric matrix block, row by row."""
    values = [[0.0] * size for _ in range(size)]
    for i in range(size):
        for j in range(i + 1):
            values[i][j] = values[j][i] = (size + 1.0 + rng.random()) * scale if i == j else rng.uniform(-0.5, 0.5)
    return [v for row in values for v in row]


def assembly_request(requests, dep, cells, commands):
    rows, cols = len(dep), len(dep[0])
    table = [len(cells), len(cells[0])]
    for row in cells:
        for lower, indexes in row:
            table += [lower, len(indexes), *indexes]
    add(requests, 'fem_assembly', rows, cols, *[v for row in dep for v in row], *table, assembly_count(commands), *commands)


def assembly_count(commands):
    count, i = 0, 0
    while i < len(commands):
        code = commands[i]
        if code == 1:
            i += 8 + int(commands[i + 6]) * int(commands[i + 7])
        elif code == 2:
            i += 5 + int(commands[i + 4])
        elif code == 3:
            i += 7 + int(commands[i + 5])
        else:
            i += 1 + (1 if code == 5 else 0)
        count += 1
    return count


def element_terms(rng, cells, dims, elements):
    commands = []
    for d in range(1, dims + 1):
        for el in range(1, elements + 1):
            size = len(cells[d - 1][el - 1][1])
            commands += [1, el, d, d, 1, 1, size, size, *spd_block(rng, size)]
            commands += [2, el, d, 1, size, *[rng.uniform(-3, 3) for _ in range(size)]]
    return commands


def chain_cells(dims, elements, width, base=1):
    """Elements sharing their end variables, dimension blocks one after the other."""
    cells, start = [], base
    for _ in range(dims):
        row = []
        for el in range(elements):
            row.append((1, list(range(start + el * (width - 1), start + el * (width - 1) + width))))
        cells.append(row)
        start = row[-1][1][-1] + 1
    return cells


def constraint(rng, index, el, d, size, value):
    return [3, index, el, d, 1, size, *[rng.uniform(-1, 1) for _ in range(size)], value]


def assembly_requests(rng, requests):
    for dims, elements, width in [(1, 1, 3), (1, 3, 4), (2, 2, 3), (3, 2, 4)]:
        cells = chain_cells(dims, elements, width)
        dep = [[1 if i == j else 0 for j in range(dims)] for i in range(dims)]
        terms = element_terms(rng, cells, dims, elements)
        plain = [5, 1, 4, 10, 5, 1, 5, 0]
        chunks = []
        for index, (el, d) in enumerate([(1, 1), (elements, dims), (1, dims)], start=1):
            chunks.append(constraint(rng, index, el, d, width, rng.uniform(-1, 1)))
        # Merges within one row: after the first, before it, between and across.
        for el, d in [(elements, 1), (1, dims), (elements, dims)]:
            chunks.append(constraint(rng, 1, el, d, width, 0.25))
        add_c = [v for chunk in chunks for v in chunk]
        again = [v for chunk in chunks[:2] for v in chunk]
        commands = [*terms, *plain, *add_c, 4, 5, 1, 9, 4, 5, 1, 8, *again, 4, 5, 2, 6, 5, 1, 4, 5, 1]
        assembly_request(requests, dep, cells, commands)
    # Segments added in reverse order (prepend and merge-with-first) and with gaps.
    cells = chain_cells(1, 4, 3)
    terms = element_terms(rng, cells, 1, 4)
    for order in [[4, 1], [3, 1, 4], [1, 4, 2], [4, 3, 2, 1], [1, 3, 4, 2], [2, 4, 1, 3]]:
        c = []
        for el in order:
            c += constraint(rng, 1, el, 1, 3, 0.5)
        c += constraint(rng, 2, order[0], 1, 3, -0.5)
        assembly_request(requests, [[1]], cells, [*terms, *c, 4, 5, 1, 10])
    # Errors: an independent dimension pair, Solution before Solve, a matrix that is
    # not positive definite, a table from index 0 (no constraint, then one).
    cells = chain_cells(2, 1, 3)
    bad = [1, 1, 1, 1, 1, 1, 3, 3, *[-1.0] * 9]
    assembly_request(requests, [[1, 0], [0, 1]], cells,
                     [1, 1, 1, 2, 1, 1, 3, 3, *[1.0] * 9, 5, 1, *bad, 4, 5, 1, 10])
    cells = [[(0, [0, 1, 2]), (1, [2, 3, 4])]]
    terms = element_terms(rng, cells, 1, 2)
    assembly_request(requests, [[1]], cells, [*terms, 4, 5, 0, 5, 3])
    # H1 is indexed from 1: Solve raises Standard_OutOfRange (after storing IsSolved,
    # so a later Solution would dereference the null Schur matrix natively).
    assembly_request(requests, [[1]], cells, [*terms, *constraint(rng, 1, 1, 1, 3, 1.0), 4])
    # The dependence table keeps its own bounds; a full table couples the dimensions.
    cells = chain_cells(2, 2, 3)
    terms = element_terms(rng, cells, 2, 2)
    cross = [1, 1, 2, 1, 1, 1, 3, 3, *[0.01 * v for v in range(9)]]
    assembly_request(requests, [[1, 1], [1, 1]], cells, [*terms, *cross, 4, 5, 1])


def append_femtool_requests(requests: list[str]) -> None:
    """Append the FEmTool requests in a fixed order (the criterion cache depends on it)."""
    rng = random.Random(19981106)
    elements_requests(requests)
    criterion_requests(rng, requests)
    profile_requests(rng, requests)
    curve_requests(rng, requests)
    assembly_requests(rng, requests)
