#!/usr/bin/env python3
"""Independent literal V1 name-envelope fixture; no production Rust imports/parsing."""
from pathlib import Path
import struct


def generate_fixture() -> bytes:
    """Encode the reviewed 97 reservations with explicit numeric role and anchor expectations."""
    names = [(1, r) for r in (1, 2)]
    names += [(s, r) for s in (2, 3, 4, 5, 6, 7, 8) for r in (3, 4, 5, 6, 7)]
    names += [(s, r) for s in (4, 5, 6, 7, 8) for r in range(8, 16)]
    names += [(9, r) for r in (*range(16, 34), 5, 6)]
    assert len(names) == len(set(names)) == 97
    result = b"Martensite.SemanticNames\0" + struct.pack("<I5QB2QQ", 1, 42, 700, 201, 990, 3, 1, 1 << 63, 0x3FF4000000000000, 97)
    for stage, role in sorted(names):
        tag, value = expected_anchor(role)
        result += struct.pack("<QBBQQBQ", stage * 256 + role, stage, role, 42, 700, tag, value)
    return result + struct.pack("<Q", 3) + bytes((0, 255, 42))


def expected_anchor(role: int) -> tuple[int, int]:
    """Explicit caller roles are preserved, including distinct source endpoint uses."""
    if role == 1:
        return 1, 42
    if role in (2, 4, 23, 24, 25):
        return 3, 201
    if role in (5, 32):
        return 4, 990
    if role in (6, 33):
        return 5, 3
    return 2, 700


if __name__ == "__main__":
    target = Path("crates/tkbo/src/sketch_region_prism/owned_request/names/fixtures/names-v1.bin")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(generate_fixture())
