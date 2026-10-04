"""Keep rustdoc impl members attached to their resolved self type."""

from __future__ import annotations

from collections.abc import Callable, Mapping
from pathlib import Path
from typing import Protocol

from api_inventory_json import JSONValue, array_value, integer_value
from rustdoc_declarations import file_hash, span_filename, stable_path
from rustdoc_bool_foreign_impls import bool_foreign_impl_members
from rustdoc_foreign_impls import foreign_impl_members
from rustdoc_const_aliases import const_specialized_alias_members, eligible_impl_body


class ImplState(Protocol):
    items: Mapping[str, dict[str, JSONValue]]
    paths: Mapping[str, JSONValue]
    source_root: Path
    unresolved: list[str]
    foreign_impl_links: dict[str, set[str]]
    foreign_impl_candidates: list[dict[str, JSONValue]]
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
    kind, body = read_inner(item)
    if kind == "type_alias":
        return const_specialized_alias_members(item, path, state, read_inner, render_impl)
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
    return render_linked_impl(
        impl_id, owner_id, path, impl_body, state, render_impl
    )


def render_linked_impl(
    impl_id: JSONValue, owner_id: str, path: str, impl: Mapping[str, JSONValue],
    state: ImplState, render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    if impl.get("for") == {"primitive": "bool"}:
        return linked_bool_impl_entries(impl_id, owner_id, path, impl, state)
    impl_owner, owner_error = resolved_impl_owner(impl, state.items)
    if impl_owner is None:
        state.unresolved.append(f"impl:{impl_id}:{owner_error}")
        return []
    if impl_owner != owner_id:
        capture_foreign_impl(impl_id, impl_owner, impl, state)
        return []
    state.owned_impl_ids.add(str(impl_id))
    return render_impl(
        impl, path, state.items, state.paths, state.source_root, state.unresolved
    )


def linked_bool_impl_entries(
    impl_id: JSONValue,
    owner_id: str,
    path: str,
    impl: Mapping[str, JSONValue],
    state: ImplState,
) -> list[dict[str, JSONValue]]:
    anchor_id = bool_from_local_anchor(impl, state.items, state.paths)
    if anchor_id is None or anchor_id != owner_id:
        anchor_path = bool_impl_anchor_path(impl, state.paths)
        state.unresolved.append(f"impl:{impl_id}:unsupported-bool-anchor:{anchor_path}")
        return []
    capture_bool_impl(impl_id, anchor_id, path, impl, state)
    return []


def bool_impl_anchor_path(
    impl: Mapping[str, JSONValue], paths: Mapping[str, JSONValue]
) -> str:
    trait = impl.get("trait")
    argument = only_trait_type_argument(trait) if isinstance(trait, dict) else None
    reference = argument.get("borrowed_ref") if isinstance(argument, dict) else None
    reference_type = reference.get("type") if isinstance(reference, dict) else None
    anchor_id = resolved_path_type_id(reference_type)
    return stable_path(paths, anchor_id, "") if anchor_id is not None else "unknown"


def capture_bool_impl(
    impl_id: JSONValue,
    anchor_id: str,
    anchor_path: str,
    impl: Mapping[str, JSONValue],
    state: ImplState,
) -> None:
    provenance = bool_anchor_provenance(anchor_id, state)
    if provenance is None:
        state.unresolved.append(f"impl:{impl_id}:unverifiable-bool-anchor")
        return
    canonical, source_file, source_digest = provenance
    identifier = str(impl_id)
    state.foreign_impl_candidates.extend(
        bool_foreign_impl_members(
            impl, anchor_path, canonical, source_file, source_digest, identifier,
            state.items, state.paths, state.source_root, state.unresolved,
        )
    )


def bool_anchor_provenance(
    anchor_id: str, state: ImplState
) -> tuple[str, str, str] | None:
    canonical = stable_path(state.paths, anchor_id, "")
    source_file = span_filename(state.items[anchor_id])
    source_digest, source_error = file_hash(state.source_root, source_file)
    if not canonical or source_error is not None or source_digest is None:
        return None
    return canonical, source_file or "", source_digest


def bool_from_local_anchor(
    impl: Mapping[str, JSONValue],
    items: Mapping[str, dict[str, JSONValue]],
    paths: Mapping[str, JSONValue],
) -> str | None:
    if (
        impl.get("for") != {"primitive": "bool"}
        or impl.get("is_negative") is not False
        or not empty_impl_generics(impl)
    ):
        return None
    anchor_id = from_reference_anchor(impl, paths)
    if anchor_id is None:
        return None
    anchor_path = paths.get(anchor_id)
    anchor = items.get(anchor_id)
    return anchor_id if public_nominal_anchor(anchor_path, anchor) else None


def empty_impl_generics(impl: Mapping[str, JSONValue]) -> bool:
    generics = impl.get("generics")
    return (
        isinstance(generics, dict)
        and generics.get("params") == []
        and generics.get("where_predicates") == []
    )


def from_reference_anchor(
    impl: Mapping[str, JSONValue], paths: Mapping[str, JSONValue]
) -> str | None:
    trait = impl.get("trait")
    if not isinstance(trait, dict):
        return None
    trait_id = trait.get("id")
    if not isinstance(trait_id, (int, str)):
        return None
    if stable_path(paths, str(trait_id), "") != "core::convert::From":
        return None
    argument = only_trait_type_argument(trait)
    reference = argument.get("borrowed_ref") if isinstance(argument, dict) else None
    if not isinstance(reference, dict) or reference.get("is_mutable") is not False:
        return None
    if reference.get("lifetime") is not None:
        return None
    return resolved_nominal_id(reference.get("type"), paths)


def only_trait_type_argument(trait: Mapping[str, JSONValue]) -> JSONValue | None:
    arguments = trait.get("args")
    angle = arguments.get("angle_bracketed") if isinstance(arguments, dict) else None
    values = angle.get("args") if isinstance(angle, dict) else None
    if not isinstance(values, list) or len(values) != 1 or angle.get("constraints") != []:
        return None
    value = values[0]
    return value.get("type") if isinstance(value, dict) else None


def resolved_nominal_id(
    value: JSONValue | None, paths: Mapping[str, JSONValue]
) -> str | None:
    identifier = resolved_path_type_id(value)
    if identifier is None:
        return None
    path = paths.get(identifier)
    if not isinstance(path, dict) or path.get("kind") not in {"struct", "enum", "union"}:
        return None
    resolved = value.get("resolved_path") if isinstance(value, dict) else None
    return identifier if isinstance(resolved, dict) and resolved.get("args") is None else None


def public_nominal_anchor(
    path: JSONValue | None, item: Mapping[str, JSONValue] | None
) -> bool:
    return (
        isinstance(path, dict)
        and path.get("kind") in {"struct", "enum", "union"}
        and item is not None
    )


def capture_foreign_impl(
    impl_id: JSONValue, owner_id: str, impl: Mapping[str, JSONValue], state: ImplState
) -> None:
    # Rustdoc indexes foreign-Self trait implementations on their local arguments.
    identifier = str(impl_id)
    state.foreign_impl_links.setdefault(identifier, set()).add(owner_id)
    owner_path = stable_path(state.paths, owner_id, "")
    state.foreign_impl_candidates.extend(
        foreign_impl_members(
            impl, owner_path, identifier, state.items, state.paths,
            state.source_root, state.unresolved,
        )
    )


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
