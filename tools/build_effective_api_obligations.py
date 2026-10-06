#!/usr/bin/env python3
"""Stage or replay an access-qualified API obligation overlay without changing primary policy."""

from __future__ import annotations

import argparse
import gzip
import json
from collections import Counter
from pathlib import Path

from api_inventory.effective_obligations import effective_obligation_union
from api_inventory.inputs import (
    JSONObject,
    digest,
    object_rows,
    read_object,
    text_field,
)
from api_inventory.merge import write_jsonl_gzip
from probe_effective_api_access import CONTROLS, EXPECTED_STDOUT, access_diagnostic
from report_returned_type_inventory import read_table
from verify_returned_private_reachability import verify_files

ROOT = Path(__file__).resolve().parents[1]
STAGED = Path("porting/staged-api-inventory/returned-types-v1/global")
OUTPUT = Path("porting/staged-api-inventory/effective-obligations-v1")
RECEIPT = Path("porting/audits/effective-api-obligations-v1.json")
INCOMPLETE_POLICY: JSONObject = {
    "implementation_credit": False,
    "api_denominator_complete": False,
    "primary_visibility_policy_changed": False,
    "private_type_names_accessible": False,
}


def verify_native_demonstration(root: Path, contracts: JSONObject) -> None:
    """Authenticate the exact successful fixture and rejected controls; e.g. `verify_native_demonstration(root, contracts)`."""
    verify_access_artifacts(root, contracts)
    report = read_object(root / text_field(contracts, "native_report"))
    controls = object_rows(report.get("controls"), "native controls")
    if (
        len(controls) != len(CONTROLS)
        or {row.get("control") for row in controls} != CONTROLS.keys()
    ):
        raise ValueError(
            f"controls {[row.get('control') for row in controls]!r}: expected {list(CONTROLS)}"
        )
    for row in controls:
        verify_control_record(row)
    if report.get("fixture_sha256") != contracts.get("fixture_sha256"):
        raise ValueError(
            "native fixture digest: expected the attested access contract fixture"
        )


def verify_access_artifacts(root: Path, contracts: JSONObject) -> None:
    """Require complete attestation and no credit; e.g. `verify_access_artifacts(root, contracts)`."""
    artifacts = object_rows(contracts.get("artifacts"), "access artifacts")
    paths = {text_field(artifact, "path") for artifact in artifacts}
    required = {
        text_field(contracts, field)
        for field in ("fixture", "native_report", "rust_inventory")
    }
    if len(paths) != len(artifacts) or not required.issubset(paths):
        raise ValueError(
            f"artifact paths {paths!r}: expected unique paths including {required!r}"
        )
    for flag in ("implementation_credit", "api_denominator_complete"):
        if contracts.get(flag) is not False:
            raise ValueError(f"{flag}={contracts.get(flag)!r}: expected false")
    for artifact in artifacts:
        if digest(root / text_field(artifact, "path")) != artifact.get("sha256"):
            raise ValueError(f"artifact {artifact!r}: expected recorded SHA256")


def verify_control_record(row: JSONObject) -> None:
    """Reject failed or promoted native evidence; e.g. `verify_control_record(control)`."""
    if row.get("expected_outcome_observed") is not True:
        raise ValueError(f"control {row!r}: expected demonstrated access outcome")
    if type(row.get("exit_code")) is not int:
        raise ValueError(f"control {row!r}: expected integer exit code")
    if row.get("control") != "positive":
        verify_negative_record(row)
        return
    run = row.get("execution")
    if (
        row.get("exit_code") != 0
        or not isinstance(run, dict)
        or run.get("exit_code") != 0
        or run.get("stdout") != EXPECTED_STDOUT
    ):
        raise ValueError(
            f"positive control {row!r}: expected successful compile and exact runtime output"
        )


def verify_negative_record(row: JSONObject) -> None:
    """Require the access rejection category; e.g. `verify_negative_record(row)`."""
    control = text_field(row, "control")
    if control not in CONTROLS or row.get("exit_code") == 0:
        raise ValueError(
            f"negative control {row!r}: expected known control and nonzero exit"
        )
    if not access_diagnostic(control, text_field(row, "stderr")):
        raise ValueError(
            f"negative control {row!r}: expected matching private/protected/abstract diagnostic"
        )


def rust_declarations(path: Path) -> list[JSONObject]:
    """Read publicly reachable compiler declarations; e.g. `rust_declarations(normalized_gzip)`."""
    with gzip.open(path, "rt") as stream:
        inventory: JSONObject = json.load(stream)
    crates = object_rows(inventory.get("crates"), "Rust crates")
    return [
        row
        for crate in crates
        for row in object_rows(crate.get("declarations"), "Rust declarations")
    ]


def select_current_rust(
    root: Path, contract: JSONObject, declarations: list[JSONObject]
) -> JSONObject | None:
    """Require one current exact Rust declaration; e.g. `select_current_rust(root, contract, rows)`."""
    target = contract.get("rust_target")
    if not isinstance(target, dict):
        return None
    candidates = [
        row
        for row in declarations
        if row.get("path") == target.get("path")
        and row.get("kind") == target.get("kind")
    ]
    if len(candidates) != 1:
        raise ValueError(
            f"Rust target {target!r}: expected one declaration, got {len(candidates)}"
        )
    return require_current_rust(root, candidates[0])


def require_current_rust(root: Path, declaration: JSONObject) -> JSONObject:
    """Reject stale source evidence; e.g. `require_current_rust(root, declaration)`."""
    if digest(root / text_field(declaration, "source_file")) != declaration.get(
        "source_sha256"
    ):
        raise ValueError(
            f"Rust declaration {declaration!r}: expected current source hash"
        )
    verify_function_body(declaration)
    return declaration


