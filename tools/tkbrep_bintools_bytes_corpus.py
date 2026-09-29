"""Hand-built BinTools byte strings for the TKBRep oracle (`br_bin_read`).

Each request reads a given byte string with one reader of
tools/oracle/tkbrep_probe_bintools_shape.cxx: valid minimal files of every version,
missing or wrong tags, unknown type codes, out-of-range indices, singular matrices,
truncations and back references (known, unread and invalid). A byte string never ends
where the shape-reference reader reads a byte without a check (the native value would
be uninitialised), nor between a table tag and its count.
"""
from __future__ import annotations
import struct

VERSIONS = {1: b'Open CASCADE Topology V1 (c)', 2: b'Open CASCADE Topology V2 (c)',
            3: b'Open CASCADE Topology V3 (c)', 4: b'Open CASCADE Topology V4, (c) Open Cascade'}
FLAGS = bytes([1, 0, 0, 1, 0, 0, 1])
EMPTY_GEOMETRY = b'Curve2ds 0\nCurves 0\nPolygon3D 0\nPolygonOnTriangulations 0\nSurfaces 0\nTriangulations 0\n'
IDENTITY = (1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0)


def d(*values: float) -> bytes:
    return struct.pack(f'<{len(values)}d', *values)


def i32(*values: int) -> bytes:
    return struct.pack(f'<{len(values)}i', *values)


def header(version: int, line_end: bytes = b'\n') -> bytes:
    return b'\n' + VERSIONS[version] + line_end


def table(tag: bytes, count: int, records: bytes = b'') -> bytes:
    return tag + b' %d\n' % count + records


def line3d() -> bytes:
    return bytes([1]) + d(0.0, 0.0, 0.0, 1.0, 0.0, 0.0)


def line2d() -> bytes:
    return bytes([1]) + d(0.0, 0.0, 1.0, 0.0)


def plane() -> bytes:
    return bytes([1]) + d(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0)


def vertex_record(reps: bytes = b'', subs: bytes = b'') -> bytes:
    return bytes([7]) + d(1e-7, 1.0, 2.0, 3.0) + reps + bytes([0]) + FLAGS + subs + b'*'


def reference(orientation: int, index: int, location: int) -> bytes:
    return bytes([orientation]) + i32(index, location)


def shape_set(version: int, shapes: list[bytes], geometry: bytes = EMPTY_GEOMETRY,
              locations: bytes = b'Locations 0\n', root: bytes | None = None) -> bytes:
    """A grouped file: header, locations, geometry, `TShapes`, the root reference."""
    body = header(version) + locations + geometry + b'\nTShapes %d\n' % len(shapes) + b''.join(shapes)
    return body + (root if root is not None else reference(0, 1, 0))


def request(reader: int, data: bytes, extra: str = '') -> str:
    return f'br_bin_read {reader} 0 {len(data)} {" ".join(map(str, data))}{extra}'.rstrip()


def grouped_edge(uv: bytes) -> bytes:
    """An edge with a 3D curve and a p-curve on the first surface; `uv` holds the UV
    points of versions 2 and 3."""
    return (bytes([6]) + d(1e-7) + bytes([1, 1, 0]) + bytes([1]) + i32(1, 0) + d(0.0, 1.0)
            + bytes([2]) + i32(1, 1, 0) + d(0.0, 1.0) + uv + bytes([0]) + FLAGS + b'*')


