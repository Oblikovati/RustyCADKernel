"""Publication regression checks without remote writes; run through run_tool_tests.py."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from publish_release import SystemGitHub, json_object, publish, validate_identity
from release_archives import write_receipt

REVISION = "a" * 40
REPOSITORY = "Oblikovati/RustyCADKernel"
TAG = "v0.2.4"


class FakeGitHub:
    """Named remote whose uploaded bytes survive a failed edit."""

    def __init__(self, assets: Path) -> None:
        self.assets = assets
        self.existing: dict[str, object] | None = None
        self.remote: dict[str, bytes] = {}
        self.arguments: list[list[str]] = []
        self.fail_edit = False
        self.tag_revision = REVISION
        self.annotated = False
        self.move_after_create = False

    def release(self, repository: str, tag: str) -> dict[str, object] | None:
        if repository != REPOSITORY or tag != TAG:
            raise AssertionError((repository, tag))
        return self.existing

    def run(self, arguments: list[str]) -> str:
        self.arguments.append(arguments)
        if arguments[1] == "api":
            kind = "tag" if self.annotated and "/git/ref/" in arguments[2] else "commit"
            return json.dumps({"object": {"type": kind, "sha": self.tag_revision}})
        operation = arguments[2]
        if operation == "create":
            self.create_draft()
        elif operation == "download":
            self.download(Path(arguments[arguments.index("--dir") + 1]))
        elif operation == "edit":
            self.edit_draft()
        else:
            raise AssertionError(arguments)
        return ""

    def create_draft(self) -> None:
        self.remote = {path.name: path.read_bytes() for path in self.assets.iterdir()}
        self.existing = {
            "draft": True,
            "tag_name": TAG,
            "assets": [{"name": name} for name in self.remote],
        }
        if self.move_after_create:
            self.tag_revision = "b" * 40

    def download(self, destination: Path) -> None:
        for name, contents in self.remote.items():
            (destination / name).write_bytes(contents)
        return None

    def edit_draft(self) -> None:
        if self.fail_edit:
            raise subprocess.CalledProcessError(1, ["gh", "release", "edit"])
        if self.existing is None:
            raise AssertionError("expected created draft")
        self.existing["draft"] = False

    def writes(self) -> list[str]:
        return [
            arguments[2]
            for arguments in self.arguments
            if arguments[1] == "release" and arguments[2] in ("create", "edit")
        ]


class PublicationFixture(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.assets = Path(self.temporary.name) / "assets"
        self.assets.mkdir()
        (self.assets / "kernel.crate").write_bytes(b"checked archive")
        write_receipt(self.assets, "0.2.4", REVISION)
        self.github = FakeGitHub(self.assets)

    def run_publish(self) -> None:
        publish(self.assets, TAG, REVISION, REPOSITORY, self.github)
        return None

    def rewrite_receipt(self, key: str, value: object) -> None:
        path = self.assets / "martensite-release.json"
        receipt = json.loads(path.read_text())
        receipt[key] = value
        path.write_text(json.dumps(receipt))
        from release_archives import file_sha256

        hashes = [
            (entry.name, file_sha256(entry))
            for entry in sorted(self.assets.iterdir())
            if entry.name != "SHA256SUMS"
        ]
        (self.assets / "SHA256SUMS").write_text(
            "".join(f"{sha}  {name}\n" for name, sha in hashes)
        )

    def assert_no_write_failure(self, message: str) -> None:
        with self.assertRaisesRegex(ValueError, message):
            self.run_publish()
        self.assertEqual(self.github.writes(), [])

    def test_new_release_uses_explicit_owner_and_verified_tag(self) -> None:
        self.run_publish()
        self.assertEqual(self.github.writes(), ["create", "edit"])
        create = next(
            command for command in self.github.arguments if command[2] == "create"
        )
        self.assertIn("--verify-tag", create)
        self.assertEqual(create[create.index("--repo") + 1], REPOSITORY)

    def test_failed_edit_recovers_identical_draft_without_reupload(self) -> None:
        self.github.fail_edit = True
        with self.assertRaises(subprocess.CalledProcessError):
            self.run_publish()
        self.github.fail_edit = False
        self.run_publish()
        self.assertEqual(self.github.writes(), ["create", "edit", "edit"])
        self.assertIs(self.github.existing["draft"], False)

    def test_published_release_is_never_modified(self) -> None:
        self.github.create_draft()
        self.github.existing["draft"] = False
        self.assert_no_write_failure("unpublished draft")

    def test_draft_with_changed_bytes_is_never_modified(self) -> None:
        self.github.create_draft()
        self.github.remote["kernel.crate"] = b"other bytes"
        self.assert_no_write_failure("byte-identical")

    def test_draft_asset_name_set_must_match_exactly(self) -> None:
        for assets in (
            [],
            [{"name": "kernel.crate"}],
            [{"name": "bad"}],
            [{"name": "kernel.crate"}, {"name": "kernel.crate"}],
        ):
            with self.subTest(assets=assets):
                self.github.create_draft()
                self.github.existing["assets"] = assets
                self.assert_no_write_failure("asset")

    def test_draft_wrong_tag_and_malformed_assets_are_rejected(self) -> None:
        for key, value in (
            ("tag_name", "v0.2.3"),
            ("assets", {}),
            ("assets", [None]),
            ("assets", [{"name": []}]),
        ):
            with self.subTest(key=key, value=value):
                self.github.create_draft()
                self.github.existing[key] = value
                self.assert_no_write_failure("draft|asset")

    def test_receipt_owner_fields_checked_even_with_valid_checksums(self) -> None:
        for key, value in (
            ("format", True),
            ("repository", "other/owner"),
            ("revision", "b" * 40),
            ("version", "0.2.3"),
            ("artifacts", {}),
            ("validation", "untested"),
        ):
            with self.subTest(key=key):
                write_receipt(self.assets, "0.2.4", REVISION)
                self.rewrite_receipt(key, value)
                self.assert_no_write_failure("receipt")

    def test_corrupted_missing_and_unsealed_assets_are_rejected(self) -> None:
        (self.assets / "kernel.crate").write_bytes(b"corrupted")
        self.assert_no_write_failure("checksum")
        write_receipt(self.assets, "0.2.4", REVISION)
        (self.assets / "kernel.crate").unlink()
        self.assert_no_write_failure("checksum")
        (self.assets / "extra").write_bytes(b"extra")
        self.assert_no_write_failure("checksum")

    def test_bad_duplicate_and_traversal_checksums_are_rejected(self) -> None:
        valid = (self.assets / "SHA256SUMS").read_text()
        for text in ("malformed", valid + valid, f"{'a' * 64}  ../outside\n"):
            (self.assets / "SHA256SUMS").write_text(text)
            self.assert_no_write_failure("checksum")

    def test_non_regular_asset_is_rejected(self) -> None:
        (self.assets / "link").symlink_to("kernel.crate")
        self.assert_no_write_failure("regular flat")
        (self.assets / "link").unlink()
        (self.assets / "directory").mkdir()
        self.assert_no_write_failure("regular flat")

    def test_moved_tag_is_rejected_before_create(self) -> None:
        self.github.tag_revision = "b" * 40
        self.assert_no_write_failure("remote tag")

    def test_annotated_tag_is_peeled_to_exact_owner(self) -> None:
        self.github.annotated = True
        self.run_publish()
        self.assertTrue(any("/git/tags/" in args[2] for args in self.github.arguments))

    def test_tag_movement_after_draft_creation_never_publishes(self) -> None:
        self.github.move_after_create = True
        with self.assertRaisesRegex(ValueError, "remote tag"):
            self.run_publish()
        self.assertEqual(self.github.writes(), ["create"])

    def test_identity_and_json_shape_rejected(self) -> None:
        for tag, revision, repo in (
            ("--bad", REVISION, REPOSITORY),
            (TAG, "short", REPOSITORY),
            (TAG, REVISION, "../bad"),
        ):
            with self.assertRaisesRegex(ValueError, "identity"):
                validate_identity(tag, revision, repo)
        with self.assertRaisesRegex(ValueError, "expected an object"):
            json_object("[]")

    def test_cli_help_is_read_only(self) -> None:
        result = subprocess.run(
            [sys.executable, "tools/publish_release.py", "--help"],
            check=True,
            text=True,
            capture_output=True,
        )
        self.assertIn("--revision", result.stdout)
        self.assertIn("--repository", result.stdout)

    def test_system_transport_preserves_failure_and_distinguishes_absence(self) -> None:
        fake = subprocess.CompletedProcess(["gh"], 1, "", "release not found")
        with patch("publish_release.subprocess.run", return_value=fake):
            self.assertIsNone(SystemGitHub().release(REPOSITORY, TAG))
        fake.stderr = "gh: unauthorized (HTTP 401)"
        with patch("publish_release.subprocess.run", return_value=fake):
            with self.assertRaises(subprocess.CalledProcessError):
                SystemGitHub().release(REPOSITORY, TAG)
        fake.returncode, fake.stdout = (
            0,
            '{"isDraft": true, "tagName": "v0.2.4", "assets": []}',
        )
        with patch("publish_release.subprocess.run", return_value=fake):
            self.assertTrue(SystemGitHub().release(REPOSITORY, TAG)["draft"])
            self.assertEqual(SystemGitHub().run(["gh", "--version"]), fake.stdout)


if __name__ == "__main__":
    unittest.main()
