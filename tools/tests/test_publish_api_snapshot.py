"""Published evidence must remain deterministic and bound to the crosswalk inputs."""

import gzip
import json
from pathlib import Path
import tempfile
import unittest

from api_inventory.inputs import digest
from publish_api_snapshot import compress_snapshot, publish_snapshot


class SnapshotPublicationTests(unittest.TestCase):
    def test_compression_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "raw.json"
            source.write_text('{"example":true}')
            first = compress_snapshot(source, root / "a.gz")
            second = compress_snapshot(source, root / "b.gz")
            self.assertEqual(first["sha256"], second["sha256"])
            self.assertEqual(
                gzip.decompress((root / "a.gz").read_bytes()), source.read_bytes()
            )

    def test_summary_keeps_gaps_and_rejects_changed_rust(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rust, report = root / "rust.json", root / "report.json"
            rust.write_text("{}")
            report.write_text(
                json.dumps(
                    {
                        "input_sha256": {"rust": digest(rust)},
                        "crosswalk_rows": [],
                        "proof_gaps": ["missing"],
                        "declaration_crosswalk_proof": False,
                    }
                )
            )
            publish_snapshot(rust, report, root / "out")
            summary = json.loads((root / "out/completeness.json").read_text())
            self.assertEqual(summary["proof_gaps"], ["missing"])
            self.assertNotIn("crosswalk_rows", summary)
            rust.write_text("changed")
            with self.assertRaisesRegex(ValueError, "expected the exact"):
                publish_snapshot(rust, report, root / "out")
