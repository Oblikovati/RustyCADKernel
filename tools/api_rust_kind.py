"""Distinguish compiler-observed fields from same-named accessor functions."""

from api_inventory.inputs import JSONValue

RustPathIndex = dict[str, list[dict[str, JSONValue]]]
RustCrateIndex = dict[str, RustPathIndex]


def select_rust_kind(crates: RustCrateIndex, kind: JSONValue) -> RustCrateIndex | None:
    """Filter explicit field/function selectors; e.g. `select_rust_kind(crates, 'function')`.

    Missing selectors retain ordinary ambiguity handling at the caller. An explicit
    selector never substitutes for source pinning or resolves duplicate same-kind rows.
    """
    if not isinstance(kind, str) or kind not in {"function", "struct_field"}:
        return None
    return {crate: _select_path_kind(paths, kind) for crate, paths in crates.items()}


def _select_path_kind(paths: RustPathIndex, kind: str) -> RustPathIndex:
    return {
        path: [item for item in items if item.get("kind") == kind]
        for path, items in paths.items()
    }