def grouped_cases() -> list[bytes]:
    """Readers 0 (BinTools::Read) and 6 (BinTools_ShapeSet::Read)."""
    vertex = vertex_record()
    curves = b'Curve2ds 1\n' + line2d() + b'Curves 1\n' + line3d() + b'Polygon3D 0\nPolygonOnTriangulations 0\nSurfaces 1\n' + plane() + b'Triangulations 0\n'
    on_curve = bytes([1]) + d(0.5) + i32(1, 0)
    on_surface = bytes([3]) + d(0.25, 0.75) + i32(1, 0)
    on_pcurve = bytes([2]) + d(0.5) + i32(1, 1, 0)
    edge = grouped_edge(b'')
    face = bytes([4, 1]) + d(1e-7) + i32(1, 0) + bytes([0]) + FLAGS + reference(0, 1, 0) + b'*'
    face_on_vertex = bytes([4, 1]) + d(1e-7) + i32(1, 0) + bytes([0]) + FLAGS + reference(0, 2, 0) + b'*'
    cases = [b'', b'garbage\nmore\n', b'x' * 150 + b'\n' + shape_set(4, [vertex]),
             header(4, b'\r\n') + shape_set(4, [vertex])[len(header(4)):],
             b'junk\n' + shape_set(2, [vertex])]
    cases += [shape_set(v, [vertex]) for v in (1, 2, 3, 4)]
    cases += [shape_set(3, [vertex_record(on_curve + on_surface + on_pcurve)], curves),
              shape_set(4, [vertex_record(on_curve)], curves),
              shape_set(3, [vertex_record(bytes([4]) + d(0.5))], curves),
              shape_set(4, [edge], curves), shape_set(2, [grouped_edge(d(0.0, 0.0, 1.0, 0.0))], curves),
              shape_set(4, [face], curves), shape_set(4, [vertex, face_on_vertex], curves)]
    matrix = bytes([1]) + d(*IDENTITY)
    cases += [shape_set(4, [vertex], locations=b'Locationz 0\n'),
              shape_set(4, [vertex], locations=table(b'Locations', 1, bytes([3]))),
              shape_set(4, [vertex], locations=table(b'Locations', 1, bytes([2]) + i32(5, 1, 0))),
              shape_set(4, [vertex], locations=table(b'Locations', 1, bytes([1]) + d(*([0.0] * 12)))),
              shape_set(4, [vertex], locations=table(b'Locations', 2, matrix + bytes([2]) + i32(1, 2, 0)),
                        root=reference(1, 1, 2)),
              shape_set(4, [vertex], root=reference(0, 1, 7)), shape_set(4, [vertex], root=reference(0, 5, 0)),
              shape_set(4, [vertex], root=b'*'), shape_set(4, [vertex], root=b'')]
    bad_geometry = [b'Curve2dz 0\n', b'Curve2ds 1\n' + bytes([12]), b'Curve2ds 0\nCurves 1\n' + bytes([0]),
                    EMPTY_GEOMETRY.replace(b'Polygon3D 0', b'xPolygon3Dy 0'),
                    EMPTY_GEOMETRY.replace(b'Polygon3D 0', b'Polygon2D 0'),
                    EMPTY_GEOMETRY.replace(b'PolygonOnTriangulations', b'PolygonOnTriangles'),
                    EMPTY_GEOMETRY.replace(b'Surfaces 0', b'Surfaces 1\n' + bytes([12])),
                    EMPTY_GEOMETRY.replace(b'Triangulations 0', b'Triangulation 0'),
                    EMPTY_GEOMETRY.replace(b'Curves 0', b'Curves 1\n' + bytes([1]) + d(0.0, 0.0))]
    cases += [header(4) + b'Locations 0\n' + g + b'\nTShapes 0\n' for g in bad_geometry]
    cases += [shape_set(4, [vertex]).replace(b'TShapes', b'TShapez'),
              shape_set(4, [bytes([9])]), shape_set(4, [bytes([8])]), header(4) + b'Locations 0\n' + EMPTY_GEOMETRY + b'\nTShapes 1\n',
              shape_set(4, [edge.replace(bytes([2]) + i32(1, 1, 0), bytes([8]) + i32(1, 1, 0))], curves),
              shape_set(4, [vertex_record(subs=reference(0, 5, 0))]),
              shape_set(4, [vertex, vertex_record(subs=reference(0, 2, 3))]),
              shape_set(4, [bytes([4, 0]) + d(1e-7) + i32(0, 0) + bytes([1]) + FLAGS + b'*']),
              shape_set(4, [bytes([4, 0]) + d(1e-7) + i32(1, 0) + bytes([2]) + i32(3) + FLAGS + b'*'], curves),
              shape_set(4, [edge.replace(bytes([1]) + i32(1, 0), bytes([1]) + i32(0, 0))], curves),
              shape_set(4, [edge.replace(bytes([1]) + i32(1, 0), bytes([1]) + i32(4, 0))], curves),
              shape_set(4, [edge.replace(bytes([2]) + i32(1, 1, 0), bytes([2]) + i32(0, 1, 0))], curves)]
    return cases


