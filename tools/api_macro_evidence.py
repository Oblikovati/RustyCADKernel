"""Collect the C++ and Rust facts that justify generated macro/special-member rows."""

from __future__ import annotations

from collections import defaultdict
import re

from api_inventory.inputs import JSONObject
from api_macro_cpp import RTTI_HEADER, declaration_owner, special_member_kind, spelling_header
from api_macro_rtti_rows import MacroEvidence
from api_macro_rust import FACADE_CRATE, RustInventoryIndex, crate_of
from api_macro_rust_source import (
    RustSourceTree,
    TransientRegistration,
    exception_factories,
    exception_parent_pairs,
    type_leaf,
)
from api_macro_rust_registrations import transient_registrations
from api_macro_source import SourceTextReader


def cpp_base_types(declarations: list[JSONObject]) -> dict[str, str]:
    """C++ parent of each RTTI class from its macro `base_type` typedef."""
    bases = {}
    for row in declarations:
        name = str(row.get("qualified_name", ""))
        if not name.endswith("::base_type") or spelling_header(row) != RTTI_HEADER:
            continue
        base = re.sub(r"^(?:class|struct) ", "", str(row.get("signature", "")).strip())
        bases[declaration_owner(row)] = base
    return bases


def exception_bases(
    declarations: list[JSONObject], reader: SourceTextReader
) -> dict[str, str]:
    """`DEFINE_STANDARD_EXCEPTION(Class, Base)` parents, read from each class header."""
    bases = {}
    for row in declarations:
        if special_member_kind(row) != "exception_type":
            continue
        cxx = declaration_owner(row)
        text = reader.text(str(row.get("source", ""))) or ""
        pattern = rf"DEFINE_STANDARD_EXCEPTION\s*\(\s*{re.escape(cxx)}\s*,\s*(\w+)\s*\)"
        match = re.search(pattern, text)
        if match:
            bases[cxx] = match.group(1)
    return bases


def factory_paths(tree: RustSourceTree, index: RustInventoryIndex) -> dict[str, list[str]]:
    """Inventory paths of every exception factory function, keyed by C++ class."""
    functions: dict[tuple[str, str], list[str]] = defaultdict(list)
    for path, items in sorted(index.by_path.items()):
        if len(items) == 1 and items[0].get("kind") == "function" and crate_of(path) != FACADE_CRATE:
            functions[(path.rsplit("::", 1)[-1], str(items[0].get("source_file")))].append(path)
    paths: dict[str, list[str]] = defaultdict(list)
    for factory in exception_factories(tree):
        paths[factory.cxx_name].extend(functions.get((factory.function, factory.file), []))
    return {name: sorted(set(found)) for name, found in paths.items() if found}


def descriptor_names(
    registrations: list[TransientRegistration], index: RustInventoryIndex
) -> dict[str, set[str]]:
    """Descriptor names reachable from a Rust type spelling's last segment."""
    names: dict[str, set[str]] = defaultdict(set)
    for registration in registrations:
        names[type_leaf(registration.rust_type)].add(registration.cxx_name)
    for cxx, owners in index.type_name_owners.items():
        for owner in owners:
            names[owner.rsplit("::", 1)[-1]].add(cxx)
    return dict(names)


def macro_evidence(
    declarations: list[JSONObject],
    index: RustInventoryIndex,
    tree: RustSourceTree,
    reader: SourceTextReader,
) -> tuple[MacroEvidence, list[TransientRegistration]]:
    """Assemble every fact source once; `macro_evidence(decls, index, tree, reader)`."""
    registrations = transient_registrations(tree)
    evidence = MacroEvidence(
        index=index,
        registrations=registrations_by_name(registrations),
        descriptor_names=descriptor_names(registrations, index),
        cpp_bases=cpp_base_types(declarations),
        factories=factory_paths(tree, index),
        exception_bases=exception_bases(declarations, reader),
        rust_exception_pairs=exception_parent_pairs(tree),
    )
    return evidence, registrations


def registrations_by_name(
    registrations: list[TransientRegistration],
) -> dict[str, list[TransientRegistration]]:
    """Group registrations by descriptor name; a split class (trait plus base struct)
    registers one name twice, and the parent check then requires both to agree."""
    grouped: dict[str, list[TransientRegistration]] = defaultdict(list)
    for registration in registrations:
        grouped[registration.cxx_name].append(registration)
    return dict(grouped)
