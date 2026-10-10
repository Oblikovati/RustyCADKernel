"""Explicit software/native device selection prevents silently testing the same GPU twice."""

from __future__ import annotations
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path
import re

CommandRunner = Callable[[list[str], dict[str, str]], str]


@dataclass(frozen=True)
class VulkanDevice:
    index: int
    name: str
    kind: str
    api_version: str
    driver: str


@dataclass(frozen=True)
class VulkanProfile:
    name: str
    device: VulkanDevice
    environment: dict[str, str]
    probe: str


def parse_devices(summary: str) -> list[VulkanDevice]:
    """Read vulkaninfo's device blocks; e.g. parse_devices(summary)."""
    devices = []
    for index, block in re.findall(
        r"GPU(\d+):\s*\n(.*?)(?=\nGPU\d+:|\Z)", summary, re.S
    ):
        fields = dict(re.findall(r"^\s*(\w+)\s*=\s*(.*?)\s*$", block, re.M))
        required = ("deviceName", "deviceType", "apiVersion", "driverName")
        if any(key not in fields for key in required):
            raise ValueError(
                f"received GPU{index} fields {fields}; expected {required}"
            )
        devices.append(VulkanDevice(int(index), *(fields[key] for key in required)))
    if not devices:
        raise ValueError("received no device blocks; expected Vulkan 1.3 devices")
    return devices


def clean_environment(environment: dict[str, str]) -> dict[str, str]:
    """Isolate a profile from inherited software/ICD overrides; e.g. clean_environment(env)."""
    removed = {
        "VK_DRIVER_FILES",
        "VK_ICD_FILENAMES",
        "LIBGL_ALWAYS_SOFTWARE",
        "GALLIUM_DRIVER",
        "MESA_LOADER_DRIVER_OVERRIDE",
        "MESA_VK_DEVICE_SELECT",
        "MARTENSITE_VULKAN_DEVICE",
    }
    return {key: value for key, value in environment.items() if key not in removed}


def choose_device(profile: str, devices: Sequence[VulkanDevice]) -> VulkanDevice:
    """Require a CPU lavapipe device or a native GPU; e.g. choose_device('host', devices)."""
    for device in devices:
        software = device.kind == "PHYSICAL_DEVICE_TYPE_CPU" and device.driver in (
            "llvmpipe",
            "lavapipe",
        )
        hardware = device.kind in (
            "PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU",
            "PHYSICAL_DEVICE_TYPE_DISCRETE_GPU",
        )
        if tuple(map(int, device.api_version.split(".")[:2])) < (1, 3):
            continue
        if (profile == "lavapipe" and software) or (profile == "host" and hardware):
            return device
    raise ValueError(
        f"received devices {devices}; expected Vulkan 1.3 {profile}, without fallback"
    )


def probe_profile(
    name: str,
    environment: dict[str, str],
    manifests: Sequence[Path],
    runner: CommandRunner,
    host_icd: Path | None = None,
) -> VulkanProfile:
    """Probe and pin the profile's actual device; e.g. probe_profile('host', env, icds, run)."""
    env = clean_environment(environment)
    if name == "lavapipe":
        candidates = [p for p in manifests if p.name.startswith("lvp_icd")]
        if not candidates:
            raise ValueError(f"received manifests {manifests}; expected a lavapipe ICD")
        env.update(VK_DRIVER_FILES=str(candidates[0]), LIBGL_ALWAYS_SOFTWARE="1")
    elif name == "host" and host_icd is not None:
        env["VK_DRIVER_FILES"] = str(host_icd)
    elif name != "host":
        raise ValueError(f"received profile {name!r}; expected lavapipe or host")
    summary = runner(["vulkaninfo", "--summary"], env)
    device = choose_device(name, parse_devices(summary))
    env["MARTENSITE_VULKAN_DEVICE"] = str(device.index)
    return VulkanProfile(name, device, env, summary)


def confirm_device(output: str, expected: VulkanDevice) -> None:
    """Require renderer identity matching the probe; e.g. confirm_device(stdout, device)."""
    names = re.findall(r"^MARTENSITE_DEVICE=(.*)$", output, re.M)
    if not names or any(name != expected.name for name in names):
        raise ValueError(
            f"received renderer devices {names}; expected {expected.name!r}"
        )
