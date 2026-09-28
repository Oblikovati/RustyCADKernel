"""Deterministic requests for GeomTools (tools/oracle/tkgeombase_probe_geomtools*.cxx).

Write cases build every Geom curve, Geom2d curve and Geom surface class (and one class
GeomTools does not know) and compare the dump and compact texts at precisions 6 and 17
byte for byte, then the geometry read back from the compact text. The numbers exercise
the `%g` layouts: signed zeros, exponents on both sides, rounding at 6 and 17 digits,
huge and tiny values. Read cases feed native-format and malformed text (unknown and
overflowing type codes, null directions, parallel axes, invalid radii, weights and knots,
bad booleans, long words, `nan`/`inf`/hexadecimal/subnormal numbers) to the readers and
compare the geometry, the stream state and the unread rest. Set cases cover indexing,
Dump/Write/Read, repeated reads, wrong tags, progress interruption, the debug dump and a
custom undefined-type handler.

Every read text keeps enough tokens after the last record: the native readers extract the
type code into an uninitialised `int`, so a type code read at end of input would be
undefined behaviour, which no oracle can pin down.
"""
from __future__ import annotations

Z = [0.0, 0.0, 1.0]
X = [1.0, 0.0, 0.0]
O = [0.0, 0.0, 0.0]
TAIL = ' 0' * 24


def req(op: str, values: list) -> str:
    """One request line; floats use repr, which both probes parse exactly."""
    return ' '.join([op, *(repr(float(v)) if isinstance(v, float) else str(v) for v in values)])


def text_bytes(text: str) -> list[int]:
    data = text.encode('latin1')
    return [len(data), *data]


def ax2(p=O, n=Z, x=X) -> list:
    return [*p, *n, *x]


def ax3(p=O, n=Z, x=X, indirect=0) -> list:
    return [*p, *n, *x, indirect]


# Curve specs of tkgeombase_probe_geomtools_build.cxx.
TILTED = ax2([1.5, -2.25, 0.1], [1.0, 2.0, 3.0], [3.0, 0.0, -1.0])
CIRCLE = [2, *ax2(), 2.5]
BEZIER3 = [6, 0, 4, 0.0, 0.0, 0.0, 1.0, 2.0, 0.0, 3.0, 2.0, 1.0, 4.0, 0.0, -0.0]
BEZIER_RATIONAL = [6, 1, 3, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0, 0.0, 0.0, 1.0, 0.7071067811865476, 1.0]
BSPLINE = [7, 0, 0, 3, 5, *[0.0, 0.0, 0.0], *[1.0, 2.0, 0.0], *[2.0, -1.0, 0.5], *[3.0, 3.0, 1.0],
           *[4.0, 0.0, 0.0], 3, 0.0, 0.5, 1.0, 4, 1, 4]
BSPLINE_RATIONAL = [7, 1, 0, 2, 3, *[1.0, 0.0, 0.0], *[1.0, 1.0, 0.0], *[0.0, 1.0, 0.0],
                    1.0, 0.7071067811865476, 1.0, 2, 0.0, 1.5707963267948966, 3, 3]
BSPLINE_PERIODIC = [7, 0, 1, 2, 3, *[1.0, 0.0, 0.0], *[-0.5, 0.8660254037844386, 0.0],
                    *[-0.5, -0.8660254037844386, 0.0], 4, 0.0, 1.0, 2.0, 3.0, 1, 1, 1, 1]
HELIX = [10, *ax2(), 1.5, 0.25]

