"""Deterministic requests for the HLRAlgo value classes (tools/oracle/tkhlr_probe_hlralgo.cxx).

- `ha_minmax`: InitMinMax, UpdateMinMax over points (none, grid, huge, signed zero) and
  EnlargeMinMax;
- `ha_encode`: EncodeMinMax, DecodeMinMax, SizeBox, Minimize, Maximize, AddMinMax and
  CopyMinMax on indices within and beyond the 15-bit fields;
- `ha_bipoint`: the six constructors and the default one, then the flag setters;
- `ha_edges` and `ha_wires`: the edge and wire blocks, with out-of-range ranks
  (Standard_OutOfRange); every read follows the setters of its rank;
- `ha_status`: EdgeStatus sequences (hides, on-face hides, hide all, show all, the flag
  setters, re-initialization) with the visible and hidden walks of EdgeIterator;
- `ha_records`: Intersection, Coincidence and Interference;
- `ha_projector`: the five constructors on the standard views (top, front, both
  axonometric types) and on skew, scaled and translated frames, Set, Scaled, and every
  projection, Shoot and Transform, including a null perspective sight
  (Standard_ConstructionError) and a zero scale.
"""
from __future__ import annotations
import random

R2 = 0.7071067811865476


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def request(op: str, values: list[float]) -> str:
    """One request line."""
    return op + ' ' + numbers(values)


def minmax_requests(rng: random.Random) -> list[str]:
    """ha_minmax over empty, grid, random and extreme point sets."""
    sets = [[], [(0.0, 0.0, 0.0)], [(-0.0, 1.0, -0.0), (2.0, -3.0, 4.0)],
            [(1e300, -1e300, 5.0), (1e-300, 3.0, -7.0)]]
    for _ in range(12):
        sets.append([tuple(rng.uniform(-10, 10) for _ in range(3)) for _ in range(rng.randint(1, 6))])
    out = []
    for i, points in enumerate(sets):
        big, tol = [(1e100, 0.0), (2e100, 1e-3), (0.0, -1.0), (1e100, 0.5)][i % 4]
        out.append(request('ha_minmax', [big, tol, len(points)] + [c for p in points for c in p]))
    return out


def minmax_values(rng: random.Random, span: int) -> list[int]:
    """Sixteen indices in [-span, span]."""
    return [rng.randint(-span, span) for _ in range(16)]


def encode_requests(rng: random.Random) -> list[str]:
    """ha_encode on small, 15-bit, and wider indices."""
    out = []
    for span in (10, 0x7fff, 0x10000, 1 << 20):
        for _ in range(6):
            values = minmax_values(rng, span) + minmax_values(rng, span) + minmax_values(rng, span)
            out.append(request('ha_encode', values))
    out.append(request('ha_encode', [0] * 48))
    return out


def bipoint_requests(rng: random.Random) -> list[str]:
    """ha_bipoint: every constructor with random fields and setter sequences."""
    out = []
    for variant in range(7):
        for _ in range(5):
            coords = [rng.uniform(-5, 5) for _ in range(12)]
            faces = [rng.randint(-3, 9) for _ in range(7)]
            bits = rng.randint(0, 31)
            setters = [(rng.randint(0, 4), rng.randint(0, 1)) for _ in range(rng.randint(0, 5))]
            out.append(request('ha_bipoint', [variant] + coords + faces + [bits, len(setters)]
                               + [v for s in setters for v in s]))
    return out


def edge_ops(rng: random.Random, n: int) -> list[list[float]]:
    """Setter and read sequences over ranks 1..n, then out-of-range ranks."""
    ops: list[list[float]] = []
    for i in range(1, n + 1):
        ops += [[0, i, rng.randint(-5, 50)], [1, i, rng.randint(0, 3)]]
        ops += [[k, i, rng.randint(0, 1)] for k in (2, 3, 4, 5)]
        ops.append([6, i])
        ops += [[rng.choice((1, 2, 3, 4, 5)), i, rng.randint(0, 3)], [6, i]]
    ops += [[6, 0], [6, n + 1], [0, 0, 1], [1, n + 1, 2], [5, -1, 1]]
    ops.append([7] + [rng.randint(-100, 100) for _ in range(16)])
    return ops


