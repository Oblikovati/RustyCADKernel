"""Real reference storage transactions must preserve the exact logical release object."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

ROOT = Path(__file__).resolve().parents[2]
RELEASE = "refs/tags/v0.2.2"


class NamedRefRepository:
    """A historical tag fixture created before installing the actual owner hook."""

    def __init__(self, root: Path, symbolic: bool = False) -> None:
        self.root = root
        self.hooks = root / "fixture-hooks"
        self.environment = dict(os.environ)
        self.environment.pop("MARTENSITE_RELEASE_RECEIPT", None)
        self.run("init", "-q", "-b", "main")
        self.run("commit", "--allow-empty", "-qm", "historical fixture")
        self.run("tag", "-a", "v0.2.2", "-m", "historical release")
        if symbolic:
            self.run("symbolic-ref", RELEASE, "refs/heads/main")
        self.object = self.run("rev-parse", RELEASE).stdout.strip()
        self.hooks = ROOT / ".githooks"

    def command(self, arguments: tuple[str, ...]) -> list[str]:
        return [
            "git",
            "-c",
            f"core.hooksPath={self.hooks}",
            "-c",
            "user.name=Named reference fixture",
            "-c",
            "user.email=fixture@example.invalid",
            *arguments,
        ]

    def run(
        self, *arguments: str, check: bool = True, input_text: str | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            self.command(arguments),
            cwd=self.root,
            env=self.environment,
            capture_output=True,
            text=True,
            check=check,
            input=input_text,
        )

    def snapshot(self) -> str:
        # Full tag-object identity matters: a reannotation can preserve the peeled commit.
        result = self.run("for-each-ref", "--format=%(refname) %(objectname)")
        return result.stdout


class ReferenceStorageTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory(prefix="martensite-ref-maintenance-")
        self.addCleanup(temporary.cleanup)
        self.fixture = NamedRefRepository(Path(temporary.name))
        self.original = self.fixture.snapshot()

    def test_pack_refs_preserves_existing_annotated_release_without_receipt(
        self,
    ) -> None:
        self.fixture.run("pack-refs", "--all")
        self.assertEqual(self.fixture.snapshot(), self.original)
        packed = self.fixture.root / ".git/packed-refs"
        self.assertIn(f"{self.fixture.object} {RELEASE}", packed.read_text())
        self.assertFalse((self.fixture.root / ".git" / RELEASE).exists())
        self.fixture.run("pack-refs", "--all")
        self.assertEqual(self.fixture.snapshot(), self.original)

    def test_gc_preserves_existing_release_without_receipt(self) -> None:
        self.fixture.run("gc", "--quiet")
        self.assertEqual(self.fixture.snapshot(), self.original)
        self.assertTrue((self.fixture.root / ".git/packed-refs").is_file())
        self.assertEqual(
            self.fixture.run("cat-file", "-t", self.fixture.object).stdout, "tag\n"
        )

    def test_new_release_still_requires_receipt(self) -> None:
        result = self.fixture.run("tag", "-a", "v0.2.3", "-m", "new", check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("MARTENSITE_RELEASE_RECEIPT", result.stderr)
        self.assertEqual(self.fixture.snapshot(), self.original)

    def test_reannotation_of_same_commit_remains_rejected(self) -> None:
        result = self.fixture.run(
            "tag", "-f", "-a", "v0.2.2", "-m", "changed", check=False
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("immutable", result.stderr)
        self.assertEqual(self.fixture.snapshot(), self.original)

    def test_retarget_to_different_commit_remains_rejected(self) -> None:
        self.fixture.run("commit", "--allow-empty", "-qm", "new source")
        before = self.fixture.snapshot()
        result = self.fixture.run(
            "tag", "-f", "-a", "v0.2.2", "-m", "changed", check=False
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("immutable", result.stderr)
        self.assertEqual(self.fixture.snapshot(), before)

    def test_noop_row_cannot_hide_new_release_in_same_transaction(self) -> None:
        updates = f"update {RELEASE} {self.fixture.object}\n"
        updates += f"create refs/tags/v0.2.3 {self.fixture.object}\n"
        result = self.fixture.run(
            "update-ref", "--stdin", check=False, input_text=updates
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("MARTENSITE_RELEASE_RECEIPT", result.stderr)
        self.assertEqual(self.fixture.snapshot(), self.original)

    def test_symbolic_alias_cannot_be_replaced_by_same_resolved_object(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = NamedRefRepository(Path(directory), symbolic=True)
            result = fixture.run(
                "update-ref", "--no-deref", RELEASE, fixture.object, check=False
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("expected direct immutable tag", result.stderr)
            self.assertEqual(
                fixture.run("symbolic-ref", RELEASE).stdout, "refs/heads/main\n"
            )


class FakeRefLookup:
    """Named exact-ref reader; unexpected Git queries fail rather than reading the host."""

    def __init__(self, result: bytes | subprocess.CalledProcessError) -> None:
        # Only the named read can succeed; an unexpected host query is a test failure.
        self.result = result
        self.calls: list[tuple[str, ...]] = []

    def read(self, *arguments: str) -> bytes:
        if arguments == ("symbolic-ref", "--quiet", RELEASE):
            raise subprocess.CalledProcessError(1, ["git", *arguments])
        expected = ("rev-parse", "--verify", "--quiet", "--end-of-options", RELEASE)
        if arguments != expected:
            raise AssertionError(f"Git query={arguments!r}: expected {expected!r}")
        self.calls.append(arguments)
        if isinstance(self.result, subprocess.CalledProcessError):
            raise self.result
        return self.result


class LogicalRefAdmissionTests(unittest.TestCase):
    def test_same_object_requires_actual_reference_lookup(self) -> None:
        from release_tag_guard import guard_transaction

        for size in (40, 64):
            oid = "a" * size
            for old in ("0" * size, oid):
                reader = FakeRefLookup((oid + "\n").encode())
                guard_transaction("prepared", f"{old} {oid} {RELEASE}", None, reader)
                self.assertEqual(len(reader.calls), 1)

    def test_existing_different_object_is_immutable_even_with_zero_old(self) -> None:
        from release_tag_guard import guard_transaction

        reader = FakeRefLookup(("a" * 40 + "\n").encode())
        for old in ("0" * 40, "a" * 40):
            with self.assertRaisesRegex(ValueError, "immutable"):
                guard_transaction(
                    "prepared", f"{old} {'b' * 40} {RELEASE}", "/unused", reader
                )

    def test_same_object_with_inconsistent_old_is_rejected(self) -> None:
        from release_tag_guard import guard_transaction

        reader = FakeRefLookup(("a" * 40 + "\n").encode())
        with self.assertRaisesRegex(ValueError, "immutable"):
            guard_transaction(
                "prepared", f"{'b' * 40} {'a' * 40} {RELEASE}", None, reader
            )

    def test_missing_ref_still_requires_original_receipt_gate(self) -> None:
        from release_tag_guard import guard_transaction

        reader = FakeRefLookup(subprocess.CalledProcessError(1, ["git", "rev-parse"]))
        with self.assertRaisesRegex(ValueError, "MARTENSITE_RELEASE_RECEIPT"):
            guard_transaction(
                "prepared", f"{'0' * 40} {'a' * 40} {RELEASE}", None, reader
            )

    def test_read_errors_fail_closed_instead_of_becoming_missing_refs(self) -> None:
        from release_tag_guard import existing_release_reference

        cases = [(128, b"fatal: unavailable"), (1, b"warning: corrupt reference")]
        for code, diagnostic in cases:
            failure = subprocess.CalledProcessError(code, ["git"], stderr=diagnostic)
            with self.assertRaises(subprocess.CalledProcessError) as raised:
                existing_release_reference(RELEASE, FakeRefLookup(failure))
            self.assertIs(raised.exception, failure)

    def test_malformed_existing_object_is_not_noop_evidence(self) -> None:
        from release_tag_guard import existing_release_reference

        for malformed in (b"", b"abc\n", b"a" * 39, b"ref: refs/heads/main\n"):
            with self.assertRaisesRegex(ValueError, "expected full object ID"):
                existing_release_reference(RELEASE, FakeRefLookup(malformed))


if __name__ == "__main__":
    unittest.main()
