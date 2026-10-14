"""Independent ADR0048 admission-field oracle; does not call/read Rust encoders."""
from pathlib import Path
import struct

def words(*values):
    """Pin unsigned integer fields independently of the Rust codec."""
    chunks = (struct.pack("<Q", value) for value in values)
    return b"".join(chunks)

def numbers(*values):
    """Pin binary64 field bytes independently, including signed zero."""
    chunks = (struct.pack("<d", value) for value in values)
    return b"".join(chunks)

cases = [
    b"\x00",
    b"\x01" + struct.pack("<II", 99, 1),
    b"\x02" + words(2, 7, 9001, 1),
    b"\x03" + words(7, 0, 2, 26),
    b"\x04" + words(7, 3, 2, 0x7FF8000000000123),
    b"\x05" + words(7) + numbers(-0.0, 1.0, 2.0, 3.0, 4.0, 5.0),
    b"\x06" + numbers(1.0, -0.0, float("inf")),
    b"\x07\x03" + words(2, 0x7FF8000000000456),
    b"\x08\x00" + words(901),
    b"\x08\x01" + words(1, 0xFFF0000000000000),
    b"\x08\x02" + words(9001) + struct.pack("<I", 2),
]
result = b"".join(cases)
root = Path(__file__).resolve().parents[3]
path = root / "crates/tkbo/src/sketch_region_prism/diagnostic_envelope/replay/fixtures/admission-v1.bin"
path.parent.mkdir(parents=True, exist_ok=True)
path.write_bytes(result)
