"""Deterministic BinTools request corpus for the TKBRep oracle (`br_bin_*`).

The requests drive tools/oracle/tkbrep_probe_bintools*.cxx and their Rust twins in
crates/tkbrep/examples/br_probe/bintools*.rs: the scalar encodings, the
BinTools_OStream/IStream primitives (every reference width), every Geom, Geom2d curve
and Geom surface class through the three geometry sets, BRep scenes (the scenarios of
tools/tkbrep_brep_corpus.py and dedicated mesh, point and location scenes) in every
format version with and without triangles and normals, and truncated shape sets. The
hand-built malformed byte strings are in tools/tkbrep_bintools_bytes_corpus.py.
"""
from __future__ import annotations
from tkbrep_bintools_bytes_corpus import append_tkbrep_bintools_bytes_requests
from tkbrep_bintools_scene_corpus import append_tkbrep_bintools_scene_requests
from tkbrep_brep_corpus import text

X, Y, Z = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
O = (0.0, 0.0, 0.0)


def scalar_requests() -> list[str]:
    values = [1.5, -2.0, 0.0, 65535.0, -1.0, 1e-300, 7.25, 123456.0, 2147483647.0,
              -2147483648.0, 0.1, -0.0, 1e-45, -0.75]
    return [f'br_bin_scalars {len(values)} {text(values)}',
            'br_bin_scalars 0',
            'br_bin_scalars 1 2.5']


