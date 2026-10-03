"""Describe the recorded compiler snapshot without promoting it to a parity proof."""

from __future__ import annotations

from pathlib import Path
from api_inventory.inputs import JSONObject, digest, read_object
from api_tool_hashes import implementation_hashes_match
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
    source = report.get("rust_capture_inputs")
    source_matches = (
        isinstance(source, dict)
        and source.get("workspace_inputs_sha256") == workspace_digest
    )
    state = (
        "source/ledger/artifact/tool fingerprints current"
        if recorded_inputs_match(root, report) and source_matches
        else "STALE: regenerate after source/ledger/artifact/tool changes"
    )
    return [
        f"Compiler API snapshot: {counts} ({state})",
        f"Unresolved ledger rows: {report.get('unresolved_row_count')}; unclaimed C++ declarations: {report.get('unclaimed_cpp_count')}",
        "Full API coverage: UNKNOWN (explicit configuration/identity/mapping gaps); port completion: NOT COMPLETE",
    ]


def recorded_inputs_match(root: Path, report: JSONObject) -> bool:
    """Verify recorded C++ inputs and archives; e.g. `recorded_inputs_match(root, report)`."""
    inputs = report.get("input_sha256")
    if not isinstance(inputs, dict):
        return False
    paths = {
        "parity": "porting/parity.json",
        "cpp": "porting/api-inventory/occt-declarations.jsonl.gz",
        "cpp_manifest": "porting/api-inventory/occt-manifest.json",
        "headers": "porting/api-header-manifest.json",
    }
    for key, name in paths.items():
        path = root / name
        if not path.is_file() or inputs.get(key) != digest(path):
            return False
    return (
        implementation_hashes_match(root, report.get("tool_implementation_sha256"))
        and archived_inputs_match(root, report)
        and cpp_archives_match(root)
    )


def archived_inputs_match(root: Path, report: JSONObject) -> bool:
    """Reject stale Rust/crosswalk archives; e.g. `archived_inputs_match(root, report)`."""
    artifacts = report.get("artifacts")
    if not isinstance(artifacts, dict):
        return False
    for name in ("rust-api", "api-crosswalk"):
        expected = artifacts.get(name)
        path = root / "porting/api-inventory" / f"{name}.json.gz"
        if not isinstance(expected, dict) or not path.is_file():
            return False
        if expected.get("sha256") != digest(path):
            return False
    return True


def cpp_archives_match(root: Path) -> bool:
    """Verify every C++ evidence table; e.g. `cpp_archives_match(root)`."""
    folder = root / "porting/api-inventory"
    manifest = read_object(folder / "occt-manifest.json")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, dict) or not artifacts:
        return False
    for name, expected in artifacts.items():
        path = folder / f"occt-{name}.jsonl.gz"
        if not isinstance(expected, dict) or not path.is_file():
            return False
        if expected.get("sha256") != digest(path):
            return False
    return True
