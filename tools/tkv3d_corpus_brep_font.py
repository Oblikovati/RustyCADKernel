"""BRepFont: regularization statuses and pools, curve evaluation, glyph and text topology.

Outlines are written as `UnitsPerEm Rule Overlaps SingleStroke NbContours {NbSegments
Segment...} NbOrphans Segment...`, each segment as `Kind X0 Y0 X1 Y1 X2 Y2 X3 Y3`
(0 line, 1 quadratic, 2 cubic; unused controls repeat a neighbouring point).
"""
from __future__ import annotations
from itertools import product

Point = tuple[float, float]


def line(a: Point, b: Point) -> str:
    return f'0 {a[0]} {a[1]} {a[0]} {a[1]} {b[0]} {b[1]} {b[0]} {b[1]}'


def quad(a: Point, c: Point, b: Point) -> str:
    return f'1 {a[0]} {a[1]} {c[0]} {c[1]} {c[0]} {c[1]} {b[0]} {b[1]}'


def cubic(a: Point, c1: Point, c2: Point, b: Point) -> str:
    return f'2 {a[0]} {a[1]} {c1[0]} {c1[1]} {c2[0]} {c2[1]} {b[0]} {b[1]}'


def polygon(points: list[Point]) -> list[str]:
    return [line(points[i], points[(i + 1) % len(points)]) for i in range(len(points))]


def outline(contours: list[list[str]], rule: int = 0, overlaps: int = 0, stroke: int = 0,
            orphans: list[str] = (), units: float = 1000.0) -> str:
    body = ' '.join(f'{len(c)} ' + ' '.join(c) for c in contours)
    tail = ' '.join(orphans)
    return f'{units} {rule} {overlaps} {stroke} {len(contours)} {body} {len(orphans)} {tail}'.strip()


SQUARE = [(0.0, 0.0), (20.0, 0.0), (20.0, 20.0), (0.0, 20.0)]
INNER_CW = [(5.0, 5.0), (5.0, 15.0), (15.0, 15.0), (15.0, 5.0)]
INNER_CCW = [(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0)]
ISLAND = [(8.0, 8.0), (12.0, 8.0), (12.0, 12.0), (8.0, 12.0)]
BOWTIE = [(0.0, 0.0), (10.0, 10.0), (2.0, 8.0), (10.0, 0.0)]
THIN = [(0.0, 0.0), (1.0e12, 0.0), (1.0e12, 0.1), (0.1, 0.1), (0.1, 0.9), (1.0e12, 0.9), (1.0e12, 1.0), (0.0, 1.0)]
BIG = 1.7976931348623157e308
CONCAVE = [(0.0, 0.0), (10.0, 0.0), (10.0, 3.0), (3.0, 3.0), (3.0, 7.0), (10.0, 7.0), (10.0, 10.0), (0.0, 10.0)]


def round_glyph() -> list[str]:
    """An 'O' of quadratic arcs with a cubic counter-clockwise hole."""
    outer = [quad((10.0, 0.0), (20.0, 0.0), (20.0, 10.0)), quad((20.0, 10.0), (20.0, 20.0), (10.0, 20.0)),
             quad((10.0, 20.0), (0.0, 20.0), (0.0, 10.0)), quad((0.0, 10.0), (0.0, 0.0), (10.0, 0.0))]
    return outer


def round_hole() -> list[str]:
    return [cubic((10.0, 5.0), (7.0, 5.0), (5.0, 7.0), (5.0, 10.0)), cubic((5.0, 10.0), (5.0, 13.0), (7.0, 15.0), (10.0, 15.0)),
            cubic((10.0, 15.0), (13.0, 15.0), (15.0, 13.0), (15.0, 10.0)), cubic((15.0, 10.0), (15.0, 7.0), (13.0, 5.0), (10.0, 5.0))]


