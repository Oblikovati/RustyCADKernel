"""Inspectable color/depth captures and localized diff reports, with no image library dependency."""

from __future__ import annotations
from pathlib import Path
from typing import Protocol
import math
import struct
import zlib
from dataclasses import asdict
from vulkan_capture_compare import compare_bytes


class CaptureStore(Protocol):
    def read(self, path: Path) -> bytes: ...
    def write(self, path: Path, content: bytes) -> None: ...


class FileCaptureStore:
    def read(self, path: Path) -> bytes:
        """Read a completed capture; e.g. store.read(path)."""
        return path.read_bytes()

    def write(self, path: Path, content: bytes) -> None:
        """Publish a complete image atomically; e.g. store.write(path, png)."""
        temporary = path.with_suffix(path.suffix + ".tmp")
        temporary.write_bytes(content)
        temporary.replace(path)


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    """Encode a CRC-checked PNG chunk; e.g. png_chunk(b'IEND', b'')."""
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", zlib.crc32(kind + payload))
    )


def rgba_png(width: int, height: int, rgba: bytes) -> bytes:
    """Encode top-left RGBA without changing color values; e.g. rgba_png(1, 1, bytes(4))."""
    if width <= 0 or height <= 0 or len(rgba) != width * height * 4:
        raise ValueError(
            f"received {width=} {height=} bytes={len(rgba)}; expected complete positive RGBA extent"
        )
    rows = b"".join(
        b"\0" + rgba[y * width * 4 : (y + 1) * width * 4] for y in range(height)
    )
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", header)
        + png_chunk(b"IDAT", zlib.compress(rows))
        + png_chunk(b"IEND", b"")
    )


def depth_rgba(depth: bytes) -> bytes:
    """Visualize depth as near-black/far-white, invalid magenta; e.g. depth_rgba(raw)."""
    pixels = bytearray()
    for (value,) in struct.iter_unpack("=f", depth):
        if not math.isfinite(value) or not 0 <= value <= 1:
            pixels.extend((255, 0, 255, 255))
            continue
        gray = round(value * 255)
        pixels.extend((gray, gray, gray, 255))
    return bytes(pixels)


def difference_image(
    width: int, native: bytes, ported: bytes
) -> tuple[bytes, list[int] | None]:
    """Highlight changed pixels and report inclusive bounds; e.g. difference_image(64, gl, vk)."""
    changed = [
        i // 4
        for i in range(0, len(native), 4)
        if native[i : i + 4] != ported[i : i + 4]
    ]
    return error_pixels(width, len(native) // 4, changed)


def error_pixels(
    width: int, count: int, changed: list[int]
) -> tuple[bytes, list[int] | None]:
    """Encode a sparse error mask; e.g. error_pixels(2, 4, [3])."""
    changed_set = set(changed)
    rgba = bytes(
        channel
        for pixel in range(count)
        for channel in ((255, 0, 255, 255) if pixel in changed_set else (0, 0, 0, 255))
    )
    if not changed:
        return rgba, None
    xs, ys = [i % width for i in changed], [i // width for i in changed]
    return rgba, [min(xs), min(ys), max(xs), max(ys)]


def depth_difference(
    width: int, native: bytes, ported: bytes
) -> tuple[bytes, list[int] | None]:
    """Localize invalid or out-of-tolerance depth; e.g. depth_difference(64, gl, vk)."""
    pairs = zip(
        struct.iter_unpack("=f", native), struct.iter_unpack("=f", ported), strict=True
    )
    changed = [
        index
        for index, ((left,), (right,)) in enumerate(pairs)
        if not all(math.isfinite(v) and 0 <= v <= 1 for v in (left, right))
        or abs(left - right) > 1e-6
    ]
    return error_pixels(width, len(native) // 4, changed)


def write_view_images(
    prefix: Path,
    width: int,
    height: int,
    color: bytes,
    depth: bytes,
    store: CaptureStore,
) -> None:
    """Publish color and independently inspectable depth; e.g. write_view_images(prefix, w, h, rgba, z, store)."""
    store.write(Path(f"{prefix}.png"), rgba_png(width, height, color))
    store.write(Path(f"{prefix}.depth.png"), rgba_png(width, height, depth_rgba(depth)))


def export_comparison(
    label: str, width: int, height: int, native: Path, ported: Path, store: CaptureStore
) -> dict[str, object]:
    """Retain evidence even when comparison fails; e.g. export_comparison(label, w, h, gl, vk, store)."""
    colors = [store.read(Path(f"{p}.rgba")) for p in (native, ported)]
    depths = [store.read(Path(f"{p}.depth")) for p in (native, ported)]
    try:
        result: dict[str, object] = {
            **asdict(compare_bytes(label, width * height, *colors, *depths)),
            "status": "passed",
        }
    except ValueError as error:
        result = {"case": label, "status": "failed", "error": str(error)}
    if any(len(buffer) != width * height * 4 for buffer in (*colors, *depths)):
        return result
    for prefix, color, depth in zip((native, ported), colors, depths, strict=True):
        write_view_images(prefix, width, height, color, depth, store)
    result.update(write_error_images(width, height, colors, depths, ported, store))
    result.update(native_capture=str(native), ported_capture=str(ported))
    return result


def write_error_images(
    width: int,
    height: int,
    colors: list[bytes],
    depths: list[bytes],
    ported: Path,
    store: CaptureStore,
) -> dict[str, object]:
    """Publish independent color/depth failure masks; e.g. write_error_images(w, h, colors, depths, vk, store)."""
    difference, bounds = difference_image(width, *colors)
    store.write(Path(f"{ported}.diff.png"), rgba_png(width, height, difference))
    depth_diff, depth_bounds = depth_difference(width, *depths)
    store.write(Path(f"{ported}.depth-diff.png"), rgba_png(width, height, depth_diff))
    return {"depth_error_bounds": depth_bounds, "color_error_bounds": bounds}
