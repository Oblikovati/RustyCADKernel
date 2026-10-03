"""Focused invariants for the no-fuzzy API crosswalk."""

import json
import unittest
from pathlib import Path
from api_inventory.inputs import JSONObject

from api_coverage_report import cpp_candidates, report_document
from api_coverage_report import declaration_toolkits, rust_inventory_gaps
from api_crosswalk import (
    canonical_parameter_type,
    cpp_candidates,
    parameter_match,
    resolve_status,
    rust_candidates,
    rust_crates,
)
from api_identity import parse_upstream_id

HEADER_PATH = "src/FoundationClasses/TKMath/GeomAbs/GeomAbs_Surface.hxx"


def cpp_decl(
    identity: str, name: str, params: list[str], visible: bool = True
) -> JSONObject:
    return {
        "id": identity,
        "usr": f"c:@{identity}",
        "qualified_name": name,
        "kind": "CXXMethod",
        "signature": f"void ({', '.join(params)}) const",
        "parameter_types": params,
        "written_parameter_types": params,
        "source": HEADER_PATH,
        "api_visible": visible,
        "exclusion_reason": "" if visible else "private_access",
        "sites": [],
    }


def source_headers() -> JSONObject:
    return {
        "headers": [
            {
                "path": HEADER_PATH,
                "owners": [
                    {
                        "module": "FoundationClasses",
                        "toolkit": "TKMath",
                        "package": "GeomAbs",
                    }
                ],
            }
        ],
        "generated_headers": [],
    }


def cpp_manifest(configured: bool = True) -> JSONObject:
    return {
        "declaration_inventory_complete": True,
        "expected_headers": 1,
        "headers_by_status": {"parsed": 1},
        "configuration_matrix_complete": configured,
        "unsupported_declarations": 0,
    }


def rust_inventory(paths: list[str]) -> JSONObject:
    return {
        "crates": [
            {
                "crate": "rustycad_tkmath",
                "declarations": [
                    {
                        "path": path,
                        "kind": "function",
                        "signature": {},
                        "source_file": "crates/tkmath/src/lib.rs",
                        "source_status": None,
                        "source_sha256": "a" * 64,
                    }
                    for path in paths
                ],
                "unresolved_links": [],
                "unsupported": {"kind_counts": {}},
            }
        ],
    }


