"""Build ledger rows for allocation operators, destructors and copy/move members.

Every builder returns `(row, None)` or `(None, reason)`; a reason names the missing
evidence so the run summary lists what still needs a person, never a silent skip.
"""

from __future__ import annotations

from dataclasses import dataclass

from api_inventory.inputs import JSONObject
from api_macro_classes import RustClass
from api_macro_cpp import allocation_macro, upstream_identity
from api_macro_rust import CLONE, RustInventoryIndex, trait_item
from api_macro_rust_source import RustSourceTree, clone_is_derived
from api_macro_source import DEFAULTED, DELETED, EMPTY_BODY, USER_BODY

GENERATOR = "tools/generate_api_macro_rows.py"
BuildResult = tuple[JSONObject | None, str | None]


@dataclass(frozen=True)
class SpecialMember:
    """One unclaimed C++ declaration with its special kind and definition form."""

    declaration: JSONObject
    kind: str
    form: str
    toolkit: str
    rust_class: RustClass | None


def ledger_row(
    member: SpecialMember, status: str, rust: str | list[str] | None, notes: str
) -> JSONObject:
    """Shape a row like hand rows; `rust` is one path, an ordered path list, or None."""
    row: JSONObject = {
        "upstream_id": upstream_identity(member.declaration),
        "toolkit": member.toolkit,
        "source": member.declaration.get("source"),
        "rust_item": rust[0] if isinstance(rust, list) else rust,
    }
    if isinstance(rust, list):
        row["rust_items"] = list(rust)
    row.update(status=status, notes=notes)
    row["cpp_declaration_id"] = member.declaration.get("id")
    row["generated_by"] = GENERATOR
    return row


def class_path(member: SpecialMember) -> str | None:
    return member.rust_class.path if member.rust_class else None


def rust_owner_phrase(member: SpecialMember) -> str:
    path = class_path(member)
    return f"Rust {path}" if path else "A Rust port of the class"


ALLOCATION_NOTES = {
    "new": "allocates one object through Standard::Allocate",
    "new[]": "allocates an object array through Standard::Allocate",
    "delete": "returns one object's memory through Standard::Free",
    "delete[]": "returns an array's memory through Standard::Free",
    "placement": "is the placement form that constructs into caller-provided storage",
    "allocator new": "allocates from an NCollection_BaseAllocator handle",
    "allocator delete": "frees through an NCollection_BaseAllocator handle",
    "inc delete": "does nothing because the incremental allocator frees in bulk",
}


def allocation_variant(declaration: JSONObject) -> str:
    """Name the operator variant from its name, parameters and generating macro."""
    name = str(declaration.get("qualified_name", "")).rsplit("::", 1)[-1]
    operator = name.removeprefix("operator ")
    parameters = declaration.get("parameter_types") or []
    macro = allocation_macro(declaration)
    if macro == "DEFINE_INC_ALLOC":
        return "inc delete"
    if macro == "DEFINE_NCOLLECTION_ALLOC":
        return f"allocator {operator}"
    if len(parameters) == 2:
        return "placement"
    return operator


def allocation_row(member: SpecialMember) -> BuildResult:
    """Exclude a macro allocation operator; Rust has no class-specific allocator hook."""
    variant = allocation_variant(member.declaration)
    macro = allocation_macro(member.declaration)
    notes = (
        f"Excluded: {macro} generates this class-specific operator, which "
        f"{ALLOCATION_NOTES[variant]}. Rust has no per-type allocation or placement "
        "operator: a value lives inline, in a Box, a Handle or a collection, and its "
        "storage comes from and returns to the owner's allocator when it is created and "
        f"dropped. {rust_owner_phrase(member)} needs no counterpart; no behavior is lost."
    )
    return ledger_row(member, "excluded", class_path(member), notes), None


