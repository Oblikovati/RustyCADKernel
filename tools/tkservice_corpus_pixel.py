"""Deterministic requests for the Image_PixMap pixel conversions
(tools/oracle/tkservice_probe_pixel.cxx).

`sv_pixel` writes a color as a pixel of every format, linear and sRGB-encoded, and reads it
back: channel values at 0, 1, half-way, just below byte boundaries and out of range (the
color constructor raises). `sv_rawpixel` decodes raw bytes, including float pixels outside
[0, 1] (raises), half floats and the unknown format. `sv_half` compares the half-float bit
hacks on normals, denormals, signed zeros, infinities, NaN and saturation.
"""
from __future__ import annotations
import random
import struct

FORMATS = list(range(20))
COLORS = [[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], [0.5, 0.25, 0.75, 0.5],
          [0.003, 0.0031308, 0.04045, 0.999], [254.5 / 255.0, 1.0 / 255.0, 0.2, 0.7],
          [1.5, 0.0, 0.0, 1.0], [-0.1, 0.5, 0.5, 1.0], [0.2, 0.4, 0.6, 2.0]]


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def float_bytes(values: list[float]) -> list[int]:
    """The little-endian bytes of single-precision values, padded to 16 bytes."""
    data = list(struct.pack('<' + 'f' * len(values), *values))
    return (data + [0] * 16)[:16]


def append_pixel_requests(requests: list[str]) -> None:
    """Append the pixel, raw-pixel and half-float requests."""
    for fmt in FORMATS:
        for linear in (0, 1):
            for color in COLORS:
                requests.append('sv_pixel ' + numbers([fmt, linear, *color]))
    raws = [float_bytes([0.25, 0.5, 0.75, 1.0]), float_bytes([2.0, -1.0, 0.5, 0.5]),
            list(range(16)), [255] * 16, float_bytes([float('nan'), 0.0, 0.0, 1.0])]
    for fmt in FORMATS:
        for raw in raws:
            requests.append('sv_rawpixel ' + numbers([fmt, fmt % 2, *raw]))
    halves = [0x0000, 0x8000, 0x0001, 0x03FF, 0x0400, 0x3C00, 0x3555, 0x7BFF, 0x7C00, 0xFC00,
              0x7E00, 0x8001, 0xFFFF]
    floats = [0.0, -0.0, 1.0, 65504.0, 65520.0, 1e9, 6e-8, 5.96e-8, 3e-8, 1e-10,
              1e39, -2.5, 0.1]  # 1e39 becomes +inf as a float
    for half, value in zip(halves, floats):
        requests.append('sv_half ' + numbers([half, value]))
    rng = random.Random(20261009)
    for _ in range(64):
        value = struct.unpack('<f', struct.pack('<I', rng.getrandbits(32)))[0]
        if value != value or abs(value) == float('inf'):  # istream cannot read nan or inf
            value = 0.5
        requests.append('sv_half ' + numbers([rng.getrandbits(16), value]))
