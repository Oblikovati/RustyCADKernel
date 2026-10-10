"""Precision controls cannot conceal stale inputs, missing scenes or altered output."""

import hashlib
import struct
import unittest
from dataclasses import asdict
from pathlib import Path

from verify_vulkan import CASES, PERSISTENCE_MODES
from verify_vulkan_raster_precision import (
    buffer_fingerprints,
    capture_control,
    persistence_arguments,
    persistence_rows,
    require_control_output,
    require_profile,
    require_renderer_sources,
)
from vulkan_profiles import VulkanDevice, VulkanProfile


class FakeRasterStore:
    def __init__(self) -> None:
        self.files: dict[Path, bytes] = {}

    def read(self, path: Path) -> bytes:
        return self.files[path]

    def write(self, path: Path, content: bytes) -> None:
        self.files[path] = content


class FakeRasterRunner:
    def __init__(self, store: FakeRasterStore, write_buffers: bool = True) -> None:
        self.store = store
        self.write_buffers = write_buffers

    def __call__(self, command: list[str], environment: dict[str, str]) -> str:
        if command[-1] != "raster8":
            raise AssertionError(f"received {command}; expected raster8 control")
        if self.write_buffers:
            self.store.write(Path(command[5] + ".rgba"), bytes([255, 0, 0, 255]))
            self.store.write(Path(command[5] + ".depth"), struct.pack("f", 0.5))
        return "GLdevice: named GPU\nMARTENSITE_RASTER_CONTROL=8"


class RasterPrecisionTests(unittest.TestCase):
    def setUp(self) -> None:
        device = VulkanDevice(0, "named GPU", "integrated", "1.3.0", "radv")
        self.profile = VulkanProfile("host", device, {}, "pinned driver")

    def test_signed_offsets_and_unknown_modes(self) -> None:
        self.assertEqual(
            persistence_arguments("48-96-1--3-pers-zoom"),
            (["48", "96", "1", "-3"], "pers-zoom"),
        )
        for label in ("64-64-2-0-pers-zoom", "64-64-0-0-pers-unknown", "ais-displayed"):
            with self.assertRaisesRegex(ValueError, "known persistence"):
                persistence_arguments(label)

    def test_every_case_is_required_exactly_once(self) -> None:
        rows = [
            {"case": "-".join(map(str, (*case, mode)))}
            for case in CASES
            for mode in PERSISTENCE_MODES
        ]
        self.assertEqual(len(persistence_rows(rows)), 114)
        for incomplete in (rows[:-1], rows[:-1] + [rows[0]]):
            with self.assertRaisesRegex(ValueError, "unique cases"):
                persistence_rows(incomplete)

    def test_stale_added_removed_and_changed_renderer_sources_are_rejected(
        self,
    ) -> None:
        recorded = {"crates/vulkan/a.rs": "original", "tools/oracle.cxx": "before"}
        current = {**recorded, "tools/oracle.cxx": "after"}
        self.assertEqual(
            require_renderer_sources(recorded, current),
            {"crates/vulkan/a.rs": "original"},
        )
        for changed in (
            {},
            {**current, "crates/new.rs": "new"},
            {**current, "crates/vulkan/a.rs": "changed"},
        ):
            with self.assertRaisesRegex(ValueError, "renderer inputs"):
                require_renderer_sources(recorded, changed)
        with self.assertRaisesRegex(ValueError, "renderer inputs"):
            require_renderer_sources({**recorded, "crates/removed.rs": "old"}, current)

    def test_driver_changes_and_missing_markers_fail(self) -> None:
        recorded = {
            "device": asdict(self.profile.device),
            "driver_info": self.profile.probe,
        }
        require_profile(recorded, self.profile)
        with self.assertRaisesRegex(ValueError, "device/driver"):
            require_profile({**recorded, "driver_info": "changed"}, self.profile)
        original = "GLdevice: named GPU"
        require_control_output(original + "\nMARTENSITE_RASTER_CONTROL=8", original)
        for output in (original, "GLdevice: software\nMARTENSITE_RASTER_CONTROL=8"):
            with self.assertRaisesRegex(ValueError, "original identity"):
                require_control_output(output, original)

    def capture_fixture(self) -> tuple[FakeRasterStore, dict[str, object]]:
        store = FakeRasterStore()
        store.write(Path("vk.rgba"), bytes([255, 0, 0, 255]))
        store.write(Path("vk.depth"), struct.pack("f", 0.5))
        row = {
            "case": "1-1-0-0-pers-zoom",
            "ported_capture": "vk",
            "status": "failed",
            "opengl_runtime": "GLdevice: named GPU",
        }
        return store, row

    def test_control_preserves_raw_failure_and_rejects_changed_pixels(self) -> None:
        store, row = self.capture_fixture()
        result = capture_control(
            Path("native"),
            Path("out"),
            self.profile,
            row,
            FakeRasterRunner(store),
            store,
        )
        self.assertEqual(
            (result["status"], result["raw_native_status"]), ("passed", "failed")
        )
        self.assertEqual(
            buffer_fingerprints(Path("vk"), store)["rgba"],
            hashlib.sha256(store.read(Path("vk.rgba"))).hexdigest(),
        )
        store.write(Path("vk.rgba"), bytes([0, 0, 255, 255]))
        result = capture_control(
            Path("native"),
            Path("out"),
            self.profile,
            row,
            FakeRasterRunner(store),
            store,
        )
        self.assertEqual(result["status"], "failed")

    def test_control_cannot_reuse_stale_buffers(self) -> None:
        store, row = self.capture_fixture()
        capture_control(
            Path("native"),
            Path("out"),
            self.profile,
            row,
            FakeRasterRunner(store),
            store,
        )
        result = capture_control(
            Path("native"),
            Path("out"),
            self.profile,
            row,
            FakeRasterRunner(store, False),
            store,
        )
        self.assertEqual(result["status"], "failed")
