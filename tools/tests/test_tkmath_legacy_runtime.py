"""Test the fail-closed exact identity helpers used by the TKMath runtime run."""

from __future__ import annotations

import hashlib
import pathlib
import tempfile
import unittest
from unittest.mock import patch

from api_inventory.inputs import JSONObject
from verify_tkmath_legacy import (
    hash_linked_native_libraries,
    linked_native_library_paths,
    native_filter,
    native_env,
    native_statuses,
    cargo_env,
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

    def test_native_status_accepts_exact_passing_identity(self) -> None:
        """A complete passing XML case produces one status; e.g. Suite.Case."""
        with tempfile.TemporaryDirectory() as temporary:
            xml_path = pathlib.Path(temporary) / "cases.xml"
            xml_path.write_text(
                '<testsuites><testsuite><testcase classname="Suite" name="Case" '
                'status="run"/></testsuite></testsuites>'
            )
            self.assertEqual(
                native_statuses(xml_path, {"Suite.Case"}), {"Suite.Case": "passed"}
            )

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

    def test_native_library_hash_requires_core_libraries_inside_build(self) -> None:
        """Oracle provenance binds shared dependencies; e.g. four core libraries."""
        with tempfile.TemporaryDirectory() as temporary:
            build = pathlib.Path(temporary)
            names = ["libTKernel.so", "libTKMath.so", "libgtest.so", "libgtest_main.so"]
            lines = []
            for name in names:
                path = build / name
                path.write_text(name)
                lines.append(f"{name} => {path} (0x0)")
            hashes = hash_linked_native_libraries("\n".join(lines), build)
        self.assertEqual(len(hashes), 4)

    def test_native_library_provenance_rejects_unresolved_dependency(self) -> None:
        """An unresolved linked OCCT library blocks evidence; e.g. ldd says not found."""
        with self.assertRaisesRegex(RuntimeError, "unresolved"):
            linked_native_library_paths("libTKOffset.so => not found")

    def test_native_environment_removes_library_overrides(self) -> None:
        """Dynamic loader overrides cannot replace the pinned build libraries."""
        with patch.dict(
            "os.environ", {"LD_LIBRARY_PATH": "/tmp", "LD_PRELOAD": "/tmp/x.so"}
        ):
            env = native_env()
        self.assertNotIn("LD_LIBRARY_PATH", env)
        self.assertNotIn("LD_PRELOAD", env)

    def test_cargo_environment_pins_private_target_directory(self) -> None:
        """Cargo artifact paths are checkout-scoped; e.g. caller override is replaced."""
        target = pathlib.Path("/tmp/rustycad-tkmath-private-target")
        with patch.dict("os.environ", {"CARGO_TARGET_DIR": "/tmp/other-worktree"}):
            env = cargo_env(target)
        self.assertEqual(env["CARGO_TARGET_DIR"], str(target.resolve()))


def mapping(digest: str) -> JSONObject:
    """Make one exact test mapping; e.g. `mapping('0' * 64)`."""
    return {"upstream_id": "Suite.Case", "source_sha256": digest}


if __name__ == "__main__":
    unittest.main()
