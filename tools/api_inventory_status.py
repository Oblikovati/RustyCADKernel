"""Describe the recorded compiler snapshot without promoting it to a parity proof."""

from __future__ import annotations

from pathlib import Path
from api_inventory.inputs import digest, read_object
from capture_rustdoc_json import (
    FINGERPRINT_FILES,
    ROOT,
    config_paths,
    digest_paths,
    workspace_source_paths,
)


def current_workspace_digest() -> str:
    """Use the same source/configuration fingerprint as rustdoc capture."""
    fixed = [path for path in FINGERPRINT_FILES if path.is_file()]
    return digest_paths(workspace_source_paths() + fixed + config_paths(), ROOT)


def compiler_status_lines(root: Path, workspace_digest: str) -> list[str]:
    """Report observed counts and detect stale snapshots; e.g. `compiler_status_lines(root, digest)`."""
    path = root / "porting/api-inventory/completeness.json"
    if not path.is_file():
        return [
            "API coverage: UNKNOWN (compiler inventory snapshot pending); port completion: NOT COMPLETE"
        ]
    report = read_object(path)
    counts = f"{report.get('observed_cpp_api_declarations')} observed C++ declarations; {report.get('matched_declared_mappings')} resolved implementation claims"
    fingerprints, source = report.get("input_sha256"), report.get("rust_capture_inputs")
    ledger_matches = isinstance(fingerprints, dict) and fingerprints.get(
        "parity"
    ) == digest(root / "porting/parity.json")
    source_matches = (
        isinstance(source, dict)
        and source.get("workspace_inputs_sha256") == workspace_digest
    )
    state = (
        "source/ledger fingerprints current"
        if ledger_matches and source_matches
        else "STALE: regenerate after source/ledger changes"
    )
    return [
        f"Compiler API snapshot: {counts} ({state})",
        f"Unresolved ledger rows: {report.get('unresolved_row_count')}; unclaimed C++ declarations: {report.get('unclaimed_cpp_count')}",
        "Full API coverage: UNKNOWN (explicit configuration/identity/mapping gaps); port completion: NOT COMPLETE",
    ]
