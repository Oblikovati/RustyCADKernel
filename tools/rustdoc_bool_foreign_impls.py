"""Render the bounded `From<&LocalRecord> for bool` rustdoc shape."""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

from api_inventory_json import JSONValue, declaration_digest
from rustdoc_foreign_impls import display_trait, render_foreign_members


def bool_foreign_impl_members(
    impl: Mapping[str, JSONValue], anchor_path: str, anchor_canonical: str,
    anchor_source: str, anchor_digest: str, impl_id: str,
    items: Mapping[str, dict[str, JSONValue]], paths: Mapping[str, JSONValue],
    source_root: Path, unresolved: list[str],
) -> list[dict[str, JSONValue]]:
    trait_path = display_trait(impl.get("trait"), paths)
    if not valid_bool_trait(impl, anchor_path, trait_path):
        unresolved.append(f"impl:{impl_id}:unsupported-bool-foreign-impl")
        return []
    return render_bool_members(
        impl, trait_path, impl_id, items, paths, source_root, unresolved,
        anchor_path, anchor_canonical, anchor_source, anchor_digest,
    )


def valid_bool_trait(
    impl: Mapping[str, JSONValue], anchor_path: str, trait_path: str | None
) -> bool:
    return (
        bool(anchor_path)
        and trait_path is not None
        and impl.get("for") == {"primitive": "bool"}
        and impl.get("is_negative") is False
        and impl.get("blanket_impl") is None
    )


def attach_anchor_provenance(
    members: list[dict[str, JSONValue]],
    anchor_path: str,
    anchor_canonical: str,
    anchor_source: str,
    anchor_digest: str,
) -> list[dict[str, JSONValue]]:
    for member in members:
        add_anchor_provenance(
            member, anchor_path, anchor_canonical, anchor_source, anchor_digest
        )
    return members


def render_bool_members(
    impl: Mapping[str, JSONValue], trait: str, impl_id: str,
    items: Mapping[str, dict[str, JSONValue]], paths: Mapping[str, JSONValue],
    source_root: Path, unresolved: list[str], anchor_path: str,
    anchor_canonical: str, anchor_source: str, anchor_digest: str,
) -> list[dict[str, JSONValue]]:
    members = render_foreign_members(
        impl, "bool", trait, impl_id, items, paths, source_root, unresolved
    )
    return attach_anchor_provenance(
        members, anchor_path, anchor_canonical, anchor_source, anchor_digest
    )


def add_anchor_provenance(
    member: dict[str, JSONValue],
    anchor_path: str,
    anchor_canonical: str,
    anchor_source: str,
    anchor_digest: str,
) -> None:
    member["foreign_impl_self_primitive"] = "bool"
    member["foreign_impl_anchor_path"] = anchor_path
    member["foreign_impl_anchor_canonical"] = anchor_canonical
    member["foreign_impl_anchor_source_file"] = anchor_source
    member["foreign_impl_anchor_source_sha256"] = anchor_digest
    member["declaration_sha256"] = declaration_digest(member)
