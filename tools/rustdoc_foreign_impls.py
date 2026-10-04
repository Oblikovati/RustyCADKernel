"""Attach reachable foreign-Self impl members to their canonical owner crate."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from pathlib import Path
import re

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
    if not owner_path or trait_path is None or self_type_is_parameterized(impl):
        error = f"impl:{impl_id}:unsupported-foreign-owner-or-trait"
        unresolved.append(error)
        return [unsupported_foreign_candidate(impl_id, owner_path, error)]
    return render_foreign_members(
        impl, owner_path, trait_path, impl_id, items, paths, source_root, unresolved
    )


def self_type_is_parameterized(impl: Mapping[str, JSONValue]) -> bool:
    self_type = impl.get("for")
    if not isinstance(self_type, dict) or len(self_type) != 1:
        return True
    kind, value = next(iter(self_type.items()))
    return kind != "resolved_path" or not isinstance(value, dict) or value.get("args") is not None


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
    aliases = public_identity_aliases(crates)
    candidates = unique_foreign_candidate_rows(crates)
    for origin, candidate in candidates:
        attach_or_report(origin, candidate, aliases, crates)
    for crate in crates:
        crate["source_files"] = source_hashes(source_declaration_rows(crate))


def source_declaration_rows(
    crate: Mapping[str, JSONValue],
) -> list[Mapping[str, JSONValue]]:
    values = array_value(crate.get("declarations"), "declarations")
    if any(not isinstance(value, dict) for value in values):
        raise ValueError("normalized declaration list must contain only objects")
    return [value for value in values if isinstance(value, dict)]


def public_identity_aliases(
    crates: Sequence[Mapping[str, JSONValue]],
) -> dict[tuple[str, str], set[str]]:
    aliases: dict[tuple[str, str], set[str]] = {}
    for crate in crates:
        crate_name = string_value(crate.get("crate"), "crate name")
        for raw in array_value(crate.get("declarations"), "declarations"):
            add_public_identity_alias(aliases, crate_name, raw)
    return aliases


def add_public_identity_alias(
    aliases: dict[tuple[str, str], set[str]], crate: str, value: JSONValue
) -> None:
    if not isinstance(value, dict):
        return
    kind = value.get("kind")
    key = "canonical_trait_path" if kind == "trait" else "canonical_type_path"
    if kind not in {"struct", "enum", "union", "type_alias", "trait"}:
        return
    canonical, public_path = value.get(key), value.get("path")
    if isinstance(canonical, str) and isinstance(public_path, str):
        aliases.setdefault((crate, canonical), set()).add(public_path)


def unique_foreign_candidate_rows(
    crates: Sequence[dict[str, JSONValue]],
) -> list[tuple[dict[str, JSONValue], dict[str, JSONValue]]]:
    rows = []
    seen: set[tuple[str, str, str]] = set()
    for crate in crates:
        values = crate.pop("foreign_impl_candidates", [])
        for value in array_value(values, "foreign impl candidates"):
            if not isinstance(value, dict):
                continue
            key = foreign_candidate_key(crate, value)
            if key in seen:
                continue
            seen.add(key)
            rows.append((crate, value))
    return rows


def foreign_candidate_key(
    crate: Mapping[str, JSONValue], candidate: Mapping[str, JSONValue]
) -> tuple[str, str, str]:
    return (
        string_value(crate.get("crate"), "origin crate"),
        str(candidate.get("foreign_impl_id", "")),
        str(candidate.get("path", candidate.get("foreign_impl_error", ""))),
    )


def attach_or_report(
    origin: dict[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    aliases: Mapping[tuple[str, str], set[str]],
    crates: list[dict[str, JSONValue]],
) -> None:
    error = candidate.get("foreign_impl_error")
    if isinstance(error, str):
        report_gap(origin, error)
        return
    owner, target = candidate_public_target(origin, candidate, aliases, crates)
    if target is None or owner is None:
        report_unreachable_owner(origin, candidate)
        return
    local_aliases, context_error = local_context_aliases(origin, candidate, aliases)
    if local_aliases is None:
        report_gap(origin, context_error)
        return
    public_entry = rewrite_foreign_candidate(candidate, owner, local_aliases)
    append_foreign_member(target, public_entry, origin)


def candidate_public_target(
    origin: dict[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    aliases: Mapping[tuple[str, str], set[str]],
    crates: Sequence[dict[str, JSONValue]],
) -> tuple[str | None, dict[str, JSONValue] | None]:
    if candidate.get("foreign_impl_self_primitive") == "bool":
        return public_bool_anchor_target(origin, candidate, aliases, crates)
    return public_owner_target(candidate, aliases, crates)


def public_bool_anchor_target(
    origin: dict[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    aliases: Mapping[tuple[str, str], set[str]],
    crates: Sequence[dict[str, JSONValue]],
) -> tuple[str | None, dict[str, JSONValue] | None]:
    crate_name = string_value(origin.get("crate"), "origin crate")
    canonical = candidate.get("foreign_impl_anchor_canonical")
    if not isinstance(canonical, str):
        return None, None
    owners = aliases.get((crate_name, canonical), set())
    target = next((crate for crate in crates if crate.get("crate") == crate_name), None)
    if len(owners) != 1 or target is not origin:
        return None, None
    public_path = next(iter(owners))
    if not bool_anchor_row_is_pinned(origin, candidate, public_path, canonical):
        return None, None
    return public_path, target


def bool_anchor_row_is_pinned(
    origin: Mapping[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    public_path: str,
    canonical: str,
) -> bool:
    matches = [
        row for row in source_declaration_rows(origin)
        if row.get("path") == public_path and row.get("canonical_type_path") == canonical
    ]
    if len(matches) != 1:
        return False
    anchor = matches[0]
    return (
        anchor.get("source_status") is None
        and anchor.get("source_file") == candidate.get("foreign_impl_anchor_source_file")
        and anchor.get("source_sha256") == candidate.get("foreign_impl_anchor_source_sha256")
    )


def public_owner_target(
    candidate: Mapping[str, JSONValue],
    aliases: Mapping[tuple[str, str], set[str]],
    crates: Sequence[dict[str, JSONValue]],
) -> tuple[str | None, dict[str, JSONValue] | None]:
    owner = candidate.get("foreign_impl_owner")
    if not isinstance(owner, str):
        return None, None
    crate_name = owner.split("::", 1)[0].replace("-", "_")
    public_owners = aliases.get((crate_name, owner), set())
    if len(public_owners) != 1:
        return None, None
    target = next((crate for crate in crates if crate.get("crate") == crate_name), None)
    return next(iter(public_owners)), target


def local_context_aliases(
    origin: Mapping[str, JSONValue],
    candidate: Mapping[str, JSONValue],
    aliases: Mapping[tuple[str, str], set[str]],
) -> tuple[dict[str, str] | None, str | None]:
    context = candidate.get("impl_context")
    paths, unresolved = context_references(context)
    if unresolved:
        return None, f"foreign-impl-unresolved-context:{candidate.get('path')}"
    crate = string_value(origin.get("crate"), "origin crate")
    local = [path for path in paths if path.startswith(f"{crate}::")]
    if not local:
        return None, f"foreign-impl-no-local-public-anchor:{candidate.get('path')}"
    return resolve_local_aliases(crate, local, aliases)


def resolve_local_aliases(
    crate: str,
    local_paths: Sequence[str],
    aliases: Mapping[tuple[str, str], set[str]],
) -> tuple[dict[str, str] | None, str | None]:
    resolved: dict[str, str] = {}
    for canonical in sorted(set(local_paths)):
        choices = aliases.get((crate, canonical), set())
        if len(choices) != 1:
            return None, f"foreign-impl-unreachable-local-type:{canonical}"
        resolved[canonical] = next(iter(choices))
    return resolved, None


def rewrite_foreign_candidate(
    candidate: Mapping[str, JSONValue],
    owner: str,
    local_aliases: Mapping[str, str],
) -> dict[str, JSONValue]:
    entry = dict(candidate)
    rewrite_foreign_owner(entry, owner, local_aliases)
    trait_path = ufcs_trait_path(entry["path"])
    canonical_trait = candidate_trait_path(entry)
    if trait_path is not None and canonical_trait is not None:
        entry["foreign_impl_trait_path"] = trait_path
        entry["foreign_impl_trait_canonical"] = canonical_trait
    return entry


def rewrite_foreign_owner(
    entry: dict[str, JSONValue], owner: str, local_aliases: Mapping[str, str]
) -> None:
    canonical_owner = string_value(entry.get("foreign_impl_owner"), "foreign owner")
    path = string_value(entry.get("path"), "foreign impl path")
    identities = dict(local_aliases)
    if entry.get("foreign_impl_self_primitive") == "bool":
        entry["foreign_impl_anchor_path"] = owner
    else:
        identities[canonical_owner] = owner
        entry["foreign_impl_owner"] = owner
        entry["foreign_impl_owner_canonical"] = canonical_owner
    entry["path"] = rewrite_path_identities(path, identities)


def candidate_trait_path(entry: Mapping[str, JSONValue]) -> str | None:
    context = entry.get("impl_context")
    trait = context.get("trait") if isinstance(context, dict) else None
    path = trait.get("resolved_item_path") if isinstance(trait, dict) else None
    return path if isinstance(path, str) else None


def ufcs_trait_path(path: str) -> str | None:
    if not path.startswith("<"):
        return None
    prefix, separator, _member = path[1:].rpartition(">::")
    _owner, trait_separator, application = prefix.partition(" as ")
    if not separator or not trait_separator:
        return None
    return application.split("<", 1)[0]


def rewrite_path_identities(path: str, aliases: Mapping[str, str]) -> str:
    for canonical in sorted(aliases, key=len, reverse=True):
        pattern = re.compile(re.escape(canonical) + r"(?=$|[^A-Za-z0-9_])")
        path = pattern.sub(lambda _match: aliases[canonical], path)
    return path


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
    entry.pop("foreign_impl_id", None)
    entry["foreign_impl_origin_crate"] = origin.get("crate")
    if entry.get("foreign_impl_self_primitive") == "bool":
        entry["foreign_impl_api_owner"] = origin.get("crate")
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