def verify_function_body(declaration: JSONObject) -> None:
    """Withhold declaration-only or specialized implementations; e.g. `verify_function_body(row)`."""
    if declaration.get("kind") != "function":
        return
    signature, context = declaration.get("signature"), declaration.get("impl_context")
    if not isinstance(signature, dict) or signature.get("has_body") is not True:
        raise ValueError(f"Rust declaration {declaration!r}: expected compiler body")
    if not isinstance(context, dict) or context.get("trait") is not None:
        raise ValueError(
            f"Rust impl {context!r}: expected inherent SparseArrayBlock body"
        )
    verify_block_impl_context(context)


def verify_block_impl_context(context: JSONObject) -> None:
    """Require the exact block implementation context; e.g. `verify_block_impl_context(context)`."""
    expected = {
        "resolved_path": {
            "args": None,
            "path": "SparseArrayBlock",
            "resolved_item_path": "rustycad_tkernel::ncollection::sparse_array_base::block::SparseArrayBlock",
        }
    }
    if context.get("for") != expected or context.get("generics") != {
        "params": [],
        "where_predicates": [],
    }:
        raise ValueError(
            f"Rust impl {context!r}: expected exact nongeneric SparseArrayBlock context"
        )


def reconciliation_rows(
    root: Path, contracts: list[JSONObject], rust: list[JSONObject]
) -> list[JSONObject]:
    """Record partial and missing surfaces without ledger changes; e.g. `reconciliation_rows(root, proofs, rust)`."""
    return [
        {
            "native_id": row["native_id"],
            "qualified_name": row["qualified_name"],
            "signature": row["signature"],
            "status": row["reconciliation_status"],
            "remaining_gaps": row["remaining_gaps"],
            "rust_declaration": select_current_rust(root, row, rust),
            "existing_evidence": row["existing_evidence"],
            "implementation_credit": False,
        }
        for row in contracts
    ]


def staged_obligation_content(
    root: Path, contracts: JSONObject
) -> tuple[list[JSONObject], list[JSONObject], JSONObject]:
    """Generate the reproducible union and scoped reconciliation; e.g. `staged_obligation_content(root, contracts)`."""
    verify_native_demonstration(root, contracts)
    declarations, edges = (
        read_table(root / STAGED, "declarations"),
        read_table(root / STAGED, "returned-type-reachability"),
    )
    proofs = object_rows(contracts.get("declarations"), "access declarations")
    obligations = effective_obligation_union(declarations, edges, proofs)
    rust = rust_declarations(root / text_field(contracts, "rust_inventory"))
    matrix = reconciliation_rows(root, proofs, rust)
    return (
        obligations,
        matrix,
        obligation_summary(declarations, edges, obligations, matrix),
    )


def obligation_summary(
    declarations: list[JSONObject],
    edges: list[JSONObject],
    obligations: list[JSONObject],
    matrix: list[JSONObject],
) -> JSONObject:
    """Keep unresolved evidence outside any completeness claim; e.g. `obligation_summary(native, edges, union, matrix)`."""
    primary_count = sum(row.get("api_visible") is True for row in declarations)
    return {
        "schema": "rustycad.effective-api-obligations.v1",
        "primary_candidates": primary_count,
        "effective_obligations": len(obligations),
        "demonstrated_inferred_obligations": len(matrix),
        "supplemental_obligations_added": len(obligations) - primary_count,
        "returned_edge_observations": len(edges),
        "returned_edge_statuses": status_counts(edges),
        "reconciliation_statuses": status_counts(matrix),
        **INCOMPLETE_POLICY,
    }


def status_counts(rows: list[JSONObject]) -> JSONObject:
    """Count all explicit statuses; e.g. `status_counts(edges)`."""
    return dict(Counter(text_field(row, "status") for row in rows))


def write_or_verify(path: Path, rows: list[JSONObject], verify: bool) -> None:
    """Replay semantic rows or write deterministic gzip; e.g. `write_or_verify(path, rows, True)`."""
    if not verify:
        write_jsonl_gzip(path, rows)
        return
    with gzip.open(path, "rt") as stream:
        actual = [json.loads(line) for line in stream]
    if actual != rows:
        raise ValueError(
            f"staged artifact {path}: expected exact regenerated obligation rows"
        )


def stage_or_verify(verify: bool) -> JSONObject:
    """Replay the immutable overlay when verify is true; e.g. `stage_or_verify(True)`."""
    destination = ROOT / OUTPUT
    if verify:
        verify_files(ROOT, read_object(ROOT / RECEIPT))
    contracts = read_object(destination / "access-contracts.json")
    obligations, matrix, summary = staged_obligation_content(ROOT, contracts)
    write_or_verify(destination / "effective-obligations.jsonl.gz", obligations, verify)
    write_or_verify(
        destination / "supplemental-reconciliation.jsonl.gz", matrix, verify
    )
    if verify and read_object(destination / "summary.json") != summary:
        raise ValueError(
            "staged summary: expected exact regenerated counts and limitation flags"
        )
    if not verify:
        (destination / "summary.json").write_text(
            json.dumps(summary, indent=2, sort_keys=True) + "\n"
        )
    return summary


def main() -> None:
    """Use --verify to replay the staged overlay; generation never changes primary inventory."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify", action="store_true")
    print(json.dumps(stage_or_verify(parser.parse_args().verify), sort_keys=True))


if __name__ == "__main__":
    main()
