"""Named in-memory capture I/O tests image integrity and failure localization."""

import struct
import unittest
import zlib
from pathlib import Path
from vulkan_capture_artifacts import rgba_png, depth_rgba, export_comparison


class FakeCaptureStore:
    def __init__(self) -> None:
        self.files: dict[Path, bytes] = {}

    def read(self, path: Path) -> bytes:
        return self.files[path]

    def write(self, path: Path, content: bytes) -> None:
        self.files[path] = content


class CaptureArtifactTests(unittest.TestCase):
    def test_png_preserves_rgba_rows_and_valid_crc(self) -> None:
        rgba = bytes((1, 2, 3, 4, 5, 6, 7, 8))
        png = rgba_png(1, 2, rgba)
        self.assertEqual(png[:8], b"\x89PNG\r\n\x1a\n")
        offset, payloads = 8, {}
        while offset < len(png):
            length = struct.unpack(">I", png[offset : offset + 4])[0]
            kind, content = (
                png[offset + 4 : offset + 8],
                png[offset + 8 : offset + 8 + length],
            )
            crc = struct.unpack(">I", png[offset + 8 + length : offset + 12 + length])[
                0
            ]
            self.assertEqual(crc, zlib.crc32(kind + content))
            payloads[kind] = content
            offset += length + 12
        self.assertEqual(
            zlib.decompress(payloads[b"IDAT"]), b"\0" + rgba[:4] + b"\0" + rgba[4:]
        )

    def test_invalid_extent_and_depth_visualization(self) -> None:
        with self.assertRaises(ValueError):
            rgba_png(2, 2, b"")
        colors = depth_rgba(struct.pack("=3f", 0.0, 1.0, float("nan")))
        self.assertEqual(
            colors, bytes((0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 255, 255))
        )

    def test_failed_comparison_retains_images_and_pixel_bounds(self) -> None:
        store = FakeCaptureStore()
        for name in ("gl", "vk"):
            store.write(Path(name + ".rgba"), bytes(16))
            store.write(Path(name + ".depth"), struct.pack("=4f", 0.5, 0.5, 0.5, 0.5))
        store.write(Path("vk.rgba"), bytes(12) + bytes((1, 0, 0, 0)))
        row = export_comparison("scene", 2, 2, Path("gl"), Path("vk"), store)
        self.assertEqual(row["status"], "failed")
        self.assertEqual(row["color_error_bounds"], [1, 1, 1, 1])
        self.assertIn(Path("vk.diff.png"), store.files)
        store.write(Path("vk.depth"), struct.pack("=4f", 0.5, 0.8, 0.5, 0.5))
        depth_row = export_comparison("depth", 2, 2, Path("gl"), Path("vk"), store)
        self.assertEqual(depth_row["depth_error_bounds"], [1, 0, 1, 0])
        store.write(Path("vk.depth"), struct.pack("=4f", 0.5, 0.5, 0.5, 0.5))
        store.write(Path("vk.rgba"), bytes(16))
        self.assertEqual(
            export_comparison("same", 2, 2, Path("gl"), Path("vk"), store)["status"],
            "passed",
        )