CURVES = [
    [1, 0.0, 0.0, 0.0, *Z],
    [1, 0.1, -0.0, 1e300, 1.0, 1.0, 1.0],
    [1, 123456.5, 1e-5, 1e21, -0.6, 0.8, 0.0],
    [1, 1 / 3, 2 / 3, -1e-300, 0.0, -1.0, 0.0],
    [1, 999999.5, 0.0001, 12345678901234567.0, 1e-3, 0.0, 1.0],
    CIRCLE,
    [2, *TILTED, 1e-300],
    [2, *ax2([1e10, -1e-10, 0.5]), 0.0],
    [3, *ax2(), 5.0, 3.0],
    [3, *TILTED, 2.0, 2.0],
    [4, *ax2(), 0.25],
    [4, *TILTED, 1234.5678],
    [5, *ax2(), 5.0, 3.0],
    [5, *TILTED, 1.0, 7.5],
    [6, 0, 2, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    BEZIER3,
    BEZIER_RATIONAL,
    [6, 1, 3, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0, 0.0, 0.0, 2.0, 2.0, 2.0],
    BSPLINE,
    BSPLINE_RATIONAL,
    BSPLINE_PERIODIC,
    [8, 0.0, 1.5707963267948966, 1, *CIRCLE],
    [8, 0.25, 2.0, 0, *CIRCLE],
    [8, 7.0, 8.0, 1, *CIRCLE],
    [8, 0.1, 0.9, 1, 8, 0.0, 1.0, 1, *BSPLINE],
    [8, -2.0, 3.0, 1, 1, 0.0, 0.0, 0.0, *X],
    [9, 0.5, *Z, *CIRCLE],
    [9, -0.25, *Z, *BSPLINE],
    [9, 0.125, *Z, 9, 0.5, *Z, *CIRCLE],
    [9, 1.0, *Z, 8, 0.0, 1.0, 1, *BEZIER3],
    HELIX,
]

# Curve2d specs.
AX22D = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0]
AX22D_INDIRECT = [1.5, -2.0, 0.6, 0.8, 0.8, -0.6]
CIRCLE2D = [2, *AX22D, 2.5]
BSPLINE2D = [7, 0, 0, 2, 4, 0.0, 0.0, 1.0, 2.0, 2.0, -1.0, 3.0, 0.5, 3, 0.0, 0.25, 1.0, 3, 1, 3]
CURVES2D = [
    [1, 0.0, 0.0, 1.0, 0.0],
    [1, 1e-5, -0.0, 3.0, 4.0],
    [1, 1e21, 123456.5, -1.0, -1.0],
    CIRCLE2D,
    [2, *AX22D_INDIRECT, 1e-7],
    [3, *AX22D, 5.0, 3.0],
    [3, *AX22D_INDIRECT, 1.1, 1.1],
    [4, *AX22D, 0.25],
    [4, *AX22D_INDIRECT, 99.5],
    [5, *AX22D, 5.0, 3.0],
    [5, *AX22D_INDIRECT, 0.5, 8.0],
    [6, 0, 3, 0.0, 0.0, 1.0, 2.0, 2.0, 0.0],
    [6, 1, 3, 0.0, 0.0, 1.0, 1.0, 2.0, 0.0, 1.0, 0.5, 1.0],
    BSPLINE2D,
    [7, 1, 0, 2, 3, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.7071067811865476, 1.0, 2, 0.0, 1.0, 3, 3],
    [7, 0, 1, 2, 3, 1.0, 0.0, -0.5, 0.8660254037844386, -0.5, -0.8660254037844386, 4, 0.0, 1.0, 2.0,
     3.0, 1, 1, 1, 1],
    [8, 0.0, 3.0, 1, *CIRCLE2D],
    [8, 0.2, 0.8, 0, *BSPLINE2D],
    [9, 0.5, *CIRCLE2D],
    [9, -0.1, *BSPLINE2D],
    [9, 0.25, 8, 0.0, 1.0, 1, *CIRCLE2D],
    [10, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 0.5],
]

# Surface specs.
PLANE = [1, *ax3()]
BSPLINE_SURFACE = [9, 0, 0, 0, 1, 2, 2, 3, *[0.0, 0.0, 0.0], *[0.0, 1.0, 0.5], *[0.0, 2.0, 0.0],
                   *[1.0, 0.0, 1.0], *[1.0, 1.0, 2.0], *[1.0, 2.0, 1.0], 2, 0.0, 1.0, 2, 2, 2, 0.0, 1.0, 3, 3]
