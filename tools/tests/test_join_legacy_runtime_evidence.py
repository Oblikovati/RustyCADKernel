"""Test declaration and per-case identity joins with synthetic evidence."""

from __future__ import annotations

import unittest
import hashlib
import pathlib
import subprocess
import tempfile

from api_inventory.inputs import JSONObject
from join_legacy_runtime_evidence import (
    join_declarations,
    joined_report,
    one_source_mapping_error,
    require_schema,
    runtime_provenance,
    source_mapping_errors,
)


class LegacyRuntimeJoinTest(unittest.TestCase):
    def test_exact_native_and_rust_cases_green_one_declaration(self) -> None:
        """Both exact identities must pass; e.g. matching toolkit/source/suite."""
        declaration = mapped_declaration()
        joined, unclaimed = join_declarations([declaration], [case_result()])
        self.assertEqual(joined[0]["runtime_status"], "passed")
        self.assertTrue(joined[0]["legacy_assertions_passed"])
        self.assertEqual(unclaimed, [])

    def test_wrong_declaration_identity_stays_unverified(self) -> None:
        """Case names alone cannot join; e.g. another source has same suite name."""
        declaration = mapped_declaration()
        evidence = case_result(source="src/Other.cxx")
        joined, unclaimed = join_declarations([declaration], [evidence])
        self.assertEqual(joined[0]["runtime_status"], "unverified")
        self.assertEqual(len(unclaimed), 1)

    def test_runtime_toolkit_scope_cannot_green_other_toolkits(self) -> None:
        """TKernel reports prove no TKBRep row; e.g. a matching foreign case is unclaimed."""
        declaration, evidence = scoped_toolkit_fixture("TKBRep")
        joined, unclaimed = join_declarations([declaration], [evidence], "TKernel")
        self.assertEqual(joined[0]["runtime_status"], "unverified")
        self.assertEqual(joined[0]["runtime_reason"], "outside_runtime_toolkit_scope")
        self.assertEqual(len(unclaimed), 1)

    def test_upstream_skip_is_never_legacy_pass(self) -> None:
        """Compatibility behavior does not assert the skipped legacy test."""
        declaration = mapped_declaration("upstream_skipped")
        evidence = case_result(native_status="upstream_skipped")
        joined, _ = join_declarations([declaration], [evidence])
        self.assertEqual(joined[0]["runtime_status"], "upstream_skipped")
        self.assertFalse(joined[0]["legacy_assertions_passed"])

    def test_missing_expansion_stays_unverified(self) -> None:
        """Every typed runtime expansion is required; e.g. one of two absent."""
        declaration = mapped_declaration(
            runtime_cases=[
                {"upstream_case": "Typed/0", "rust_test": "typed_0"},
                {"upstream_case": "Typed/1", "rust_test": "typed_1"},
            ]
        )
        joined, _ = join_declarations(
            [declaration], [case_result(native_case="Typed/0", rust_test="typed_0")]
        )
        self.assertEqual(joined[0]["runtime_status"], "unverified")

    def test_missing_rust_execution_cannot_pass_declaration(self) -> None:
        """A native pass alone is incomplete; e.g. Rust output is missing."""
        evidence = case_result()
        evidence["rust_status"] = "missing"
        joined, _ = join_declarations([mapped_declaration()], [evidence])
        self.assertEqual(joined[0]["runtime_status"], "unverified")
        self.assertEqual(joined[0]["runtime_reason"], "runtime_pair_not_passed")

    def test_case_source_digest_must_match_mapping(self) -> None:
        """Source-key matching still checks bytes; e.g. mismatched SHA stays unverified."""
        evidence = case_result()
        evidence["source_sha256"] = "f" * 64
        joined, _ = join_declarations([mapped_declaration()], [evidence])
        self.assertEqual(joined[0]["runtime_status"], "unverified")
        self.assertEqual(joined[0]["runtime_reason"], "source_digest_mismatch")

    def test_duplicate_runtime_identity_fails_closed(self) -> None:
        """Ambiguous reports are rejected; e.g. duplicate same native/Rust pair."""
        case = case_result()
        with self.assertRaisesRegex(ValueError, "duplicate runtime case identity"):
            join_declarations([mapped_declaration()], [case, dict(case)])

    def test_incremental_green_rows_do_not_set_global_proof(self) -> None:
        """A passing subset is not full acceptance; e.g. one TKA declaration."""
        report: JSONObject = {"revision": "pinned"}
        inventory: JSONObject = {"revision": "pinned"}
        rows = [{**mapped_declaration(), "runtime_status": "passed"}]
        result = joined_report(report, inventory, rows, [], [], provenance_fixture())
        self.assertFalse(result["runtime_proof"])
        self.assertEqual(result["counts"]["passed"], 1)
        self.assertEqual(result["schema_version"], 2)

    def test_join_provenance_binds_exact_report_and_rust_inputs(self) -> None:
        """The artifact fingerprints its report and input set; e.g. hashes change."""
        with tempfile.TemporaryDirectory() as temporary:
            first, changed = changed_provenance_pair(pathlib.Path(temporary))
        self.assertEqual(first["runtime_report_sha256"], "b" * 64)
        self.assertEqual(first["runtime_report_revision"], "occt-revision")
        self.assertNotEqual(
            first["runtime_input_manifest_sha256"],
            changed["runtime_input_manifest_sha256"],
        )
        self.assertNotEqual(
            first["runtime_report_sha256"], changed["runtime_report_sha256"]
        )
        self.assertEqual(first["rust_source_clean"], True)
        self.assertEqual(first["rust_revision"], "a" * 40)
        self.assertIn("joined_on_rust_revision", first)

    def test_join_provenance_rejects_missing_manifest(self) -> None:
        """A report without input hashes cannot produce a self-contained join."""
        with self.assertRaisesRegex(ValueError, "source_hashes"):
            runtime_provenance(
                {
                    "revision": "x",
                    "rust_git_commit": "a" * 40,
                    "rust_source_clean": True,
                },
                pathlib.Path("."),
                "b" * 64,
            )

    def test_join_rejects_reports_without_execution_revision(self) -> None:
        """Join-time commits cannot stand in for run provenance; e.g. old reports fail."""
        with tempfile.TemporaryDirectory() as temporary:
            initialize_git(pathlib.Path(temporary))
            report: JSONObject = {
                "revision": "occt-revision",
                "source_hashes": {"Cargo.lock": "a" * 64},
            }
            with self.assertRaisesRegex(ValueError, "rust_git_commit"):
                runtime_provenance(report, pathlib.Path(temporary), "b" * 64)

    def test_join_rejects_dirty_execution_source(self) -> None:
        """Run-time dirty trees cannot be reassigned a later clean commit."""
        with tempfile.TemporaryDirectory() as temporary:
            initialize_git(pathlib.Path(temporary))
            report: JSONObject = {
                "revision": "occt-revision",
                "source_hashes": {"Cargo.lock": "a" * 64},
                "rust_git_commit": "a" * 40,
                "rust_source_clean": False,
            }
            with self.assertRaisesRegex(ValueError, "rust_source_clean"):
                runtime_provenance(report, pathlib.Path(temporary), "b" * 64)

    def test_source_mapping_errors_accept_tkmath_scope(self) -> None:
        """Source freshness checks target the report toolkit; e.g. TKMath rows."""
        with tempfile.TemporaryDirectory() as temporary:
            parity, source = tkmath_source_scope_fixture(pathlib.Path(temporary))
            errors = source_mapping_errors(parity, source, "TKMath")
        self.assertEqual(errors, [])

    def test_boolean_is_not_accepted_as_runtime_schema_version(self) -> None:
        """Python bool equality cannot satisfy schema identity; e.g. True is rejected."""
        with self.assertRaisesRegex(ValueError, "schema 1"):
            require_schema(
                {"runtime_evidence_schema": True, "runtime_case_results": []}
            )

    def test_mapping_source_digest_is_checked_against_occt_bytes(self) -> None:
        """A parity hash must match source content; e.g. stale test file fails."""
        with tempfile.TemporaryDirectory() as temporary:
            source = pathlib.Path(temporary)
            errors = checked_source_fixture(source)
        self.assertEqual(errors, [])

    def test_stale_mapping_source_digest_is_rejected(self) -> None:
        """Changed native source cannot be joined; e.g. mutate pinned test bytes."""
        with tempfile.TemporaryDirectory() as temporary:
            source = pathlib.Path(temporary)
            errors = stale_source_fixture(source)
        self.assertEqual(len(errors), 1)
        self.assertIn("source SHA mismatch", errors[0])


