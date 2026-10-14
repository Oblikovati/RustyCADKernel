"""Named effect fixtures bind tag receipts to original source and sealed assets."""

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from pre_release_operations import NativePreReleaseOperations
from release_archives import write_receipt

REVISION = "a" * 40
PLAN = b'GATE_NAMES = ("required",)\n\n'


class NativeReleaseFilesFixture:
    """Own tiny valid sealed payloads without compiling any kernel code."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.checkout = root / "checkout"
        self.evidence = root / "evidence"
        self.checkout.mkdir()
        self.evidence.mkdir()
        self.original_assets = self.checkout / "target/releases/v0.2.7"
        self.original_assets.mkdir(parents=True)
        for index in range(31):
            (self.original_assets / f"fixture-{index}.crate").write_bytes(
                b"owned fixture"
            )
        for name in (
            "LICENSE",
            "Cargo.lock",
            "rust-toolchain.toml",
            "martensite-test-support.tar.gz",
        ):
            (self.original_assets / name).write_bytes(b"owned fixture")
        write_receipt(self.original_assets, "0.2.7", REVISION)


class FakeNativeReleaseEffects:
    """Replace external Git/Cargo/driver effects with named, recorded outcomes."""

    def __init__(self, files: NativeReleaseFilesFixture) -> None:
        self.files = files
        self.revision = REVISION
        self.tags: list[tuple[list[str], Path, dict[str, str]]] = []
        self.disposals: list[Path] = []

    def candidate(self, checkout: Path, tag: str, commands: object) -> Path:
        assert checkout == self.files.checkout and tag == "v0.2.7"
        return self.files.original_assets

    def clean_revision(self, checkout: Path, commands: object) -> str:
        assert checkout == self.files.checkout
        return self.revision

    def git_read(self, argv: list[str], cwd: Path) -> bytes:
        assert argv[:2] == ["git", "show"] and cwd == self.files.checkout
        return PLAN

    def execute(
        self, argv: list[str], cwd: Path, env: dict[str, str], check: bool
    ) -> subprocess.CompletedProcess[str]:
        self.tags.append((argv, cwd, env))
        return subprocess.CompletedProcess(argv, 0)

    def dispose(self, root: Path, checkout: Path) -> None:
        self.disposals.append(checkout)


class PreReleaseNativeOperationsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.files = NativeReleaseFilesFixture(Path(self.temporary.name))
        self.effects = FakeNativeReleaseEffects(self.files)
        self.operations = NativePreReleaseOperations(
            self.files.root, Path("oracle"), {}
        )
        self.operations.evidence = self.files.evidence
        self.operations.revision = REVISION

    def test_untagged_verified_assets_are_moved_outside_disposable_checkout(
        self,
    ) -> None:
        with patch(
            "pre_release_operations.build_release_candidate", new=self.effects.candidate
        ):
            assets = self.operations.package(self.files.checkout, "v0.2.7")
        self.assertEqual(assets, self.files.evidence / "artifacts")
        self.assertEqual(len(list(assets.iterdir())), 37)
        self.assertFalse(self.files.original_assets.exists())

    def test_receipt_hashes_exact_git_blob_including_trailing_newlines(self) -> None:
        self.files.original_assets.rename(self.files.evidence / "artifacts")
        with (
            patch(
                "pre_release_operations.clean_release_revision",
                new=self.effects.clean_revision,
            ),
            patch(
                "pre_release_operations.subprocess.check_output",
                new=self.effects.git_read,
            ),
        ):
            path = self.operations.receipt(
                self.files.checkout, "v0.2.7", self.files.evidence / "artifacts"
            )
        receipt = json.loads(path.read_text())
        self.assertEqual(
            receipt["verification_plan_sha256"], hashlib.sha256(PLAN).hexdigest()
        )
        self.assert_receipt_owner(receipt)

    def assert_receipt_owner(self, receipt: dict[str, object]) -> None:
        self.assertEqual(
            (receipt["revision"], receipt["tag"], receipt["complete"]),
            (REVISION, "v0.2.7", True),
        )

    def test_changed_source_cannot_produce_success_receipt(self) -> None:
        self.effects.revision = "b" * 40
        with patch(
            "pre_release_operations.clean_release_revision",
            new=self.effects.clean_revision,
        ):
            with self.assertRaisesRegex(ValueError, "expected"):
                self.operations.receipt(
                    self.files.checkout, "v0.2.7", self.files.original_assets
                )
        self.assertFalse((self.files.evidence / "acceptance.json").exists())

    def test_tag_command_supplies_receipt_to_guard_and_disposal_is_owned(self) -> None:
        receipt = self.files.evidence / "acceptance.json"
        with patch("pre_release_operations.subprocess.run", new=self.effects.execute):
            self.operations.tag(self.files.checkout, "v0.2.7", receipt)
        self.assertEqual(
            self.effects.tags[0][2]["MARTENSITE_RELEASE_RECEIPT"], str(receipt)
        )
        self.assertEqual(self.effects.tags[0][0][-1], REVISION)
        with patch(
            "pre_release_operations.delete_release_checkout", new=self.effects.dispose
        ):
            self.operations.dispose(self.files.checkout)
        self.assertEqual(self.effects.disposals, [self.files.checkout])


if __name__ == "__main__":
    unittest.main()
