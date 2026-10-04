"""Resolve inherent methods through a narrowly supported const alias shape."""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path
from typing import Protocol

from api_inventory_json import JSONValue, array_value
from rustdoc_declarations import stable_path


class AliasState(Protocol):
    items: Mapping[str, dict[str, JSONValue]]
    paths: Mapping[str, JSONValue]
    source_root: Path
    unresolved: list[str]


class InnerReader(Protocol):
    def __call__(self, item: Mapping[str, JSONValue]) -> tuple[str, dict[str, JSONValue]]: ...


class ImplRenderer(Protocol):
    def __call__(
        self, impl: Mapping[str, JSONValue], path: str,
        items: Mapping[str, dict[str, JSONValue]], paths: Mapping[str, JSONValue],
        source_root: Path, unresolved: list[str],
    ) -> list[dict[str, JSONValue]]: ...


def const_specialized_alias_members(
    item: Mapping[str, JSONValue],
    path: str,
    state: AliasState,
    read_inner: InnerReader,
    render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    """Attach exact inherent members; `const_specialized_alias_members(alias, ...)`.

    Only aliases of `Record<T, N>` with a literal const binding are expanded.
    """
    return alias_members_for_item(item, path, state, read_inner, render_impl)


def alias_members_for_item(
    item: Mapping[str, JSONValue], path: str, state: AliasState,
    read_inner: InnerReader, render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    target_id, const_value, reason = const_specialized_alias_target(item, state, read_inner)
    if reason is not None:
        state.unresolved.append(f"const-specialized-alias:{path}:{reason}")
        return []
    if target_id is None or const_value is None:
        return []
    alias_type = alias_type_parameter(item, read_inner)
    if alias_type is None:
        state.unresolved.append(f"const-specialized-alias:{path}:unsupported-type-parameter")
        return []
    return alias_target_impl_members(
        target_id, const_value, alias_type, path, state, read_inner, render_impl
    )


def const_specialized_alias_target(
    item: Mapping[str, JSONValue],
    state: AliasState,
    read_inner: InnerReader,
) -> tuple[str | None, int | None, str | None]:
    _, alias = read_inner(item)
    target_id = resolved_alias_path_type_id(alias.get("type"))
    target = state.items.get(target_id or "")
    if target is None:
        return None, None, None
    kind, body = read_inner(target)
    if kind != "struct" or not has_two_param_const_shape(body):
        return None, None, None
    return alias_const_binding(alias)


def has_two_param_const_shape(body: Mapping[str, JSONValue]) -> bool:
    generics = body.get("generics")
    params = generics.get("params") if isinstance(generics, dict) else None
    return (
        isinstance(params, list)
        and len(params) == 2
        and parameter_kind(params[0]) == "type"
        and parameter_kind(params[1]) == "const"
    )


def alias_const_binding(
    alias: Mapping[str, JSONValue],
) -> tuple[str | None, int | None, str | None]:
    target = alias.get("type")
    resolved = target.get("resolved_path") if isinstance(target, dict) else None
    target_id = resolved_alias_path_type_id(target)
    args = path_type_arguments(resolved)
    alias_name = only_type_generic_name(alias.get("generics"))
    if target_id is None or args is None or alias_name is None or len(args) != 2:
        return None, None, "unsupported-type-arguments"
    if generic_argument_name(args[0]) != alias_name:
        return None, None, "unsupported-type-arguments"
    const_value = literal_integer_argument(args[1])
    if const_value is None:
        return None, None, "unsupported-const-expression"
    return target_id, const_value, None


def only_type_generic_name(generics: JSONValue | None) -> str | None:
    if not isinstance(generics, dict) or generics.get("where_predicates") != []:
        return None
    params = generics.get("params")
    if not isinstance(params, list) or len(params) != 1:
        return None
    param = params[0]
    name = param.get("name") if isinstance(param, dict) else None
    return name if parameter_kind(param) == "type" and isinstance(name, str) else None


def parameter_kind(parameter: JSONValue) -> str | None:
    kind = parameter.get("kind") if isinstance(parameter, dict) else None
    return next(iter(kind)) if isinstance(kind, dict) and len(kind) == 1 else None


def path_type_arguments(resolved: JSONValue | None) -> list[JSONValue] | None:
    args = resolved.get("args") if isinstance(resolved, dict) else None
    angle = args.get("angle_bracketed") if isinstance(args, dict) else None
    values = angle.get("args") if isinstance(angle, dict) else None
    if not isinstance(values, list) or not isinstance(angle, dict):
        return None
    return values if angle.get("constraints") == [] else None


def generic_argument_name(argument: JSONValue) -> str | None:
    value = argument.get("type") if isinstance(argument, dict) else None
    name = value.get("generic") if isinstance(value, dict) else None
    return name if isinstance(name, str) else None


def literal_integer_argument(argument: JSONValue) -> int | None:
    value = argument.get("const") if isinstance(argument, dict) else None
    expression = value.get("expr") if isinstance(value, dict) else None
    if not isinstance(value, dict) or value.get("is_literal") is not True:
        return None
    if not isinstance(expression, str) or not expression.isascii() or not expression.isdecimal():
        return None
    return int(expression)


def alias_target_impl_members(
    target_id: str,
    const_value: int,
    alias_type: str,
    path: str,
    state: AliasState,
    read_inner: InnerReader,
    render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    owner = state.items.get(target_id)
    if owner is None:
        return []
    _, body = read_inner(owner)
    return [
        member
        for impl_id in array_value(body.get("impls", []), "impls")
        for member in matching_alias_impl(
            impl_id, target_id, const_value, alias_type, path, state, read_inner, render_impl
        )
    ]


def matching_alias_impl(
    impl_id: JSONValue,
    target_id: str,
    const_value: int,
    alias_type: str,
    path: str,
    state: AliasState,
    read_inner: InnerReader,
    render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    impl = eligible_impl_body(impl_id, state.items, read_inner)
    if impl is None or impl.get("trait") is not None:
        return []
    return alias_impl_members(impl, target_id, const_value, alias_type, path, state, render_impl)


def alias_impl_members(
    impl: Mapping[str, JSONValue], target_id: str, const_value: int,
    alias_type: str, path: str, state: AliasState, render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    specialization = specialized_alias_context(
        impl, target_id, const_value, alias_type, path, state
    )
    if specialization is None:
        return []
    if not impl_has_empty_where_clause(impl):
        state.unresolved.append(f"const-specialized-alias:{path}:unsupported-impl-where-predicates")
        return []
    return render_specialized_alias_members(impl, path, state, render_impl, specialization)


def render_specialized_alias_members(
    impl: Mapping[str, JSONValue], path: str, state: AliasState,
    render_impl: ImplRenderer, specialization: dict[str, JSONValue],
) -> list[dict[str, JSONValue]]:
    members = render_impl(impl, path, state.items, state.paths, state.source_root, state.unresolved)
    return preserve_alias_specialization(members, specialization)


def alias_type_parameter(item: Mapping[str, JSONValue], read_inner: InnerReader) -> str | None:
    _, alias = read_inner(item)
    return only_type_generic_name(alias.get("generics"))


def specialized_alias_context(
    impl: Mapping[str, JSONValue], target_id: str, const_value: int,
    alias_type: str, alias_path: str, state: AliasState,
) -> dict[str, JSONValue] | None:
    bindings = specialized_alias_bindings(impl, target_id, const_value, alias_type)
    target_path = stable_item_path(state, target_id)
    if bindings is None or target_path is None:
        return None
    type_parameter, const_parameter = bindings
    return alias_specialization_record(
        alias_path, target_id, target_path, alias_type, type_parameter,
        const_value, const_parameter,
    )


def specialized_alias_bindings(
    impl: Mapping[str, JSONValue], target_id: str, const_value: int, alias_type: str,
) -> tuple[str, str | None] | None:
    owner_id = resolved_alias_path_type_id(impl.get("for"))
    if owner_id != target_id:
        return None
    owner = impl.get("for")
    resolved = owner.get("resolved_path") if isinstance(owner, dict) else None
    arguments = path_type_arguments(resolved)
    generics = impl.get("generics")
    params = generics.get("params") if isinstance(generics, dict) else None
    if arguments is None or len(arguments) != 2 or not isinstance(params, list):
        return None
    type_parameter = matching_type_parameter(arguments[0], params, alias_type)
    const_parameter = matching_const_parameter(arguments[1], params, const_value)
    if type_parameter is None or const_parameter is False:
        return None
    return type_parameter, const_parameter


def alias_specialization_record(
    alias_path: str, target_id: str, target_path: str, alias_type: str,
    type_parameter: str, const_value: int, const_parameter: str | None,
) -> dict[str, JSONValue]:
    return {
        "alias_path": alias_path,
        "target_item_id": target_id,
        "target_path": target_path,
        "alias_type_parameter": alias_type,
        "impl_type_parameter": type_parameter,
        "const_value": const_value,
        "impl_const_parameter": const_parameter,
    }


def impl_has_empty_where_clause(impl: Mapping[str, JSONValue]) -> bool:
    generics = impl.get("generics")
    return isinstance(generics, dict) and generics.get("where_predicates") == []


def stable_item_path(state: AliasState, item_id: str) -> str | None:
    path = stable_path(state.paths, item_id, "")
    return path or None


def matching_type_parameter(
    argument: JSONValue, params: list[JSONValue], alias_type: str
) -> str | None:
    name = generic_argument_name(argument)
    if name is None or not any(is_named_type_parameter(param, name) for param in params):
        return None
    return name if alias_type else None


def is_named_type_parameter(parameter: JSONValue, name: str) -> bool:
    return isinstance(parameter, dict) and parameter.get("name") == name and parameter_kind(parameter) == "type"


def matching_const_parameter(
    argument: JSONValue, params: list[JSONValue], const_value: int
) -> str | None | bool:
    literal = literal_integer_argument(argument)
    if literal is not None:
        return None if literal == const_value else False
    name = const_generic_argument_name(argument)
    return name if name is not None and has_named_const_parameter(params, name) else False


def const_generic_argument_name(argument: JSONValue) -> str | None:
    value = argument.get("const") if isinstance(argument, dict) else None
    expression = value.get("expr") if isinstance(value, dict) else None
    return expression if isinstance(expression, str) and value.get("is_literal") is False else None


def has_named_const_parameter(params: list[JSONValue], name: str) -> bool:
    return any(
        isinstance(param, dict) and param.get("name") == name and parameter_kind(param) == "const"
        for param in params
    )


def preserve_alias_specialization(
    members: list[dict[str, JSONValue]], context: dict[str, JSONValue]
) -> list[dict[str, JSONValue]]:
    for member in members:
        member["alias_specialization"] = context.copy()
    return members


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


def resolved_alias_path_type_id(value: JSONValue | None) -> str | None:
    if not isinstance(value, dict) or len(value) != 1:
        return None
    resolved = value.get("resolved_path")
    if not isinstance(resolved, dict):
        return None
    identifier = resolved.get("id")
    if isinstance(identifier, bool) or not isinstance(identifier, (int, str)):
        return None
    return str(identifier)