SURFACES = [
    PLANE,
    [1, *ax3([1e-5, -0.0, 1e21], [1.0, 2.0, 3.0], [3.0, 0.0, -1.0], 1)],
    [2, *ax3(), 2.0],
    [2, *ax3([0.1, 0.2, 0.3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], 1), 123456.5],
    [3, *ax3(), 0.5, 1.0],
    [3, *ax3(indirect=1), -0.3, 0.0],
    [4, *ax3(), 3.0],
    [4, *ax3([1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 0.0, 1.0], 1), 1e-8],
    [5, *ax3(), 5.0, 1.0],
    [5, *ax3(indirect=1), 2.5, 2.5],
    [6, *Z, *BSPLINE],
    [6, 1.0, 1.0, 0.0, *CIRCLE],
    [7, 0.0, 0.0, 0.0, *Z, 1, 2.0, 0.0, 0.0, *Z],
    [7, 1.0, 0.0, 0.0, *X, *BEZIER3],
    [8, 0, 2, 2, *[0.0, 0.0, 0.0], *[0.0, 1.0, 0.0], *[1.0, 0.0, 0.0], *[1.0, 1.0, 1.0]],
    [8, 1, 3, 2, *[0.0, 0.0, 0.0], *[0.0, 1.0, 0.0], *[1.0, 0.0, 1.0], *[1.0, 1.0, 1.0],
     *[2.0, 0.0, 0.0], *[2.0, 1.0, 0.0], 1.0, 1.0, 0.5, 0.5, 1.0, 1.0],
    BSPLINE_SURFACE,
    [9, 1, 0, 0, 1, 1, 2, 2, *[0.0, 0.0, 0.0], *[0.0, 1.0, 0.0], *[1.0, 0.0, 0.0], *[1.0, 1.0, 1.0],
     1.0, 2.0, 3.0, 1.0, 2, 0.0, 1.0, 2, 2, 2, 0.0, 1.0, 2, 2],
    [9, 0, 1, 0, 1, 1, 3, 2, *[1.0, 0.0, 0.0], *[1.0, 0.0, 1.0], *[-0.5, 0.9, 0.0], *[-0.5, 0.9, 1.0],
     *[-0.5, -0.9, 0.0], *[-0.5, -0.9, 1.0], 4, 0.0, 1.0, 2.0, 3.0, 1, 1, 1, 1, 2, 0.0, 1.0, 2, 2],
    [10, 0.0, 3.0, -1.0, 2.0, *PLANE],
    [10, 0.5, 4.0, 0.0, 1.0, 2, *ax3(), 2.0],
    [10, 0.1, 0.9, 0.2, 0.8, *BSPLINE_SURFACE],
    [11, 0.5, *PLANE],
    [11, -0.25, 2, *ax3(), 2.0],
    [11, 0.125, *BSPLINE_SURFACE],
    [11, 1.0, 10, 0.0, 1.0, 0.0, 1.0, *PLANE],
    [12, *ax3(), 3.0, 2.0, 1.0],
]


def write_requests(requests: list[str]) -> None:
    for spec in CURVES:
        requests.append(req('gt_curve', spec))
    for spec in CURVES2D:
        requests.append(req('gt_curve2d', spec))
    for spec in SURFACES:
        requests.append(req('gt_surface', spec))


