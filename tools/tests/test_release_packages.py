"""Release contract tests with named command and archive fixtures."""

from __future__ import annotations

import hashlib
import io
import json
import os
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
    release_crate_from_metadata,
    release_feature_names,
    release_revision,
    validate_inventory,
)
from release_packages import (
    build_release,
    package_test_support,
    release_metadata,
    test_archives,
)

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

    def write_archive(
        self, name: str, symlink: bool = False, crate: ReleaseCrate = CRATE
    ) -> None:
        archive = self.root / "target/package" / crate.archive
        with tarfile.open(archive, "w:gz") as stream:
            member = tarfile.TarInfo(name)
            content = f'[package]\nname = "{crate.name}"\nversion = "{crate.version}"\n'.encode()
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
        self.stale_suites = False
        self.unformatted_inputs = False
        self.features: object = {}
        self.calls: list[list[str]] = []

    def run(self, arguments: list[str], root: Path) -> str:
        self.calls.append(arguments)
        if arguments == ["python3", "tools/check_rust_format.py"]:
            if self.unformatted_inputs:
                raise subprocess.CalledProcessError(3, arguments)
            return ""
        if arguments == ["python3", "tools/toolkit_test_suites.py"]:
            if self.stale_suites:
                raise subprocess.CalledProcessError(2, arguments)
            return ""
        if arguments[0] == "git":
            return self.git_result(arguments)
        if arguments[1] == "metadata":
            package = {
                "id": "kernel",
                "name": CRATE.name,
                "version": CRATE.version,
                "features": self.features,
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


class LateReleaseMutation(FakeCommands):
    """Expose a real source/tag gate transition after extracted tests finish."""

    mutation: str = ""
    tests_finished: bool = False

    def run(self, arguments: list[str], root: Path) -> str:
        result = super().run(arguments, root)
        if arguments[:2] == ["cargo", "test"]:
            self.tests_finished = True
            if self.mutation == "dirty":
                self.dirty = " M Cargo.lock"
        return result

    def git_result(self, arguments: list[str]) -> str:
        if (
            self.tests_finished
            and self.mutation == "moved"
            and arguments[1] == "rev-parse"
        ):
            return "b" * 40
        return super().git_result(arguments)


class ReleasePackageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.fixture = ReleaseFixture(Path(self.temporary.name))
        self.runner = FakeCommands(self.fixture)

    def test_inventory_and_named_archive(self) -> None:
        self.assertEqual(release_inventory(self.fixture.root, self.runner), [CRATE])
        self.assertEqual(CRATE.archive, "rustycad-kernel-0.1.0.crate")

    def test_stale_suites_refuse_release_before_packaging(self) -> None:
        self.runner.stale_suites = True
        with self.assertRaises(subprocess.CalledProcessError):
            build_release(self.fixture.root, "v0.1.0", self.runner)
        self.assertFalse(
            any(call[:2] == ["cargo", "package"] for call in self.runner.calls)
        )

    def test_featureless_release_does_not_repeat_identical_cargo_tests(self) -> None:
        build_release(self.fixture.root, "v0.1.0", self.runner)
        tests = [call for call in self.runner.calls if call[:2] == ["cargo", "test"]]
        self.assertEqual(len(tests), 2)
        self.assertFalse(any("--all-features" in call for call in tests))

    def test_unformatted_included_tests_refuse_release_before_packaging(self) -> None:
        self.runner.unformatted_inputs = True
        with self.assertRaises(subprocess.CalledProcessError) as failure:
            build_release(self.fixture.root, "v0.1.0", self.runner)
        self.assertEqual(failure.exception.returncode, 3)
        self.assertFalse(
            any(call[:2] == ["cargo", "package"] for call in self.runner.calls)
        )

    def test_cargo_metadata_preserves_available_features(self) -> None:
        self.runner.features = {"default": [], "debug": ["default"]}
        crate = release_inventory(self.fixture.root, self.runner)[0]
        self.assertEqual(crate.features, ("debug", "default"))

    def test_malformed_features_cannot_silently_skip_release_tests(self) -> None:
        for features in (None, [], {"debug": "dependency"}, {"debug": [12]}, {12: []}):
            with self.subTest(features=features), self.assertRaisesRegex(
                ValueError, "expected"
            ):
                release_feature_names({"features": features})

    def test_feature_bearing_release_still_runs_both_profiles(self) -> None:
        self.runner.features = {"debug": []}
        build_release(self.fixture.root, "v0.1.0", self.runner)
        tests = [call for call in self.runner.calls if call[:2] == ["cargo", "test"]]
        self.assertEqual(len(tests), 4)
        self.assertEqual(sum("--all-features" in call for call in tests), 2)

    def test_malformed_package_fields_are_rejected_before_profile_selection(
        self,
    ) -> None:
        for field in ("name", "version", "manifest_path"):
            package = {
                "name": CRATE.name,
                "version": CRATE.version,
                "manifest_path": str(
                    self.fixture.root / CRATE.directory / "Cargo.toml"
                ),
                "features": {},
            }
            package[field] = 12
            with self.subTest(field=field), self.assertRaisesRegex(
                ValueError, "expected a string"
            ):
                release_crate_from_metadata(package, self.fixture.root)

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

    def test_post_test_source_or_tag_changes_prevent_sealing(self) -> None:
        for mutation in ["dirty", "moved"]:
            runner = LateReleaseMutation(self.fixture)
            runner.mutation = mutation
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                build_release(self.fixture.root, "v0.1.0", runner)
            self.assertFalse((self.fixture.root / "target/releases/v0.1.0").exists())

    def test_packaging_cannot_update_the_owner_lockfile(self) -> None:
        build_release(self.fixture.root, "v0.1.0", self.runner)
        package = next(
            call for call in self.runner.calls if call[:2] == ["cargo", "package"]
        )
        self.assertIn("--locked", package)

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

    def test_each_profile_uses_and_cleans_its_own_target_directory(self) -> None:
        build_release(self.fixture.root, "v0.1.0", self.runner)
        cargo_calls = [
            call
            for call in self.runner.calls
            if call[:2] in (["cargo", "test"], ["cargo", "clean"])
        ]
        self.assertEqual(
            [call[1] for call in cargo_calls],
            ["test", "test", "clean"],
        )
        for call in cargo_calls:
            if call[1] == "test":
                self.assertNotIn("--workspace", call)
                self.assertEqual(call[call.index("--package") + 1], CRATE.name)
        targets = {call[call.index("--target-dir") + 1] for call in cargo_calls}
        self.assertEqual(len(targets), 1)
        self.assertTrue(next(iter(targets)).endswith("workspace/target"))

    def test_release_reuses_dependencies_across_packages_and_cleans_after_profiles(
        self,
    ) -> None:
        second = ReleaseCrate("rustycad-cmath", "0.1.0", "crates/cmath", ("debug",))
        self.fixture.write_archive(
            f"{second.name}-{second.version}/Cargo.toml", crate=second
        )
        package_test_support(self.fixture.root, self.runner)
        test_archives(
            self.fixture.root, self.fixture.root / "stage", self.runner, [CRATE, second]
        )
        calls = [call for call in self.runner.calls if call[0] == "cargo"]
        self.assertEqual(
            [call[1] for call in calls],
            ["test", "test", "test", "test", "clean", "test", "test", "clean"],
        )
        tests = [call for call in calls if call[1] == "test"]
        for index, package in enumerate([CRATE, second, second]):
            pair = tests[index * 2 : index * 2 + 2]
            self.assertEqual(
                [call[call.index("--package") + 1] for call in pair], [package.name] * 2
            )
            self.assertIn("--all-targets", pair[0])
            self.assertIn("--doc", pair[1])
            self.assertEqual("--all-features" in pair[0], index >= 2)

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

    def test_test_support_is_independent_of_checkout_timestamps(self) -> None:
        package_test_support(self.fixture.root, self.runner)
        archive = self.fixture.root / "target/package/martensite-test-support.tar.gz"
        first = archive.read_bytes()
        os.utime(self.fixture.root / "tools/oracle/check.expected", (10, 10))
        package_test_support(self.fixture.root, self.runner)
        self.assertEqual(first, archive.read_bytes())
        member = tarfile.TarInfo("linked-fixture")
        member.type = tarfile.SYMTYPE
        with self.assertRaisesRegex(ValueError, "regular tracked file"):
            release_metadata(member)

    def test_system_commands_propagates_failures(self) -> None:
        runner = SystemCommands()
        self.assertIn(
            "git version", runner.run(["git", "--version"], self.fixture.root)
        )
        with self.assertRaises(subprocess.CalledProcessError):
            runner.run(["git", "rev-parse", "HEAD"], self.fixture.root)


if __name__ == "__main__":
    unittest.main()
