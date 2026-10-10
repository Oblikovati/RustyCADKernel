"""Actual-window comparisons reject stale/truncated buffers and expose screen differences."""

import unittest
from pathlib import Path
from vulkan_window_cases import compare_window_pixels, WINDOW_MODES


class FakeWindowCaptureStore:
    def __init__(self, native: bytes, ported: bytes) -> None:
        self.files: dict[Path, bytes] = {
            Path("gl.window.rgba"): native,
            Path("vk.window.rgba"): ported,
        }

    def read(self, path: Path) -> bytes:
        return self.files[path]

    def write(self, path: Path, content: bytes) -> None:
        self.files[path] = content


class WindowCaptureTests(unittest.TestCase):
    def test_equal_pixels_and_localized_difference(self) -> None:
        for right, expected in (
            (bytes([255, 0, 0, 255]), "passed"),
            (bytes([0, 255, 0, 255]), "failed"),
        ):
            store = FakeWindowCaptureStore(bytes([255, 0, 0, 255]), right)
            row = compare_window_pixels(1, 1, Path("gl"), Path("vk"), store)
            self.assertEqual(row["status"], expected)
            self.assertEqual(row["different_pixels"], int(expected == "failed"))
            self.assertTrue(store.files[Path("vk.window.png")].startswith(b"\x89PNG"))
        self.assertEqual(len(WINDOW_MODES), 6)

    def test_incomplete_surface_buffers_are_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "complete positive RGBA extent"):
            compare_window_pixels(
                2, 1, Path("gl"), Path("vk"), FakeWindowCaptureStore(bytes(4), bytes(8))
            )
