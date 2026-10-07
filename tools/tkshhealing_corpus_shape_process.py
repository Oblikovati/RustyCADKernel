"""Requests for ShapeProcess and ShapeProcessAPI (tools/oracle/tkshhealing_probe_shape_process.cxx).

`sp_context` sets parameters of the fixed SP_PARAMS table, pushes and pops scopes and runs
every getter on every name of SP_NAMES: scoped and unscoped lookups, `&name` aliases (also
to a missing name), blank values (the native Standard_OutOfRange of GetReal), integer
prefixes, booleans and continuities. `sp_process` runs the eight predefined sequences of
the table and operation-flag sets on the faces and the compound of the ShapeCustom spline
scene with several parameter sets and trace levels; `sp_apply` runs
ShapeProcessAPI_ApplySequence with and without a sequence.

Known divergence kept out of the corpus: ToBezier with SurfaceMode on a face whose surface
is a surface of linear extrusion (bare or offset) gives other p-curves than OCCT; the
divergence is in ShapeUpgrade_ShapeConvertToBezier (its Perform row is partial), not in
ShapeProcess.
"""
from __future__ import annotations

from tkshhealing_corpus_shapecustom import request, spline_scene

SEQUENCES = range(8)
# Every operation flag; natively FixFaceSize or FixShape after ToBezier (bit 8) crashes
# (SIGSEGV) on the Bezier-converted plane face, so the full set is used without ToBezier.
ALL_FLAGS = (1 << 18) - 1
PARAM_SETS = [
    [],
    [8, 9],
    [10, 11, 12, 17, 21],
    [13, 19],
    [14, 15, 16],
    [20, 8],
    [25],
]


def context_requests() -> list[str]:
    queries = [op for op in range(3, 10)]
    out = []
    for prefix in ([], [0, 12, 0, 22, 0, 23, 0, 24, 0, 28, 0, 29, 0, 8, 0, 11],
                   [0, 12, 0, 22, 0, 23, 0, 24, 0, 28, 0, 29, 0, 8, 0, 11, 1, 0]):
        for op in queries:
            ops = list(prefix)
            for name in range(10):
                ops += [op, name]
            out.append(request('sp_context', len(ops_of(ops)), *ops))
    scoped = [0, 8, 1, 0, 1, 1, 3, 1, 2, 3, 1, 2, 2, 3, 1, 4, 1]
    out.append(request('sp_context', len(ops_of(scoped)), *scoped))
    return out


def ops_of(ops: list[int]) -> list[int]:
    """The op codes of an sp_context op list (op 2 has no operand)."""
    codes, k = [], 0
    while k < len(ops):
        codes.append(ops[k])
        k += 1 if ops[k] == 2 else 2
    return codes


def process_requests() -> list[str]:
    s = spline_scene()
    targets = [s.faces[name] for name in ('plane', 'ext_circle', 'rev_cyl', 'cyl', 'rev_bezier')]
    targets.append(s.compound)
    out = []
    for target in targets:
        for sequence in SEQUENCES:
            for params in PARAM_SETS:
                fields = [sequence, *params]
                out.append(request('sp_process', s.tokens(), target, 0, 0, len(fields), *fields))
        out.append(request('sp_process', s.tokens(), target, 0, 2, 2, 3, 8))
        for flags in (1 << 15, (1 << 0) | (1 << 15), (1 << 3) | (1 << 1) | (1 << 2), ALL_FLAGS & ~(1 << 8)):
            out.append(request('sp_process', s.tokens(), target, -flags, 0, 2, 26, 27))
        if target not in (s.faces['ext_circle'], s.compound):
            out.append(request('sp_process', s.tokens(), target, -(1 << 8), 0, 3, 30, 31, 32))
        out.append(request('sp_apply', s.tokens(), target, 0))
        out.append(request('sp_apply', s.tokens(), target, 3, 0, 8, 9))
    return out


def append_shape_process_requests(requests: list[str]) -> None:
    """Append the ShapeProcess requests."""
    requests.extend(context_requests())
    requests.extend(process_requests())