def ref_vertex(location: bytes = bytes([7]), reps: bytes = b'') -> bytes:
    return bytes([228]) + location + d(1e-7, 1.0, 2.0, 3.0) + reps + bytes([0, 75, 199])


def reference_edge(tail: bytes) -> bytes:
    """An edge with a 3D curve, a p-curve on a plane, then `tail` (more representations or
    an unknown type), the end byte, the flags and EndShape."""
    return (bytes([224, 7]) + d(1e-7) + bytes([3, 1]) + bytes([9]) + line3d() + bytes([7]) + d(0.0, 2.0)
            + bytes([2, 11]) + line2d() + bytes([13]) + plane() + bytes([7]) + d(0.0, 2.0)
            + tail + bytes([0, 75, 199]))


def reference_cases() -> list[tuple[bytes, int, int]]:
    """Reader 1 (BinTools_ShapeReader): bytes, start position and shape count."""
    trsf = d(*IDENTITY[:3], 1.0, *IDENTITY[4:7], 2.0, *IDENTITY[8:11], 3.0)
    vertex = ref_vertex()
    # The delta counts back from the reference type byte to the vertex record at 0.
    compound = bytes([200, 7, 75]) + bytes([1, len(vertex) + 3]) + bytes([7, 1, 199])
    curve_rep = bytes([1]) + d(0.5) + bytes([9]) + line3d() + bytes([7])
    pcurve_rep = bytes([2]) + d(0.5) + bytes([11]) + line2d() + bytes([14, 7])
    pcurve_surface_rep = bytes([2]) + d(0.5) + bytes([11]) + line2d() + bytes([13]) + plane() + bytes([7])
    surface_rep = bytes([3]) + d(0.25, 0.75) + bytes([13]) + plane() + bytes([7])
    edge = reference_edge(bytes([4, 2, 13]) + plane() + bytes([7, 13]) + plane() + bytes([7, 4, 1, 14, 7, 14, 7]))
    polygon = (bytes([224, 7]) + d(1e-7) + bytes([0, 5, 15]) + i32(2) + bytes([1]) + d(0.1)
               + d(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0) + bytes([7, 6, 17]) + i32(2, 1, 2) + d(0.2)
               + bytes([0, 19]) + i32(3, 1) + bytes([1, 1]) + d(0.3) + d(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0)
               + d(0.0, 0.0, 1.0, 0.0, 0.0, 1.0) + i32(1, 2, 3) + struct.pack('<9f', *([0.0, 0.0, 1.0] * 3))
               + bytes([7, 0, 75, 199]))
    return [(bytes([198]), 0, 1), (bytes([199]), 0, 1), (b'', 0, 1), (vertex, 0, 1),
            (ref_vertex(bytes([6]) + trsf), 0, 1),
            (ref_vertex(bytes([5, 6]) + trsf + i32(2) + bytes([6]) + d(*IDENTITY) + i32(-1) + bytes([8])), 0, 1),
            (ref_vertex(bytes([20])), 0, 1), (ref_vertex(bytes([5]))[:3], 0, 1),
            (ref_vertex(reps=curve_rep + pcurve_rep + pcurve_surface_rep + surface_rep), 0, 1),
            (ref_vertex(reps=bytes([4]) + d(0.5)), 0, 1),
            (vertex + compound, 0, 2), (vertex + compound, len(vertex), 1),
            (bytes([200, 7, 75, 1, 0]), 0, 1), (bytes([200, 7, 75, 3]) + struct.pack('<I', 70000), 0, 1),
            (edge, 0, 1), (reference_edge(bytes([9])), 0, 1), (polygon, 0, 1),
            (bytes([224, 7]) + d(1e-7) + bytes([0, 1, 9, 0]), 0, 1), (bytes([5]), 0, 1), (bytes([232]), 0, 1),
            (bytes([228, 7]) + d(1e-7)[:4], 0, 1), (bytes([228, 7, 75]), 0, 1)]


