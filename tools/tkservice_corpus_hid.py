"""Deterministic requests for WNT_HIDSpaceMouse (tools/oracle/tkservice_probe_hid.cxx).

`sv_hid ProductId Quadric N byte...` decodes one raw HID chunk: translation chunks of 7 and
13 bytes, rotation chunks, key-state chunks, unknown chunk types and sizes, values inside
the dead zones, at the 350 range limit and beyond it (the range widens to 32767), for
every known product and an unknown one, linear and quadric.
"""
from __future__ import annotations
import random
import struct

PRODUCTS = [0xC625, 0xC627, 0xC626, 0xC62B, 0xC635, 0xC603, 0x1234]


def chunk(kind: int, values: list[int]) -> list[int]:
    """A chunk type byte followed by little-endian int16 values (x86-64 byte order)."""
    return [kind] + list(struct.pack('<' + 'h' * len(values), *values))


def hid_request(product: int, quadric: bool, data: list[int]) -> str:
    """One `sv_hid` request line."""
    fields = [product, int(quadric), len(data), *data]
    return 'sv_hid ' + ' '.join(repr(float(v)) for v in fields)


def fixed_chunks() -> list[list[int]]:
    """Chunks chosen for the dead zones, the range limit and the chunk-type rules."""
    return [chunk(1, [350, 3, -6]), chunk(1, [-350, -4, 8]), chunk(1, [0, 0, 0]),
            chunk(1, [1000, -2000, 7]), chunk(1, [10, 20, 30, -40, 50, -60]),
            chunk(2, [0, 0, 0]), chunk(2, [-351, 1, 2]), chunk(2, [1, 2, 3, 4, 5, 6]),
            [3, 255, 255, 255, 255], [3, 5, 0, 0, 128], [3], [1, 1, 2], [4, 0, 0, 0, 0, 0, 0]]


def append_hid_requests(requests: list[str]) -> None:
    """Append the HID requests: fixed chunks for every product, then seeded random ones."""
    for product in PRODUCTS:
        for data in fixed_chunks():
            requests.append(hid_request(product, product % 2 == 0, data))
    rng = random.Random(20261008)
    for _ in range(40):
        kind = rng.choice([1, 2, 3])
        size = rng.choice([7, 13]) if kind == 1 else (7 if kind == 2 else 5)
        data = [kind] + [rng.randrange(256) for _ in range(size - 1)]
        requests.append(hid_request(rng.choice(PRODUCTS), rng.random() < 0.5, data))
