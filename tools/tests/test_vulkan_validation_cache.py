"""Pinned dependency integrity and child-process selection avoid the legacy layer crash."""

import hashlib
import io
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
import tomllib
import unittest
from unittest.mock import patch

from vulkan_validation_cache import (
    ValidationLayerPaths, ValidationPackagePin, VerifiedValidationCache, noble_validation_pin,
)
from with_vulkan_validation import (
    NativeValidationCacheIO, command_environment, export_github_environment,
    WORKSPACE_VALIDATION_CACHE, main, require_noble_platform, shell_exit_status, validation_arguments,
)


class ValidationCacheFilesFake:
    def __init__(self) -> None:
        self.files: dict[Path, bytes] = {}
        self.downloads: list[str] = []
        self.extractions: list[Path] = []
        self.archive = b"package"

    def exists(self, path: Path) -> bool:
        return path in self.files

    def read(self, path: Path) -> bytes:
        return self.files[path]

    def download(self, url: str, path: Path) -> None:
        self.downloads.append(url)
        self.files[path] = self.archive

    def extract(self, archive: Path, destination: Path) -> None:
        self.extractions.append(archive)
        self.files[destination / "usr/lib/x86_64-linux-gnu/libVkLayer_khronos_validation.so"] = b"layer"
        self.files[destination / "usr/share/vulkan/explicit_layer.d/VkLayer_khronos_validation.json"] = b"manifest"


class ValidationCommandFake:
    def __init__(self) -> None:
        self.calls: list[list[str]] = []

    def __call__(self, command: list[str], **options: object) -> subprocess.CompletedProcess[str]:
        self.calls.append(command)
        if command[0] == "curl":
            Path(command[command.index("--output") + 1]).write_bytes(b"package")
        return subprocess.CompletedProcess(command, 23 if command[0] == "child" else 0)


class ValidationEnvironmentFake:
    def __call__(self, *arguments: object) -> dict[str, str]:
        return {"VK_LAYER_PATH": "/verified/layers", "LD_LIBRARY_PATH": "/verified/lib"}


def fake_validation_pin() -> ValidationPackagePin:
    return ValidationPackagePin(
        "fake", "https://example.invalid/layer.deb",
        hashlib.sha256(b"package").hexdigest(), hashlib.sha256(b"layer").hexdigest(),
        hashlib.sha256(b"manifest").hexdigest(),
    )


class ValidationCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.files = ValidationCacheFilesFake()
        self.cache = VerifiedValidationCache(Path("cache"), fake_validation_pin(), self.files)

    def test_verified_download_extract_and_cache_reuse(self) -> None:
        first = self.cache.prepare()
        self.assertEqual(first, self.cache.paths())
        self.assertEqual(self.cache.prepare(), first)
        self.assertEqual(len(self.files.downloads), 1)
        self.assertEqual(len(self.files.extractions), 1)

    def test_untrusted_archive_is_rejected_before_extraction(self) -> None:
        self.files.archive = b"changed package"
        with self.assertRaisesRegex(ValueError, "validation-layer.deb.*expected"):
            self.cache.prepare()
        self.assertEqual(self.files.extractions, [])

    def test_cached_library_tampering_is_rejected_without_redownload(self) -> None:
        paths = self.cache.prepare()
        self.files.files[paths.library] = b"wrong library"
        with self.assertRaisesRegex(ValueError, "libVkLayer.*expected"):
            self.cache.prepare()
        self.assertEqual(len(self.files.downloads), 1)

    def test_cached_manifest_tampering_is_rejected(self) -> None:
        paths = self.cache.prepare()
        self.files.files[paths.manifest] = b"wrong manifest"
        with self.assertRaisesRegex(ValueError, "VkLayer.*expected"):
            self.cache.verify(paths)

    def test_environment_preserves_device_validation_and_existing_libraries(self) -> None:
        paths = ValidationLayerPaths(Path("cache/layers/layer.json"), Path("cache/lib/layer.so"))
        original = {"LD_LIBRARY_PATH": "/existing", "VK_DRIVER_FILES": "/gpu.json"}
        selected = paths.environment(original)
        self.assertEqual(selected["VK_DRIVER_FILES"], "/gpu.json")
        self.assertEqual(selected["LD_LIBRARY_PATH"], str(Path("cache/lib").resolve()) + ":/existing")
        self.assertEqual(original["LD_LIBRARY_PATH"], "/existing")

    def test_pin_contains_exact_checksum_provenance(self) -> None:
        pin = noble_validation_pin()
        self.assertIn(pin.version, pin.url)
        for checksum in (pin.archive_sha256, pin.library_sha256, pin.manifest_sha256):
            self.assertEqual(len(bytes.fromhex(checksum)), 32)

    def test_platform_and_cli_fail_with_offending_values(self) -> None:
        require_noble_platform("Linux", "x86_64")
        with self.assertRaisesRegex(ValueError, "Darwin.*expected Linux x86_64"):
            require_noble_platform("Darwin", "arm64")
        self.assertEqual(validation_arguments(["--", "cargo", "test-all"]).command, ["cargo", "test-all"])
        self.assertIsNotNone(validation_arguments(["--github-env", "env.txt"]).github_env)

    def test_missing_command_fails_before_preparing_dependency(self) -> None:
        with patch("sys.stderr", new=io.StringIO()), self.assertRaises(SystemExit) as error:
            validation_arguments([])
        self.assertEqual(error.exception.code, 2)

    def test_native_io_download_and_extract_use_bounded_argument_lists(self) -> None:
        command = ValidationCommandFake()
        with TemporaryDirectory() as folder, patch("subprocess.run", new=command):
            archive = Path(folder) / "package.deb"
            native_io = NativeValidationCacheIO()
            native_io.download("https://example.invalid/package", archive)
            self.assertTrue(native_io.exists(archive))
            self.assertEqual(native_io.read(archive), b"package")
            native_io.extract(archive, Path(folder) / "extracted")
        self.assertIn("--fail", command.calls[0])
        self.assertEqual(command.calls[1][0], "dpkg-deb")

    def test_command_environment_prepares_injected_verified_dependency(self) -> None:
        pin = noble_validation_pin()
        cache = Path("verified")
        paths = VerifiedValidationCache(cache, pin, self.files).paths()
        self.files.files[paths.manifest], self.files.files[paths.library] = b"manifest", b"layer"
        with patch("with_vulkan_validation.noble_validation_pin", return_value=fake_validation_pin()):
            selected = command_environment(cache, {}, self.files, "Linux", "x86_64")
        self.assertEqual(selected["VK_LAYER_PATH"], str(paths.manifest.parent.resolve()))
        self.assertEqual(self.files.downloads, [])

    def test_child_exit_status_and_github_environment_are_preserved(self) -> None:
        command = ValidationCommandFake()
        with patch("with_vulkan_validation.command_environment", new=ValidationEnvironmentFake()):
            with patch("subprocess.run", new=command):
                self.assertEqual(main(["--", "child", "argument"]), 23)
            with TemporaryDirectory() as folder:
                output = Path(folder) / "env"
                self.assertEqual(main(["--github-env", str(output)]), 0)
                self.assertEqual(output.read_text(), "VK_LAYER_PATH=/verified/layers\nLD_LIBRARY_PATH=/verified/lib\n")
        self.assertEqual(command.calls, [["child", "argument"]])

    def test_signalled_child_reports_shell_status_instead_of_a_wrapped_negative_code(self) -> None:
        self.assertEqual(shell_exit_status(-11), 139)
        self.assertEqual(shell_exit_status(101), 101)
        self.assertEqual(shell_exit_status(0), 0)


class CargoRunnerPinTests(unittest.TestCase):
    """Plain `cargo test` must select the pinned layer from every package working directory."""

    def test_default_cache_is_rooted_at_the_workspace_not_the_child_directory(self) -> None:
        root = Path(__file__).resolve().parents[2]
        self.assertEqual(WORKSPACE_VALIDATION_CACHE, root / "target" / "vulkan-validation")
        self.assertEqual(validation_arguments(["--", "child"]).cache, WORKSPACE_VALIDATION_CACHE)

    def test_linux_cargo_runner_is_the_executable_validation_wrapper(self) -> None:
        root = Path(__file__).resolve().parents[2]
        configuration = tomllib.loads((root / ".cargo/config.toml").read_text())
        runner = configuration["target"]["x86_64-unknown-linux-gnu"]["runner"]
        self.assertEqual(runner, ["tools/with_vulkan_validation.py", "--"])
        self.assertTrue((root / runner[0]).stat().st_mode & 0o111, f"{runner[0]!r}; expected executable")


if __name__ == "__main__":
    unittest.main()