class ApiCoverageCrosswalkTest(unittest.TestCase):
    def test_inventory_conflict_withholds_a_unique_candidate(self) -> None:
        declaration = cpp_decl("alias", "math_Vector", [], visible=True)
        declaration["inventory_conflict"] = True
        declaration["_toolkits"] = {"TKMath"}
        status, candidates = cpp_candidates(
            "math_Vector", "TKMath", {"math_Vector": [declaration]}
        )
        self.assertEqual(status, "conflicting_inventory")
        self.assertEqual(candidates, [declaration])

    def test_unresolved_using_member_blocks_automatic_credit(self) -> None:
        declaration = cpp_decl(
            "unresolved", "fixture::Derived::member", [], visible=True
        )
        declaration["api_resolution"] = "unresolved_using_value_target"
        declaration["_toolkits"] = {"TKFixture"}
        status, rows = cpp_candidates(
            "fixture::Derived::member",
            "TKFixture",
            {"fixture::Derived::member": [declaration]},
        )
        self.assertEqual(status, "unresolved_member_target")
        self.assertEqual(rows, [declaration])

    def test_unstable_template_identity_blocks_automatic_credit(self) -> None:
        declaration = cpp_decl("unstable", "fixture::choose", ["int"])
        declaration["api_resolution"] = (
            "unsupported_template_signature_normalization"
        )
        declaration["unstable_identity"] = True
        declaration["_toolkits"] = {"TKFixture"}
        status, rows = cpp_candidates(
            "fixture::choose(int)",
            "TKFixture",
            {"fixture::choose": [declaration]},
        )
        self.assertEqual(status, "unstable_cpp_identity")
        self.assertEqual(rows, [declaration])
        self.assertEqual(
            resolve_status("implemented", status, "unique"),
            "unstable_cpp_identity",
        )

    def test_exact_qualified_signature_selects_matching_overload(self) -> None:
        overloads = [
            cpp_decl("int", "GeomAbs::Convert", ["int"]),
            cpp_decl("double", "GeomAbs::Convert", ["double"]),
        ]
        status, rows = cpp_candidates(
            "GeomAbs::Convert(int) const", "TKMath", index(overloads)
        )
        self.assertEqual(status, "unique")
        self.assertEqual([row["id"] for row in rows], ["int"])

    def test_quantity_period_uses_real_occt_top_level_const_parameters(self) -> None:
        fixture_path = (
            Path(__file__).parent / "fixtures/occt-quantity-period-constructor.json"
        )
        declaration = json.loads(fixture_path.read_text(encoding="utf-8"))
        declaration["_toolkits"] = {"TKernel"}
        by_name = {"Quantity_Period::Quantity_Period": [declaration]}

        status, rows = cpp_candidates(
            "Quantity_Period::Quantity_Period(int,int)", "TKernel", by_name
        )

        self.assertEqual(status, "unique")
        self.assertEqual(rows[0]["parameter_types"], ["const int", "const int"])

    def test_parameter_cv_canonicalization_preserves_pointer_and_reference_cv(self) -> None:
        self.assertEqual(
            canonical_parameter_type("std::vector<int const> const"),
            "std::vector<int const>",
        )
        self.assertEqual(canonical_parameter_type("const int* const"), "const int*")
        self.assertEqual(canonical_parameter_type("const int&"), "const int&")
        self.assertNotEqual(
            canonical_parameter_type("int*"), canonical_parameter_type("const int*")
        )
        self.assertNotEqual(
            canonical_parameter_type("int&"), canonical_parameter_type("const int&")
        )

    def test_unsupported_parenthesized_declarators_remain_unmatched(self) -> None:
        self.assertIsNone(canonical_parameter_type("int (*)(int)"))
        self.assertFalse(
            parameter_match(
                ["int (*)(int)"], {"parameter_types": ["int (*)(int)"]}
            )
        )

    def test_parameter_match_keeps_nested_and_indirected_cv_semantics(self) -> None:
        self.assertTrue(
            parameter_match(
                ["std::vector<int const>"],
                {"parameter_types": ["std::vector<int const> const"]},
            )
        )
        self.assertTrue(
            parameter_match(["const int*"], {"parameter_types": ["const int* const"]})
        )
        self.assertFalse(parameter_match(["int*"], {"parameter_types": ["const int*"]}))
        self.assertTrue(
            parameter_match(["const int&"], {"parameter_types": ["const int&"]})
        )
        self.assertFalse(parameter_match(["int&"], {"parameter_types": ["const int&"]}))

    def test_bare_overload_name_stays_ambiguous(self) -> None:
        overloads = [
            cpp_decl("int", "GeomAbs::Convert", ["int"]),
            cpp_decl("double", "GeomAbs::Convert", ["double"]),
        ]
        status, rows = cpp_candidates("GeomAbs::Convert", "TKMath", index(overloads))
        self.assertEqual(status, "ambiguous")
        self.assertEqual(len(rows), 2)

    def test_qualified_mismatch_does_not_fall_back_to_leaf_name(self) -> None:
        declaration = cpp_decl("d1", "OtherScope::Convert", ["int"])
        status, rows = cpp_candidates(
            "GeomAbs::Convert(int) const", "TKMath", index([declaration])
        )
        self.assertEqual(status, "missing")
        self.assertEqual(rows, [])

    def test_hidden_template_pattern_cannot_claim_a_public_declaration(self) -> None:
        public = cpp_decl("public", "GeomAbs::Convert", ["int"])
        hidden = cpp_decl("template", "GeomAbs::Convert", ["int"], visible=False)
        hidden["exclusion_reason"] = "template_pattern"
        parity = {
            "api": [
                api_row(
                    "GeomAbs::Convert(int) const", "rustycad_tkmath::geom_abs::convert"
                )
            ]
        }
        report = report_document(
            [public, hidden],
            cpp_manifest(),
            source_headers(),
            rust_inventory(["rustycad_tkmath::geom_abs::convert"]),
            parity,
        )
        self.assertEqual(report["crosswalk_rows"][0]["cpp_status"], "unique")
        self.assertEqual(report["crosswalk_rows"][0]["cpp_ids"], ["public"])

    def test_hidden_only_declaration_does_not_receive_implementation_credit(self) -> None:
        hidden = cpp_decl("template", "GeomAbs::Convert", ["int"], visible=False)
        hidden["exclusion_reason"] = "template_pattern"
        report = report_document(
            [hidden],
            cpp_manifest(),
            source_headers(),
            rust_inventory(["rustycad_tkmath::geom_abs::convert"]),
            {
                "api": [
                    api_row(
                        "GeomAbs::Convert(int) const",
                        "rustycad_tkmath::geom_abs::convert",
                    )
                ]
            },
        )
        self.assertEqual(report["crosswalk_rows"][0]["resolution"], "missing_cpp")

    def test_rust_target_without_pinned_source_cannot_receive_credit(self) -> None:
        rust = rust_inventory(["rustycad_tkmath::geom_abs::convert"])
        item = rust["crates"][0]["declarations"][0]
        item["source_status"] = "source_file_missing"
        item["source_sha256"] = None
        report = report_document(
            [cpp_decl("public", "GeomAbs::Convert", ["int"])],
            cpp_manifest(),
            source_headers(),
            rust,
            {
                "api": [
                    api_row(
                        "GeomAbs::Convert(int) const",
                        "rustycad_tkmath::geom_abs::convert",
                    )
                ]
            },
        )
        self.assertEqual(
            report["crosswalk_rows"][0]["rust_status"], "unverifiable_source"
        )
        self.assertEqual(
            report["crosswalk_rows"][0]["resolution"], "unverifiable_rust_source"
        )

    def test_real_normalized_inventory_path_is_found_with_source_evidence(self) -> None:
        fixture_path = (
            Path(__file__).parent / "fixtures/rust-api-inventory-cmath-asinh.json"
        )
        fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
        rust = rust_crates({"crates": [fixture]})

        status, candidates = rust_candidates("rustycad_cmath::asinh", rust)

        self.assertEqual(status, "unique")
        self.assertEqual(candidates[0]["source_file"], "crates/cmath/src/lib.rs")

    def test_template_commas_and_operator_names_parse_without_guessing(self) -> None:
        template = parse_upstream_id(
            "Bnd_Range::Split(double,NCollection_List<Bnd_Range>&,double) const"
        )
        operator = parse_upstream_id("Bnd_Range::operator<(const Bnd_Range&) const")
        self.assertEqual(
            template[1], ["double", "NCollection_List<Bnd_Range>&", "double"]
        )
        self.assertEqual(operator[0], "Bnd_Range::operator<")
        self.assertEqual(operator[2], "const")

    def test_missing_rust_item_and_deleted_cpp_declaration_remain_unresolved(
        self,
    ) -> None:
        declaration = cpp_decl("d1", "GeomAbs::Convert", ["int"])
        parity = {
            "api": [
                api_row(
                    "GeomAbs::Convert(int) const", "rustycad_tkmath::missing::convert"
                ),
                api_row(
                    "GeomAbs::Deleted(int) const", "rustycad_tkmath::geom_abs::deleted"
                ),
            ]
        }
        report = report_document(
            [declaration],
            cpp_manifest(),
            source_headers(),
            rust_inventory(["rustycad_tkmath::geom_abs::unrelated"]),
            parity,
        )
        states = {
            row["upstream_id"]: row["resolution"] for row in report["unresolved_rows"]
        }
        self.assertEqual(states["GeomAbs::Convert(int) const"], "missing_rust_target")
        self.assertEqual(states["GeomAbs::Deleted(int) const"], "missing_cpp")
        self.assertEqual(report["unclaimed_cpp_count"], 0)
        self.assertFalse(report["declaration_crosswalk_proof"])

    def test_stale_exclusion_is_distinct_from_partial_mapping(self) -> None:
        declaration = cpp_decl("d1", "GeomAbs::Convert", ["int"])
        parity = {
            "api": [
                {
                    **api_row("GeomAbs::Gone()", "rustycad_tkmath::geom_abs::gone"),
                    "source": "src/FoundationClasses/TKMath/GeomAbs/GeomAbs_Gone.hxx",
                    "status": "excluded",
                },
                {
                    **api_row(
                        "GeomAbs::Convert(int) const",
                        "rustycad_tkmath::geom_abs::convert",
                    ),
                    "status": "partial",
                },
            ]
        }
        report = report_document(
            [declaration],
            cpp_manifest(),
            source_headers(),
            rust_inventory(["rustycad_tkmath::geom_abs::convert"]),
            parity,
        )
        states = {
            row["upstream_id"]: row["resolution"] for row in report["unresolved_rows"]
        }
        self.assertEqual(states["GeomAbs::Gone()"], "stale_exclusion")
        self.assertEqual(report["partial_mappings"], 1)

    def test_configuration_gap_forces_proof_false(self) -> None:
        declaration = cpp_decl("d1", "GeomAbs::Convert", ["int"])
        parity = {
            "api": [
                api_row(
                    "GeomAbs::Convert(int) const", "rustycad_tkmath::geom_abs::convert"
                )
            ]
        }
        report = report_document(
            [declaration],
            cpp_manifest(False),
            source_headers(),
            rust_inventory(["rustycad_tkmath::geom_abs::convert"]),
            parity,
        )
        self.assertFalse(report["configuration_matrix_complete"])
        self.assertIn("cxx_configuration_matrix_incomplete", report["proof_gaps"])
        self.assertFalse(report["declaration_crosswalk_proof"])
        self.assertFalse(report["behavioral_equivalence_proven"])

    def test_unclassified_rust_items_and_cpp_unsupported_records_are_gaps(self) -> None:
        declaration = cpp_decl("d1", "GeomAbs::Convert", ["int"])
        manifest = {**cpp_manifest(), "unsupported_records": 2}
        rust = rust_inventory(["rustycad_tkmath::geom_abs::convert"])
        rust["crates"][0]["unclassified_kinds"] = {"counts": {"new_kind": 1}}
        report = report_document(
            [declaration],
            manifest,
            source_headers(),
            rust,
            {
                "api": [
                    api_row(
                        "GeomAbs::Convert(int) const",
                        "rustycad_tkmath::geom_abs::convert",
                    )
                ]
            },
        )
        self.assertIn("unsupported_cpp_declarations:2", report["proof_gaps"])
        self.assertIn("rust_unclassified_items:rustycad_tkmath", report["proof_gaps"])
        self.assertFalse(report["declaration_crosswalk_proof"])

    def test_declaration_source_owner_precedes_incidental_observation(self) -> None:
        owners = {
            "src/TKMath.hxx": {"TKMath"},
            "src/TKBin.hxx": {"TKBin"},
        }
        declaration = {
            "source": "src/TKMath.hxx",
            "main_header": "src/TKBin.hxx",
            "sites": [{"observed_in": ["src/TKBin.hxx"]}],
        }
        self.assertEqual(declaration_toolkits(declaration, owners), {"TKMath"})

    def test_cpp_unsupported_records_accept_count_or_record_array(self) -> None:
        from api_coverage_report import cpp_unsupported_count

        self.assertEqual(cpp_unsupported_count({"unsupported_records": 3}), 3)
        self.assertEqual(cpp_unsupported_count({"unsupported_records": [{}, {}]}), 2)


def index(declarations: list[dict]) -> JSONObject:
    return {
        name: [
            dict(row, _toolkits={"TKMath"})
            for row in declarations
            if row["qualified_name"] == name
        ]
        for name in {row["qualified_name"] for row in declarations}
    }


def api_row(identity: str, rust_item: str) -> JSONObject:
    return {
        "upstream_id": identity,
        "toolkit": "TKMath",
        "source": HEADER_PATH,
        "rust_item": rust_item,
        "status": "implemented",
        "notes": "fixture",
    }


if __name__ == "__main__":
    unittest.main()
