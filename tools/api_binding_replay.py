"""Replay exact partial mappings without allowing path ambiguity to grant credit."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

from api_crosswalk import cpp_candidates, row_rust_candidates
from api_inventory.inputs import JSONObject, text_field
from api_rust_kind import RustCrateIndex


def binding_object(row: JSONObject, field: str) -> JSONObject:
    """Require a named object; e.g. `binding_object(record, 'proposed_row')`."""
    value = row.get(field)
    if not isinstance(value, dict):
        raise ValueError(f"{field}={value!r}; expected a JSON object")
    return value


def binding_equal(actual: object, expected: object, label: str) -> None:
    """Require exact proof values; e.g. `binding_equal(status, 'unique', 'Rust')`."""
    if actual == expected:
        return
    raise ValueError(f"{label}={actual!r}; expected {expected!r}")


@dataclass(frozen=True)
class BindingReplay:
    """Inject inventories and source hashing; `BindingReplay(native, rust, digest)`."""

    native_by_name: dict[str, list[JSONObject]]
    rust_by_crate: RustCrateIndex
    source_hash: Callable[[str], str]

    def verify(self, record: JSONObject) -> None:
        """Check both declarations and their body; e.g. `replay.verify(record)`."""
        proposed = binding_object(record, "proposed_row")
        binding_equal(proposed.get("status"), "partial", "ledger status")
        self._verify_native(record, proposed)
        self._verify_rust(record, proposed)

    def _verify_native(self, record: JSONObject, proposed: JSONObject) -> None:
        status, matches = cpp_candidates(
            text_field(proposed, "upstream_id"),
            text_field(proposed, "toolkit"),
            self.native_by_name,
        )
        binding_equal(status, "unique", "native resolution")
        binding_equal(
            [row.get("id") for row in matches], [record.get("cpp_id")], "native ID"
        )
        binding_equal(
            matches[0].get("signature"), record.get("cpp_signature"), "native signature"
        )

    def _verify_rust(self, record: JSONObject, proposed: JSONObject) -> None:
        status, matches = row_rust_candidates(proposed, self.rust_by_crate)
        binding_equal(status, "unique", "Rust resolution")
        binding_equal(len(matches), 1, "Rust body count")
        body = matches[0]
        binding_equal(body.get("kind"), "function", "Rust declaration kind")
        binding_equal(
            binding_object(body, "signature").get("has_body"), True, "Rust body"
        )
        self._verify_body_identity(body, binding_object(record, "rust_body"))

    def _verify_body_identity(self, body: JSONObject, expected: JSONObject) -> None:
        for field in (
            "path",
            "signature",
            "impl_context",
            "declaration_sha256",
            "source_file",
            "source_sha256",
        ):
            binding_equal(body.get(field), expected.get(field), f"Rust {field}")
        filename = text_field(body, "source_file")
        binding_equal(self.source_hash(filename), body.get("source_sha256"), filename)


def verify_binding_ledger(records: list[JSONObject], ledger: list[JSONObject]) -> None:
    """Require each exact partial row once; e.g. `verify_binding_ledger(proof, api)`."""
    for record in records:
        proposed = binding_object(record, "proposed_row")
        matches = [
            row
            for row in ledger
            if row.get("upstream_id") == proposed.get("upstream_id")
        ]
        binding_equal(matches, [proposed], f"ledger {proposed.get('upstream_id')}")