def mapped_declaration(
    status: str = "translated", runtime_cases: list[JSONObject] | None = None
) -> JSONObject:
    return {
        "toolkit": "TKA",
        "source": "src/A_Test.cxx",
        "name": "Suite.Case",
        "declaration_id": "declaration-1",
        "mapping_status": "mapped",
        "mapped_rust_test": fixture_mapping(status, runtime_cases),
    }


def fixture_mapping(status: str, runtime_cases: list[JSONObject] | None) -> JSONObject:
    """Build source mapping data; e.g. `fixture_mapping('translated', None)`."""
    mapping: JSONObject = {
        "toolkit": "TKA",
        "source": "src/A_Test.cxx",
        "upstream_id": "Suite.Case",
        "rust_target": "suite_test",
        "rust_test": "case_test",
        "status": status,
        "source_sha256": "0" * 64,
    }
    if runtime_cases is not None:
        mapping["runtime_cases"] = runtime_cases
    return mapping


def scoped_toolkit_fixture(toolkit: str) -> tuple[JSONObject, JSONObject]:
    """Move fixtures outside one report scope; e.g. `scoped_toolkit_fixture('TKBRep')`."""
    declaration, evidence = mapped_declaration(), case_result()
    declaration["toolkit"] = toolkit
    declaration["mapped_rust_test"]["toolkit"] = toolkit
    evidence["toolkit"] = toolkit
    return declaration, evidence


