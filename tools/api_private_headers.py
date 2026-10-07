"""Recognize ledger rows that describe private OCCT implementation headers.

OCCT ships `.pxx` (private), `.gxx` (generic body) and `.lxx` (inline body) files. The
ones that the public header manifest does not list are implementation detail, not public
API (coordinator decision 2026-10-07): their Rust ports and unit tests stay, but API rows
for them are removed from porting/parity.json and the crosswalk counts nothing for them.
"""

from __future__ import annotations

from api_inventory.inputs import JSONValue

PRIVATE_SUFFIXES = (".pxx", ".gxx", ".lxx")


def is_private_header_row(row: dict[str, JSONValue], public_headers: set[str]) -> bool:
    """True for a row whose source is an unlisted private header.

    Example: `is_private_header_row({"source": "src/A/B/B_Utils.pxx"}, set())` is True.
    """
    source = row.get("source")
    if not isinstance(source, str):
        return False
    return source.endswith(PRIVATE_SUFFIXES) and source not in public_headers


def split_private_header_rows(
    rows: list[dict[str, JSONValue]], public_headers: set[str]
) -> tuple[list[dict[str, JSONValue]], list[dict[str, JSONValue]]]:
    """Partition rows into (public rows, private-header rows), keeping order."""
    public = [row for row in rows if not is_private_header_row(row, public_headers)]
    private = [row for row in rows if is_private_header_row(row, public_headers)]
    return public, private


def public_header_paths(manifest: dict[str, JSONValue]) -> set[str]:
    """Header paths of the public manifest; e.g. `public_header_paths(header_manifest)`."""
    headers = manifest.get("headers")
    if not isinstance(headers, list):
        raise ValueError(f"header manifest field 'headers' {type(headers).__name__}: expected a list")
    return {row["path"] for row in headers if isinstance(row, dict) and isinstance(row.get("path"), str)}
