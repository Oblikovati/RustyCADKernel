"""Account upstream GTests that wait for an unported C++ class of a higher toolkit.

A GTest of one toolkit may build its fixture with a class of a toolkit that is not ported yet
(TKBRep's BRepTools_Test cuts faces with TKBO's BRepAlgoAPI_Cut). Such a declaration is a
`pending_dependency` ledger row: the unchanged native test still runs and must pass, but there
is no Rust test, so every report counts it as pending, never as passed or mapped. The row
names the missing classes in `blocked_by`; the approval lapses as soon as one of them gets an
implemented class record, so the test is translated once it can be.
"""

from __future__ import annotations

import re
from collections.abc import Mapping, Sequence
from dataclasses import dataclass

PENDING_DEPENDENCY = "pending_dependency"
# A GTest of a C++-only language property (see legacy_not_applicable); it has no Rust side either.
NOT_APPLICABLE_TO_RUST = "not_applicable_to_rust"
WITHOUT_RUST_SIDE = frozenset({PENDING_DEPENDENCY, NOT_APPLICABLE_TO_RUST})
# A pending row has no Rust side; any of these fields would claim one.
RUST_SIDE_FIELDS = ("rust_package", "rust_target", "rust_test", "rust_source", "rust_reference", "runtime_cases")
# Native-side approvals belong to translated rows only; a pending test must pass unchanged.
NATIVE_APPROVAL_FIELDS = ("native_expected_failure", "native_disabled_reason", "native_skip_reason", "behavior_decision")
CPP_CLASS = re.compile(r"[A-Za-z][A-Za-z0-9]*_[A-Za-z0-9_]+")


@dataclass(frozen=True)
class PendingDependencyCase:
    """One upstream declaration whose Rust translation waits for unported C++ classes."""

    identity: str
    source: str
    blocked_by: tuple[str, ...]


def is_pending_dependency(row: Mapping[str, object]) -> bool:
    """True for a pending-dependency ledger row; is_pending_dependency({'status': 'pending_dependency'})."""
    return row.get("status") == PENDING_DEPENDENCY


def blocking_classes(row: Mapping[str, object]) -> tuple[str, ...]:
    """The validated `blocked_by` list; blocking_classes({'blocked_by': ['BRepAlgoAPI_Cut']})."""
    value = row.get("blocked_by")
    if not isinstance(value, list) or not value:
        raise ValueError(f"{row.get('upstream_id')!r} blocked_by={value!r}; expected a nonempty list of C++ class names")
    if any(not isinstance(name, str) or not CPP_CLASS.fullmatch(name) for name in value):
        raise ValueError(f"{row.get('upstream_id')!r} blocked_by={value!r}; expected names like 'BRepAlgoAPI_Cut'")
    if len(set(value)) != len(value):
        raise ValueError(f"{row.get('upstream_id')!r} blocked_by={value!r}; expected each class once")
    return tuple(value)


def require_native_only_row(row: Mapping[str, object]) -> str:
    """Check a row without a Rust side: no Rust or native-approval fields and a pinned source;
    returns its identity; require_native_only_row(ledger_row) == 'Suite.Case'."""
    identity = str(row.get("upstream_id"))
    claimed = [field for field in (*RUST_SIDE_FIELDS, *NATIVE_APPROVAL_FIELDS) if field in row]
    if claimed:
        raise ValueError(f"{identity!r} carries {claimed!r}; expected a {row.get('status')} row without Rust or native-approval fields")
    digest = row.get("source_sha256")
    if not isinstance(digest, str) or not re.fullmatch("[0-9a-f]{64}", digest):
        raise ValueError(f"{identity!r} source_sha256={digest!r}; expected a pinned source SHA256")
    return identity


def pending_dependency_case(row: Mapping[str, object]) -> PendingDependencyCase:
    """Validate one pending row; pending_dependency_case(ledger_row).blocked_by == ('BRepAlgoAPI_Cut',)."""
    identity = require_native_only_row(row)
    return PendingDependencyCase(identity, str(row.get("source")), blocking_classes(row))


def pending_dependency_rows(rows: Sequence[Mapping[str, object]], implemented: set[str]) -> dict[str, PendingDependencyCase]:
    """Validate every pending row and lapse those whose blocker is now ported;
    pending_dependency_rows(rows, implemented_classes(ledger))."""
    pending = {case.identity: case for case in map(pending_dependency_case, filter(is_pending_dependency, rows))}
    for case in pending.values():
        ported = sorted(set(case.blocked_by) & implemented)
        if ported:
            raise ValueError(f"{case.identity!r} blocked_by {ported!r}, which are now ported; translate the test or drop them from blocked_by")
    return pending


def runnable_rows(rows: Sequence[Mapping[str, object]]) -> list[Mapping[str, object]]:
    """Rows that have a Rust side (every status except pending_dependency and
    not_applicable_to_rust); runnable_rows(rows)."""
    return [row for row in rows if row.get("status") not in WITHOUT_RUST_SIDE]


def implemented_classes(api_rows: Sequence[Mapping[str, object]]) -> set[str]:
    """C++ classes with an implemented ledger record; implemented_classes(ledger['api'])."""
    return {str(row["upstream_id"]) for row in api_rows if row.get("status") == "implemented" and CPP_CLASS.fullmatch(str(row.get("upstream_id")))}


def require_blockers_in_source(case: PendingDependencyCase, text: str) -> None:
    """A blocker must be named by the GTest source it blocks; require_blockers_in_source(case, cxx_text)."""
    absent = [name for name in case.blocked_by if not re.search(rf"\b{name}\b", text)]
    if absent:
        raise ValueError(f"{case.identity!r} blocked_by {absent!r}, which {case.source} never names; expected classes the test uses")
