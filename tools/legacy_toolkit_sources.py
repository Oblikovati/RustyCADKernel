"""Validate complete OCCT GTest source inventories for runtime toolkits."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re

from api_inventory.inputs import JSONObject

SUPPORTED_RUNTIME_TOOLKITS = ("TKMath", "TKG2d", "TKG3d")
GTEST_MANIFESTS = {
    "TKMath": "src/FoundationClasses/TKMath/GTests/FILES.cmake",
    "TKG2d": "src/ModelingData/TKG2d/GTests/FILES.cmake",
    "TKG3d": "src/ModelingData/TKG3d/GTests/FILES.cmake",
}
DECLARATION = re.compile(r"(?:TEST(?:_F)?|TYPED_TEST)\((\w+), (\w+)\)")
SOURCE_FILE = re.compile(r"^\s+([A-Za-z0-9_]+\.cxx)$", re.M)


def toolkit_rows(ledger_path: Path, toolkit: str = "TKMath") -> list[JSONObject]:
    """Load translated declarations; e.g. `toolkit_rows(ledger, 'TKG2d')`."""
    require_supported_toolkit(toolkit)
    ledger = json.loads(ledger_path.read_text())
    rows = [row for row in ledger["tests"] if row["toolkit"] == toolkit]
    if not rows or any(row["status"] != "translated" for row in rows):
        raise RuntimeError(f"{toolkit} mappings must be nonempty and translated")
    return rows


def require_supported_toolkit(toolkit: str) -> None:
    """Reject toolkits without a pinned GTest manifest; e.g. `require_supported_toolkit('TKG2d')`."""
    if toolkit not in SUPPORTED_RUNTIME_TOOLKITS:
        raise ValueError(f"Unsupported runtime toolkit {toolkit!r}")


def verify_source_rows(source: Path, rows: list[JSONObject]) -> set[str]:
    """Validate files, declarations, and hashes; e.g. `verify_source_rows(occt, rows)`."""
    if not rows:
        raise ValueError("Expected nonempty toolkit source rows")
    toolkit = str(rows[0]["toolkit"])
    require_one_toolkit(rows, toolkit)
    verify_test_source_manifest(source, toolkit, rows)
    grouped = group_source_rows(rows)
    for relative, mappings in grouped.items():
        verify_source_group(source, relative, mappings)
    return set(grouped)


def require_one_toolkit(rows: list[JSONObject], toolkit: str) -> None:
    """Reject mixed-toolkit evidence inputs; e.g. `require_one_toolkit(rows, 'TKG2d')`."""
    if not rows or any(row["toolkit"] != toolkit for row in rows):
        raise ValueError(f"Expected nonempty {toolkit} rows without mixed toolkits")


def verify_test_source_manifest(
    source: Path, toolkit: str, rows: list[JSONObject]
) -> None:
    """Require parity rows to cover every CMake-listed GTest file for a toolkit."""
    require_supported_toolkit(toolkit)
    relative = GTEST_MANIFESTS[toolkit]
    manifest = source / relative
    listed = listed_test_sources(source, manifest)
    mapped = {str(row["source"]) for row in rows}
    if listed != mapped:
        raise RuntimeError(
            f"{toolkit} test source coverage differs from {manifest}: missing={sorted(listed - mapped)!r}, extra={sorted(mapped - listed)!r}"
        )


def listed_test_sources(source: Path, manifest: Path) -> set[str]:
    """Read CMake's GTest source list; e.g. `listed_test_sources(occt, files_cmake)`."""
    parent = manifest.relative_to(source).parent
    filenames = SOURCE_FILE.findall(manifest.read_text())
    if not filenames:
        raise RuntimeError(f"{manifest}: expected listed GTest source filenames")
    if len(filenames) != len(set(filenames)):
        raise RuntimeError(f"{manifest}: duplicate GTest source filename")
    return {(parent / filename).as_posix() for filename in filenames}


def group_source_rows(rows: list[JSONObject]) -> dict[str, list[JSONObject]]:
    """Group exact rows by file; e.g. `group_source_rows(toolkit_rows)`."""
    grouped: dict[str, list[JSONObject]] = {}
    for row in rows:
        grouped.setdefault(str(row["source"]), []).append(row)
    return grouped


def verify_source_group(
    source: Path, relative: str, mappings: list[JSONObject]
) -> None:
    """Require source declarations and digest to match; e.g. `verify_source_group(root, path, rows)`."""
    if not mappings:
        raise RuntimeError(f"Stale or incomplete TKMath source mapping: {relative}")
    contents = (source / relative).read_bytes()
    digest = hashlib.sha256(contents).hexdigest()
    declarations = {
        f"{suite}.{case}" for suite, case in DECLARATION.findall(contents.decode())
    }
    mapped = {str(row["upstream_id"]) for row in mappings}
    toolkit = str(mappings[0].get("toolkit", "TKMath"))
    if declarations != mapped or any(
        row["source_sha256"] != digest for row in mappings
    ):
        raise RuntimeError(f"Stale or incomplete {toolkit} source mapping: {relative}")
