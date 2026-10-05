"""Bind Quantity exception helpers to their captured C++ declarations."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]
EXCEPTION_BINDINGS = (
    (
        "Quantity_DateDefinitionError::Quantity_DateDefinitionError(const char*)",
        "0a3c56d6d1d424e27d00d2f5269c58b52eb9165868c3f7d6c7eaeed6b46b4f5f",
        "rustycad_tkernel::quantity::exceptions::exception_bytes::date_definition_error",
        [
            "rustycad_tkernel::quantity::exceptions::exception_bytes::date_definition_error",
            "rustycad_tkernel::quantity::exceptions::default_date_definition_error",
        ],
    ),
    (
        "Quantity_DateDefinitionError::Quantity_DateDefinitionError(const char*,const char*)",
        "b81d2db77fa010c46f994898d7218ed04ba392c6787e54bd93823ecb8a3c76d6",
        "rustycad_tkernel::quantity::exceptions::exception_stacks::date_definition_error",
        None,
    ),
    (
        "Quantity_DateDefinitionError::ExceptionType() const noexcept",
        "629b0b418e5ca587ec68415939a46099c5abf0cc01356dc01354b4993c111aa5",
        "rustycad_tkernel::standard::Failure::exception_type",
        None,
    ),
    (
        "Quantity_PeriodDefinitionError::Quantity_PeriodDefinitionError(const char*)",
        "8ecc720c5be0d253b05af2515f2d860ede0d3a3d34f9210af9322f7cb07e5b57",
        "rustycad_tkernel::quantity::exceptions::exception_bytes::period_definition_error",
        [
            "rustycad_tkernel::quantity::exceptions::exception_bytes::period_definition_error",
            "rustycad_tkernel::quantity::exceptions::default_period_definition_error",
        ],
    ),
    (
        "Quantity_PeriodDefinitionError::Quantity_PeriodDefinitionError(const char*,const char*)",
        "d311f8dabb2dc0b07015bfddfa84adf5c2954a82695874f521053764d42cb81d",
        "rustycad_tkernel::quantity::exceptions::exception_stacks::period_definition_error",
        None,
    ),
    (
        "Quantity_PeriodDefinitionError::ExceptionType() const noexcept",
        "cfd81e71b7552e14049bb99ed8d15180f11852344b0c359fe89a830538473db7",
        "rustycad_tkernel::standard::Failure::exception_type",
        None,
    ),
)


class QuantityDefinitionErrorBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["QuantityDefinitionErrorBindingsTest"]) -> None:
        declaration_path = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
        with gzip.open(declaration_path, "rt") as stream:
            declarations = [json.loads(line) for line in stream]
        manifest_path = ROOT / "porting/api-header-manifest.json"
        manifest = json.loads(manifest_path.read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity_path = ROOT / "porting/parity.json"
        api_rows = json.loads(parity_path.read_text())["api"]
        cls.by_selector = {row["upstream_id"]: row for row in api_rows}

    def test_all_exception_members_bind_unique_compiler_declarations(
        self: "QuantityDefinitionErrorBindingsTest",
    ) -> None:
        for selector, compiler_id, rust_item, rust_items in EXCEPTION_BINDINGS:
            self.assert_unique_binding(selector, compiler_id, rust_item, rust_items)

    def assert_unique_binding(
        self: "QuantityDefinitionErrorBindingsTest",
        selector: str,
        compiler_id: str,
        rust_item: str,
        rust_items: list[str] | None,
    ) -> None:
        status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
        self.assertEqual(status, "unique", selector)
        self.assertEqual([row["id"] for row in candidates], [compiler_id])
        row = self.by_selector[selector]
        self.assertEqual(row["rust_item"], rust_item)
        self.assertEqual(row["status"], "implemented")
        if rust_items is not None:
            self.assertEqual(row["rust_items"], rust_items)


if __name__ == "__main__":
    unittest.main()
