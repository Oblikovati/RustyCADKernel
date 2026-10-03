"""Crosswalk manifests must identify the exact declaration bytes being counted."""

from pathlib import Path
import tempfile
import unittest

from api_inventory.inputs import digest
from api_report_inputs import (
    crosswalk_input_hashes,
    validate_header_manifest,
    mark_conflicted_declarations,
)
from api_crosswalk import cpp_candidates
from api_coverage_report import implemented_declaration_ids


class CrosswalkInputTests(unittest.TestCase):
    def test_conflicting_compiler_identity_cannot_resolve(self) -> None:
        declaration = {"id": "one", "api_visible": True, "_toolkits": ["TKernel"]}
        mark_conflicted_declarations([declaration], [{"id": "one"}])
        status, _ = cpp_candidates("Thing", "TKernel", {"Thing": [declaration]})
        self.assertEqual(status, "conflicting_inventory")

    def test_header_manifest_must_match_compiler_capture(self) -> None:
        validate_header_manifest(
            {"headers": "expected"}, {"inputs": {"manifest_sha256": "expected"}}
        )
        with self.assertRaisesRegex(ValueError, "expected compiler input"):
            validate_header_manifest(
                {"headers": "changed"}, {"inputs": {"manifest_sha256": "expected"}}
            )

    def test_repeated_mapping_cannot_inflate_declaration_count(self) -> None:
        rows = [
            {"resolution": "implemented", "cpp_ids": ["one"]},
            {"resolution": "implemented", "cpp_ids": ["one"]},
            {"resolution": "partial", "cpp_ids": ["two"]},
        ]
        self.assertEqual(implemented_declaration_ids(rows), {"one"})

    def test_changed_cpp_artifact_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "cpp.gz"
            artifact.write_bytes(b"fixture")
            manifest = {"artifacts": {"declarations": {"sha256": digest(artifact)}}}
            self.assertEqual(
                crosswalk_input_hashes({"cpp": artifact}, manifest)["cpp"],
                digest(artifact),
            )
            artifact.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "expected"):
                crosswalk_input_hashes({"cpp": artifact}, manifest)

    def test_missing_artifact_metadata_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected hash metadata"):
            crosswalk_input_hashes({}, {})


if __name__ == "__main__":
    unittest.main()
