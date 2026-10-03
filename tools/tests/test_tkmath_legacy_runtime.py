"""Test the fail-closed exact identity helpers used by the TKMath runtime run."""

from __future__ import annotations

import hashlib
import pathlib
import tempfile
import unittest
from unittest.mock import patch

from api_inventory.inputs import JSONObject
from verify_tkmath_legacy import (
    arguments,
    hash_linked_native_libraries,
    linked_native_library_paths,
    native_filter,
    native_env,
    native_statuses,
    main,
    report_header_fields,
    cargo_env,
    rust_statuses,
    verify_source_group,
)
from tkmath_native_provenance import (
    require_native_library_set,
    required_toolkit_libraries,
)


class FailedToolkitRun:
    """Named fake that simulates an execution failure after CLI setup."""

    def __call__(self, _args: object) -> None:
        raise RuntimeError("simulated native execution failure")


class PackageScopedRustRuns:
    """Record package/target requests and return each exact mapped identity."""

    def __init__(self) -> None:
        self.calls: list[tuple[str, str]] = []

    def __call__(self, package, target, expected, _env, _output):
        self.calls.append((package, target))
        return {case for case in expected if case[:2] == (package, target)}


class TkMathLegacyRuntimeTest(unittest.TestCase):
    def test_cli_keeps_tkmath_default_and_selects_tkg2d_output(self) -> None:
        """The old default remains stable and explicit toolkit runs isolate outputs."""
        with patch("sys.argv", ["verify_tkmath_legacy.py"]):
            default = arguments()
        with patch("sys.argv", ["verify_tkmath_legacy.py", "--toolkit", "TKG2d"]):
            tkg2d = arguments()
        self.assertEqual(default.toolkit, "TKMath")
        self.assertEqual(default.output.name, "oracle-tkmath-legacy")
        self.assertEqual(tkg2d.toolkit, "TKG2d")
        self.assertEqual(tkg2d.output.name, "oracle-tkg2d-legacy")

    def test_failed_rerun_removes_previous_success_report(self) -> None:
        """Old green receipts are removed before a fresh run starts."""
        with tempfile.TemporaryDirectory() as temporary:
            output = pathlib.Path(temporary)
            report_path = output / "report.json"
            report_path.write_text('{"toolkit_complete": true}\n')
            argv = ["verify_tkmath_legacy.py", "--output", str(output)]
            with patch("sys.argv", argv), patch(
                "verify_tkmath_legacy.run_verification",
                side_effect=FailedToolkitRun(),
            ):
                with self.assertRaisesRegex(RuntimeError, "simulated native"):
                    main()
            self.assertFalse(report_path.exists())

    def test_tkg2d_runtime_report_does_not_claim_toolkit_completion(self) -> None:
        """Mapped runtime evidence stays scoped and non-complete; e.g. TKG2d header."""
        report = report_header_fields(
            "TKG2d",
            {"revision": "occt-revision"},
            {},
            {},
            {"commit": "rust-revision", "clean": True},
            pathlib.Path("target/private"),
        )
        self.assertEqual(report["runtime_evidence_toolkit"], "TKG2d")
        self.assertFalse(report["toolkit_complete"])

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
            with self.assertRaisesRegex(
                RuntimeError, "TKG2d native skips require explicit review"
            ):
                native_statuses(xml_path, {"Suite.Case"}, "TKG2d")

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

    def test_native_status_rejects_duplicate_identity(self) -> None:
        """Duplicated XML identities cannot satisfy an exact case set."""
        with tempfile.TemporaryDirectory() as temporary:
            xml_path = pathlib.Path(temporary) / "cases.xml"
            testcase = '<testcase classname="Suite" name="Case" status="run"/>'
            xml_path.write_text(
                f"<testsuites><testsuite>{testcase}{testcase}</testsuite></testsuites>"
            )
            with self.assertRaisesRegex(RuntimeError, "exact set"):
                native_statuses(xml_path, {"Suite.Case"}, "TKG2d")

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

    def test_tkg2d_native_library_requirements_include_core_dependencies(self) -> None:
        """TKG2d proof requires itself and its dependency core; e.g. TKG2d/TKMath/TKernel."""
        self.assertEqual(
            required_toolkit_libraries("TKG2d"),
            ("libTKernel.so", "libTKMath.so", "libTKG2d.so"),
        )
        libraries = {
            name: f"/build/{name}.8.1"
            for name in (
                "libTKernel.so.8.1",
                "libTKMath.so.8.1",
                "libTKG2d.so.8.1",
                "libgtest.so.1",
                "libgtest_main.so.1",
            )
        }
        require_native_library_set(libraries, "TKG2d")

    def test_tkg3d_native_library_requirements_reject_missing_module(self) -> None:
        """A TKG3d report needs both 3D and 2D linked modules."""
        libraries = {
            name: f"/build/{name}.8.1"
            for name in (
                "libTKernel.so.8.1",
                "libTKMath.so.8.1",
                "libTKG2d.so.8.1",
                "libgtest.so.1",
                "libgtest_main.so.1",
            )
        }
        with self.assertRaisesRegex(RuntimeError, "libTKG3d.so"):
            require_native_library_set(libraries, "TKG3d")

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

    def test_rust_runs_are_grouped_by_package_and_target(self) -> None:
        """Every crate gets a separate exact Cargo invocation; e.g. TKG3d dependencies."""
        expected = {
            ("rustycad-tkg3d", "mesh_test", "case"),
            ("rustycad-tkgeomalgo", "mesh_test", "case"),
        }
        runner = PackageScopedRustRuns()
        with patch("verify_tkmath_legacy.run_rust_target", side_effect=runner):
            passed = rust_statuses(expected, {}, pathlib.Path("out"))
        self.assertEqual(
            set(runner.calls),
            {("rustycad-tkg3d", "mesh_test"), ("rustycad-tkgeomalgo", "mesh_test")},
        )
        self.assertEqual(passed, expected)


def mapping(digest: str) -> JSONObject:
    """Make one exact test mapping; e.g. `mapping('0' * 64)`."""
    return {"upstream_id": "Suite.Case", "source_sha256": digest}


if __name__ == "__main__":
    unittest.main()
