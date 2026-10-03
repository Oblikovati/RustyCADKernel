"""Keep rustdoc impl members attached to their resolved self type."""

from __future__ import annotations

from collections.abc import Callable, Mapping
from pathlib import Path
from typing import Protocol

from api_inventory_json import JSONValue, array_value, integer_value


class ImplState(Protocol):
    items: Mapping[str, dict[str, JSONValue]]
    paths: Mapping[str, JSONValue]
    source_root: Path
    unresolved: list[str]
    foreign_impl_links: dict[str, set[str]]
    owned_impl_ids: set[str]


class InnerReader(Protocol):
    def __call__(
        self, item: Mapping[str, JSONValue]
    ) -> tuple[str, dict[str, JSONValue]]: ...


class ImplRenderer(Protocol):
    def __call__(
        self,
        impl: Mapping[str, JSONValue],
        path: str,
        items: Mapping[str, dict[str, JSONValue]],
        paths: Mapping[str, JSONValue],
        source_root: Path,
        unresolved: list[str],
    ) -> list[dict[str, JSONValue]]: ...


def inherent_members(
    item: Mapping[str, JSONValue],
    path: str,
    state: ImplState,
    read_inner: InnerReader,
    render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    _, body = read_inner(item)
    owner_id = str(integer_value(item.get("id"), "impl owner id"))
    return [
        entry
        for impl_id in array_value(body.get("impls", []), "impls")
        for entry in linked_impl_entries(
            impl_id, owner_id, path, state, read_inner, render_impl
        )
    ]


def linked_impl_entries(
    impl_id: JSONValue,
    owner_id: str,
    path: str,
    state: ImplState,
    read_inner: InnerReader,
    render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    impl_body = eligible_impl_body(impl_id, state.items, read_inner)
    if impl_body is None:
        return []
    impl_owner, owner_error = resolved_impl_owner(impl_body, state.items)
    if impl_owner is None:
        state.unresolved.append(f"impl:{impl_id}:{owner_error}")
        return []
    if impl_owner != owner_id:
        # Rustdoc indexes trait implementations on local trait arguments too.
        state.foreign_impl_links.setdefault(str(impl_id), set()).add(impl_owner)
        return []
    state.owned_impl_ids.add(str(impl_id))
    return render_impl(
        impl_body, path, state.items, state.paths, state.source_root, state.unresolved
    )


def eligible_impl_body(
    impl_id: JSONValue,
    items: Mapping[str, dict[str, JSONValue]],
    read_inner: InnerReader,
) -> dict[str, JSONValue] | None:
    impl = items.get(str(impl_id))
    if impl is None:
        return None
    impl_kind, impl_body = read_inner(impl)
    if impl_kind != "impl" or impl_body.get("is_synthetic") is True:
        return None
    return None if impl_body.get("blanket_impl") is not None else impl_body


def resolved_impl_owner(
    impl: Mapping[str, JSONValue], items: Mapping[str, dict[str, JSONValue]]
) -> tuple[str | None, str]:
    owner_id = resolved_path_type_id(impl.get("for"))
    if owner_id is None:
        return None, "unknown-self-type"
    resolved, alias_unknown = resolve_alias_owner(owner_id, items, set())
    return resolved, "unknown-self-alias" if alias_unknown else "unknown-self-type"


def resolved_path_type_id(type_value: JSONValue | None) -> str | None:
    if not isinstance(type_value, dict) or len(type_value) != 1:
        return None
    kind, value = next(iter(type_value.items()))
    if kind != "resolved_path" or not isinstance(value, dict):
        return None
    identifier = value.get("id")
    if isinstance(identifier, (int, str)) and not isinstance(identifier, bool):
        return str(identifier)
    return None


def resolve_alias_owner(
    type_id: str, items: Mapping[str, dict[str, JSONValue]], visited: set[str]
) -> tuple[str | None, bool]:
    if type_id in visited:
        return None, True
    item = items.get(type_id)
    if item is None:
        return type_id, False
    inner = item.get("inner")
    if not isinstance(inner, dict) or len(inner) != 1:
        return None, True
    kind, body = next(iter(inner.items()))
    if kind != "type_alias" or not isinstance(body, dict):
        return type_id, False
    target_id = resolved_path_type_id(body.get("type"))
    if target_id is None:
        return None, True
    return resolve_alias_owner(target_id, items, visited | {type_id})


def unresolved_impl_owner_links(
    foreign_impl_links: dict[str, set[str]], owned_impl_ids: set[str]
) -> list[str]:
    return [
        f"impl:{impl_id}:unreachable-self-owner:{owner_id}"
        for impl_id, owner_ids in foreign_impl_links.items()
        if impl_id not in owned_impl_ids
        for owner_id in sorted(owner_ids)
    ]
