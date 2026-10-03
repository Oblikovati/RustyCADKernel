"""Bind crosswalk output to the exact inventory and ledger inputs consumed."""

from __future__ import annotations

from pathlib import Path
from api_inventory.inputs import JSONObject, digest


def crosswalk_input_hashes(
    paths: dict[str, Path], cpp_manifest: JSONObject
) -> JSONObject:
    """Validate and record input bytes; e.g. `crosswalk_input_hashes(paths, manifest)`."""
    hashes: JSONObject = {name: digest(path) for name, path in paths.items()}
    artifacts = cpp_manifest.get("artifacts")
    declaration_artifact = (
        artifacts.get("declarations") if isinstance(artifacts, dict) else None
    )
    if not isinstance(declaration_artifact, dict):
        raise ValueError(
            f"C++ declaration artifact {declaration_artifact!r}: expected hash metadata"
        )
    if declaration_artifact.get("sha256") != hashes["cpp"]:
        raise ValueError(
            f"C++ declarations {hashes['cpp']}: expected {declaration_artifact.get('sha256')}"
        )
    validate_header_manifest(hashes, cpp_manifest)
    return hashes


def validate_header_manifest(hashes: JSONObject, cpp_manifest: JSONObject) -> None:
    """Require the same ownership boundary that the compiler capture used."""
    if "headers" not in hashes:
        return
    inputs = cpp_manifest.get("inputs")
    expected = inputs.get("manifest_sha256") if isinstance(inputs, dict) else None
    if expected != hashes["headers"]:
        raise ValueError(
            f"header manifest {hashes['headers']}: expected compiler input {expected!r}"
        )


def mark_conflicted_declarations(
    declarations: list[JSONObject], conflicts: list[JSONObject]
) -> None:
    """Withhold mappings whose compiler identity has inconsistent observations."""
    identifiers = {row.get("id") for row in conflicts if isinstance(row.get("id"), str)}
    for declaration in declarations:
        if declaration.get("id") in identifiers:
            declaration["inventory_conflict"] = True


def verify_conflict_artifact(manifest: JSONObject, directory: Path) -> Path:
    """Locate the hash-bound conflict table; e.g. `verify_conflict_artifact(manifest, folder)`."""
    artifacts = manifest.get("artifacts")
    row = artifacts.get("unsupported") if isinstance(artifacts, dict) else None
    if not isinstance(row, dict) or not isinstance(row.get("path"), str):
        raise ValueError(f"conflict artifact {row!r}: expected path and SHA256")
    path = directory / row["path"]
    if digest(path) != row.get("sha256"):
        raise ValueError(
            f"conflict artifact {path}: expected recorded hash {row.get('sha256')!r}"
        )
    return path
