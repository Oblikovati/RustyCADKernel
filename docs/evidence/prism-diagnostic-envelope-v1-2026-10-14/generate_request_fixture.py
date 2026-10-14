#!/usr/bin/env python3
"""Independent explicit V1 fixture: little-endian fields for the owned request test corpus."""
import pathlib, struct

def root_options(width):
    version, bisections, degree = 1, 4096, 32
    return struct.pack('<IdHH', version, width, bisections, degree)

def publication_options():
    header = struct.pack('<IIIIH', 1, 1, 1, 1, 32) + root_options(1e-6)
    validation = struct.pack('<IBd', 1, 2, 1e-6)
    history = struct.pack('<IQQQ', 1, 32, 4096, 16384)
    integral = struct.pack('<IHHd', 1, 16, 1, 1e-12)
    measures = struct.pack('<I', 1) + integral * 2
    measures += struct.pack('<IHId', 1, 16, 4096, 0.01)
    measures += struct.pack('<IHd', 1, 3, 0.02)
    return header + validation + history + measures

def policies():
    census = root_options(0.001) + struct.pack('<d', 1e-8)
    source = root_options(0.001) + struct.pack('<dd', 0., 0.)
    contact = root_options(0.001) + struct.pack('<ddd', 1e-8, 1e-8, 1e-8)
    classification = root_options(0.001) + struct.pack('<dd', 1e-8, 1e-8)
    return census + source + contact + classification + publication_options()

def request():
    header = b'Martensite.OwnedPolynomialPrismRequest/v1\0' + struct.pack('<IQQ', 1, 257, 1)
    source = struct.pack('<QQQQQBQ', 0, 71, 7000, 901, 19, 0, 4)
    source += struct.pack('<12d', 0.,0.,0., 4.,0.,0., 0.,4.,0., 0.,0.,0.)
    plane = struct.pack('<12d', 0.,0.,0., 0.,0.,1., 1.,0.,0., 0.,1.,0.)
    intent = struct.pack('<Bdd', 1, 1., 1.)
    extrusion = struct.pack('<ddd', 0., 0., 2.)
    return header + source + plane + intent + extrusion + policies()

if __name__ == '__main__':
    target = pathlib.Path('crates/tkbo/src/sketch_region_prism/owned_request/request_replay/fixtures/request-v1.bin')
    target.write_bytes(request())
    print(len(request()))
