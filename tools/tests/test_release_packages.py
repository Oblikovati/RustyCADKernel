"""Release contract tests with named command and archive fixtures."""

from __future__ import annotations

import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

from release_archives import extract_crate, file_sha256, stage_workspace
from release_inventory import (
    ReleaseCrate,
    SystemCommands,
    release_inventory,
    release_revision,
    validate_inventory,
)
from release_packages import build_release

REVISION = "a" * 40
CRATE = ReleaseCrate("rustycad-kernel", "0.1.0", "crates/kernel")


class ReleaseFixture:
    """Real temporary release bytes with a fake external command boundary."""

    def __init__(self, root: Path) -> None:
        self.root = root
        (root / "target/package").mkdir(parents=True)
        (root / "tools/oracle").mkdir(parents=True)
        (root / "tools/oracle/check.expected").write_text("exact oracle result\n")
        (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.98.1"\n')
        (root / "LICENSE").write_text("GPL-2.0-only\n")
        (root / "Cargo.lock").write_text("version = 4\n")
        self.archive = root / "target/package" / CRATE.archive
        self.write_archive(f"{CRATE.name}-{CRATE.version}/Cargo.toml")

    def write_archive(self, name: str, symlink: bool = False) -> None:
        with tarfile.open(self.archive, "w:gz") as stream:
            member = tarfile.TarInfo(name)
            content = b'[package]\nname = "rustycad-kernel"\nversion = "0.1.0"\n'
            member.size = len(content)
            if symlink:
                member.type = tarfile.SYMTYPE
                member.linkname = "/etc/passwd"
            stream.addfile(member, io.BytesIO(content))


class FakeCommands:
    """Model Git/Cargo responses and record checks without compiling fake sources."""

    def __init__(self, fixture: ReleaseFixture) -> None:
        self.fixture = fixture
        self.dirty = ""
        self.tagged = REVISION
        self.fail_tests = False
        self.calls: list[list[str]] = []

    def run(self, arguments: list[str], root: Path) -> str:
        self.calls.append(arguments)
        if arguments[0] == "git":
            return self.git_result(arguments)
        if arguments[1] == "metadata":
            package = {
                "id": "kernel",
                "name": CRATE.name,
                "version": CRATE.version,
                "manifest_path": str(
                    self.fixture.root / CRATE.directory / "Cargo.toml"
                ),
            }
            return json.dumps({"workspace_members": ["kernel"], "packages": [package]})
        if arguments[1] == "test":
            self.check_isolated_workspace(root)
        return ""

    def git_result(self, arguments: list[str]) -> str:
        if arguments[1] == "status":
            return self.dirty
        if arguments[1] == "ls-files":
            return "tools/oracle/check.expected\0"
        return REVISION if arguments[-1] == "HEAD" else self.tagged

    def check_isolated_workspace(self, root: Path) -> None:
        if not (root / CRATE.directory / "Cargo.toml").is_file():
            raise AssertionError("Expected extracted Cargo source")
        if not (root / "tools/oracle/check.expected").is_file():
            raise AssertionError("Expected packaged oracle corpus")
        if self.fail_tests:
            raise subprocess.CalledProcessError(1, ["cargo", "test"])


class ReleasePackageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.fixture = ReleaseFixture(Path(self.temporary.name))
        self.runner = FakeCommands(self.fixture)

    def test_inventory_and_named_archive(self) -> None:
        self.assertEqual(release_inventory(self.fixture.root, self.runner), [CRATE])
        self.assertEqual(CRATE.archive, "rustycad-kernel-0.1.0.crate")

    def test_invalid_inventories_are_rejected(self) -> None:
        invalid = [
            [],
            [CRATE, CRATE],
            [ReleaseCrate("foreign", "0.1.0", "crates/foreign")],
            [ReleaseCrate(CRATE.name, "01.1.0", CRATE.directory)],
            [ReleaseCrate(CRATE.name, "0.1.0-dev", CRATE.directory)],
            [ReleaseCrate(CRATE.name, "0.1.0", "../outside")],
            [CRATE, ReleaseCrate("martensite-vulkan", "0.2.0", "crates/vulkan")],
        ]
        for crates in invalid:
            with self.subTest(crates=crates), self.assertRaisesRegex(
                ValueError, "expected"
            ):
                validate_inventory(crates)

    def test_dirty_checkout_and_wrong_tag_are_rejected(self) -> None:
        self.runner.dirty = " M Cargo.toml"
        with self.assertRaisesRegex(ValueError, "clean release"):
            release_revision(self.fixture.root, self.runner, "v0.1.0")
        self.runner.dirty = ""
        self.runner.tagged = "b" * 40
        with self.assertRaisesRegex(ValueError, "expected HEAD"):
            release_revision(self.fixture.root, self.runner, "v0.1.0")

    def test_tag_must_match_package_version(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected v0.1.0"):
            build_release(self.fixture.root, "v0.2.0", self.runner)

    def test_test_failure_never_creates_release_output(self) -> None:
        self.runner.fail_tests = True
        with self.assertRaises(subprocess.CalledProcessError):
            build_release(self.fixture.root, "v0.1.0", self.runner)
        self.assertFalse((self.fixture.root / "target/releases/v0.1.0").exists())

    def test_success_seals_exact_artifacts_and_hashes_after_both_tests(self) -> None:
        output = build_release(self.fixture.root, "v0.1.0", self.runner)
        receipt = json.loads((output / "martensite-release.json").read_text())
        self.assertEqual(receipt["revision"], REVISION)
        self.assertEqual(
            set(receipt["artifacts"]),
            {
                CRATE.archive,
                "Cargo.lock",
                "LICENSE",
                "rust-toolchain.toml",
                "martensite-test-support.tar.gz",
            },
        )
        self.assertEqual(
            len([call for call in self.runner.calls if call[:2] == ["cargo", "test"]]),
            2,
        )
        for line in (output / "SHA256SUMS").read_text().splitlines():
            digest, name = line.split("  ")
            self.assertEqual(
                digest, hashlib.sha256((output / name).read_bytes()).hexdigest()
            )
            self.assertEqual(digest, file_sha256(output / name))
        with self.assertRaisesRegex(ValueError, "new directory"):
            build_release(self.fixture.root, "v0.1.0", self.runner)

    def test_archive_rejects_wrong_root_traversal_and_links(self) -> None:
        for name, symlink in [
            ("outside/Cargo.toml", False),
            ("rustycad-kernel-0.1.0/../../escaped", False),
            ("rustycad-kernel-0.1.0/link", True),
        ]:
            self.fixture.write_archive(name, symlink)
            with self.assertRaisesRegex(ValueError, "expected regular files"):
                extract_crate(
                    self.fixture.archive, self.fixture.root / "unpacked", CRATE
                )

    def test_archive_requires_manifest(self) -> None:
        self.fixture.write_archive("rustycad-kernel-0.1.0/readme")
        with self.assertRaisesRegex(ValueError, "Cargo.toml"):
            extract_crate(self.fixture.archive, self.fixture.root / "unpacked", CRATE)

    def test_workspace_is_new_and_patch_paths_are_portable(self) -> None:
        output = build_release(self.fixture.root, "v0.1.0", self.runner)
        stage = self.fixture.root / "standalone"
        stage_workspace(output, stage, [CRATE], output / "Cargo.lock")
        self.assertIn('path = "crates/kernel"', (stage / "Cargo.toml").read_text())
        with self.assertRaises(FileExistsError):
            stage_workspace(output, stage, [CRATE], output / "Cargo.lock")

    def test_system_commands_propagates_failures(self) -> None:
        runner = SystemCommands()
        self.assertIn(
            "git version", runner.run(["git", "--version"], self.fixture.root)
        )
        with self.assertRaises(subprocess.CalledProcessError):
            runner.run(["git", "rev-parse", "HEAD"], self.fixture.root)


if __name__ == "__main__":
    unittest.main()
