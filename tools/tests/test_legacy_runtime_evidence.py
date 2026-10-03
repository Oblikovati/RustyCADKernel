"""Fail-closed freshness checks for aggregate legacy runtime reports."""

from __future__ import annotations

import pathlib
import subprocess
import tempfile
import unittest

from api_inventory.inputs import JSONObject
from legacy_runtime_evidence import (
    HASHED_FIXED_INPUTS,
    assess_report,
    file_digest,
    tracked_runtime_inputs,
)


class LegacyRuntimeEvidenceTest(unittest.TestCase):
    def test_matching_aggregate_report_stays_unverified(self) -> None:
        """A current report still lacks declaration-level proof; e.g. this fixture."""
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            source = pathlib.Path(temporary) / "OCCT"
            root.mkdir()
            source.mkdir()
            initialize_occt(source)
            revision = git(source, "rev-parse", "HEAD")
            create_fixture_inputs(root, revision)
            report = report_for(root, revision=revision)
            result = assess_report(report, root, source)
        self.assertEqual(result["status"], "aggregate_only")
        self.assertFalse(result["runtime_proof"])
        self.assertEqual(result["declaration_runtime_status"], "unverified")
        self.assertEqual(result["declaration_instance_join"], "unavailable")

    def test_missing_occt_checkout_stays_stale(self) -> None:
        """A pin alone cannot verify source execution; e.g. no checkout is stale."""
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root, "pinned-occt-revision")
            result = assess_report(report_for(root), root)
        self.assertEqual(result["status"], "stale")
        self.assertIn("occt_checkout_not_verified", result["reasons"])

    def test_changed_rust_input_marks_report_stale(self) -> None:
        """Changed test bytes invalidate evidence; e.g. edit one hashed crate file."""
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root, "pinned-occt-revision")
            report = report_for(root)
            target = root / "crates/tkernel/src/lib.rs"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("changed")
            result = assess_report(report, root)
        self.assertEqual(result["status"], "stale")
        self.assertIn(
            "runtime_input_changed:crates/tkernel/src/lib.rs", result["reasons"]
        )

    def test_missing_helper_hash_marks_report_stale(self) -> None:
        """Required direct verifier modules must be fingerprinted; e.g. baseline helper."""
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root, "pinned-occt-revision")
            report = report_for(root)
            del report["source_hashes"]["tools/tkernel_native_baseline.py"]
            result = assess_report(report, root)
        self.assertEqual(result["status"], "stale")
        self.assertTrue(
            any("tkernel_native_baseline.py" in row for row in result["reasons"])
        )

    def test_missing_transitive_helper_hash_marks_report_stale(self) -> None:
        """Imported status helpers are covered; e.g. missing rustdoc fingerprint tool."""
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root, "pinned-occt-revision")
            report = report_for(root)
            del report["source_hashes"]["tools/capture_rustdoc_json.py"]
            result = assess_report(report, root)
        self.assertEqual(result["status"], "stale")
        self.assertTrue(
            any("capture_rustdoc_json.py" in reason for reason in result["reasons"])
        )

    def test_case_rows_are_not_joined_without_declaration_identity(self) -> None:
        """Unlinked runtime rows cannot green declarations; e.g. name/status only."""
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            source = pathlib.Path(temporary) / "OCCT"
            root.mkdir()
            source.mkdir()
            initialize_occt(source)
            revision = git(source, "rev-parse", "HEAD")
            create_fixture_inputs(root, revision)
            report = report_for(root, revision=revision)
            report["cases"] = [{"name": "Some.Test", "status": "passed"}]
            result = assess_report(report, root, source)
        self.assertEqual(result["status"], "granular_unjoined")
        self.assertFalse(result["runtime_proof"])

    def test_occt_revision_and_cleanliness_are_required(self) -> None:
        """The baseline pin and clean source checkout must match; e.g. a dirty tree fails."""
        with tempfile.TemporaryDirectory() as temporary:
            root, source, revision = pinned_fixture(pathlib.Path(temporary))
            report = report_for(root, revision=revision)
            clean = assess_report(report, root, source)
            (source / "dirty.txt").write_text("uncommitted")
            dirty = assess_report(report, root, source)
        self.assertEqual(clean["status"], "aggregate_only")
        self.assertEqual(dirty["status"], "stale")
        self.assertIn("occt_source_dirty", dirty["reasons"])

    def test_report_revision_must_match_repository_pin(self) -> None:
        """Reject old source reports; e.g. report SHA differs from baseline.json."""
        with tempfile.TemporaryDirectory() as temporary:
            root, source, revision = pinned_fixture(pathlib.Path(temporary))
            report = report_for(root, revision=revision)
            report["revision"] = "different-pinned-revision"
            result = assess_report(report, root, source)
        self.assertEqual(result["status"], "stale")
        self.assertIn("report_revision_differs_from_pinned_occt", result["reasons"])

    def test_historical_report_shape_without_hashes_is_invalid(self) -> None:
        """Incomplete aggregate records are invalid; e.g. missing hash table."""
        result = assess_report({"revision": "x"}, pathlib.Path("."))
        self.assertEqual(result["status"], "invalid")

    def test_negative_aggregate_count_is_invalid(self) -> None:
        """Negative report counts are malformed; e.g. count -1 is invalid."""
        report: JSONObject = {
            "revision": "x",
            "source_hashes": {},
            "legacy_tests_passed": -1,
        }
        result = assess_report(report, pathlib.Path("."))
        self.assertEqual(result["status"], "invalid")


