"""Build ledger rows for RTTI, exception and class-record declarations.

RTTI rows are claimed only where Rust registers the same descriptor name; the parent the
Rust registration names must equal the C++ `base_type`, otherwise the row is `partial`
and says which parent differs. Exception rows require a Rust factory whose whole body is
`Failure::with_type("<class>", message, "")`.
"""

from __future__ import annotations

from dataclasses import dataclass

from api_macro_classes import toolkit_crate
from api_macro_cpp import declaration_owner
from api_macro_rows import BuildResult, SpecialMember, class_path, ledger_row
from api_macro_rust import RUNTIME_TYPED, TRANSIENT, RustInventoryIndex, literal_value
from api_macro_rust import preferred_path, trait_item
from api_macro_rust_source import TransientRegistration, type_leaf

FAILURE_TYPE = "rustycad_tkernel::standard::Failure::exception_type"
FAILURE_WITH_TYPE = "rustycad_tkernel::standard::Failure::with_type"
RTTI_ITEMS = {
    "rtti_name": (RUNTIME_TYPED, "TYPE_NAME", "get_type_name()"),
    "rtti_descriptor": (RUNTIME_TYPED, "static_type", "get_type_descriptor()"),
    "rtti_dynamic": (TRANSIENT, "dynamic_type", "DynamicType()"),
}


@dataclass(frozen=True)
class MacroEvidence:
    """Source facts the RTTI and exception builders check against."""

    index: RustInventoryIndex
    registrations: dict[str, list[TransientRegistration]]
    descriptor_names: dict[str, set[str]]
    cpp_bases: dict[str, str]
    factories: dict[str, list[str]]
    exception_bases: dict[str, str]
    rust_exception_pairs: set[tuple[str, str]]


def rtti_target(member: SpecialMember, evidence: MacroEvidence) -> tuple[str | None, str]:
    """The Rust item for one RTTI member, or None with the reason it is unproven."""
    rust_class, cxx = member.rust_class, declaration_owner(member.declaration)
    if rust_class is None:
        return None, "rtti_without_rust_class"
    if rust_class.interface:
        ok = any(item.interface for item in evidence.registrations.get(cxx, []))
        return (rust_class.path, "") if ok else (None, "rtti_interface_unregistered")
    trait, item, _ = RTTI_ITEMS[member.kind]
    name = trait_item(rust_class.path, RUNTIME_TYPED, "TYPE_NAME")
    items = evidence.index.by_path.get(name, [])
    if len(items) != 1 or literal_value(items[0].get("signature")) != cxx:
        return None, "rtti_type_name_mismatch"
    target: str | None = trait_item(rust_class.path, trait, item)
    if member.kind == "rtti_dynamic":
        target = evidence.index.dynamic_type_item(rust_class.path)
    elif not evidence.index.unique(str(target)):
        target = None
    return (target, "") if target else (None, "rtti_item_missing")


def rust_parent_name(cxx: str, evidence: MacroEvidence) -> str | None:
    """Descriptor name of the parent that every Rust registration of `cxx` names."""
    parents: set[str | None] = set()
    for registration in evidence.registrations.get(cxx, []):
        names = evidence.descriptor_names.get(type_leaf(registration.parent_type), set())
        parents.add(next(iter(names)) if len(names) == 1 else None)
    return parents.pop() if len(parents) == 1 else None


def rtti_row(member: SpecialMember, evidence: MacroEvidence) -> BuildResult:
    """Claim a DEFINE_STANDARD_RTTI* member through the Rust descriptor registration."""
    target, reason = rtti_target(member, evidence)
    if target is None:
        return None, reason
    cxx = declaration_owner(member.declaration)
    base, parent = evidence.cpp_bases.get(cxx), rust_parent_name(cxx, evidence)
    registration = member.rust_class.registration if member.rust_class else None
    notes = rtti_notes(member, registration, base)
    if base is not None and parent == base:
        return ledger_row(member, "implemented", target, notes), None
    notes += f" Not proven: the Rust registration names parent {parent or 'unknown'}."
    return ledger_row(member, "partial", target, notes), None


def rtti_notes(
    member: SpecialMember, registration: TransientRegistration | None, base: str | None
) -> str:
    cxx = declaration_owner(member.declaration)
    _, _, spelling = RTTI_ITEMS[member.kind]
    where = (
        f"{registration.macro}!"
        f"({registration.rust_type}, \"{cxx}\", {registration.parent_type}) at "
        f"{registration.file}:{registration.line}"
        if registration
        else f"RuntimeTyped::TYPE_NAME == \"{cxx}\""
    )
    shape = (
        "Abstract class: the trait is the Rust target because rustdoc does not index "
        "impls on `dyn` types; the descriptor is `<dyn Trait as RuntimeTyped>`. "
        if member.rust_class and member.rust_class.interface
        else ""
    )
    return (
        f"DEFINE_STANDARD_RTTI {spelling}: {where} registers descriptor \"{cxx}\"; C++ "
        f"base_type is {base or 'unknown'}. {shape}get_type_name is TYPE_NAME, "
        "get_type_descriptor is RuntimeTyped::static_type and DynamicType is the "
        "virtual dynamic_type of the Rust target (most-derived descriptor)."
    ).replace("  ", " ")


def exception_row(member: SpecialMember, evidence: MacroEvidence) -> BuildResult:
    """Claim a DEFINE_STANDARD_EXCEPTION member through its Rust Failure factory."""
    cxx = declaration_owner(member.declaration)
    factory = preferred_path(evidence.factories.get(cxx, []), toolkit_crate(member.toolkit))
    if factory is None:
        return None, "exception_without_rust_factory"
    if member.kind == "exception_type":
        notes = (
            f"DEFINE_STANDARD_EXCEPTION ExceptionType() returns \"{cxx}\"; {factory} builds "
            f"Failure::with_type(\"{cxx}\", ..) and Failure::exception_type returns that name."
        )
        return ledger_row(member, "implemented", [FAILURE_TYPE, factory], notes), None
    return exception_constructor_row(member, evidence, cxx, factory)


def exception_constructor_row(
    member: SpecialMember, evidence: MacroEvidence, cxx: str, factory: str
) -> BuildResult:
    two = len(member.declaration.get("parameter_types") or []) == 2
    rust: str | list[str] = [FAILURE_WITH_TYPE, factory] if two else factory
    call = f"Failure::with_type(\"{cxx}\", message, stack)" if two else f"{factory}(message)"
    notes = f"DEFINE_STANDARD_EXCEPTION constructor: Rust {call} builds the same exception."
    base = evidence.exception_bases.get(cxx)
    # Failure::is_kind treats every unknown class as a direct Standard_Failure child.
    if base == "Standard_Failure" or (cxx, str(base)) in evidence.rust_exception_pairs:
        return ledger_row(member, "implemented", rust, notes), None
    notes += (
        f" Partial: C++ derives {cxx} from {base or 'an unknown base'}, but the Rust "
        "exception hierarchy tables lack that pair, so Failure::is_kind answers only "
        "Standard_Failure for a catch-by-base check."
    )
    return ledger_row(member, "partial", rust, notes), None


def record_row(member: SpecialMember) -> BuildResult:
    """Claim a class record whose members already have rows, for its one Rust type."""
    rust_class = member.rust_class
    if rust_class is None:
        return None, "record_without_rust_class"
    shape = "a trait reached as Handle<dyn Trait>" if rust_class.interface else "a type"
    notes = (
        f"Class record: ported as {shape}, {rust_class.path} (evidence: "
        f"{rust_class.evidence}); each member is mapped by its own row."
    )
    return ledger_row(member, "implemented", class_path(member), notes), None