# Texts for the single-record readers, by kind (0 curve, 1 curve2d, 2 surface).
CURVE_TEXTS = [
    '1 0 0 0 0 0 1 \n', '1 0, 0, 0 0, 0, 1 ', '1 1E3 -2.5e-3 +7 0 0 2 ', '1 007 .5 -.5e1 0 1 1 ',
    '1 nan -inf inf 0 0 1 ', '1 0 0 0 nan 0 0 ', '1 0 0 0 inf 0 0 ', '1 0 0 0 1e200 1e200 0 ',
    '1 0x1p-3 0X10 -0x.8 0 0 1 ', '1 4.9e-324 2.2250738585072014e-308 1e-320 0 0 1 ',
    '1 1e400 -1e400 1e-400 0 1 0 ', '1 1.5abc 2,5 3;4 0 0 1 ',
    '1 0.1234567890123456789012345678901234 0 0 0 0 1 ',
    '1 12345678901234567890123456789012345678 0 0 1 ',
    '1 0 0 0 0 0 0 5 ', '1 0 0 0 1e-200 0 0 ',
    '2 0 0 0 0 0 1 1 0 0 0 1 0 2.5\n', '2 0 0 0 0 0 1 1 0 0 0 1 0 -1 ',
    '2 0 0 0 0 0 1 0 0 1 0 1 0 1 ', '2 0 0 0 0 0 0 1 0 0 0 1 0 1 ',
    '2 0 0 0 0 0 1 1 1 0 5 5 5 1 ', '2 1 2 3 0 0 1 1 0 0 0 0 0 1 ',
    '3 0 0 0 0 0 1 1 0 0 0 1 0 5 3 ', '3 0 0 0 0 0 1 1 0 0 0 1 0 3 5 ',
    '4 0 0 0 0 0 1 1 0 0 0 1 0 0.5 ', '4 0 0 0 0 0 1 1 0 0 0 1 0 -0.5 ',
    '5 0 0 0 0 0 1 1 0 0 0 1 0 5 3 ', '5 0 0 0 0 0 1 1 0 0 0 1 0 -5 3 ',
    '6 0 1 0 0 0 1 1 1 ', '6 0 0 1 2 3 ', '6 0 -1 ', '6 1 1 0 0 0 2 1 1 1 4 ',
    '6 1 1 0 0 0 0 1 1 1 1 ', '6 1 1 0 0 0 3 1 1 1 3 ', '6 2 1 0 0 0 1 1 1 ',
    '6 0 30 ' + '1 2 3 ' * 31,
    '7 0 0 1 2 2 0 0 0 1 0 0 0 2 1 2\n', '7 1 0 1 2 2 0 0 0 1 1 0 0 3 0 2 1 2 ',
    '7 0 0 1 2 2 0 0 0 1 0 0 1 2 0 2 ', '7 0 0 1 2 2 0 0 0 1 0 0 0 1 0 1 ',
    '7 0 1 1 3 4 0 0 0 1 0 0 0 1 0 0 1 1 1 2 1 3 1 ', '7 0 0 -1 2 2 0 0 0 1 0 0 0 2 1 2 ',
    '7 0 0 1 2 0 0 0 0 1 0 0 ', '7 0 0 1 2 2 0 0 0 1 0 0 0 2 1 x ',
    '8 0 1 1 0 0 0 1 0 0 \n', '8 1 1 1 0 0 0 1 0 0 \n', '8 0 1 42 5 ', '8 0 1 x 1 2 3 ',
    '8 0 1 2 0 0 0 0 0 0 1 0 0 0 1 0 1 ', '8 7 8 2 0 0 0 0 0 1 1 0 0 0 1 0 1 ',
    '8 0 3 8 0 1 1 0 0 0 1 0 0 ', '8 0.5 0.25 7 0 0 1 2 2 0 0 0 1 0 0 0 2 1 2 ',
    '9 0.5 0 0 1 2 0 0 0 0 0 1 1 0 0 0 1 0 1 \n', '9 0.5 0 0 0 1 0 0 0 1 0 0 ',
    '9 1 0 0 1 1 0 0 0 0 0 1 ', '9 1 0 0 1 42 ', '9 1 0 0 1 9 2 0 1 0 1 0 0 0 1 0 0 ',
    '9 2 0 0 1 7 0 0 1 3 2 0 0 0 1 1 0 2 0 0 2 0 2 1 2 ',
    '42 1 2 3 \n', '0 5 ', '-7 ', '99999999999 1 ', '-99999999999 1 ', 'abc 1 ', '+1 0 0 0 0 0 1 ',
    '1.5 0 0 0 0 0 1 ',
]
CURVE2D_TEXTS = [
    '1 0 0 1 0 \n', '1 0, 0, 0, 1 ', '1 nan inf 1 1 ', '1 0 0 0 0 5 ', '1 1e400 -0 1e-400 1 ',
    '2 0 0 1 0 0 1 2.5\n', '2 0 0 1 0 0 -1 2.5 ', '2 0 0 1 0 1 0 2.5 ', '2 0 0 0 0 0 1 2.5 ',
    '2 0 0 1 0 0 1 -2 ', '3 0 0 1 0 0 1 5 3 ', '3 0 0 1 0 0 1 3 5 ', '4 0 0 1 0 0 1 0.5 ',
    '4 0 0 1 0 0 1 -1 ', '5 0 0 1 0 0 1 5 3 ', '5 0 0 1 0 0 1 5 -3 ',
    '6 0 1 0 0 1 1 ', '6 0 0 1 2 ', '6 1 2 0 0 1 1 2 0 1 2 1 ', '6 1 1 0 0 1 1 -1 1 ',
    '7 0 0 1 2 2 0 0 1 0 2 0 2 1 2\n', '7 1 0 1 2 2 0 0 1 1 0 3 2 0 2 1 2 ',
    '7 0 0 1 2 2 0 0 1 0 2 0 1 1 1 ', '7 0 1 1 3 4 0 0 1 0 0 1 0 1 1 1 2 1 3 1 ',
    '8 0 1 1 0 0 1 0 \n', '8 1 1 1 0 0 1 0 \n', '8 0 1 43 1 ', '8 0 1 2 0 0 0 0 0 1 1 ',
    '9 0.5 2 0 0 1 0 0 1 1 \n', '9 1 1 0 0 1 0 ', '9 1 42 1 ', '9 0.5 9 0.5 2 0 0 1 0 0 1 1 ',
    '42 1 2 3 ', '0 5 ', '99999999999 1 ', 'abc 1 ',
]
SURFACE_TEXTS = [
    '1 0 0 0 0 0 1 1 0 0 0 1 0 \n', '1 0 0 0 0 0 1 1 0 0 0 -1 0 \n', '1 0 0 0 0 0 1 0 0 1 0 1 0 ',
    '1 0 0 0 0 0 1 1 0 0 0 0 0 ', '1 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0 ',
    '2 0 0 0 0 0 1 1 0 0 0 1 0 2 \n', '2 0 0 0 0 0 1 1 0 0 0 1 0 -2 ', '2 0 0 0 0 0 1 0 0 1 0 1 0 2 ',
    '3 0 0 0 0 0 1 1 0 0 0 1 0 1 0.5 ', '3 0 0 0 0 0 1 1 0 0 0 1 0 1 0 ',
    '3 0 0 0 0 0 1 1 0 0 0 1 0 -1 0.5 ', '3 0 0 0 0 0 1 1 0 0 0 -1 0 1 2 ',
    '4 0 0 0 0 0 1 1 0 0 0 1 0 3 ', '4 0 0 0 0 0 1 1 0 0 0 1 0 -3 ',
    '5 0 0 0 0 0 1 1 0 0 0 1 0 5 1 ', '5 0 0 0 0 0 1 1 0 0 0 1 0 1 5 ',
    '5 0 0 0 0 0 1 1 0 0 0 1 0 -5 1 ',
    '6 0 0 1 1 0 0 0 1 0 0 \n', '6 0 0 0 1 0 0 0 1 0 0 ', '6 0 0 1 42 ', '6 1 0 0 1 0 0 0 1 0 0 ',
    '7 0 0 0 0 0 1 1 2 0 0 0 0 1 \n', '7 0 0 0 0 0 0 1 2 0 0 0 0 1 ', '7 0 0 0 0 0 1 42 ',
    '8 0 0 1 1 0 0 0 0 1 0 1 0 0 1 1 1 \n', '8 1 0 1 1 0 0 0 1 0 1 0 1 1 0 0 2 1 1 1 1 ',
    '8 0 0 0 1 0 0 0 0 1 0 ', '8 0 0 -1 1 ', '8 0 1 1 1 0 0 0 1 0 1 0 1 1 0 0 1 1 1 1 1 ',
    '9 0 0 0 0 1 1 2 2 2 2 0 0 0 0 1 0 1 0 0 1 1 1 0 2 1 2 0 2 1 2 \n',
    '9 1 0 0 0 1 1 2 2 2 2 0 0 0 1 0 1 0 1 1 0 0 1 1 1 1 1 0 2 1 2 0 2 1 2 ',
    '9 0 0 0 0 1 1 2 2 2 2 0 0 0 0 1 0 1 0 0 1 1 1 0 2 1 1 0 2 1 2 ',
    '9 0 0 1 0 1 1 3 2 4 2 1 0 0 1 0 1 0 1 0 0 1 1 -1 0 0 -1 0 1 0 1 1 1 2 1 3 1 0 2 1 2 ',
    '10 0 1 0 1 1 0 0 0 0 0 1 1 0 0 0 1 0 \n', '10 1 1 0 1 1 0 0 0 0 0 1 1 0 0 0 1 0 ',
    '10 0 1 0 1 44 1 2 3 ', '10 0 7 0 1 2 0 0 0 0 0 1 1 0 0 0 1 0 1 ',
    '11 0.5 1 0 0 0 0 0 1 1 0 0 0 1 0 \n', '11 -0.5 4 0 0 0 0 0 1 1 0 0 0 1 0 1 ',
    '11 0.5 42 ', '11 0.5 11 0.25 1 0 0 0 0 0 1 1 0 0 0 1 0 ',
    '12 1 2 3 ', '44 1 2 3 ', '0 1 ', '99999999999 1 ', 'abc 1 ',
]

