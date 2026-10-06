"""Historical contexts are hints; fresh source hashes and compile commands remain required."""

from __future__ import annotations

import unittest
from pathlib import Path
from unittest.mock import patch

from api_inventory.context_replay import CompileContext, context_flags_hash
from api_inventory.inputs import JSONObject
from rebind_occt_context_plan import rebind_row, selected_rows


class FakePinnedSources:
    """Named source-digest fake for pure plan validation tests."""

    def digest(self, path: Path) -> str:
        return "header-hash" if path.suffix == ".hxx" else "source-hash"


class RebindPlanTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = Path("/occt")
        self.context = CompileContext(
            Path("/occt/src/TKernel/example.cxx"),
            Path("/new-build"),
            "TKernel",
            ("-DNEW",),
        )
        self.contexts = {"src/TKernel/example.cxx": self.context}
        self.row: JSONObject = {
            "path": "src/TKernel/example.hxx",
            "compile_source": "src/TKernel/example.cxx",
            "compile_toolkit": "TKernel",
            "header_sha256": "header-hash",
            "compile_source_sha256": "source-hash",
            "compile_flags_sha256": "old-flags",
        }
        self.fake = FakePinnedSources()

    def test_rebind_preserves_old_flags_without_parse_credit(self) -> None:
        with patch("rebind_occt_context_plan.digest", side_effect=self.fake.digest):
            row = rebind_row(self.row, self.source, self.contexts)
        self.assertEqual(row["previous_compile_flags_sha256"], "old-flags")
        self.assertEqual(
            row["compile_flags_sha256"], context_flags_hash(self.context.flags)
        )
        self.assertEqual(
            row["context_basis"], "historical_source_hint_requires_fresh_success"
        )
        self.assertEqual(self.row["compile_flags_sha256"], "old-flags")

    def test_missing_or_wrong_toolkit_command_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected current command"):
            rebind_row(self.row, self.source, {})
        self.row["compile_toolkit"] = "TKMath"
        with self.assertRaisesRegex(ValueError, "expected current command"):
            rebind_row(self.row, self.source, self.contexts)

    def test_changed_sources_and_outside_paths_are_rejected(self) -> None:
        for field, value in [
            ("header_sha256", "changed"),
            ("compile_source_sha256", "changed"),
            ("path", "../outside.hxx"),
        ]:
            row = {**self.row, field: value}
            with (
                self.subTest(field=field),
                patch("rebind_occt_context_plan.digest", side_effect=self.fake.digest),
                self.assertRaisesRegex(ValueError, "expected historical source SHA256"),
            ):
                rebind_row(row, self.source, self.contexts)

    def test_only_failed_current_scope_headers_are_selected(self) -> None:
        prior: JSONObject = {
            "rows": [{"path": "failed"}, {"path": "passed"}, {"path": "absent"}]
        }
        capture: JSONObject = {
            "results": [
                {"path": "failed", "status": "parse_failed"},
                {"path": "passed", "status": "parsed"},
            ]
        }
        self.assertEqual(selected_rows(prior, capture), [{"path": "failed"}])


if __name__ == "__main__":
    unittest.main()
