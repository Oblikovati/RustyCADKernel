"""Fingerprint the project-owned code that normalizes and crosswalks API evidence."""

from __future__ import annotations

import hashlib
from pathlib import Path
import re
from typing import TypeAlias

ToolHashMap: TypeAlias = dict[str, dict[str, str]]

IMPLEMENTATION_FILES: dict[str, tuple[str, ...]] = {
    "rust_normalizer": (
        "tools/normalize_rustdoc_inventory.py",
        "tools/api_inventory_capture.py",
        "tools/api_inventory_json.py",
        "tools/api_inventory_reexports.py",
        "tools/rustdoc_declarations.py",
        "tools/rustdoc_impl_ownership.py",
        "tools/rustdoc_foreign_impls.py",
        "tools/rustdoc_trait_paths.py",
        "tools/api_tool_hashes.py",
    ),
    "api_crosswalk": (
        "tools/api_coverage_report.py",
        "tools/api_crosswalk.py",
        "tools/api_identity.py",
        "tools/api_report_inputs.py",
        "tools/api_inventory/__init__.py",
        "tools/api_inventory/inputs.py",
        "tools/api_tool_hashes.py",
    ),
}


def implementation_hashes(root: Path) -> ToolHashMap:
    """Hash every normalization and crosswalk input; `implementation_hashes(repo_root)`."""
    hashes: ToolHashMap = {}
    for stage in IMPLEMENTATION_FILES:
        hashes[stage] = stage_hashes(root, stage)
    return hashes


def stage_hashes(root: Path, stage: str) -> dict[str, str]:
    """Hash one stage's implementation files; `stage_hashes(repo_root, 'rust_normalizer')`."""
    files = IMPLEMENTATION_FILES.get(stage)
    if files is None:
        raise ValueError(f"implementation stage {stage!r}: expected a configured stage")
    return {relative: file_sha256(require_file(root, relative)) for relative in files}


def implementation_hashes_match(root: Path, recorded: object) -> bool:
    """Fail closed when metadata is absent, malformed, stale, or references deleted code."""
    if not valid_hash_map(recorded):
        return False
    try:
        return recorded == implementation_hashes(root)
    except (OSError, ValueError):
        return False


def valid_hash_map(value: object) -> bool:
    if not isinstance(value, dict) or set(value) != set(IMPLEMENTATION_FILES):
        return False
    for stage, paths in IMPLEMENTATION_FILES.items():
        hashes = value.get(stage)
        if not isinstance(hashes, dict) or set(hashes) != set(paths):
            return False
        if any(not is_sha256(digest) for digest in hashes.values()):
            return False
    return True


def is_sha256(value: object) -> bool:
    if not isinstance(value, str):
        return False
    match = re.fullmatch(r"[0-9a-f]{64}", value)
    return match is not None


def require_file(root: Path, relative: str) -> Path:
    path = root / relative
    if not path.is_file():
        raise ValueError(f"implementation input {relative}: expected an existing file")
    return path


def file_sha256(path: Path) -> str:
    with path.open("rb") as stream:
        file_digest = hashlib.file_digest(stream, "sha256")
    return file_digest.hexdigest()
