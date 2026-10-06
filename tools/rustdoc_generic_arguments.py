"""Render rustdoc generic arguments only when their spelling is lossless."""

from __future__ import annotations

from collections.abc import Mapping
import re

from api_inventory_json import JSONValue


def stable_path(
    path_table: Mapping[str, JSONValue], item_id: str, fallback: str
) -> str:
    """Resolve a rustdoc item path; `stable_path(paths, item_id, fallback)`."""
    raw = path_table.get(item_id)
    if not isinstance(raw, dict):
        return fallback
    parts = raw.get("path")
    if not isinstance(parts, list) or not all(isinstance(part, str) for part in parts):
        return fallback
    return "::".join(parts)


def trait_arguments(
    value: JSONValue | None, paths: Mapping[str, JSONValue]
) -> str | None:
    """Render captured angle arguments; `trait_arguments(args, path_table)`."""
    if value is None:
        return ""
    if not isinstance(value, dict):
        return None
    angle = value.get("angle_bracketed")
    if not isinstance(angle, dict):
        return None
    arguments = angle.get("args")
    if not isinstance(arguments, list):
        return None
    rendered = [display_generic_argument(argument, paths) for argument in arguments]
    if any(argument is None for argument in rendered) or angle.get("constraints") != []:
        return None
    return "<" + ", ".join(argument for argument in rendered if argument) + ">"


def display_generic_argument(
    value: JSONValue, paths: Mapping[str, JSONValue]
) -> str | None:
    if not isinstance(value, dict) or len(value) != 1:
        return None
    kind, body = next(iter(value.items()))
    if kind == "const":
        return display_named_const_argument(body)
    return display_type(body, paths) if kind == "type" else None


def display_named_const_argument(value: JSONValue) -> str | None:
    """Keep a compiler-captured const name; `display_named_const_argument(body)`."""
    if not isinstance(value, dict) or set(value) != {"expr", "value", "is_literal"}:
        return None
    expression = value["expr"]
    if value["is_literal"] is not False or value["value"] is not None:
        return None
    if not isinstance(expression, str) or expression == "_":
        return None
    # Preserve the compiler's generic identity; never evaluate or erase expressions.
    if re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*", expression) is None:
        return None
    return expression


def display_type(value: JSONValue, paths: Mapping[str, JSONValue]) -> str | None:
    if not isinstance(value, dict) or len(value) != 1:
        return None
    kind, body = next(iter(value.items()))
    if kind == "resolved_path":
        return display_resolved_path(body, paths)
    if kind == "borrowed_ref":
        return display_borrowed_ref(body, paths)
    if kind == "slice":
        element = display_type(body, paths)
        return None if element is None else f"[{element}]"
    if kind == "array":
        return display_array_type(body, paths)
    if kind in {"generic", "primitive"} and isinstance(body, str):
        return body
    return None


def display_array_type(value: JSONValue, paths: Mapping[str, JSONValue]) -> str | None:
    """Preserve a captured array type; `display_array_type(array, paths)`."""
    if not isinstance(value, dict) or set(value) != {"type", "len"}:
        return None
    length = value["len"]
    if not isinstance(length, str) or length == "_":
        return None
    # Rustdoc expressions require a separate identity policy; never guess their value.
    if re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*|[0-9]+", length) is None:
        return None
    element = display_type(value["type"], paths)
    return None if element is None else f"[{element}; {length}]"


def display_resolved_path(
    value: JSONValue, paths: Mapping[str, JSONValue]
) -> str | None:
    if not isinstance(value, dict):
        return None
    identifier, fallback = value.get("id"), value.get("path")
    if not isinstance(identifier, (int, str)) or not isinstance(fallback, str):
        return None
    if str(identifier) not in paths:
        return None
    path = stable_path(paths, str(identifier), fallback)
    arguments = trait_arguments(value.get("args"), paths)
    return None if arguments is None else path + arguments


def display_borrowed_ref(
    value: JSONValue, paths: Mapping[str, JSONValue]
) -> str | None:
    if not isinstance(value, dict) or not isinstance(value.get("is_mutable"), bool):
        return None
    inner = display_type(value.get("type"), paths)
    if inner is None:
        return None
    lifetime = value.get("lifetime")
    if lifetime is not None and not isinstance(lifetime, str):
        return None
    lifetime_text = f"{lifetime} " if isinstance(lifetime, str) else ""
    mutability = "mut " if value["is_mutable"] else ""
    return f"&{lifetime_text}{mutability}{inner}"
