"""Checksum-pinned validation dependency for Martensite Linux rendering tests."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
from pathlib import Path
from typing import Protocol


@dataclass(frozen=True)
class ValidationPackagePin:
    version: str
    url: str
    archive_sha256: str
    library_sha256: str
    manifest_sha256: str


def noble_validation_pin() -> ValidationPackagePin:
    """Select the package verified by parallel replay; e.g. `noble_validation_pin().version`."""
    return ValidationPackagePin(
        "1.4.313.0~rc2-1lunarg24.04-1",
        "https://packages.lunarg.com/vulkan/pool/main/v/vulkan-validationlayers/"
        "vulkan-validationlayers_1.4.313.0~rc2-1lunarg24.04-1_amd64.deb",
        "b100bfafac3df98c5d8ef5a572b423bcd9ebf3fab963e2e68957f4bf6f2423f0",
        "89e618526d6d1532096dbbd62ee1dc710dda647061cd3b7b66b542dc7dbce104",
        "f328caf95d75607075575cf5849b4e765f9eabe111ea5be7f971c92a9a7d1a58",
    )


class ValidationCacheIO(Protocol):
    def exists(self, path: Path) -> bool: ...
    def read(self, path: Path) -> bytes: ...
    def download(self, url: str, path: Path) -> None: ...
    def extract(self, archive: Path, destination: Path) -> None: ...


@dataclass(frozen=True)
class ValidationLayerPaths:
    manifest: Path
    library: Path

    def environment(self, inherited: dict[str, str]) -> dict[str, str]:
        """Select this layer without disabling validation; e.g. `paths.environment(os.environ.copy())`."""
        environment = dict(inherited)
        environment["VK_LAYER_PATH"] = str(self.manifest.parent.resolve())
        previous = environment.get("LD_LIBRARY_PATH", "")
        directory = str(self.library.parent.resolve())
        environment["LD_LIBRARY_PATH"] = directory + (":" + previous if previous else "")
        return environment


class VerifiedValidationCache:
    def __init__(self, folder: Path, pin: ValidationPackagePin, cache_io: ValidationCacheIO) -> None:
        self.folder = folder
        self.pin = pin
        self.cache_io = cache_io

    def paths(self) -> ValidationLayerPaths:
        """Locate extracted package files; e.g. `cache.paths().manifest`."""
        extracted = self.folder / "extracted" / "usr"
        return ValidationLayerPaths(
            extracted / "share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json",
            extracted / "lib/x86_64-linux-gnu/libVkLayer_khronos_validation.so",
        )

    def prepare(self) -> ValidationLayerPaths:
        """Reuse verified files or extract the pinned package; e.g. `cache.prepare()`."""
        paths = self.paths()
        if self.cache_io.exists(paths.manifest) and self.cache_io.exists(paths.library):
            self.verify(paths)
            return paths
        archive = self.folder / "validation-layer.deb"
        if not self.cache_io.exists(archive):
            self.cache_io.download(self.pin.url, archive)
        self.require_digest(archive, self.pin.archive_sha256)
        self.cache_io.extract(archive, self.folder / "extracted")
        self.verify(paths)
        return paths

    def verify(self, paths: ValidationLayerPaths) -> None:
        """Reject changed extracted assets; e.g. `cache.verify(cache.paths())`."""
        self.require_digest(paths.manifest, self.pin.manifest_sha256)
        self.require_digest(paths.library, self.pin.library_sha256)

    def require_digest(self, path: Path, expected: str) -> None:
        """Bind downloaded/extracted bytes to provenance; e.g. `cache.require_digest(path, sha)`."""
        actual = hashlib.sha256(self.cache_io.read(path)).hexdigest()
        if actual == expected:
            return
        raise ValueError(f"artifact={str(path)!r}, sha256={actual!r}; expected {expected!r}")
