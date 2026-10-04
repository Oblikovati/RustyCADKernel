"""Resolve inherent methods through a narrowly supported const alias shape."""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path
from typing import Protocol

from api_inventory_json import JSONValue, array_value


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
    target_id, const_value, reason = const_specialized_alias_target(item, state, read_inner)
    if reason is not None:
        state.unresolved.append(f"const-specialized-alias:{path}:{reason}")
        return []
    if target_id is None or const_value is None:
        return []
    return alias_target_impl_members(target_id, const_value, path, state, read_inner, render_impl)


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
        for member in matching_alias_impl(impl_id, target_id, const_value, path, state, read_inner, render_impl)
    ]


def matching_alias_impl(
    impl_id: JSONValue,
    target_id: str,
    const_value: int,
    path: str,
    state: AliasState,
    read_inner: InnerReader,
    render_impl: ImplRenderer,
) -> list[dict[str, JSONValue]]:
    impl = eligible_impl_body(impl_id, state.items, read_inner)
    if impl is None or impl.get("trait") is not None:
        return []
    if not specialized_owner_matches(impl, target_id, const_value):
        return []
    return render_impl(impl, path, state.items, state.paths, state.source_root, state.unresolved)


def specialized_owner_matches(
    impl: Mapping[str, JSONValue], target_id: str, const_value: int
) -> bool:
    owner_id = resolved_alias_path_type_id(impl.get("for"))
    if owner_id != target_id:
        return False
    owner = impl.get("for")
    resolved = owner.get("resolved_path") if isinstance(owner, dict) else None
    arguments = path_type_arguments(resolved)
    generics = impl.get("generics")
    params = generics.get("params") if isinstance(generics, dict) else None
    if arguments is None or len(arguments) != 2 or not isinstance(params, list):
        return False
    return impl_type_argument_is_generic(arguments[0], params) and impl_const_argument_matches(
        arguments[1], params, const_value
    )


def impl_type_argument_is_generic(argument: JSONValue, params: list[JSONValue]) -> bool:
    name = generic_argument_name(argument)
    return name is not None and any(
        param.get("name") == name and parameter_kind(param) == "type"
        for param in params if isinstance(param, dict)
    )


def impl_const_argument_matches(
    argument: JSONValue, params: list[JSONValue], const_value: int
) -> bool:
    literal = literal_integer_argument(argument)
    if literal is not None:
        return literal == const_value
    value = argument.get("const") if isinstance(argument, dict) else None
    expression = value.get("expr") if isinstance(value, dict) else None
    return (
        isinstance(value, dict)
        and value.get("is_literal") is False
        and isinstance(expression, str)
        and any(
            param.get("name") == expression and parameter_kind(param) == "const"
            for param in params if isinstance(param, dict)
        )
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