GET_REAL_TEXTS = [
    (6, ' 12.5  -3e2\tabc\n1e400 0x10 inf'),
    (8, '-nan NAN -Infinity 4.9e-324 1.7976931348623157e308 1e-5 .5 -.e1'),
    (5, '99999999999999999999999999999999999 1,5 +0 -0 00000000000000000000000000000000001'),
    (4, '0.1234567890123456789012345678901234567890123456789012345678901234'),
    (3, '1e'),
    (2, ''),
    (3, '   '),
    (4, '1 2'),
    (2, '\x01\x02 7 \xff9'),
]


def read_requests(requests: list[str]) -> None:
    for kind, texts in enumerate([CURVE_TEXTS, CURVE2D_TEXTS, SURFACE_TEXTS]):
        for text in texts:
            requests.append(req('gt_read', [kind, *text_bytes(text + TAIL)]))
    for count, text in GET_REAL_TEXTS:
        requests.append(req('gt_get_real', [count, *text_bytes(text)]))


def set_specs(specs: list[list], duplicates: list[int]) -> list:
    """`n (dup | 0 spec)...`: `duplicates` lists the positions that re-add the previous one."""
    values: list = [len(specs) + len(duplicates)]
    for index, spec in enumerate(specs):
        values += [0, *spec]
        if index in duplicates:
            values.append(1)
    return values


