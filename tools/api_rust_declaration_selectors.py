"""Select explicit compiler declarations without guessing Rust specialization intent."""

from __future__ import annotations

from collections.abc import Callable
import re

from api_inventory.inputs import JSONObject, JSONValue
from api_rust_kind import RustCrateIndex

RustResolution = tuple[str, list[JSONObject]]
RustResolver = Callable[[JSONValue, RustCrateIndex], RustResolution]
SELECTOR_FIELDS = {"path", "declaration_sha256", "impl_context"}
SELECTION_STATUS = {
    "missing": "missing_rust_selection",
    "ambiguous": "ambiguous_rust_selection",
    "unknown_rust_crate": "missing_rust_selection",
    "unverifiable_source": "unverifiable_rust_selection",
}
SELECTION_FAILURES = {"invalid_rust_selectors", *SELECTION_STATUS.values()}


def declaration_selectors(row: JSONObject, paths: list[str]) -> list[JSONObject] | None:
    """Validate a complete explicit selection; e.g. `declaration_selectors(row, paths)`."""
    supplied = row.get("rust_declaration_selectors")
    if not isinstance(supplied, list) or not supplied:
        return None
    if any(not valid_declaration_selector(item) for item in supplied):
        return None
    selectors = [item for item in supplied if isinstance(item, dict)]
    identities = [(item["path"], item["declaration_sha256"]) for item in selectors]
    if len(set(identities)) != len(identities):
        return None
    selected_paths = list(dict.fromkeys(item["path"] for item in selectors))
    return selectors if selected_paths == paths else None


def valid_declaration_selector(value: JSONValue) -> bool:
    """Require digest and impl context together; e.g. `valid_declaration_selector(item)`."""
    if not isinstance(value, dict) or set(value) != SELECTOR_FIELDS:
        return False
    path, digest = value.get("path"), value.get("declaration_sha256")
    if not isinstance(path, str) or not path or not isinstance(digest, str):
        return False
    context = value.get("impl_context")
    return re.fullmatch(r"[0-9a-f]{64}", digest) is not None and (
        context is None or isinstance(context, dict)
    )


def matching_declarations(
    items: list[JSONObject], selector: JSONObject
) -> list[JSONObject]:
    """Match both identities exactly; e.g. `matching_declarations(items, selector)`."""
    return [
        item
        for item in items
        if all(item.get(field) == selector[field] for field in SELECTOR_FIELDS)
    ]


def selected_declaration_index(
    crates: RustCrateIndex, selector: JSONObject
) -> RustCrateIndex:
    """Keep crate ownership for existing resolution; `selected_declaration_index(crates, item)`."""
    path = selector["path"]
    assert isinstance(path, str)
    return {
        crate: {path: matching_declarations(paths.get(path, []), selector)}
        for crate, paths in crates.items()
    }


def selected_rust_candidates(
    row: JSONObject, paths: list[str], crates: RustCrateIndex, resolve: RustResolver
) -> RustResolution:
    """Resolve every declared body independently; `selected_rust_candidates(row, paths, index, resolve)`."""
    selectors = declaration_selectors(row, paths)
    if selectors is None:
        return "invalid_rust_selectors", []
    matches = [
        resolve(selector["path"], selected_declaration_index(crates, selector))
        for selector in selectors
    ]
    status = next((status for status, _ in matches if status != "unique"), "unique")
    return SELECTION_STATUS.get(status, status), [
        item for _, items in matches for item in items
    ]