def edges_requests(rng: random.Random) -> list[str]:
    """ha_edges on blocks of 0 to 4 edges."""
    out = []
    for n in (0, 1, 2, 4):
        ops = edge_ops(rng, n)
        out.append(request('ha_edges', [n, len(ops)] + [v for op in ops for v in op]))
    return out


def wires_requests(rng: random.Random) -> list[str]:
    """ha_wires: null wires, set wires, out-of-range ranks and the box."""
    out = []
    for n in (0, 1, 3):
        ops: list[list[float]] = [[1, i] for i in range(1, n + 1)]
        ops += [[0, i, rng.randint(0, 3)] for i in range(1, n + 1)]
        ops += [[1, i] for i in range(0, n + 2)]
        ops += [[0, n + 1, 2], [2] + [rng.randint(-9, 9) for _ in range(16)]]
        out.append(request('ha_wires', [n, len(ops)] + [v for op in ops for v in op]))
    return out


def status_op(rng: random.Random) -> list[float]:
    """One random EdgeStatus operation (hides dominate)."""
    kind = rng.choice((1, 1, 1, 1, 0, 2, 3, 4, 5))
    if kind <= 1:
        s = rng.choice((rng.uniform(-0.2, 1.2), 0.0, 0.5, 1.0, 0.25))
        e = s + rng.choice((rng.uniform(0.0, 0.6), 0.0, 0.25, -0.1))
        tols = [rng.choice((0.0, 1e-6, 1e-3, 0.05)) for _ in range(2)]
        op = [kind, s, tols[0], e, tols[1]]
        return op + [rng.randint(0, 1) if rng.random() < 0.2 else 0, rng.randint(0, 1)] if kind == 1 else op
    if kind >= 4:
        return [kind, rng.randint(0, 1)]
    return [kind]


def status_requests(rng: random.Random) -> list[str]:
    """ha_status: fixed and random operation sequences."""
    fixed = [[0, 0], [1, 0.0, 1e-6, 1.0, 1e-6, 0],
             [1, 0.0, 1e-6, 1.0, 1e-6, 2, 1, 0.25, 1e-6, 0.5, 1e-6, 0, 0, 1, 0.75, 1e-6, 0.8, 1e-6, 0, 0],
             [1, 0.0, 1e-6, 1.0, 1e-6, 1, 1, -1.0, 0.0, 2.0, 0.0, 0, 0],
             [1, 0.0, 1e-6, 1.0, 1e-6, 1, 1, 0.2, 0.0, 0.4, 0.0, 1, 0],
             [0, 3, 5, 0, 4, 0, 1, 0.0, 0.0, 1.0, 0.0, 0, 1],
             [1, 0.0, 0.01, 1.0, 0.01, 2, 1, 0.0, 0.0, 0.5, 0.0, 0, 0, 5, 1],
             [1, 0.0, 1e-6, 1.0, 1e-6, 3, 1, 0.0, 0.0, 0.5, 0.0, 0, 0, 4, 1, 5, 0]]
    out = [request('ha_status', values) for values in fixed]
    for _ in range(120):
        head = [1, 0.0, rng.choice((1e-6, 1e-3)), 1.0, rng.choice((1e-6, 1e-3))] if rng.random() < 0.9 else [0]
        ops = [status_op(rng) for _ in range(rng.randint(1, 6))]
        out.append(request('ha_status', head + [len(ops)] + [v for op in ops for v in op]))
    return out


