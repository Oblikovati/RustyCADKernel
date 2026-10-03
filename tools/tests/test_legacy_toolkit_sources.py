"""Fail-closed checks for complete OCCT toolkit test source manifests."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from legacy_toolkit_sources import (
    GTEST_MANIFESTS,
    group_source_rows,
    listed_test_sources,
    require_one_toolkit,
    require_supported_toolkit,
    toolkit_rows,
    verify_source_rows,
)
from api_inventory.inputs import JSONObject


class LegacyToolkitSourcesTest(unittest.TestCase):
    def test_manifest_and_every_declaration_are_mapped(self) -> None:
        """CMake source files and declarations must exactly match rows."""
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            rows = source_rows(source, ["Geom2d_Line_Test.cxx"])
            self.assertEqual(
                verify_source_rows(source, rows),
                {"src/ModelingData/TKG2d/GTests/Geom2d_Line_Test.cxx"},
            )

    def test_geom_base_manifest_and_every_declaration_are_mapped(self) -> None:
        """Geometry base proofs require their complete declared GTest source set."""
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            rows = source_rows(source, ["GeomConvert_Test.cxx"], "TKGeomBase")
            self.assertEqual(
                verify_source_rows(source, rows),
                {"src/ModelingData/TKGeomBase/GTests/GeomConvert_Test.cxx"},
            )

    def test_cmake_manifest_rejects_an_unmapped_test_file(self) -> None:
        """An unrepresented FILES.cmake entry blocks toolkit runtime proof."""
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            rows = source_rows(source, ["Geom2d_Line_Test.cxx"])
            manifest = source / GTEST_MANIFESTS["TKG2d"]
            manifest.write_text(
                manifest.read_text().replace(
                    "Geom2d_Line_Test.cxx", "Geom2d_Line_Test.cxx\n  Extra_Test.cxx"
                )
            )
            with self.assertRaisesRegex(RuntimeError, "test source coverage differs"):
                verify_source_rows(source, rows)

    def test_stale_source_digest_blocks_toolkit_proof(self) -> None:
        """Changed source bytes invalidate an otherwise matching declaration row."""
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            rows = source_rows(source, ["Geom2d_Line_Test.cxx"])
            rows[0]["source_sha256"] = "0" * 64
            with self.assertRaisesRegex(RuntimeError, "Stale or incomplete TKG2d"):
                verify_source_rows(source, rows)

    def test_duplicate_declaration_mapping_is_rejected(self) -> None:
        """One source declaration cannot inflate the runtime row count twice."""
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            rows = source_rows(source, ["Geom2d_Line_Test.cxx"])
            rows.append(dict(rows[0]))
            with self.assertRaisesRegex(RuntimeError, "Duplicate TKG2d mapping"):
                verify_source_rows(source, rows)

    def test_mixed_toolkit_rows_are_rejected(self) -> None:
        """Rows from different toolkits cannot be joined into one runtime report."""
        with self.assertRaisesRegex(ValueError, "without mixed toolkits"):
            require_one_toolkit([{"toolkit": "TKG2d"}, {"toolkit": "TKG3d"}], "TKG2d")

    def test_unconfigured_toolkit_is_rejected(self) -> None:
        """Only toolkits with explicit CMake GTest manifests are accepted."""
        with self.assertRaisesRegex(ValueError, "Unsupported runtime toolkit"):
            require_supported_toolkit("TKBO")

    def test_toolkit_rows_require_translated_entries(self) -> None:
        """Only rows marked translated can contribute runtime evidence."""
        with tempfile.TemporaryDirectory() as temporary:
            ledger = Path(temporary) / "parity.json"
            ledger.write_text(
                json.dumps({"tests": [{"toolkit": "TKG2d", "status": "unmapped"}]})
            )
            with self.assertRaisesRegex(
                RuntimeError, "must be nonempty and translated"
            ):
                toolkit_rows(ledger, "TKG2d")

    def test_grouping_keeps_distinct_source_rows(self) -> None:
        """Multiple declarations stay grouped by their exact source path."""
        rows: list[JSONObject] = [
            {"source": "one.cxx"},
            {"source": "one.cxx"},
            {"source": "two.cxx"},
        ]
        self.assertEqual(set(group_source_rows(rows)), {"one.cxx", "two.cxx"})

    def test_manifest_parser_uses_test_source_filenames(self) -> None:
        """CMake FILES.cmake filenames resolve under its own directory."""
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            relative = Path(GTEST_MANIFESTS["TKG2d"])
            manifest = source / relative
            manifest.parent.mkdir(parents=True)
            manifest.write_text("set(files\n  Geom2d_Line_Test.cxx\n)\n")
            self.assertEqual(
                listed_test_sources(source, manifest),
                {relative.parent.as_posix() + "/Geom2d_Line_Test.cxx"},
            )


def source_rows(
    source: Path, filenames: list[str], toolkit: str = "TKG2d"
) -> list[JSONObject]:
    """Create a small manifest fixture; e.g. `source_rows(tmp, ['One_Test.cxx'])`."""
    relative = Path(GTEST_MANIFESTS[toolkit])
    manifest = source / relative
    manifest.parent.mkdir(parents=True, exist_ok=True)
    manifest.write_text(
        "set(files\n" + "".join(f"  {name}\n" for name in filenames) + ")\n"
    )
    rows = []
    for filename in filenames:
        contents = b"TEST(Geom2d_Line, Eval) {}\n"
        (manifest.parent / filename).write_bytes(contents)
        rows.append(
            {
                "toolkit": toolkit,
                "source": (relative.parent / filename).as_posix(),
                "upstream_id": "Geom2d_Line.Eval",
                "source_sha256": hashlib.sha256(contents).hexdigest(),
            }
        )
    return rows


if __name__ == "__main__":
    unittest.main()
