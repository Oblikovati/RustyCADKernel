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
    return hashes
