"""Outcomes of the 2026-10-15 D1 review of the TKernel partial ledger rows.

Older audits pinned many TKernel rows as `partial` while their notes named only differences that
plan D1 (docs/tkernel-rust-refactoring-plan.md) does not count as obligations. The review
(porting/audits/tkernel-d1-partial-review-2026-10-15.json) moved each such row to one of three
outcomes, and every row keeps the note it had as evidence:

- `implemented`: the old note plus a sentence that starts with `IMPLEMENTED_MARK` and names the
  D1 classes left open;
- `excluded`: a protected member (class B6), whose note starts with `EXCLUDED_MARK` and ends with
  the old note after `Former partial note: `;
- `partial`: a class-A gap remains; the audit lists the row in `kept_partial_rows`.

`review_outcome(row, kept)` validates a row against these rules, and `supersedes(row, proposed)`
lets an older partial binding receipt accept the reviewed row in its place.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from pathlib import Path

AUDIT = Path("porting/audits/tkernel-d1-partial-review-2026-10-15.json")
IMPLEMENTED_MARK = "D1 review 2026-10-15"
EXCLUDED_MARK = "Excluded (plan D1 class B6"
FORMER_NOTE = "Former partial note: "
# Fields the review may change; every other field of a receipt's row must stay as recorded.
REVIEWED_FIELDS = frozenset({"status", "notes", "rust_declaration_selectors"})

Row = Mapping[str, object]


def kept_partial_rows(root: Path) -> frozenset[str]:
    """Upstream ids the review kept partial; `kept_partial_rows(repo_root)`."""
    audit = json.loads((root / AUDIT).read_text())
    return frozenset(audit["kept_partial_rows"])


def review_outcome(row: Row, kept: frozenset[str]) -> str:
    """The checked outcome of a reviewed row: 'implemented', 'excluded' or 'partial';
    `review_outcome({'status': 'partial', 'upstream_id': 'X'}, frozenset({'X'})) == 'partial'`."""
    status, notes, identity = row.get("status"), str(row.get("notes", "")), row.get("upstream_id")
    if status == "partial" and identity in kept:
        return "partial"
    if status == "implemented" and IMPLEMENTED_MARK in notes:
        return "implemented"
    if status == "excluded" and notes.startswith(EXCLUDED_MARK):
        return "excluded"
    raise ValueError(
        f"{identity!r} status={status!r}; expected a D1-reviewed row (implemented with "
        f"{IMPLEMENTED_MARK!r}, excluded with {EXCLUDED_MARK!r}, or partial listed in {AUDIT})"
    )


def keeps_former_note(row: Row, former: str) -> bool:
    """True when the reviewed note still carries the receipt's note as its evidence."""
    notes = str(row.get("notes", ""))
    if row.get("status") == "excluded":
        return notes.endswith(FORMER_NOTE + former)
    return notes.startswith(former + " " + IMPLEMENTED_MARK)


def supersedes(row: Row, proposed: Row) -> bool:
    """True when `row` is the D1-reviewed form of a receipt's partial `proposed` row: same
    fields apart from status, notes and declaration selectors, the old note kept, and a
    reviewed status; `supersedes(ledger_row, record['proposed_row'])`."""
    fixed = (set(row) | set(proposed)) - REVIEWED_FIELDS
    if any(row.get(field) != proposed.get(field) for field in fixed):
        return False
    if row.get("status") not in ("implemented", "excluded"):
        return False
    return keeps_former_note(row, str(proposed.get("notes", "")))
