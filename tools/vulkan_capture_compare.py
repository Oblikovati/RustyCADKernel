"""Strict color/depth comparison for Martensite's controlled renderer scenes."""

from __future__ import annotations

import math
import struct
from dataclasses import asdict, dataclass
from pathlib import Path


@dataclass(frozen=True)
class CaptureComparison:
    case: str
    pixels: int
    color_mismatch_pixels: int
    maximum_depth_delta: float


def compare_bytes(
    label: str,
    pixels: int,
    native_color: bytes,
    ported_color: bytes,
    native_depth: bytes,
    ported_depth: bytes,
) -> CaptureComparison:
    """Require exact RGBA and finite depth within 1e-6; e.g. compare_bytes('scene', ...)."""
    lengths = tuple(map(len, (native_color, ported_color, native_depth, ported_depth)))
    if pixels <= 0 or lengths != (pixels * 4,) * 4:
        raise ValueError(
            f"received {pixels=} {lengths=}; expected four complete RGBA/depth buffers"
        )
    delta = depth_delta(label, pixels, native_depth, ported_depth)
    mismatches = sum(
        native_color[i : i + 4] != ported_color[i : i + 4]
        for i in range(0, pixels * 4, 4)
    )
    result = CaptureComparison(label, pixels, mismatches, delta)
    if mismatches or delta > 1e-6:
        raise ValueError(
            f"received {result}; expected identical RGBA and depth delta <= 1e-6"
        )
    return result


def compare_captures(
    label: str, pixels: int, native: Path, ported: Path
) -> dict[str, str | int | float]:
    """Read two explicit captures; e.g. compare_captures('ortho', 4096, gl, vk)."""
    buffers = [
        Path(f"{prefix}.{suffix}").read_bytes()
        for suffix in ("rgba", "depth")
        for prefix in (native, ported)
    ]
    return asdict(compare_bytes(label, pixels, *buffers))


def depth_delta(
    label: str, pixels: int, native_depth: bytes, ported_depth: bytes
) -> float:
    """Reject invalid values before measuring depth; e.g. depth_delta(label, pixels, gl, vk)."""
    native = struct.unpack(f"={pixels}f", native_depth)
    ported = struct.unpack(f"={pixels}f", ported_depth)
    if not all(math.isfinite(v) and 0 <= v <= 1 for v in (*native, *ported)):
        raise ValueError(
            f"received invalid depth in {label}; expected finite depth in [0,1]"
        )
    return max(abs(a - b) for a, b in zip(native, ported, strict=True))
