"""Deterministic requests for ShapeExtend (tools/oracle/tkshhealing_probe_shapeextend.cxx).

`sh_status` encodes every ShapeExtend_Status enumerator and decodes it from bit fields:
no bit, each single bit, each byte, every bit, bits beyond the 16 the flags use, and a
negative field.
"""
from __future__ import annotations

# The 19 enumerators of ShapeExtend_Status, OK..FAIL.
STATUS_COUNT = 19


def status_request(flag: int, status: int) -> str:
    """One `sh_status flag status` request line."""
    return f'sh_status {flag} {status}'


def status_flags() -> list[int]:
    """The bit fields decoded against every status."""
    single = [1 << bit for bit in range(16)]
    return [0, 0x00ff, 0xff00, 0xffff, 0x10000, 0x10001, -1, 0x0101, 0x8001] + single


def append_shapeextend_requests(requests: list[str]) -> None:
    """Append the ShapeExtend requests: every flag against every status."""
    requests.extend(status_request(flag, status)
                    for flag in status_flags() for status in range(STATUS_COUNT))