def set_requests(requests: list[str]) -> None:
    requests.append(req('gt_set', [0, *set_specs(CURVES[:-1], [0, 5, 20])]))
    requests.append(req('gt_set', [1, *set_specs(CURVES2D[:-1], [3])]))
    requests.append(req('gt_set', [2, *set_specs(SURFACES[:-1], [0, 9])]))
    requests.append(req('gt_set', [0, 0]))
    requests.append(req('gt_set_null', [0]))
    tables = [
        (0, 'Curves 2\n1 0 0 0 0 0 1 \n2 0 0 0 0 0 1 1 0 0 0 1 0 2.5\n'),
        (0, 'Curvez 1\n1 0 0 0 0 0 1 \n'), (0, 'Curves 0\n'), (0, 'Curves -2\n1 0 0 0 0 0 1 \n'),
        (0, 'Curves 3\n42 1 2\n1 0 0 0 0 0 1 \n2 0 0 0 0 0 1 1 0 0 0 1 0 -1\n'),
        (0, 'Curves 2\n2 0 0 0 0 0 0 1 0 0 0 1 0 1\n1 0 0 0 0 0 1\n'),
        (0, 'curves 1\n1 0 0 0 0 0 1 \n'), (0, '3D 1\n'), (0, 'Surfaces 1\n1 0 0 0 0 0 1 1 0 0 0 1 0\n'),
        (0, 'Curves' + 'x' * 300 + ' 1\n1 0 0 0 0 0 1 \n'),
        (1, 'Curve2ds 2\n1 0 0 1 0 \n2 0 0 1 0 0 1 2.5\n'), (1, 'Curves 1\n1 0 0 1 0\n'),
        (1, 'Curve2ds 2\n42\n42\n'),
        (2, 'Surfaces 2\n1 0 0 0 0 0 1 1 0 0 0 1 0 \n2 0 0 0 0 0 1 1 0 0 0 1 0 2\n'),
        (2, 'Not a surface table\n'), (2, 'Surfaces 2\n44\n0\n'),
    ]
    for kind, table in tables:
        requests.append(req('gt_set_read', [kind, *text_bytes(table + TAIL)]))


