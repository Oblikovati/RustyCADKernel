"""Check retained images independently: python3 check_framebuffers.py RAW_ROOT."""

import hashlib
import json
import struct
import sys
from pathlib import Path

root = Path(sys.argv[1])
directories = sorted(root.glob("*-rapid-*"))
assert len(directories) == 7, (directories, "expected five AMD and two lavapipe runs")
width, height = 1440, 900
blue = bytes([0, 0, 255, 255])
green = bytes([0, 255, 0, 255])
background = green * (width * height)
rectangle = bytearray(background)
for y in range(8, 118):
    rectangle[(y * width + 8) * 4 : (y * width + 308) * 4] = blue * 300
stages = {
    "blue": blue * (width * height),
    "green": background,
    **{f"rapid3gui-{index}": bytes(rectangle) for index in range(4)},
    "cleared": background,
    "restored": bytes(rectangle),
}
expected_depth = struct.pack("<f", 1.0) * (width * height)
hashes: dict[str, dict[str, str]] = {}
for stage, expected_rgba in stages.items():
    expected_rgb = bytearray(width * height * 3)
    for channel in range(3):
        expected_rgb[channel::3] = expected_rgba[channel::4]
    expected_ppm = b"P6\n1440 900\n255\n" + expected_rgb
    expected_files = {"rgba": expected_rgba, "depth": expected_depth, "ppm": expected_ppm}
    hashes[stage] = {}
    for suffix, expected in expected_files.items():
        for directory in directories:
            path = directory / f"{stage}.{suffix}"
            assert path.read_bytes() == expected, (str(path), "expected exact analytic image")
        hashes[stage][suffix] = hashlib.sha256(expected).hexdigest()
for suffix in ("rgba", "depth", "ppm"):
    native = (root / "amd-acceptance" / f"overlay.{suffix}").read_bytes()
    software = (root / "lavapipe-acceptance" / f"overlay.{suffix}").read_bytes()
    assert native == software, (suffix, "expected byte-identical GUI overlay across drivers")
print(json.dumps({"processes": 7, "stages": hashes, "overlay_cross_driver_equal": True}, indent=2))
