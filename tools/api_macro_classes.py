"""Resolve a C++ class to the one Rust type that ports it, with the evidence used.

A class resolves only when every evidence source that speaks agrees on one canonical Rust
type; disagreement is reported, never guessed. Sources, in order of strength:
ledger record row, `RuntimeTyped::TYPE_NAME` literal, `impl_transient_interface!` call,
and the common owner type of the class's resolved member rows.
"""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass

from api_inventory.inputs import JSONObject
from api_macro_rust import FACADE_CRATE, RustInventoryIndex, crate_of, preferred_path
from api_macro_rust_source import TransientRegistration, pick_registration, type_leaf

CLAIMING_STATUSES = {"implemented", "partial"}


@dataclass(frozen=True)
class RustClass:
    """The Rust type of a C++ class; `interface` marks a trait reached as `dyn Trait`."""

    path: str
    interface: bool
    evidence: str
    registration: TransientRegistration | None


@dataclass(frozen=True)
class LedgerClaim:
    """An existing ledger row joined to the C++ declaration it uniquely claims."""

    row: JSONObject
    qualified_name: str
    cpp_kind: str


def toolkit_crate(toolkit: str) -> str:
    """`toolkit_crate('TKG2d') == 'rustycad_tkg2d'`."""
    return f"rustycad_{toolkit.lower()}"


def member_owner_path(rust_path: str) -> str | None:
    """`a::T::f` -> `a::T`; `a::T::<core::clone::Clone>::clone` -> `a::T`."""
    if rust_path.startswith("<") or "::" not in rust_path:
        return None
    if "::<" in rust_path:
        return rust_path.split("::<", 1)[0]
    return rust_path.rsplit("::", 1)[0]


class ClassResolver:
    """`ClassResolver(index, registrations, claims).resolve('Geom2d_Line', 'TKG2d')`."""

    def __init__(
        self,
        index: RustInventoryIndex,
        registrations: list[TransientRegistration],
        claims: list[LedgerClaim],
    ) -> None:
        self.index = index
        self.registrations: dict[str, list[TransientRegistration]] = defaultdict(list)
        for item in registrations:
            self.registrations[item.cxx_name].append(item)
        self.records, self.member_owners = record_and_member_paths(claims, index)
        self.type_leaves = type_leaf_paths(index)
        self.conflicts: dict[str, list[str]] = {}

    def resolve(self, cxx_class: str, toolkit: str) -> RustClass | None:
        candidates = self.candidates(cxx_class, toolkit)
        canonical = {self.canonical(path) for _, path in candidates}
        if len(canonical) > 1:
            self.conflicts[cxx_class] = sorted(f"{name}:{path}" for name, path in candidates)
        if len(canonical) != 1:
            return None
        evidence, path = candidates[0]
        interface = self.index.kinds(path) == ["trait"]
        registration = pick_registration(self.registrations.get(cxx_class, []), interface)
        return RustClass(path, interface, evidence, registration)

    def descriptor_class(self, cxx_class: str, toolkit: str) -> RustClass | None:
        """The Rust type that registers descriptor `cxx_class`, for RTTI members only.

        A C++ class may be split in Rust into a trait plus a base struct that carries the
        descriptor; the descriptor owner is then the right RTTI target even though the
        class as a whole has no single Rust type.
        """
        registrations = self.registrations.get(cxx_class, [])
        concrete = self.type_name_path(cxx_class, toolkit_crate(toolkit))
        if concrete is not None:
            registration = pick_registration(registrations, False)
            return RustClass(concrete, False, "RuntimeTyped::TYPE_NAME", registration)
        interface = self.interface_path(cxx_class)
        if interface is None:
            return None
        registration = pick_registration(registrations, True)
        return RustClass(interface, True, "impl_transient_interface!", registration)

    def candidates(self, cxx_class: str, toolkit: str) -> list[tuple[str, str]]:
        crate = toolkit_crate(toolkit)
        found = [
            ("ledger record row", preferred_path(self.records.get(cxx_class, []), crate)),
            ("RuntimeTyped::TYPE_NAME", self.type_name_path(cxx_class, crate)),
            ("impl_transient_interface!", self.interface_path(cxx_class)),
            ("member row owner", self.member_owner(cxx_class, crate)),
        ]
        return [(name, path) for name, path in found if path is not None]

    def type_name_path(self, cxx_class: str, crate: str) -> str | None:
        owners = self.index.type_name_owners.get(cxx_class, [])
        path = preferred_path(owners, crate)
        return path if path is not None and self.index.is_type(path) else None

    def interface_path(self, cxx_class: str) -> str | None:
        registration = pick_registration(self.registrations.get(cxx_class, []), True)
        if registration is None or not registration.interface:
            return None
        crate = "rustycad_" + registration.file.split("/")[1]
        paths = self.type_leaves.get(type_leaf(registration.rust_type), [])
        traits = [path for path in paths if self.index.kinds(path) == ["trait"]]
        return preferred_path(traits, crate)

    def member_owner(self, cxx_class: str, crate: str) -> str | None:
        owners = self.member_owners.get(cxx_class, set())
        canonical = {self.canonical(path) for path in owners}
        return preferred_path(sorted(owners), crate) if len(canonical) == 1 else None

    def canonical(self, path: str) -> str:
        items = self.index.by_path.get(path, [])
        value = items[0].get("canonical_type_path") if len(items) == 1 else None
        return value if isinstance(value, str) else path


def record_and_member_paths(
    claims: list[LedgerClaim], index: RustInventoryIndex
) -> tuple[dict[str, list[str]], dict[str, set[str]]]:
    """Rust type paths claimed for each class by record rows and by member rows."""
    records: dict[str, list[str]] = defaultdict(list)
    members: dict[str, set[str]] = defaultdict(set)
    for claim in claims:
        if claim.row.get("status") not in CLAIMING_STATUSES:
            continue
        for path in row_paths(claim.row):
            add_claim_path(claim, path, index, records, members)
    return dict(records), dict(members)


def add_claim_path(
    claim: LedgerClaim,
    path: str,
    index: RustInventoryIndex,
    records: dict[str, list[str]],
    members: dict[str, set[str]],
) -> None:
    if claim.cpp_kind in {"CXXRecord", "ClassTemplate"}:
        if index.is_type(path):
            records[claim.qualified_name].append(path)
        return
    owner = member_owner_path(path)
    cxx_owner = claim.qualified_name.rsplit("::", 1)[0]
    if owner and "::" in claim.qualified_name and index.is_type(owner):
        members[cxx_owner].add(owner)


def row_paths(row: JSONObject) -> list[str]:
    """Declared Rust targets of a row, excluding the shared kernel facade crate."""
    targets = row.get("rust_items")
    paths = targets if isinstance(targets, list) else [row.get("rust_item")]
    return [
        path
        for path in paths
        if isinstance(path, str) and path and crate_of(path) != FACADE_CRATE
    ]


def type_leaf_paths(index: RustInventoryIndex) -> dict[str, list[str]]:
    """Public type paths by last segment, for resolving a macro's type spelling."""
    leaves: dict[str, list[str]] = defaultdict(list)
    for path in sorted(index.by_path):
        if crate_of(path) != FACADE_CRATE and index.is_type(path):
            leaves[path.rsplit("::", 1)[-1]].append(path)
    return dict(leaves)
