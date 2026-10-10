"""Profile gates must distinguish physical rendering from software fallback."""

import unittest
from pathlib import Path
from vulkan_profiles import (
    parse_devices,
    choose_device,
    clean_environment,
    probe_profile,
    confirm_device,
)

SUMMARY = """GPU0:
 apiVersion = 1.3.280
 deviceName = Named hardware GPU
 deviceType = PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU
 driverName = radv
GPU1:
 apiVersion = 1.3.280
 deviceName = Named software GPU
 deviceType = PHYSICAL_DEVICE_TYPE_CPU
 driverName = llvmpipe
"""


class FakeVulkanProbe:
    def __init__(self, summary: str = SUMMARY) -> None:
        self.summary = summary
        self.environment: dict[str, str] = {}

    def __call__(self, command: list[str], environment: dict[str, str]) -> str:
        self.environment = environment
        if command != ["vulkaninfo", "--summary"]:
            raise AssertionError(f"received {command}; expected device probe")
        return self.summary


class VulkanProfileTests(unittest.TestCase):
    def test_software_and_host_select_different_explicit_indices(self) -> None:
        runner = FakeVulkanProbe()
        software = probe_profile("lavapipe", {}, [Path("lvp_icd.json")], runner)
        host = probe_profile("host", software.environment, [], runner)
        self.assertEqual(software.device.index, 1)
        self.assertEqual(host.device.index, 0)
        self.assertNotIn("VK_DRIVER_FILES", host.environment)
        self.assertNotIn("LIBGL_ALWAYS_SOFTWARE", host.environment)
        self.assertEqual(host.environment["MARTENSITE_VULKAN_DEVICE"], "0")

    def test_missing_hardware_or_icd_cannot_pass_by_fallback(self) -> None:
        with self.assertRaisesRegex(ValueError, "without fallback"):
            choose_device("host", parse_devices(SUMMARY)[1:])
        with self.assertRaisesRegex(ValueError, "lavapipe ICD"):
            probe_profile("lavapipe", {}, [], FakeVulkanProbe())

    def test_device_report_must_match_probe(self) -> None:
        expected = parse_devices(SUMMARY)[0]
        confirm_device("MARTENSITE_DEVICE=Named hardware GPU", expected)
        for output in ("", "MARTENSITE_DEVICE=Named software GPU"):
            with self.assertRaisesRegex(ValueError, "renderer devices"):
                confirm_device(output, expected)

    def test_invalid_probe_and_profiles_are_rejected(self) -> None:
        for summary in ("", "GPU0:\n apiVersion = 1.3.0\n"):
            with self.assertRaises(ValueError):
                parse_devices(summary)
        with self.assertRaises(ValueError):
            probe_profile("missing", {}, [], FakeVulkanProbe())
        old = SUMMARY.replace("1.3.280", "1.2.100")
        with self.assertRaises(ValueError):
            choose_device("host", parse_devices(old))

    def test_host_icd_is_explicit_and_environment_is_not_mutated(self) -> None:
        env = {"VK_ICD_FILENAMES": "software", "DISPLAY": ":1"}
        profile = probe_profile("host", env, [], FakeVulkanProbe(), Path("native.json"))
        self.assertEqual(profile.environment["VK_DRIVER_FILES"], "native.json")
        self.assertEqual(clean_environment(env), {"DISPLAY": ":1"})
        self.assertEqual(env["VK_ICD_FILENAMES"], "software")
