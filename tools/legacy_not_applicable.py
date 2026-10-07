"""Account upstream GTests that check a C++-only language property Rust cannot express.

Some GTests assert C++ type traits, for example `std::is_move_constructible<T>` is false for
BRepAlgoAPI_BuilderAlgo. Every Rust value can be moved, so no Rust test can check that property
honestly, and a translation would pass for the wrong reason. Such a declaration is a
`not_applicable_to_rust` ledger row (user decision 2026-10-07): the unchanged native test still
runs and must pass, the row names its `reason`, and every report counts it apart from mapped
and unmapped tests, never as passed.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass

from legacy_pending_dependency import NOT_APPLICABLE_TO_RUST, require_native_only_row

# A reason must explain the language gap, not just restate the status.
MINIMUM_REASON_LENGTH = 40


@dataclass(frozen=True)
class NotApplicableCase:
    """One upstream declaration whose property has no Rust counterpart."""

    identity: str
    source: str
    reason: str


def is_not_applicable(row: Mapping[str, object]) -> bool:
    """True for a not-applicable ledger row; is_not_applicable({'status': 'not_applicable_to_rust'})."""
    return row.get("status") == NOT_APPLICABLE_TO_RUST


def not_applicable_case(row: Mapping[str, object]) -> NotApplicableCase:
    """Validate one not-applicable row; not_applicable_case(ledger_row).reason."""
    identity = require_native_only_row(row)
    reason = row.get("reason")
    if not isinstance(reason, str) or len(reason.strip()) < MINIMUM_REASON_LENGTH:
        raise ValueError(f"{identity!r} reason={reason!r}; expected at least {MINIMUM_REASON_LENGTH} characters that name the C++-only property")
    return NotApplicableCase(identity, str(row.get("source")), reason.strip())


def not_applicable_rows(rows: Sequence[Mapping[str, object]]) -> dict[str, NotApplicableCase]:
    """Validate every not-applicable row; not_applicable_rows(ledger['tests'])."""
    return {case.identity: case for case in map(not_applicable_case, filter(is_not_applicable, rows))}