def ostream_requests() -> list[str]:
    ops = [[4, 1.5], [6, -7], [5, 1], [3, 250], [7, 65535], [2, 199], [1, 6, 1], [1, 7, 3],
           [8, 1.0, 2.0, 3.0], [9, 0.0, 0.0, 2.0], [10, 0.5, -0.5], [11, 3.0, 4.0],
           [12, 1.0, 2.0, 3.0, 0.0, 0.0, 1.0, 0.5, 2.0], [12, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.5],
           [13, 1, 2, 3], [14, 0.1, 0.2, 0.3], [15, 1, 0, 1], [16, 1, 1, 0, 1, 0, 0, 1],
           [0, 0.0], [0, 60.0], [9, 0.0, 0.0, 0.0], [12, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
           [0, 1e19]]
    flat = [v for op in ops for v in op]
    requests = [f'br_bin_ostream 0 {len(ops)} {text(flat)}']
    # 8, 16, 32 and 64-bit references: the delta from the position after the prefix.
    for prefix, target in [(10, 0.0), (300, 0.0), (65535, 0.0), (65536, 0.0), (70000, 5.0), (3, 1e19)]:
        requests.append(f'br_bin_ostream {prefix} 3 0 {target!r} 17 2 0 {target!r}')
    return requests


def le(value: int, size: int) -> list[int]:
    return list((value % (1 << (8 * size))).to_bytes(size, 'little'))


def byte_list(values: list[int]) -> str:
    return f'{len(values)} {" ".join(map(str, values))}'


def istream_requests() -> list[str]:
    import struct
    reals = list(struct.pack('<ddd', 1.5, -2.0, 0.25))
    requests = []
    # Every reference width, a zero delta and a reference past the start.
    cases = [([1, 5], [0, 2, 3, 16]), ([2, *le(300, 2)], [0, 3]), ([3, *le(70000, 4)], [0, 3]),
             ([4, *le(1 << 40, 8)], [0, 3]), ([1, 0], [0, 3]), ([2, 0, 0], [0, 3]),
             ([4, 0, 0, 0, 0, 0, 0, 0, 0], [0, 3]), ([1], [0, 3, 15]), ([3, 1], [0, 3, 15])]
    for data, ops in cases:
        requests.append(f'br_bin_istream 0 {byte_list(data)} {len(ops)} {" ".join(map(str, ops))}')
    requests.append(f'br_bin_istream 100000 {byte_list([3, *le(99990, 4)])} 3 4 100000 0 3')
    # Shape codes, primitive reads, positions and the stream state.
    for code in (200, 203, 228, 231, 232, 255, 5, 0):
        requests.append(f'br_bin_istream 0 {byte_list([code])} 4 0 1 2 16')
    data = reals + list(struct.pack('<i', -9)) + [7, 0x5A] + list(struct.pack('<f', 0.5))
    requests.append(f'br_bin_istream 0 {byte_list(data)} 11 5 14 7 4 8 6 8 9 13 10 15 14')
    requests.append(f'br_bin_istream 0 {byte_list(reals)} 6 7 5 15 4 3 5 15')
    trsf = list(struct.pack('<12d', 1, 0, 0, 1, 0, 1, 0, 2, 0, 0, 1, 3))
    singular = list(struct.pack('<12d', *([0.0] * 12)))
    requests.append(f'br_bin_istream 0 {byte_list(trsf)} 2 11 15')
    requests.append(f'br_bin_istream 0 {byte_list(singular)} 2 11 15')
    requests.append(f'br_bin_istream 0 {byte_list(trsf[:40])} 2 11 15')
    requests.append(f'br_bin_istream 0 {byte_list([0x7F, 0x55])} 3 12 13 14')
    requests.append(f'br_bin_istream 0 {byte_list(reals)} 4 4 1e19 15 4 8 15')
    return requests


def geometry_specs() -> list[list[float]]:
    """Every curve, surface and 2D curve class, with failing constructions."""
    p = [(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (3.0, 1.0, 1.0), (4.0, 0.0, 0.0)]
    curves = [
        [0, *O, *X], [1, 1.0, 2.0, 3.0, *Z, *X, 2.5], [2, *O, *Z, *X, 3.0, 1.0],
        [3, *O, *Y, *Z, 0.5], [4, *O, *X, *Y, 2.0, 1.0],
        [5, 0, 3, *p[0], *p[1], *p[2]], [5, 1, 3, *p[0], 1.0, *p[1], 2.0, *p[2], 0.5],
        [6, 0, 0, 2, 4, *p[0], *p[1], *p[2], *p[3], 3, 0.0, 3, 0.5, 1, 1.0, 3],
        [6, 1, 1, 2, 4, *p[0], 1.0, *p[1], 2.0, *p[2], 1.0, *p[3], 0.5, 5,
         0.0, 1, 1.0, 1, 2.0, 1, 3.0, 1, 4.0, 1],
        [7, 1, 0.5, 2.0], [7, 7, 0.2, 0.8], [7, 9, 0.6, 0.7],
        [8, 1, 0.25, *Z], [8, 0, 1.0, *Z],
        [1, *O, *Z, *X, -1.0], [0, *O, *O],
    ]
    q = [(0.0, 0.0, 0.0), (1.0, 0.0, 0.5), (0.0, 1.0, 0.0), (1.0, 1.0, 1.0), (0.0, 2.0, 0.0), (1.0, 2.0, 0.0)]
    surfaces = [
        [10, *O, *Z, *X, 0], [10, 1.0, 1.0, 1.0, *Z, *X, 1], [11, *O, *Z, *X, 0, 2.0],
        [12, *O, *Z, *X, 1, 1.5, 0.3], [13, 1.0, 1.0, 1.0, *Z, *X, 0, 4.0],
        [14, *O, *Z, *X, 0, 5.0, 1.0], [15, 5, *Z], [16, 5, *O, *Z],
        [17, 0, 2, 2, *q[0], *q[1], *q[2], *q[3]],
        [17, 1, 2, 3, *q[0], 1.0, *q[1], 2.0, *q[2], 1.0, *q[3], 0.5, *q[4], 1.0, *q[5], 1.0],
        [18, 0, 0, 0, 1, 1, 2, 2, *q[0], *q[1], *q[2], *q[3], 2, 0.0, 2, 1.0, 2, 2, 0.0, 2, 1.0, 2],
        [18, 1, 0, 0, 1, 1, 2, 3, *q[0], 1.0, *q[1], 1.0, *q[2], 2.0, *q[3], 1.0, *q[4], 1.0, *q[5], 3.0,
         2, 0.0, 2, 1.0, 2, 2, 0.0, 3, 1.0, 3],
        [19, 0, -1.0, 1.0, -2.0, 2.0], [20, 2, 0.5], [19, 12, -0.5, 0.5, -1.0, 1.0],
        [11, *O, *Z, *X, 0, -2.0],
    ]
    pcurves = [
        [30, 0.0, 0.0, 1.0, 0.0], [31, 1.0, 1.0, 1.0, 0.0, 1, 2.0], [31, 1.0, 1.0, 0.0, 1.0, 0, 2.0],
        [32, 0.0, 0.0, 1.0, 0.0, 1, 3.0, 1.0], [33, 0.0, 0.0, 1.0, 0.0, 1, 0.5],
        [34, 0.0, 0.0, 1.0, 0.0, 0, 2.0, 1.0], [35, 0, 3, 0.0, 0.0, 1.0, 1.0, 2.0, 0.0],
        [35, 1, 3, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 0.0, 1.0],
        [36, 0, 0, 1, 2, 0.0, 0.0, 1.0, 1.0, 2, 0.0, 2, 1.0, 2],
        [37, 1, 0.0, 1.5], [38, 1, 0.5], [37, 9, 0.2, 0.4], [31, 0.0, 0.0, 1.0, 0.0, 1, -1.0],
    ]
    return curves + surfaces + pcurves


def geometry_requests() -> list[str]:
    specs = geometry_specs()
    flat = [v for spec in specs for v in spec]
    requests = [f'br_bin_geometry {len(specs)} {text(flat)}']
    # Each class alone, so that each set holds one member.
    for spec in specs:
        if spec[0] not in (7, 8, 15, 16, 19, 20, 37, 38):
            requests.append(f'br_bin_geometry 1 {text(spec)}')
    requests.append('br_bin_geometry 0')
    return requests


def append_tkbrep_bintools_requests(requests: list[str]) -> None:
    """Append every BinTools request."""
    requests += scalar_requests()
    requests += ostream_requests()
    requests += istream_requests()
    requests += geometry_requests()
    append_tkbrep_bintools_scene_requests(requests)
    append_tkbrep_bintools_bytes_requests(requests)
