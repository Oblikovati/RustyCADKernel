"""Expose trait impls whose Self type is a raw pointer, anchored on a local nominal type.

`impl<T> PartialEq<Handle<T>> for *const T` ports C++ `operator==(const T*, const
handle<T>&)`. Rustdoc lists it under `Handle`, the local type in its trait arguments, but
its Self has no path, so the impl owner walk dropped it as `unknown-self-type`. Such an
impl is emitted once, by the crate that defines the anchor, with the UFCS path
`<*const T as core::cmp::PartialEq<krate::...::Handle<T>>>::eq`, and the crosswalk
resolves that path by exact match across crates.
"""

from __future__ import annotations

from collections.abc import Mapping

from api_inventory_json import JSONValue, declaration_digest
from rustdoc_generic_arguments import display_raw_pointer


def pointer_self_text(
    impl: Mapping[str, JSONValue], paths: Mapping[str, JSONValue]
) -> str | None:
    """Rendered raw-pointer Self, or None; e.g. `*const T` for `impl ... for *const T`."""
    receiver = impl.get("for")
    if not isinstance(receiver, dict) or set(receiver) != {"raw_pointer"}:
        return None
    return display_raw_pointer(receiver["raw_pointer"], paths)


def referenced_ids(value: JSONValue) -> set[str]:
    """Every `resolved_path` id inside a rustdoc type tree."""
    if isinstance(value, list):
        return set().union(*(referenced_ids(item) for item in value)) if value else set()
    if not isinstance(value, dict):
        return set()
    found = set().union(*(referenced_ids(item) for item in value.values())) if value else set()
    resolved = value.get("resolved_path")
    identifier = resolved.get("id") if isinstance(resolved, dict) else None
    if isinstance(identifier, (int, str)) and not isinstance(identifier, bool):
        found.add(str(identifier))
    return found


def pointer_anchor_status(
    impl: Mapping[str, JSONValue], owner_id: str, paths: Mapping[str, JSONValue]
) -> str:
    """`emit` for the defining crate, `skip` for a re-exporting crate, else a gap reason."""
    if owner_id not in referenced_ids(impl.get("trait")):
        return "pointer-self-anchor-not-in-trait-arguments"
    record = paths.get(owner_id)
    if not isinstance(record, dict) or not isinstance(record.get("crate_id"), int):
        return "pointer-self-anchor-without-path"
    # crate_id 0 is the documented crate; a facade re-export sees the same impl again.
    return "emit" if record["crate_id"] == 0 else "skip"


def pointer_impl_entries(
    rendered: list[dict[str, JSONValue]], self_text: str, trait_path: str
) -> list[dict[str, JSONValue]]:
    """Rename rendered members to UFCS; `pointer_impl_entries(rows, "*const T", trait)`."""
    output = []
    for entry in rendered:
        member = str(entry.get("path", "")).rpartition("::")[2]
        renamed = dict(entry)
        renamed["path"] = f"<{self_text} as {trait_path}>::{member}"
        renamed["pointer_self_type"] = self_text
        renamed["declaration_sha256"] = declaration_digest(renamed)
        output.append(renamed)
    return output
