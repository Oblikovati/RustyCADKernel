"""Generate, verify and merge macro/special-member rows into the parity ledger.

Hand-written rows always win: a declaration that any hand row claims is never generated,
and every earlier generated row is dropped and rebuilt, so a re-run is idempotent. Each
generated row must resolve through the same crosswalk the coverage report uses; a row
that does not resolve to its own declaration is reported and not written.
"""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass, field

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners, match_api_row, rust_crates
from api_inventory.inputs import JSONObject
from api_macro_classes import ClassResolver, LedgerClaim, toolkit_crate
from api_macro_cpp import RTTI_KINDS, declaration_owner, special_member_kind, upstream_identity
from api_macro_evidence import macro_evidence
from api_macro_merge import keep_superseded_notes, merged_rows, retire_stale_rows, rtti_ids_by_class
from api_macro_rows import GENERATOR, BuildResult, SpecialMember, special_row
from api_macro_rtti_rows import MacroEvidence, exception_row, record_row, rtti_row
from api_macro_rust import RustInventoryIndex
from api_macro_rust_source import RustSourceTree, TransientRegistration
from api_macro_source import SourceTextReader, UNKNOWN, definition_form

RESOLVED = {"implemented", "excluded", "partial"}
FORM_KINDS = {"destructor", "copy_constructor", "move_constructor", "copy_assignment", "move_assignment"}


@dataclass
class GeneratorInputs:
    """Everything one run reads; file and source access stay injectable for tests."""

    declarations: list[JSONObject]
    headers: JSONObject
    rust: JSONObject
    parity: JSONObject
    occt: SourceTextReader
    rust_sources: RustSourceTree


@dataclass
class GenerationResult:
    """The rewritten ledger plus counts of what was written, skipped and removed."""

    parity: JSONObject
    written: Counter[tuple[str, str, str]] = field(default_factory=Counter)
    skipped: Counter[tuple[str, str, str]] = field(default_factory=Counter)
    skipped_ids: dict[tuple[str, str, str], list[str]] = field(default_factory=dict)
    removed_macro_rows: list[str] = field(default_factory=list)
    class_conflicts: dict[str, list[str]] = field(default_factory=dict)


@dataclass
class LedgerContext:
    """Indexes shared by every step of one run."""

    by_name: dict[str, list[JSONObject]]
    crates: dict[str, dict[str, list[JSONObject]]]
    known_sources: set[str]
    hand_rows: list[JSONObject]
    claims: list[LedgerClaim]
    claimed_ids: set[str]


def ledger_context(inputs: GeneratorInputs) -> LedgerContext:
    """Index declarations and hand rows; `ledger_context(inputs).claimed_ids`."""
    by_name, _, _ = cpp_indexes(inputs.declarations, header_owners(inputs.headers))
    rows = inputs.parity.get("api")
    hand_rows = [row for row in rows if row.get("generated_by") != GENERATOR] if isinstance(rows, list) else []
    claims, claimed = hand_claims(hand_rows, by_name)
    known = {str(row.get("path")) for row in inputs.headers.get("headers", []) if isinstance(row, dict)}
    return LedgerContext(by_name, rust_crates(inputs.rust), known, hand_rows, claims, claimed)


def hand_claims(
    rows: list[JSONObject], by_name: dict[str, list[JSONObject]]
) -> tuple[list[LedgerClaim], set[str]]:
    """Join each hand row to the one declaration it claims, as the report does."""
    claims, claimed = [], set()
    for row in rows:
        status, found = cpp_candidates(str(row.get("upstream_id", "")), str(row.get("toolkit", "")), by_name)
        if status != "unique" or not found:
            continue
        claimed.add(str(found[0].get("id")))
        claims.append(LedgerClaim(row, str(found[0].get("qualified_name")), str(found[0].get("kind"))))
    return claims, claimed


def crate_toolkits(inputs: GeneratorInputs, context: LedgerContext) -> set[str]:
    """Toolkits that have a Rust crate in the inventory, e.g. {'TKernel', 'TKG2d'}."""
    names = {str(row.get("name")) for row in inputs.headers.get("toolkits", []) if isinstance(row, dict)}
    return {name for name in names if toolkit_crate(name) in context.crates}


def declaration_toolkit(declaration: JSONObject, toolkits: set[str]) -> str | None:
    """The `src/<Module>/<Toolkit>/` of the declaration when that toolkit owns it."""
    parts = str(declaration.get("source", "")).split("/")
    toolkit = parts[2] if len(parts) > 3 else None
    owned = declaration.get("_toolkits", set())
    return toolkit if toolkit in toolkits and toolkit in owned else None


