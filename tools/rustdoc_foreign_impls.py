"""Attach reachable foreign-Self impl members to their canonical owner crate."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from pathlib import Path

from api_inventory_json import JSONValue, array_value, declaration_digest, string_value
from api_inventory_reexports import source_hashes
from rustdoc_declarations import record_entry, stable_path, stable_signature


def foreign_impl_members(
    impl: Mapping[str, JSONValue],
    owner_path: str,
    impl_id: str,
    items: Mapping[str, dict[str, JSONValue]],
    paths: Mapping[str, JSONValue],
    source_root: Path,
    unresolved: list[str],
) -> list[dict[str, JSONValue]]:
    trait = impl.get("trait")
    trait_path = display_trait(trait, paths)
    if not owner_path or trait_path is None:
        error = f"impl:{impl_id}:unsupported-foreign-identity"
        unresolved.append(error)
        return [unsupported_foreign_candidate(impl_id, owner_path, error)]
    return render_foreign_members(
        impl, owner_path, trait_path, impl_id, items, paths, source_root, unresolved
    )


def unsupported_foreign_candidate(
    impl_id: str, owner_path: str, error: str
) -> dict[str, JSONValue]:
    return {
        "foreign_impl_id": impl_id,
        "foreign_impl_owner": owner_path,
        "foreign_impl_error": error,
    }


def display_trait(
    trait: JSONValue | None, paths: Mapping[str, JSONValue]
) -> str | None:
    if not isinstance(trait, dict):
        return None
    trait_id = trait.get("id")
    trait_name = trait.get("path")
    if not isinstance(trait_id, (int, str)) or not isinstance(trait_name, str):
        return None
    if str(trait_id) not in paths:
        return None
    path = stable_path(paths, str(trait_id), trait_name)
    arguments = trait_arguments(trait.get("args"), paths)
    return None if arguments is None else path + arguments


def trait_arguments(value: JSONValue | None, paths: Mapping[str, JSONValue]) -> str | None:
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
    if any(argument is None for argument in rendered):
        return None
    constraints = angle.get("constraints")
    if constraints != []:
        return None
    return "<" + ", ".join(argument for argument in rendered if argument) + ">"


def display_generic_argument(
    value: JSONValue, paths: Mapping[str, JSONValue]
) -> str | None:
    if not isinstance(value, dict) or len(value) != 1:
        return None
    kind, body = next(iter(value.items()))
    return display_type(body, paths) if kind == "type" else None


def display_type(value: JSONValue, paths: Mapping[str, JSONValue]) -> str | None:
    if not isinstance(value, dict) or len(value) != 1:
        return None
    kind, body = next(iter(value.items()))
    if kind == "resolved_path":
        return display_resolved_path(body, paths)
    if kind == "borrowed_ref":
        return display_borrowed_ref(body, paths)
    if kind in {"generic", "primitive"} and isinstance(body, str):
        return body
    return None


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


def render_foreign_members(
    impl: Mapping[str, JSONValue],
    owner_path: str,
    trait_path: str,
    impl_id: str,
    items: Mapping[str, dict[str, JSONValue]],
    paths: Mapping[str, JSONValue],
    source_root: Path,
    unresolved: list[str],
) -> list[dict[str, JSONValue]]:
    context = stable_signature(
        {"for": impl.get("for"), "generics": impl.get("generics"), "trait": impl.get("trait")},
        paths,
        unresolved,
    )
    return render_foreign_member_ids(
        impl, owner_path, trait_path, impl_id, context, items, source_root, paths,
        unresolved,
    )


def render_foreign_member_ids(
    impl: Mapping[str, JSONValue], owner_path: str, trait_path: str, impl_id: str,
    context: JSONValue, items: Mapping[str, dict[str, JSONValue]],
    source_root: Path, paths: Mapping[str, JSONValue], unresolved: list[str],
) -> list[dict[str, JSONValue]]:
    return [
        foreign_member(member_id, owner_path, trait_path, impl_id, context, items,
                       source_root, paths, unresolved)
        for member_id in array_value(impl.get("items"), "impl.items")
        if str(member_id) in items
    ]


def foreign_member(
    member_id: JSONValue,
    owner_path: str,
    trait_path: str,
    impl_id: str,
    context: JSONValue,
    items: Mapping[str, dict[str, JSONValue]],
    source_root: Path,
    paths: Mapping[str, JSONValue],
    unresolved: list[str],
) -> dict[str, JSONValue]:
    member = items[str(member_id)]
    name = string_value(member.get("name"), "foreign impl member name")
    path = f"<{owner_path} as {trait_path}>::{name}"
    entry = record_entry(path, str(member_id), member, source_root, paths, unresolved, context)
    entry["foreign_impl_id"] = impl_id
    entry["foreign_impl_owner"] = owner_path
    entry["foreign_impl_api_owner"] = owner_path.split("::", 1)[0]
    entry["declaration_sha256"] = declaration_digest(entry)
    return entry


def resolve_foreign_impl_candidates(crates: list[dict[str, JSONValue]]) -> None:
    catalog = public_type_catalog(crates)
    public_paths = public_identity_paths(crates)
    candidates = foreign_candidate_rows(crates)
    for origin, candidate in candidates:
        attach_or_report(origin, candidate, catalog, public_paths, crates)
    for crate in crates:
        crate["source_files"] = source_hashes(source_declaration_rows(crate))


def source_declaration_rows(
    crate: Mapping[str, JSONValue],
) -> list[Mapping[str, JSONValue]]:
    values = array_value(crate.get("declarations"), "declarations")
    if any(not isinstance(value, dict) for value in values):
        raise ValueError("normalized declaration list must contain only objects")
    return [value for value in values if isinstance(value, dict)]


def public_type_catalog(
    crates: Sequence[Mapping[str, JSONValue]],
) -> dict[str, dict[str, JSONValue]]:
    catalog: dict[str, dict[str, JSONValue]] = {}
    for crate in crates:
        for raw in array_value(crate.get("declarations"), "declarations"):
            if not isinstance(raw, dict):
                continue
            if raw.get("kind") not in {"struct", "enum", "union", "type_alias"}:
                continue
            path = raw.get("path")
            if isinstance(path, str):
                catalog[path] = raw
    return catalog


def public_identity_paths(
    crates: Sequence[Mapping[str, JSONValue]],
) -> dict[str, set[str]]:
    accepted = {"struct", "enum", "union", "type_alias", "trait"}
    paths: dict[str, set[str]] = {}
    for crate in crates:
        name = string_value(crate.get("crate"), "crate name")
        paths[name] = {
            path
            for raw in array_value(crate.get("declarations"), "declarations")
            if isinstance(raw, dict) and raw.get("kind") in accepted
            for path in [raw.get("path")]
            if isinstance(path, str)
        }
    return paths


def foreign_candidate_rows(
    crates: Sequence[dict[str, JSONValue]],
) -> list[tuple[dict[str, JSONValue], dict[str, JSONValue]]]:
    rows = []
    for crate in crates:
        values = crate.pop("foreign_impl_candidates", [])
        for value in array_value(values, "foreign impl candidates"):
            if isinstance(value, dict):
                rows.append((crate, value))
    return rows


def attach_or_report(
    origin: dict[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    catalog: Mapping[str, dict[str, JSONValue]],
    public_paths: Mapping[str, set[str]],
    crates: list[dict[str, JSONValue]],
) -> None:
    error = candidate.get("foreign_impl_error")
    if isinstance(error, str):
        report_gap(origin, error)
        return
    target = reachable_owner_crate(candidate, catalog, crates)
    if target is None:
        report_unreachable_owner(origin, candidate)
        return
    context_error = foreign_context_error(origin, candidate, public_paths)
    if context_error is not None:
        report_gap(origin, context_error)
        return
    append_foreign_member(target, candidate, origin)


def reachable_owner_crate(
    candidate: Mapping[str, JSONValue],
    catalog: Mapping[str, dict[str, JSONValue]],
    crates: Sequence[dict[str, JSONValue]],
) -> dict[str, JSONValue] | None:
    owner = candidate.get("foreign_impl_owner")
    if not isinstance(owner, str) or owner not in catalog:
        return None
    crate_name = owner.split("::", 1)[0].replace("-", "_")
    return next((crate for crate in crates if crate.get("crate") == crate_name), None)


def foreign_context_error(
    origin: Mapping[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    public_paths: Mapping[str, set[str]],
) -> str | None:
    context = candidate.get("impl_context")
    paths, unresolved = context_references(context)
    if unresolved:
        return f"foreign-impl-unresolved-context:{candidate.get('path')}"
    crate = string_value(origin.get("crate"), "origin crate")
    local = [path for path in paths if path.startswith(f"{crate}::")]
    if not local:
        return f"foreign-impl-no-local-public-anchor:{candidate.get('path')}"
    allowed = public_paths.get(crate, set())
    missing = sorted({path for path in local if path not in allowed})
    return None if not missing else f"foreign-impl-unreachable-local-type:{missing[0]}"


def context_references(value: JSONValue | None) -> tuple[list[str], bool]:
    if isinstance(value, list):
        children = [context_references(item) for item in value]
        return [path for paths, _ in children for path in paths], any(
            unresolved for _, unresolved in children
        )
    if not isinstance(value, dict):
        return [], False
    path = value.get("resolved_item_path")
    paths = [path] if isinstance(path, str) else []
    unresolved = "unresolved_item_id" in value or "unresolved_target" in value
    children = [context_references(item) for item in value.values()]
    paths.extend(path for nested, _ in children for path in nested)
    return paths, unresolved or any(gap for _, gap in children)


def report_unreachable_owner(
    origin: dict[str, JSONValue], candidate: Mapping[str, JSONValue]
) -> None:
    owner = candidate.get("foreign_impl_owner")
    path = candidate.get("path")
    report_gap(origin, f"foreign-impl-unreachable-owner:{owner}:{path}")


def report_gap(origin: dict[str, JSONValue], gap: str) -> None:
    gaps = array_value(origin.get("unresolved_links"), "unresolved links")
    gaps.append(gap)
    origin["unresolved_links"] = sorted(set(str(value) for value in gaps))


def append_foreign_member(
    target: dict[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    origin: Mapping[str, JSONValue],
) -> None:
    entry = dict(candidate)
    entry["foreign_impl_origin_crate"] = origin.get("crate")
    entry["declaration_sha256"] = declaration_digest(entry)
    declarations = array_value(target.get("declarations"), "declarations")
    declarations.append(entry)
    target["declarations"] = declarations
    clear_resolved_owner_gap(origin, candidate)


def clear_resolved_owner_gap(
    origin: dict[str, JSONValue], candidate: Mapping[str, JSONValue]
) -> None:
    owner_id = candidate.get("foreign_impl_id")
    gaps = array_value(origin.get("unresolved_links"), "unresolved links")
    marker = f"impl:{owner_id}:unreachable-self-owner:"
    origin["unresolved_links"] = [
        gap for gap in gaps if not (isinstance(gap, str) and gap.startswith(marker))
    ]
