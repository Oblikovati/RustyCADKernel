"""Resolve exact concrete type arguments of public one-parameter aliases."""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path
from typing import Protocol

from api_inventory_json import JSONValue, array_value
from rustdoc_const_aliases import (
    ImplRenderer,
    InnerReader,
    eligible_impl_body,
    resolved_alias_path_type_id,
)
from rustdoc_declarations import declaration_digest, stable_path


class TypeAliasState(Protocol):
    items: Mapping[str, dict[str, JSONValue]]
    paths: Mapping[str, JSONValue]
    source_root: Path
    unresolved: list[str]


def specialized_type_alias_members(
    item: Mapping[str, JSONValue], path: str, state: TypeAliasState,
    read_inner: InnerReader, render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    target = concrete_alias_target(item, path, state, read_inner)
    if target is None:
        return []
    target_id, parameter, concrete_id = target
    return render_specialized_alias_impls(
        target_id, parameter, concrete_id, path, state, read_inner, render_impl
    )


def concrete_alias_target(
    item: Mapping[str, JSONValue], path: str, state: TypeAliasState, read_inner: InnerReader,
) -> tuple[str, str, str] | None:
    body = zero_generic_alias_body(item, read_inner)
    target_id = resolved_alias_path_type_id(body.get("type")) if body else None
    target = state.items.get(target_id or "")
    if target is None:
        if target_id is not None:
            state.unresolved.append(f"generic-type-alias:{path}:target-type-not-captured")
        return None
    return alias_target_binding(body or {}, path, target_id or "", target, state, read_inner)


def zero_generic_alias_body(
    item: Mapping[str, JSONValue], read_inner: InnerReader
) -> dict[str, JSONValue] | None:
    kind, body = read_inner(item)
    generics = body.get("generics")
    if kind != "type_alias" or not isinstance(generics, dict):
        return None
    if generics.get("params") != [] or generics.get("where_predicates") != []:
        return None
    return body


def alias_target_binding(
    alias: Mapping[str, JSONValue], path: str, target_id: str, target: Mapping[str, JSONValue],
    state: TypeAliasState, read_inner: InnerReader,
) -> tuple[str, str, str] | None:
    kind, body = read_inner(target)
    parameter = generic_parameter(body)
    generics = body.get("generics")
    concrete_id = alias_concrete_type_id(alias.get("type"))
    concrete = state.items.get(concrete_id or "")
    if kind != "struct" or parameter is None or concrete_id is None:
        return None
    if concrete is None:
        state.unresolved.append(f"generic-type-alias:{path}:concrete-type-not-captured")
        return None
    if not alias_target_is_supported(path, generics, concrete, read_inner, state):
        return None
    if not target_bounds_proven(target_id, parameter, concrete_id, body, state):
        return None
    return target_id, parameter, concrete_id


def alias_target_is_supported(
    path: str, generics: JSONValue | None, concrete: Mapping[str, JSONValue],
    read_inner: InnerReader, state: TypeAliasState,
) -> bool:
    if not isinstance(generics, dict) or generics.get("where_predicates") != []:
        return False
    if not is_nominal(concrete, read_inner):
        return False
    if concrete_type_is_closed(concrete, read_inner):
        return True
    state.unresolved.append(f"generic-type-alias:{path}:concrete-type-has-generics")
    return False


def target_bounds_proven(
    target_id: str, parameter: str, concrete_id: str,
    body: Mapping[str, JSONValue], state: TypeAliasState,
) -> bool:
    bounds = target_type_bounds(body, parameter)
    if bounds is None:
        state.unresolved.append(
            f"generic-type-alias:{stable_path(state.paths, target_id, '')}:unsupported-target-bounds"
        )
        return False
    failure = concrete_bounds_failure(bounds, concrete_id, state)
    if failure is not None:
        state.unresolved.append(
            f"generic-type-alias:{stable_path(state.paths, target_id, '')}:{failure}"
        )
        return False
    return True


def generic_parameter(body: Mapping[str, JSONValue]) -> str | None:
    generics = body.get("generics")
    params = generics.get("params") if isinstance(generics, dict) else None
    if not isinstance(params, list) or len(params) != 1:
        return None
    parameter = params[0]
    if not isinstance(parameter, dict) or not isinstance(parameter.get("name"), str):
        return None
    if not isinstance(parameter.get("kind"), dict) or "type" not in parameter["kind"]:
        return None
    return parameter["name"]


def target_type_bounds(body: Mapping[str, JSONValue], name: str) -> list[str] | None:
    generics = body.get("generics")
    params = generics.get("params") if isinstance(generics, dict) else None
    if not isinstance(params, list) or len(params) != 1:
        return None
    return parameter_bounds(params[0], name)


def alias_concrete_type_id(value: JSONValue | None) -> str | None:
    resolved = value.get("resolved_path") if isinstance(value, dict) else None
    args = resolved.get("args") if isinstance(resolved, dict) else None
    angle = args.get("angle_bracketed") if isinstance(args, dict) else None
    values = angle.get("args") if isinstance(angle, dict) else None
    if not isinstance(values, list) or len(values) != 1 or angle.get("constraints") != []:
        return None
    argument = values[0].get("type") if isinstance(values[0], dict) else None
    concrete_id = resolved_alias_path_type_id(argument)
    concrete = argument.get("resolved_path") if isinstance(argument, dict) else None
    return concrete_id if isinstance(concrete, dict) and concrete.get("args") is None else None


def is_nominal(item: Mapping[str, JSONValue], read_inner: InnerReader) -> bool:
    kind, body = read_inner(item)
    if not isinstance(body, dict):
        return False
    return kind in {"struct", "enum", "union"}


def concrete_type_is_closed(
    item: Mapping[str, JSONValue], read_inner: InnerReader
) -> bool:
    _, body = read_inner(item)
    generics = body.get("generics")
    return isinstance(generics, dict) and generics.get("params") == [] and generics.get("where_predicates") == []


def render_specialized_alias_impls(
    target_id: str, parameter: str, concrete_id: str, path: str,
    state: TypeAliasState, read_inner: InnerReader, render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    target = state.items[target_id]
    _, body = read_inner(target)
    result = []
    for impl_id in array_value(body.get("impls", []), "type alias target impls"):
        impl = eligible_impl_body(impl_id, state.items, read_inner)
        if impl is not None:
            result.extend(alias_impl_entries(
                impl_id, target_id, impl, parameter, concrete_id, path, state, render_impl
            ))
    return result


def alias_impl_entries(
    impl_id: JSONValue, target_id: str, impl: Mapping[str, JSONValue], parameter: str,
    concrete_id: str, path: str, state: TypeAliasState, render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    impl_parameter = alias_impl_parameter(impl, target_id, parameter)
    if impl_parameter is None:
        return []
    failure = unsupported_alias_impl(impl, impl_parameter, concrete_id, state)
    if failure is not None:
        state.unresolved.append(f"generic-type-alias:{path}:impl:{impl_id}:{failure}")
        return []
    return emit_specialized_impl(impl, impl_parameter, concrete_id, path, state, render_impl)


def alias_impl_parameter(
    impl: Mapping[str, JSONValue], target_id: str, target_parameter: str
) -> str | None:
    if not exact_generic_self(impl.get("for"), target_id, target_parameter):
        return None
    generics = impl.get("generics")
    impl_params = generics.get("params") if isinstance(generics, dict) else None
    if not isinstance(impl_params, list) or len(impl_params) != 1:
        return None
    parameter = impl_params[0]
    name = parameter.get("name") if isinstance(parameter, dict) else None
    return name if isinstance(name, str) and name == target_parameter else None


def exact_generic_self(
    value: JSONValue | None, target_id: str, parameter: str
) -> bool:
    resolved = value.get("resolved_path") if isinstance(value, dict) else None
    if not isinstance(resolved, dict) or str(resolved.get("id")) != target_id:
        return False
    args = resolved.get("args") if isinstance(resolved, dict) else None
    angle = args.get("angle_bracketed") if isinstance(args, dict) else None
    entries = angle.get("args") if isinstance(angle, dict) else None
    generic = entries[0].get("type") if isinstance(entries, list) and len(entries) == 1 and isinstance(entries[0], dict) else None
    return isinstance(generic, dict) and generic.get("generic") == parameter and angle.get("constraints") == []


def unsupported_alias_impl(
    impl: Mapping[str, JSONValue], parameter: str, concrete_id: str,
    state: TypeAliasState,
) -> str | None:
    generics = impl.get("generics")
    if not isinstance(generics, dict) or generics.get("where_predicates") != []:
        return "unsupported-where-predicates"
    params = generics.get("params")
    item_parameter = params[0] if isinstance(params, list) and params else None
    bounds = parameter_bounds(item_parameter, parameter)
    if bounds is None:
        return "unsupported-impl-bounds"
    return concrete_bounds_failure(bounds, concrete_id, state)


def parameter_bounds(value: JSONValue | None, parameter: str) -> list[str] | None:
    if not isinstance(value, dict) or value.get("name") != parameter:
        return None
    kind = value.get("kind")
    type_kind = kind.get("type") if isinstance(kind, dict) else None
    bounds = type_kind.get("bounds") if isinstance(type_kind, dict) else None
    if not isinstance(bounds, list):
        return None
    identifiers = [bound_trait_id(bound) for bound in bounds]
    return [identifier for identifier in identifiers if identifier is not None] if all(item is not None for item in identifiers) else None


def bound_trait_id(value: JSONValue) -> str | None:
    trait_bound = value.get("trait_bound") if isinstance(value, dict) else None
    trait = trait_bound.get("trait") if isinstance(trait_bound, dict) else None
    identifier = trait.get("id") if isinstance(trait, dict) else None
    if not isinstance(identifier, (str, int)) or isinstance(identifier, bool):
        return None
    if trait_bound.get("modifier") != "none" or trait_bound.get("generic_params") != []:
        return None
    if trait.get("args") is not None:
        return None
    return str(identifier)


def concrete_bounds_failure(
    required: list[str], concrete_id: str, state: TypeAliasState,
) -> str | None:
    concrete = state.items[concrete_id]
    _, body = next(iter(concrete["inner"].items()))
    impl_ids = body.get("impls") if isinstance(body, dict) else None
    if not isinstance(impl_ids, list):
        return "concrete-type-impl-index-missing"
    proven = concrete_trait_impl_ids(impl_ids, concrete_id, state)
    return None if set(required) <= proven else "concrete-type-bound-unproven"


def concrete_trait_impl_ids(
    impl_ids: list[JSONValue], concrete_id: str, state: TypeAliasState,
) -> set[str]:
    result = set()
    for impl_id in impl_ids:
        impl = state.items.get(str(impl_id))
        trait_id = concrete_trait_impl_id(impl, concrete_id)
        if trait_id is not None:
            result.add(trait_id)
    return result


def concrete_trait_impl_id(
    item: Mapping[str, JSONValue] | None, concrete_id: str,
) -> str | None:
    if item is None:
        return None
    inner = item.get("inner")
    impl = inner.get("impl") if isinstance(inner, dict) else None
    generics = impl.get("generics") if isinstance(impl, dict) else None
    if not isinstance(impl, dict) or not isinstance(generics, dict):
        return None
    if generics.get("where_predicates") != []:
        return None
    if generics.get("params") != []:
        return None
    if impl.get("is_negative") is not False:
        return None
    owner = closed_self_type_id(impl.get("for"))
    trait_id = concrete_trait_id(impl.get("trait"))
    return trait_id if owner == concrete_id else None


def closed_self_type_id(value: JSONValue | None) -> str | None:
    resolved = value.get("resolved_path") if isinstance(value, dict) else None
    if not isinstance(resolved, dict) or resolved.get("args") is not None:
        return None
    return resolved_alias_path_type_id(value)


def concrete_trait_id(value: JSONValue | None) -> str | None:
    trait_id = value.get("id") if isinstance(value, dict) else None
    if not isinstance(value, dict) or value.get("args") is not None:
        return None
    if isinstance(trait_id, (str, int)) and not isinstance(trait_id, bool):
        return str(trait_id)
    return None


def emit_specialized_impl(
    impl: Mapping[str, JSONValue], parameter: str, concrete_id: str, path: str,
    state: TypeAliasState, render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    rows = render_impl(impl, path, state.items, state.paths, state.source_root, state.unresolved)
    metadata = alias_specialization(path, parameter, concrete_id, state.paths)
    for row in rows:
        row["alias_specialization"] = metadata.copy()
        row["declaration_sha256"] = declaration_digest(row)
    return rows


def alias_specialization(
    path: str, parameter: str, concrete_id: str,
    paths: Mapping[str, JSONValue],
) -> dict[str, JSONValue]:
    return {
        "alias_path": path,
        "impl_type_parameter": parameter,
        "concrete_type_path": stable_path(paths, concrete_id, ""),
    }