def candidate_kind(
    declaration: JSONObject, context: LedgerContext, member_owners: set[str], exceptions: set[str]
) -> str | None:
    """The generator kind of an unclaimed visible declaration, or None to leave it."""
    if declaration.get("api_visible") is not True or declaration.get("id") in context.claimed_ids:
        return None
    if declaration.get("kind") == "CXXRecord":
        return "record" if declaration.get("qualified_name") in member_owners else None
    kind = special_member_kind(declaration)
    if kind == "exception_constructor" and declaration_owner(declaration) not in exceptions:
        return None
    return kind


def build_row(
    member: SpecialMember, evidence: MacroEvidence, sources: RustSourceTree
) -> BuildResult:
    """Route one member to its builder; `build_row(member, evidence, sources)`."""
    if member.kind in RTTI_KINDS.values():
        return rtti_row(member, evidence)
    if member.kind.startswith("exception"):
        return exception_row(member, evidence)
    if member.kind == "record":
        return record_row(member)
    return special_row(member, evidence.index, sources)


def verify_row(row: JSONObject, declaration: JSONObject, context: LedgerContext) -> str | None:
    """Fail unless the crosswalk resolves the row to exactly this declaration."""
    match = match_api_row(row, context.by_name, context.crates, context.known_sources)
    if match["resolution"] not in RESOLVED:
        return f"crosswalk_{match['resolution']}"
    return None if match["cpp_ids"] == [declaration.get("id")] else "crosswalk_other_declaration"


def member_owner_names(claims: list[LedgerClaim]) -> set[str]:
    """Classes with at least one implemented or partial hand-mapped member."""
    return {
        claim.qualified_name.rsplit("::", 1)[0]
        for claim in claims
        if "::" in claim.qualified_name
        and claim.cpp_kind not in {"CXXRecord", "ClassTemplate"}
        and claim.row.get("status") in {"implemented", "partial"}
    }


def exception_classes(declarations: list[JSONObject]) -> set[str]:
    return {declaration_owner(row) for row in declarations if special_member_kind(row) == "exception_type"}


def all_registrations(evidence: MacroEvidence) -> list[TransientRegistration]:
    return [item for name in sorted(evidence.registrations) for item in evidence.registrations[name]]


class MacroRowGenerator:
    """Run the whole pass; `MacroRowGenerator(inputs).run().parity`."""

    def __init__(self, inputs: GeneratorInputs) -> None:
        self.inputs = inputs
        self.context = ledger_context(inputs)
        index = RustInventoryIndex(inputs.rust)
        self.evidence, _ = macro_evidence(inputs.declarations, index, inputs.rust_sources, inputs.occt)
        self.resolver = ClassResolver(index, all_registrations(self.evidence), self.context.claims)
        self.toolkits = crate_toolkits(inputs, self.context)
        self.result = GenerationResult(parity=inputs.parity)
        self.generated: dict[str, JSONObject] = {}

    def run(self) -> GenerationResult:
        owners = member_owner_names(self.context.claims)
        exceptions = exception_classes(self.inputs.declarations)
        for declaration in self.inputs.declarations:
            toolkit = declaration_toolkit(declaration, self.toolkits)
            kind = candidate_kind(declaration, self.context, owners, exceptions) if toolkit else None
            if kind is not None and toolkit is not None:
                self.generate(declaration, kind, toolkit)
        self.merge()
        self.result.class_conflicts = dict(sorted(self.resolver.conflicts.items()))
        return self.result

    def generate(self, declaration: JSONObject, kind: str, toolkit: str) -> None:
        form = definition_form(self.inputs.occt, declaration) if kind in FORM_KINDS else UNKNOWN
        cxx_class = declaration_owner(declaration) or str(declaration.get("qualified_name"))
        rtti = kind in RTTI_KINDS.values()
        resolve = self.resolver.descriptor_class if rtti else self.resolver.resolve
        rust_class = resolve(cxx_class, toolkit)
        member = SpecialMember(declaration, kind, form, toolkit, rust_class)
        row, reason = build_row(member, self.evidence, self.inputs.rust_sources)
        reason = reason if row is None else verify_row(row, declaration, self.context)
        if row is None or reason is not None:
            key = (toolkit, kind, str(reason))
            self.result.skipped[key] += 1
            self.result.skipped_ids.setdefault(key, []).append(upstream_identity(declaration))
            return
        self.generated[str(declaration.get("id"))] = row
        self.result.written[(toolkit, kind, str(row["status"]))] += 1

    def merge(self) -> None:
        """Retire covered macro rows, then write hand rows plus generated rows."""
        rtti = rtti_ids_by_class(self.inputs.declarations)
        keep_superseded_notes(self.inputs.parity, self.generated)
        kept, removed = retire_stale_rows(
            self.context.hand_rows, rtti, self.context.claimed_ids, self.generated
        )
        self.result.removed_macro_rows = removed
        parity = dict(self.inputs.parity)
        parity["api"] = merged_rows(kept, list(self.generated.values()))
        self.result.parity = parity
