"""Deterministic empty-bounds requests (tools/oracle/tkmath_empty_bounds_probe.cxx).

Every math_DoubleTab, math_Matrix and math_VectorBase constructor and member over
row, column and vector ranges of 0, 1 and 2 elements (the empty range is
`upper = lower - 1`), at lower bound 1 and at shifted bounds, with a regular, a zero
and a negative scalar, and the solvers and decompositions over no variable. Native
builds empty tables on its stack buffer and their loops never run; indexing them
raises Standard_OutOfRange. A range below the empty one is left out: native
sign-extends its negative size over the stack buffer (undefined behaviour), which the
port refuses with Standard_RangeError.
"""
from __future__ import annotations

SIZES = (0, 1, 2)
LOWERS = ((1, 1, 1), (-3, 5, 0))
SCALARS = (2.5, 0.0, -1.0)


def append_empty_bounds_requests(requests: list[str]) -> None:
    """Append one request per shape, lower-bound set and scalar, then the solvers."""
    append_empty_table_requests(requests)
    # No variable: BFGS, FRPR, Powell, both set roots and the decompositions, with
    # callbacks that succeed or fail (math_NewtonMinimum is left out: it reads
    # Values()(0) before its empty eigenvalue vector natively).
    for value, ok in [(1.0, 1), (-2.5, 1), (1.0, 0), (0.0, 0)]:
        requests.append(f'empty_solvers {value} {ok}')


def append_empty_table_requests(requests: list[str]) -> None:
    """One request per row, column and vector size, lower-bound set and scalar."""
    for rows in SIZES:
        for cols in SIZES:
            for length in SIZES:
                for rl, cl, vl in LOWERS:
                    for scalar in SCALARS:
                        values = [rl, rl + rows - 1, cl, cl + cols - 1, vl, vl + length - 1, scalar]
                        requests.append('empty_bounds ' + ' '.join(map(str, values)))