def destructor_row(member: SpecialMember) -> BuildResult:
    """Exclude trivial destructors; a destructor with a body needs a person's review."""
    if member.form not in {DEFAULTED, EMPTY_BODY}:
        return None, f"destructor_{member.form}"
    spelling = "= default" if member.form == DEFAULTED else "an empty body"
    notes = (
        f"Excluded: the C++ destructor is trivial ({spelling} in the pinned source), so "
        "destruction only releases the members. Rust drop glue releases the same fields "
        f"when the owner goes out of scope. {rust_owner_phrase(member)} needs no callable "
        "counterpart."
    )
    return ledger_row(member, "excluded", class_path(member), notes), None


def deleted_member_row(member: SpecialMember, index: RustInventoryIndex) -> BuildResult:
    """Exclude a `= delete` copy/move member: it is not callable C++ API."""
    action = member.kind.replace("_", " ")
    notes = f"Excluded: C++ declares the {action} `= delete`, so no caller can use it. "
    path = class_path(member)
    if member.kind.startswith("copy"):
        notes += deleted_copy_clause(path, index)
    else:
        notes += (
            "A Rust move cannot be disabled, but it only transfers the same value and the "
            "moved-from binding becomes inaccessible, so no extra operation exists."
        )
    return ledger_row(member, "excluded", path, notes), None


def deleted_copy_clause(path: str | None, index: RustInventoryIndex) -> str:
    """State whether the mapped Rust type adds a Clone the C++ class forbids."""
    if path is None:
        return "No Rust type is mapped to this class yet."
    if index.has_trait_item(path, CLONE, "clone"):
        return f"Rust {path} implements Clone, an explicit copy that C++ lacks."
    return f"Rust {path} does not implement Clone either."


def copy_row(
    member: SpecialMember, index: RustInventoryIndex, sources: RustSourceTree
) -> BuildResult:
    """Map a defaulted copy member to the Rust Clone impl of the same type."""
    path = class_path(member)
    if member.form != DEFAULTED:
        return None, f"{member.kind}_{member.form}"
    if path is None or not index.has_trait_item(path, CLONE, "clone"):
        return None, f"{member.kind}_without_rust_clone"
    clone = trait_item(path, CLONE, "clone")
    source = index.by_path[clone][0].get("source_file")
    derived = isinstance(source, str) and clone_is_derived(sources, source, path.rsplit("::", 1)[-1])
    use = "a.clone()" if member.kind == "copy_constructor" else "*a = b.clone()"
    notes = (
        f"The C++ {member.kind.replace('_', ' ')} is `= default` (member-wise copy); Rust "
        f"`{use}` with {'derived' if derived else 'hand-written'} Clone"
    )
    if derived:
        return ledger_row(member, "implemented", clone, notes + " copies the same fields."), None
    notes += " is not proven member-wise against the C++ default; review the impl."
    return ledger_row(member, "partial", clone, notes), None


def move_row(member: SpecialMember) -> BuildResult:
    """Exclude a defaulted or user move: Rust moves are ownership transfers."""
    if member.form not in {DEFAULTED, USER_BODY, EMPTY_BODY}:
        return None, f"{member.kind}_{member.form}"
    origin = "`= default`" if member.form == DEFAULTED else "user-written"
    notes = (
        f"Excluded: the C++ {member.kind.replace('_', ' ')} is {origin}. Every Rust move "
        "transfers the value without a callable hook, and the moved-from binding is "
        "inaccessible, so the C++ moved-from source state has no Rust counterpart. "
        f"{rust_owner_phrase(member)} needs no method for it."
    )
    return ledger_row(member, "excluded", class_path(member), notes), None


def special_row(
    member: SpecialMember, index: RustInventoryIndex, sources: RustSourceTree
) -> BuildResult:
    """Dispatch allocation, destructor and copy/move members to their builders."""
    if member.kind == "allocation":
        return allocation_row(member)
    if member.kind == "destructor":
        return destructor_row(member)
    if member.form == DELETED:
        return deleted_member_row(member, index)
    if member.kind.startswith("move"):
        return move_row(member)
    return copy_row(member, index, sources)
