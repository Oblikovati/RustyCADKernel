"""Crosswalk manifests must identify the exact declaration bytes being counted."""

from pathlib import Path
import tempfile
import unittest

from api_inventory.inputs import digest
from api_report_inputs import crosswalk_input_hashes


class CrosswalkInputTests(unittest.TestCase):
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