def progress_requests(requests: list[str]) -> None:
    lines = [[1, 0.0, 0.0, float(i), *Z] for i in range(3)]
    lines2d = [[1, float(i), 0.0, 1.0, 0.0] for i in range(3)]
    planes = [[1, *ax3([0.0, 0.0, float(i)])] for i in range(3)]
    for limit in [0, 1, 2, 3, 10]:
        for kind, specs in enumerate([lines, lines2d, planes]):
            requests.append(req('gt_progress', [kind, limit, 3, *[v for s in specs for v in s]]))


def dump_requests(requests: list[str]) -> None:
    for spec in [CURVES[1], CIRCLE, BSPLINE, HELIX, [8, 0.0, 1.0, 1, *HELIX]]:
        requests.append(req('gt_debug_dump', [0, *spec]))
    for spec in [CIRCLE2D, BSPLINE2D, CURVES2D[-1]]:
        requests.append(req('gt_debug_dump', [1, *spec]))
    for spec in [PLANE, BSPLINE_SURFACE, SURFACES[-1], [10, 0.0, 1.0, 0.0, 1.0, *SURFACES[-1]]]:
        requests.append(req('gt_debug_dump', [2, *spec]))
    requests.append(req('gt_debug_dump', [3, 1.0, 2.0, 3.0]))
    requests.append(req('gt_debug_dump', [4]))


def handler_requests(requests: list[str]) -> None:
    cases = [
        (0, HELIX, '42 2 0.5 '), (0, [8, 0.0, 1.0, 1, *HELIX], '8 0 2 42 1 0.25 '),
        (0, [9, 0.5, *Z, *HELIX], '42 -1 1 '), (0, CIRCLE, '43 1 2 3 '),
        (1, CURVES2D[-1], '43 2 3 0.5 '), (1, [8, 0.0, 1.0, 1, *CURVES2D[-1]], '43 0 1 0 '),
        (2, SURFACES[-1], '44 3 2 1 '), (2, [10, 0.0, 1.0, 0.0, 1.0, *SURFACES[-1]], '10 0 1 0 1 44 1 1 1 '),
        (2, [11, 0.5, *SURFACES[-1]], '44 1 1 -1 '),
    ]
    for kind, spec, text in cases:
        requests.append(req('gt_handler', [kind, *spec, *text_bytes(text + TAIL)]))


def append_geomtools_requests(requests: list[str]) -> None:
    """Append every GeomTools request family in a fixed order."""
    write_requests(requests)
    read_requests(requests)
    set_requests(requests)
    progress_requests(requests)
    dump_requests(requests)
    handler_requests(requests)
