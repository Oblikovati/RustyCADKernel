"""Test the fail-closed exact identity helpers used by the TKMath runtime run."""

from __future__ import annotations

import hashlib
import pathlib
import tempfile
import unittest

from api_inventory.inputs import JSONObject
from verify_tkmath_legacy import (
    native_filter,
    native_statuses,
    verify_source_group,
)


class TkMathLegacyRuntimeTest(unittest.TestCase):
    def test_native_filter_is_exact_and_sorted(self) -> None:
        """Filtering keeps only named mapped cases; e.g. stable sorted identities."""
        self.assertEqual(native_filter({"B.Test", "A.Test"}), "A.Test:B.Test")

    def test_native_filter_rejects_empty_or_separator_cases(self) -> None:
        """GoogleTest syntax cannot silently widen scope; e.g. ':' is rejected."""
        with self.assertRaisesRegex(ValueError, "GoogleTest filter metacharacters"):
            native_filter({"Suite.Test:Other.Test"})
        with self.assertRaisesRegex(ValueError, "nonempty"):
            native_filter(set())

    def test_native_status_requires_exact_cases_and_no_skips(self) -> None:
        """Skipped legacy cases cannot count as passed; e.g. XML includes one skip."""
        with tempfile.TemporaryDirectory() as temporary:
            xml_path = pathlib.Path(temporary) / "cases.xml"
            xml_path.write_text(
                '<testsuites><testsuite><testcase classname="Suite" name="Case" '
                'status="run"><skipped message="todo"/></testcase></testsuite></testsuites>'
            )
            with self.assertRaisesRegex(RuntimeError, "skips require explicit review"):
                native_statuses(xml_path, {"Suite.Case"})

    def test_native_status_rejects_missing_identity(self) -> None:
        """A successful XML subset cannot prove the mapping set; e.g. absent case."""
        with tempfile.TemporaryDirectory() as temporary:
            xml_path = pathlib.Path(temporary) / "cases.xml"
            xml_path.write_text("<testsuites/>")
            with self.assertRaisesRegex(RuntimeError, "exact set"):
                native_statuses(xml_path, {"Suite.Case"})

    def test_source_group_requires_every_declaration_and_digest(self) -> None:
        """Mapped cases bind to exact source bytes; e.g. source file hash and names."""
        with tempfile.TemporaryDirectory() as temporary:
            source = pathlib.Path(temporary)
            test_file = source / "Suite_Test.cxx"
            contents = b"TEST_F(Suite, Case) {}\n"
            test_file.write_bytes(contents)
            digest = hashlib.sha256(contents).hexdigest()
            verify_source_group(source, "Suite_Test.cxx", [mapping(digest)])
            with self.assertRaisesRegex(RuntimeError, "Stale or incomplete"):
                verify_source_group(source, "Suite_Test.cxx", [])


def mapping(digest: str) -> JSONObject:
    """Make one exact test mapping; e.g. `mapping('0' * 64)`."""
    return {"upstream_id": "Suite.Case", "source_sha256": digest}


if __name__ == "__main__":
    unittest.main()
