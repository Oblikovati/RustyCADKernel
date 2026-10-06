"""Named file fixtures exercise artifact attestation and current Rust binding selection."""

from __future__ import annotations

import gzip
import json
import tempfile
import unittest
from pathlib import Path

from api_inventory.inputs import JSONObject, digest
from build_effective_api_obligations import (
    require_current_rust,
    select_current_rust,
    verify_access_artifacts,
    write_or_verify,
)


class FakeObligationArtifacts:
    """Own temporary attested files so tests cannot depend on host inventory state."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.contract: JSONObject = {
            "fixture": "fixture",
            "native_report": "report",
            "rust_inventory": "rust",
            "implementation_credit": False,
            "api_denominator_complete": False,
        }
        artifacts = []
        for name in ("fixture", "report", "rust"):
            path = root / name
            path.write_text(name)
            artifacts.append({"path": name, "sha256": digest(path)})
        self.contract["artifacts"] = artifacts

    def rust_row(self) -> JSONObject:
        return {
            "kind": "struct_field",
            "path": "public::Block::count",
            "source_file": "rust",
            "source_sha256": digest(self.root / "rust"),
        }


class EffectiveArtifactTests(unittest.TestCase):
    def test_require_all_mandatory_artifacts_and_current_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fake = FakeObligationArtifacts(Path(temporary))
            verify_access_artifacts(fake.root, fake.contract)
            (fake.root / "fixture").write_text("changed")
            with self.assertRaisesRegex(ValueError, "SHA256"):
                verify_access_artifacts(fake.root, fake.contract)

    def test_reject_omitted_or_duplicated_attestation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fake = FakeObligationArtifacts(Path(temporary))
            original = fake.contract["artifacts"]
            for artifacts in [original[:-1], original + [original[0]]]:
                with self.assertRaisesRegex(ValueError, "unique paths"):
                    verify_access_artifacts(
                        fake.root, {**fake.contract, "artifacts": artifacts}
                    )

    def test_reject_completeness_or_credit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fake = FakeObligationArtifacts(Path(temporary))
            for flag in ["implementation_credit", "api_denominator_complete"]:
                with self.assertRaisesRegex(ValueError, "expected false"):
                    verify_access_artifacts(fake.root, {**fake.contract, flag: True})

    def test_rust_selection_rejects_ambiguity_and_stale_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fake = FakeObligationArtifacts(Path(temporary))
            row = fake.rust_row()
            contract = {"rust_target": {"path": row["path"], "kind": row["kind"]}}
            self.assertEqual(select_current_rust(fake.root, contract, [row]), row)
            for declarations in [[], [row, row]]:
                with self.assertRaisesRegex(ValueError, "one declaration"):
                    select_current_rust(fake.root, contract, declarations)
            (fake.root / "rust").write_text("changed")
            with self.assertRaisesRegex(ValueError, "current source hash"):
                require_current_rust(fake.root, row)

    def test_replay_detects_modified_or_truncated_rows(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "obligations.gz"
            rows: list[JSONObject] = [{"native_id": "one"}, {"native_id": "two"}]
            write_or_verify(path, rows, False)
            write_or_verify(path, rows, True)
            for changed in [rows[:1], [{"native_id": "modified"}], rows + rows]:
                path.write_bytes(
                    gzip.compress(
                        "\n".join(json.dumps(row) for row in changed).encode()
                    )
                )
                with self.assertRaisesRegex(ValueError, "regenerated obligation"):
                    write_or_verify(path, rows, True)


if __name__ == "__main__":
    unittest.main()
