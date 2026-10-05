"""Preserve borrowed Self when rendering locally owned trait implementations."""

from __future__ import annotations

from collections.abc import Mapping

from api_inventory_json import JSONValue, declaration_digest


def reference_self_target(impl: Mapping[str, JSONValue]) -> JSONValue | None:
    """Read a fully shaped reference receiver; e.g. `reference_self_target(impl)`.

    Unknown reference metadata stays unresolved rather than becoming an owned impl.
    """
    receiver = impl.get("for")
    reference = receiver.get("borrowed_ref") if isinstance(receiver, dict) else None
    if not isinstance(reference, dict) or not valid_reference_shape(reference):
        return None
    return reference.get("type")


def valid_reference_shape(reference: Mapping[str, JSONValue]) -> bool:
    """Require the compiler reference fields; e.g. `valid_reference_shape(receiver)`.

    Mutability cannot be inferred from missing metadata.
    """
    if set(reference) != {"lifetime", "is_mutable", "type"}:
        return False
    lifetime = reference.get("lifetime")
    return isinstance(reference.get("is_mutable"), bool) and (
        lifetime is None
        or isinstance(lifetime, str)
        and valid_reference_lifetime(lifetime)
    )


def valid_reference_lifetime(lifetime: str) -> bool:
    """Accept compiler lifetime names; e.g. `valid_reference_lifetime("'a")`.

    Empty or malformed names must not produce callable paths.
    """
    return lifetime.startswith("'") and lifetime[1:].isidentifier()


def implementation_member_path(
    owner: str, member: str, trait: str | None, impl: Mapping[str, JSONValue]
) -> str:
    """Keep reference receivers in UFCS syntax; e.g. `implementation_member_path(owner, 'add', trait, impl)`."""
    if trait is None:
        return f"{owner}::{member}"
    if reference_self_target(impl) is None:
        return f"{owner}::<{trait}>::{member}"
    receiver = impl["for"]
    assert isinstance(receiver, dict)
    reference = receiver["borrowed_ref"]
    assert isinstance(reference, dict)
    lifetime = reference["lifetime"]
    prefix = "&" if lifetime is None else f"&{lifetime} "
    qualifier = "mut " if reference["is_mutable"] else ""
    return f"<{prefix}{qualifier}{owner} as {trait}>::{member}"


def annotate_reference_trait(
    entry: dict[str, JSONValue], trait: str | None, impl: Mapping[str, JSONValue]
) -> dict[str, JSONValue]:
    """Retain identity for public trait alias expansion; e.g. `annotate_reference_trait(row, trait, impl)`."""
    if trait is None or reference_self_target(impl) is None:
        return entry
    context = entry.get("impl_context")
    captured = context.get("trait") if isinstance(context, dict) else None
    canonical = (
        captured.get("resolved_item_path") if isinstance(captured, dict) else None
    )
    if not isinstance(canonical, str):
        return entry
    entry["reference_impl_trait_path"] = trait.partition("<")[0]
    entry["reference_impl_trait_canonical"] = canonical
    entry["declaration_sha256"] = declaration_digest(entry)
    return entry