def set_cases() -> list[tuple[int, bytes]]:
    """Readers 2 to 5: the curve, 2D curve, surface and location sets."""
    bezier = bytes([6, 1]) + struct.pack('<H', 2) + d(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 2.0, 2.0, 0.0, 0.0, 1.0)
    bspline = (bytes([7, 0, 0]) + struct.pack('<H', 1) + i32(2, 2) + d(0.0, 0.0, 0.0, 1.0, 1.0, 0.0)
               + d(0.0) + i32(2) + d(1.0) + i32(2))
    trimmed = bytes([8]) + d(0.0, 1.0) + bytes([3]) + d(0.0) * 3 + d(0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 1.0)
    circle2d = bytes([2]) + d(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 2.0)
    cylinder = bytes([2]) + d(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 3.0)
    matrix = bytes([1]) + d(*IDENTITY)
    return [(2, table(b'Curves', 1, line3d())), (2, table(b'Curves', 3, bezier + bspline + trimmed)),
            (2, table(b'Curves', 1, bytes([10]))), (2, b'Curvez 1\n' + line3d()),
            (2, table(b'Curves', 2, line3d()[:20])), (2, table(b'Curves', 1, bytes([8]) + d(0.0, 1.0) + bytes([42]))),
            (2, table(b'Curves', 1, bytes([1]) + d(0.0, 0.0, 0.0, 0.0, 0.0, 0.0))),
            (2, b'Curves 1 ' + line3d()), (2, table(b'Curves', -1)), (2, table(b'Curves', 0)),
            (3, table(b'Curve2ds', 2, circle2d + line2d())), (3, table(b'Curve2ds', 1, bytes([0]))),
            (3, table(b'Curve2ds', 1, bytes([2]) + d(0.0, 0.0, 1.0, 0.0, 0.0, 1.0, -2.0))),
            (4, table(b'Surfaces', 2, plane() + cylinder)), (4, table(b'Surfaces', 1, bytes([12]))),
            (4, table(b'Surfaces', 1, bytes([11]) + d(1.0) + bytes([1]) + plane()[1:])),
            (4, table(b'Surfaces', 1, bytes([9, 0, 0, 0, 0]) + struct.pack('<HH', 1, 1) + i32(2, 2, 2, 2)
                      + d(*([0.0] * 12)) + d(0.0) + i32(2) + d(0.0) + i32(2) + d(0.0) + i32(2) + d(1.0) + i32(2))),
            (5, table(b'Locations', 2, matrix + bytes([2]) + i32(1, 3, 0))), (5, table(b'Locations', 1, bytes([0]))),
            (5, table(b'Locations', 1, bytes([2]) + i32(4, 1, 0))), (5, table(b'Locations', 1, bytes([1]) + d(*([0.0] * 12)))),
            (5, b'Locationz 0\n'), (5, table(b'Locations', 3, matrix)), (5, table(b'Locations', 1, bytes([2]) + i32(0)))]


def append_tkbrep_bintools_bytes_requests(requests: list[str]) -> None:
    """Every hand-built byte string through its reader (the grouped ones through both
    grouped readers)."""
    for data in grouped_cases():
        requests.append(request(0, data))
        requests.append(request(6, data))
    for data, skip, count in reference_cases():
        requests.append(request(1, data, f' {skip} {count}'))
    for reader, data in set_cases():
        requests.append(request(reader, data))