def report_for(
    root: pathlib.Path, revision: str = "pinned-occt-revision"
) -> JSONObject:
    """Make current verifier evidence; e.g. `report_for(root, revision)`."""
    hashes: JSONObject = {}
    for relative in tracked_runtime_inputs(root):
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists():
            path.write_text(f"fixture:{relative}")
        hashes[relative] = file_digest(path)
    return {
        "revision": revision,
        "source_hashes": hashes,
        "legacy_tests_passed": 1,
        "declarations_mapped": 1,
        "rust_translations_passed": 1,
        "rust_runtime_cases_passed": 1,
    }


def create_fixture_inputs(root: pathlib.Path, revision: str) -> None:
    """Write the fixture's verifier files; e.g. `create_fixture_inputs(root, sha)`."""
    required = [*HASHED_FIXED_INPUTS, "crates/tkernel/src/lib.rs"]
    for relative in required:
        write_fixture_file(root / relative, relative)
    write_fixture_file(
        root / "porting/baseline.json", '{"revision":"' + revision + '"}'
    )


def write_fixture_file(path: pathlib.Path, contents: str) -> None:
    """Create a fixture input and parents; e.g. `write_fixture_file(path, 'contents')`."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contents)


def git(source: pathlib.Path, *arguments: str) -> str:
    """Run git for source fixtures; e.g. `git(source, 'rev-parse', 'HEAD')`."""
    result = subprocess.run(
        ["git", "-C", str(source), *arguments],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def initialize_occt(source: pathlib.Path) -> None:
    """Create a clean source fixture; e.g. `initialize_occt(source)`."""
    git(source, "init", "-q")
    git(
        source,
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.invalid",
        "commit",
        "--allow-empty",
        "-qm",
        "base",
    )


def pinned_fixture(temporary: pathlib.Path) -> tuple[pathlib.Path, pathlib.Path, str]:
    """Create pinned source and Rust fixtures; e.g. `pinned_fixture(Path(tmp))`."""
    root, source = temporary / "rust", temporary / "OCCT"
    root.mkdir()
    source.mkdir()
    initialize_occt(source)
    revision = git(source, "rev-parse", "HEAD")
    create_fixture_inputs(root, revision)
    return root, source, revision


if __name__ == "__main__":
    unittest.main()