def checked_source_fixture(source: pathlib.Path) -> list[str]:
    """Check a matching source fixture; e.g. `checked_source_fixture(Path(tmp))`."""
    content = b"TEST_F(Suite, Case) {}\n"
    (source / "A_Test.cxx").write_bytes(content)
    row = {"source": "A_Test.cxx", "source_sha256": hashlib.sha256(content).hexdigest()}
    return one_source_mapping_error(row, source)


def stale_source_fixture(source: pathlib.Path) -> list[str]:
    """Check a changed source fixture; e.g. `stale_source_fixture(Path(tmp))`."""
    checked_source_fixture(source)
    (source / "A_Test.cxx").write_bytes(b"changed")
    row = {"source": "A_Test.cxx", "source_sha256": hashlib.sha256(b"old").hexdigest()}
    return one_source_mapping_error(row, source)


def provenance_fixture() -> JSONObject:
    """Build minimal report provenance; e.g. `provenance_fixture()`."""
    return {
        "runtime_report_sha256": "b" * 64,
        "runtime_report_revision": "pinned",
        "rust_revision": "a" * 40,
        "rust_source_clean": True,
        "joined_on_rust_revision": "rust-revision",
        "joined_on_rust_worktree_clean": True,
        "runtime_input_manifest_sha256": "c" * 64,
        "runtime_input_manifest": {"Cargo.lock": "a" * 64},
        "join_tool_sha256": "d" * 64,
    }


def initialize_git(root: pathlib.Path) -> None:
    """Create a clean Rust revision fixture; e.g. `initialize_git(root)`."""
    run_git_fixture(root, ["init", "-q", str(root)])
    run_git_fixture(
        root,
        [
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--allow-empty",
            "-qm",
            "base",
        ],
    )


def run_git_fixture(root: pathlib.Path, arguments: list[str]) -> None:
    """Run one fixture git command; e.g. `run_git_fixture(root, ['status'])`."""
    subprocess.run(["git", "-C", str(root), *arguments], check=True)


def changed_provenance_pair(root: pathlib.Path) -> tuple[JSONObject, JSONObject]:
    """Make two provenance records with changed inputs; e.g. `changed_provenance_pair(root)`."""
    initialize_git(root)
    report: JSONObject = {
        "revision": "occt-revision",
        "source_hashes": {"Cargo.lock": "a" * 64},
        "rust_git_commit": "a" * 40,
        "rust_source_clean": True,
    }
    first = runtime_provenance(report, root, "b" * 64)
    changed_report = {**report, "source_hashes": {"Cargo.lock": "c" * 64}}
    return first, runtime_provenance(changed_report, root, "d" * 64)


def tkmath_source_scope_fixture(root: pathlib.Path) -> tuple[JSONObject, pathlib.Path]:
    """Create TKMath and unrelated rows; e.g. `tkmath_source_scope_fixture(root)`."""
    contents = b"TEST_F(MathSuite, Case) {}\n"
    (root / "Math_Test.cxx").write_bytes(contents)
    digest = hashlib.sha256(contents).hexdigest()
    rows = [{"toolkit": "TKMath", "source": "Math_Test.cxx", "source_sha256": digest}]
    rows.append({"toolkit": "TKernel", "source": "missing.cxx", "source_sha256": "x"})
    return {"tests": rows}, root


def case_result(
    source: str = "src/A_Test.cxx",
    native_case: str = "Suite.Case",
    rust_test: str = "case_test",
    native_status: str = "passed",
) -> JSONObject:
    return {
        "toolkit": "TKA",
        "source": source,
        "upstream_id": "Suite.Case",
        "native_case": native_case,
        "native_status": native_status,
        "source_sha256": "0" * 64,
        "rust_target": "suite_test",
        "rust_test": rust_test,
        "rust_status": "passed",
    }


if __name__ == "__main__":
    unittest.main()
