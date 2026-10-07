"""Pending-dependency GTest rows: native-only, never counted as a Rust pass, lapse once ported."""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from legacy_pending_dependency import (implemented_classes, is_pending_dependency, pending_dependency_case,
                                       pending_dependency_rows, require_blockers_in_source, runnable_rows,
                                       source_with_fixtures)

DIGEST = "a" * 64


def pending(identity: str = "S.Case", blocked_by: object = ("BRepAlgoAPI_Cut",), **extra: object) -> dict:
    row = {"upstream_id": identity, "toolkit": "TKBRep", "source": "src/S_Test.cxx", "source_sha256": DIGEST,
           "status": "pending_dependency", "blocked_by": list(blocked_by) if isinstance(blocked_by, tuple) else blocked_by}
    row.update(extra)
    return row


def translated(identity: str = "S.Other") -> dict:
    return {"upstream_id": identity, "status": "translated", "rust_package": "p", "rust_target": "t", "rust_test": "c"}


class PendingRowValidationTests(unittest.TestCase):
    def test_a_valid_row_names_its_blockers(self) -> None:
        case = pending_dependency_case(pending(blocked_by=("BRepAlgoAPI_Fuse", "BRepOffsetAPI_ThruSections")))
        self.assertEqual(case.blocked_by, ("BRepAlgoAPI_Fuse", "BRepOffsetAPI_ThruSections"))
        self.assertEqual(case.source, "src/S_Test.cxx")

    def test_blocked_by_must_be_a_nonempty_list_of_unique_class_names(self) -> None:
        for value in ([], None, "BRepAlgoAPI_Cut", ["not a class"], ["BRepAlgoAPI_Cut", "BRepAlgoAPI_Cut"]):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "blocked_by"):
                pending_dependency_case(pending(blocked_by=value))

    def test_a_rust_side_or_native_approval_is_refused(self) -> None:
        for field in ("rust_test", "rust_target", "runtime_cases", "native_expected_failure"):
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, field):
                pending_dependency_case(pending(**{field: "x"}))

    def test_the_source_hash_is_pinned(self) -> None:
        with self.assertRaisesRegex(ValueError, "source_sha256"):
            pending_dependency_case(pending(source_sha256="stale"))


class PendingRowSelectionTests(unittest.TestCase):
    def test_runnable_rows_drop_only_pending_rows(self) -> None:
        rows = [translated(), pending()]
        self.assertEqual(runnable_rows(rows), [rows[0]])
        self.assertTrue(is_pending_dependency(rows[1]))
        self.assertEqual(set(pending_dependency_rows(rows, set())), {"S.Case"})

    def test_the_row_lapses_once_a_blocker_is_ported(self) -> None:
        with self.assertRaisesRegex(ValueError, "now ported"):
            pending_dependency_rows([pending()], {"BRepAlgoAPI_Cut"})

    def test_implemented_classes_are_class_records_only(self) -> None:
        api = [{"upstream_id": "BRepAlgoAPI_Cut", "status": "implemented"},
               {"upstream_id": "BRepAlgoAPI_Fuse", "status": "excluded"},
               {"upstream_id": "BRepAlgoAPI_Common::Build", "status": "implemented"}]
        self.assertEqual(implemented_classes(api), {"BRepAlgoAPI_Cut"})

    def test_a_blocker_must_appear_in_the_source(self) -> None:
        case = pending_dependency_case(pending())
        require_blockers_in_source(case, "#include <BRepAlgoAPI_Cut.hxx>")
        with self.assertRaisesRegex(ValueError, "never names"):
            require_blockers_in_source(case, "#include <BRepAlgoAPI_CutX.hxx>")

    def test_a_blocker_may_appear_in_a_local_fixture_header(self) -> None:
        case = pending_dependency_case(pending(blocked_by=("BRepFilletAPI_MakeFillet",)))
        with tempfile.TemporaryDirectory() as folder:
            source = Path(folder)/"S_Test.cxx"
            source.write_text('#include "S_Utilities.pxx"\n#include <gtest/gtest.h>\n')
            (Path(folder)/"S_Utilities.pxx").write_text("BRepFilletAPI_MakeFillet aFillet(aShape);\n")
            require_blockers_in_source(case, source_with_fixtures(source))
            with self.assertRaisesRegex(ValueError, "never names"):
                require_blockers_in_source(case, source.read_text())


if __name__ == "__main__":
    unittest.main()
