"""Expand member-local aliases only when captured declarations prove their scope."""

from __future__ import annotations

import re

from api_inventory.inputs import JSONValue


def qualify_member_aliases(
    parameters: list[str],
    declaration: dict[str, JSONValue],
    by_name: dict[str, list[dict[str, JSONValue]]],
) -> list[str]:
    """Expand a proven local alias; e.g. `value_type&` becomes `Buffer::value_type&`."""
    aliases = captured_member_aliases(declaration, by_name)
    qualified = list(parameters)
    for alias, scoped_name in aliases.items():
        pattern = rf"(?<![\w:]){re.escape(alias)}(?![\w:])"
        qualified = [re.sub(pattern, lambda _: scoped_name, item) for item in qualified]
    return qualified


def captured_member_aliases(
    declaration: dict[str, JSONValue],
    by_name: dict[str, list[dict[str, JSONValue]]],
) -> dict[str, str]:
    """Find aliases with matching scope USRs; e.g. `{'value_type': 'Buffer::value_type'}`."""
    if declaration.get("kind") not in {"CXXMethod", "CXXConstructor"}:
        return {}
    name, parent = declaration.get("qualified_name"), declaration.get("parent_usr")
    if not isinstance(name, str) or not isinstance(parent, str) or not parent:
        return {}
    if "::" not in name or declaration.get("template_signature"):
        return {}
    owner = name.rsplit("::", 1)[0]
    return {
        alias: f"{owner}::{alias}"
        for alias in written_alias_names(declaration, owner)
        if unique_scope_alias(by_name.get(f"{owner}::{alias}", []), parent)
    }


def written_alias_names(declaration: dict[str, JSONValue], owner: str) -> set[str]:
    """Read qualified aliases from captured parameters; e.g. `{'value_type'}`."""
    parameters = declaration.get("written_parameter_types")
    if not isinstance(parameters, list):
        return set()
    pattern = rf"(?<![\w:]){re.escape(owner)}::([A-Za-z_]\w*)(?![\w:])"
    return {
        match
        for item in parameters
        if isinstance(item, str)
        for match in re.findall(pattern, item)
    }


def unique_scope_alias(candidates: list[dict[str, JSONValue]], parent: str) -> bool:
    """Require one stable alias in the exact owner; e.g. `unique_scope_alias(rows, usr)`."""
    if len(candidates) != 1:
        return False
    alias = candidates[0]
    return (
        alias.get("kind") in {"TypeAlias", "Typedef"}
        and alias.get("parent_usr") == parent
        and alias.get("stable_identity") is True
        and not alias.get("inventory_conflict")
    )