def records_requests(rng: random.Random) -> list[str]:
    """ha_records with every orientation and state."""
    out = []
    for i in range(24):
        out.append(request('ha_records', [i % 4, rng.randint(-3, 9), rng.randint(0, 9), rng.randint(0, 9),
                                          rng.uniform(-2, 2), rng.choice((0.0, 1e-3, 0.5)), (i // 4) % 4,
                                          rng.randint(0, 9), rng.uniform(0, 1), rng.randint(0, 3),
                                          rng.randint(0, 3), rng.randint(0, 3), rng.randint(0, 3),
                                          rng.randint(0, 3), rng.uniform(-1, 1)]))
    return out


FRAMES = [((0, 0, 0), (0, 0, 1), (1, 0, 0)),            # top view (type 1)
          ((0, 0, 0), (0, -1, 0), (1, 0, 0)),           # front view (type 2)
          ((0, 0, 0), (0.5, -0.5, R2), (R2, R2, 0)),    # axonometric (type 3)
          ((0, 0, 0), (0, 1, 1), (R2, -0.5, 0.5)),      # axonometric of type 0
          ((1, 2, 3), (0.3, -0.4, 0.5), (0.7, 0.1, -0.2)),
          ((-5, 0, 2), (1, 1, 1), (1, -1, 0))]


def trsf_fields(rng: random.Random, frame: tuple, scale: float) -> list[float]:
    """`P N X k V` for hl_read_trsf."""
    p, n, x = frame
    return [*p, *n, *x, scale, *(rng.uniform(-3, 3) for _ in range(3))]


def projector_points(rng: random.Random, count: int) -> list[float]:
    """`(P D x y)*count`, with a sight at the origin among them."""
    values: list[float] = [count]
    for i in range(count):
        xy = (0.0, 0.0) if i == 0 else (rng.uniform(-4, 4), rng.uniform(-4, 4))
        values += [rng.uniform(-5, 5) for _ in range(6)] + list(xy)
    return values


def projector_head(rng: random.Random, kind: int, frame: tuple) -> list[float]:
    """The constructor fields of one kind."""
    p, n, x = frame
    focus = rng.choice((10.0, -5.0, 100.0))
    if kind == 1:
        return [1, *p, *n, *x]
    if kind == 2:
        return [2, *p, *n, *x, focus]
    persp = rng.randint(0, 1)
    head = [kind] + trsf_fields(rng, frame, rng.choice((1.0, 2.5, -1.0))) + [persp, focus]
    if kind == 4:
        head += [rng.uniform(-1, 1) for _ in range(6)]
    return head


def projector_requests(rng: random.Random) -> list[str]:
    """ha_projector over the constructors, frames, Set and Scaled."""
    out = [request('ha_projector', [0, 0, -1] + projector_points(rng, 3))]
    for frame in FRAMES:
        for kind in (1, 2, 3, 4):
            scaled = rng.choice((-1, 0, 1))
            tail = [0, scaled] + projector_points(rng, 3)
            out.append(request('ha_projector', projector_head(rng, kind, frame) + tail))
        setter = [1] + trsf_fields(rng, frame, 2.0) + [rng.randint(0, 1), 7.0, rng.choice((-1, 0, 1))]
        out.append(request('ha_projector', [0] + setter + projector_points(rng, 2)))
    out.append(request('ha_projector', [2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0.0, 0, -1] + projector_points(rng, 2)))
    out.append(request('ha_projector', [3] + trsf_fields(rng, FRAMES[0], 0.0) + [0, 1.0, 0, -1, 0]))
    out.append(request('ha_projector', [1, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, -1, 0]))
    return out


def append_hlralgo_requests(requests: list[str]) -> None:
    """Append the HLRAlgo value-class requests in a fixed order."""
    rng = random.Random(20261005)
    requests.extend(minmax_requests(rng))
    requests.extend(encode_requests(rng))
    requests.extend(bipoint_requests(rng))
    requests.extend(edges_requests(rng))
    requests.extend(wires_requests(rng))
    requests.extend(status_requests(rng))
    requests.extend(records_requests(rng))
    requests.extend(projector_requests(rng))
