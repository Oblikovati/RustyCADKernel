"""Fail-closed freshness checks for aggregate legacy runtime reports."""
from __future__ import annotations

import pathlib
import subprocess
import tempfile
import unittest

from legacy_runtime_evidence import (
    REQUIRED_VERIFIER_HELPERS,
    assess_report,
    sha256_file,
    tracked_runtime_inputs,
)


class LegacyRuntimeEvidenceTest(unittest.TestCase):
    def test_matching_aggregate_report_stays_unverified(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root)
            report = report_for(root)
            result = assess_report(report, root)
        self.assertEqual(result["status"], "aggregate_only")
        self.assertFalse(result["runtime_proof"])
        self.assertEqual(result["declaration_runtime_status"], "unverified")
        self.assertEqual(result["declaration_instance_join"], "unavailable")

    def test_changed_rust_input_marks_report_stale(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root)
            report = report_for(root)
            target = root / "crates/tkernel/src/lib.rs"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("changed")
            result = assess_report(report, root)
        self.assertEqual(result["status"], "stale")
        self.assertIn("runtime_input_changed:crates/tkernel/src/lib.rs", result["reasons"])

    def test_missing_helper_hash_marks_report_stale(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root)
            report = report_for(root)
            del report["source_hashes"]["tools/tkernel_native_baseline.py"]
            result = assess_report(report, root)
        self.assertEqual(result["status"], "stale")
        self.assertTrue(any("tkernel_native_baseline.py" in row for row in result["reasons"]))

    def test_case_rows_are_not_joined_without_declaration_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            root.mkdir()
            create_fixture_inputs(root)
            report = report_for(root)
            report["cases"] = [{"name": "Some.Test", "status": "passed"}]
            result = assess_report(report, root)
        self.assertEqual(result["status"], "granular_unjoined")
        self.assertFalse(result["runtime_proof"])

    def test_occt_revision_and_cleanliness_are_required(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "rust"
            source = pathlib.Path(temporary) / "OCCT"
            root.mkdir()
            source.mkdir()
            create_fixture_inputs(root)
            git(source, "init", "-q")
            git(source, "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "--allow-empty", "-qm", "base")
            revision = git(source, "rev-parse", "HEAD")
            report = report_for(root, revision=revision)
            clean = assess_report(report, root, source)
            report["revision"] = "different-pinned-revision"
            mismatched = assess_report(report, root, source)
            (source / "dirty.txt").write_text("uncommitted")
            dirty = assess_report(report, root, source)
        self.assertEqual(clean["status"], "aggregate_only")
        self.assertIn("occt_revision_mismatch", mismatched["reasons"])
        self.assertEqual(dirty["status"], "stale")
        self.assertIn("occt_source_dirty", dirty["reasons"])

    def test_historical_report_shape_without_hashes_is_invalid(self) -> None:
        self.assertEqual(assess_report({"revision": "x"}, pathlib.Path("."))["status"], "invalid")


def report_for(root: pathlib.Path, revision: str = "pinned-occt-revision") -> dict:
    hashes = {}
    for relative in tracked_runtime_inputs(root):
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"fixture:{relative}")
        hashes[relative] = sha256_file(path)
    return {
        "revision": revision,
        "source_hashes": hashes,
        "legacy_tests_passed": 1,
        "declarations_mapped": 1,
        "rust_translations_passed": 1,
        "rust_runtime_cases_passed": 1,
    }


def create_fixture_inputs(root: pathlib.Path) -> None:
    required = [
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "porting/units-data.json",
        "porting/baseline.json",
        "porting/parity.json", "crates/tkernel/src/lib.rs",
        "tools/verify_geometry.py", "tools/verify_units.py",
        "tools/generate_units_data.py", "tools/generate_quantity_color_table.py",
        "tools/generate_resource_tables.py", "tools/verify_kdtree.py",
        "tools/verify_tkernel.py", "tools/oracle/tkernel_probe.cxx",
        "tools/oracle/strtod_probe.cxx", "tools/verify_strtod.py",
        "tools/verify_ascii_string.py", "tools/oracle/ascii_string_probe.cxx",
        "tools/verify_extended_string.py", "tools/oracle/extended_string_probe.cxx",
        "tools/verify_tkernel_api.py", *REQUIRED_VERIFIER_HELPERS,
    ]
    for relative in required:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"fixture:{relative}")


def git(source: pathlib.Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(source), *arguments], check=True, capture_output=True, text=True
    )
    return result.stdout.strip()


if __name__ == "__main__":
    unittest.main()
