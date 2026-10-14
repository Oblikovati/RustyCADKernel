#!/usr/bin/env python3
"""Independent literal extension of the reviewed V1 expected dictionary, without Rust inputs."""
from pathlib import Path
import runpy
import struct


def generate_fixture() -> bytes:
    """Preserve all V1 entries, adding only the four reviewed aggregate symbols."""
    old = runpy.run_path(str(Path(__file__).parent.parent / 'prism-semantic-names-v1/generate_expected.py'))
    expected = old['generate_fixture']()
    marker = b'Martensite.SemanticNames\0'
    header_size = len(marker) + struct.calcsize('<I5QB2QQ')
    entry_size = struct.calcsize('<QBBQQBQ')
    rows = [expected[header_size + i * entry_size:header_size + (i + 1) * entry_size] for i in range(97)]
    assert len(rows) == len(set(rows)) == 97
    for token, stage, role in [(1058, 4, 34), (1059, 4, 35), (1314, 5, 34), (1315, 5, 35)]:
        rows.append(struct.pack('<QBBQQBQ', token, stage, role, 42, 700, 2, 700))
    header = bytearray(expected[:header_size])
    struct.pack_into('<I', header, len(marker), 2)
    struct.pack_into('<Q', header, header_size - 8, 101)
    rows.sort(key=lambda row: struct.unpack_from('<Q', row)[0])
    return bytes(header) + b''.join(rows) + expected[header_size + 97 * entry_size:]


if __name__ == '__main__':
    target = Path('crates/tkbo/src/sketch_region_prism/owned_request/names/fixtures/names-v2.bin')
    target.write_bytes(generate_fixture())