def outlines() -> list[str]:
    """Every regularizer status and topology branch of the port."""
    healed = [line((0.0, 0.0), (10.0, 0.0)), line((10.0, 0.0), (10.0005, 0.0)), line((10.0005, 0.0), (10.0, 10.0)),
              line((10.0, 10.0), (0.0, 10.0)), line((0.0, 10.0), (0.0, -0.0005)), line((0.0, -0.0005), (0.0, 0.0))]
    return [
        outline([polygon(SQUARE)]),
        outline([polygon(SQUARE), polygon(INNER_CW)]),
        outline([polygon(SQUARE), polygon(INNER_CCW)]),
        outline([polygon(SQUARE), polygon(INNER_CCW)], rule=1),
        outline([polygon([(0.0, 0.0), (30.0, 0.0), (30.0, 30.0), (0.0, 30.0)]), polygon(INNER_CW), polygon(ISLAND)]),
        outline([polygon(BOWTIE)]),
        outline([polygon(BOWTIE)], overlaps=1),
        outline([polygon([(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]),
                 polygon([(8.0, 2.0), (12.0, 2.0), (12.0, 8.0), (8.0, 8.0)])], overlaps=1),
        outline([polygon([(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]),
                 polygon([(10.0, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)])]),
        outline([[line((0.0, 0.0), (10.0, 0.0)), quad((10.0, 0.0), (15.0, 5.0), (20.0, 0.0))]], stroke=1),
        outline([[line((0.0, 0.0), (10.0, 0.0)), line((11.0, 0.0), (0.0, 0.0))]]),
        outline([healed]),
        outline([[quad((0.0, 0.0), (0.0004, 0.0003), (0.0002, 0.0001))]]),
        outline([[line((0.0, 0.0), (0.0, 1.0)), line((0.0, 1.0), (0.0, 0.0))]]),
        outline([polygon([(BIG, BIG), (-BIG, BIG), (-BIG, -BIG), (BIG, -BIG)])]),
        outline([polygon(SQUARE)], orphans=[line((20.0, 0.0), (21.0, 0.0))]),
        outline([], orphans=[line((0.0, 0.0), (1.0, 0.0))]),
        outline([]),
        outline([polygon(SQUARE)], units=0.0),
        outline([[quad((0.0, 0.0), (10.0, 0.0), (0.0, 0.0)), line((0.0, 0.0), (0.0, 10.0)), line((0.0, 10.0), (-10.0, 10.0)),
                  line((-10.0, 10.0), (-10.0, 0.0)), line((-10.0, 0.0), (0.0, 0.0))]]),
        outline([[quad((0.0, 0.0), (5.0, 10.0), (10.0, 0.0)), line((10.0, 0.0), (0.0, 8.0)), line((0.0, 8.0), (0.0, 0.0))]]),
        outline([[quad((0.0, 0.0), (5.0, 10.0), (10.0, 0.0)), line((10.0, 0.0), (0.0, 0.0))],
                 polygon([(4.0, 5.0), (6.0, 5.0), (6.0, 7.0), (4.0, 7.0)])]),
        outline([[cubic(p, q, p, q) for p, q in zip(CONCAVE, CONCAVE[1:] + CONCAVE[:1])]]),
        outline([round_glyph(), round_hole()]),
        outline([round_glyph(), round_hole()], rule=1),
        outline([[cubic((0.0, 0.0), (10.0, 10.0), (0.0, 10.0), (10.0, 0.0)), line((10.0, 0.0), (0.0, 0.0))]]),
        outline([[cubic((0.0, 0.0), (20.0, 10.0), (-10.0, 10.0), (10.0, 0.0)), line((10.0, 0.0), (0.0, 0.0))]], overlaps=1),
        outline([polygon(THIN)]),
    ]


def curve_requests() -> list[str]:
    segments = [line((1.0, 2.0), (-3.0, 5.0)), quad((0.0, 0.0), (1.0, 2.0), (2.0, 0.0)),
                quad((BIG, 0.0), (-BIG, 1.0), (BIG, 0.0)), cubic((0.0, 2.0), (4.0, 2.0), (4.0, 2.0), (0.0, 2.0)),
                cubic((0.0, 0.0), (1.0, 3.0), (2.0, -3.0), (3.0, 0.0)), cubic((0.1, 0.2), (0.3, 0.7), (1.9, 0.4), (2.3, 1.1))]
    params = '5 0 0.25 0.5 0.75 1'
    return [f'v3_brepfont_curve {segment} {params}' for segment in segments]


def append_brep_font_requests(requests: list[str]) -> None:
    """Regularize each outline at three tolerances; build glyphs and texts of the valid ones."""
    for shape, tolerance in product(outlines(), (1.0e-7, 0.001, 0.01)):
        requests.append(f'v3_brepfont_regularize {shape} {tolerance}')
    requests += curve_requests()
    glyph_options = ('10 1e-07 0', '10 1e-07 1', '1000 1e-07 0', '1e308 1e-07 0', '1e100 1e-07 0', '10 0 0')
    for shape, options in product(outlines()[:12] + outlines()[23:25], glyph_options):
        requests.append(f'v3_brepfont_glyph {shape} 1e-07 {options}')
    pens = ('0 0 0 0 0 1 1 0 0', '3 4 5 0 0 1 1 0 0', '1 2 3 0 1 0 1 0 0', '0 0 0 0 0 1 0 0 1', '1e100 0 0 0 0 1 1 0 0')
    items = ('2 0 0 0 0 20 0', '3 0 0 0 1 5 5 0 -7.5 2', '1 1 0 0', '0', '1 2 0 0')
    for shape, item, pen, concatenate in product(outlines()[:2] + outlines()[9:10] + outlines()[23:24], items, pens, (0, 1)):
        requests.append(f'v3_brepfont_text {shape} 1e-07 12 1e-07 {item} {pen} {concatenate}')
