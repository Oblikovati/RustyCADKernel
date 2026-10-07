"""Merge generated rows into the hand ledger and retire superseded macro-spelling rows.

A former `Class::DEFINE_STANDARD_RTTIEXT` row names a macro, not a declaration, so the
crosswalk can never resolve it. It is removed only after every RTTI declaration that the
macro expands to is claimed (by a hand row or a generated row); its notes move to the
generated `get_type_descriptor()` row so evidence links are kept.
"""

from __future__ import annotations

from collections import defaultdict

from api_inventory.inputs import JSONObject
from api_macro_cpp import RTTI_KINDS, declaration_owner, special_member_kind

STALE_MARKER = "::DEFINE_STANDARD_"
SUPERSEDES = " Supersedes the former "


def rtti_ids_by_class(declarations: list[JSONObject]) -> dict[tuple[str, str], list[JSONObject]]:
    """Visible RTTI macro declarations keyed by (class, source-path toolkit)."""
    found: dict[tuple[str, str], list[JSONObject]] = defaultdict(list)
    for row in declarations:
        if row.get("api_visible") is True and special_member_kind(row) in RTTI_KINDS.values():
            for toolkit in sorted(row.get("_toolkits", set())):
                found[(declaration_owner(row), toolkit)].append(row)
    return dict(found)


def stale_macro_class(row: JSONObject) -> str | None:
    """`Geom2d_Line::DEFINE_STANDARD_RTTIEXT` -> `Geom2d_Line`; other rows -> None."""
    identity = str(row.get("upstream_id", ""))
    return identity.split(STALE_MARKER, 1)[0] if STALE_MARKER in identity else None


def retire_stale_rows(
    hand_rows: list[JSONObject],
    rtti: dict[tuple[str, str], list[JSONObject]],
    claimed: set[str],
    generated: dict[str, JSONObject],
) -> tuple[list[JSONObject], list[str]]:
    """Drop macro rows whose expansion is fully claimed; return kept rows and removed ids."""
    kept, removed = [], []
    for row in hand_rows:
        cxx = stale_macro_class(row)
        members = rtti.get((cxx, str(row.get("toolkit"))), []) if cxx else []
        ids = {str(item.get("id")) for item in members}
        if not ids or not ids <= claimed | set(generated):
            kept.append(row)
            continue
        removed.append(str(row.get("upstream_id")))
        carry_notes(row, members, generated)
    return kept, removed


def carry_notes(
    row: JSONObject, members: list[JSONObject], generated: dict[str, JSONObject]
) -> None:
    """Append the retired row's notes to the generated descriptor row, if any."""
    for member in members:
        target = generated.get(str(member.get("id")))
        if target is not None and special_member_kind(member) == "rtti_descriptor":
            target["notes"] = (
                f"{target['notes']}{SUPERSEDES}{row.get('upstream_id')} row: "
                f"{row.get('notes')}"
            )


def keep_superseded_notes(parity: JSONObject, generated: dict[str, JSONObject]) -> None:
    """Carry a retired row's notes across re-runs, where the macro row no longer exists."""
    rows = parity.get("api")
    for row in rows if isinstance(rows, list) else []:
        notes = str(row.get("notes", ""))
        target = generated.get(str(row.get("cpp_declaration_id")))
        marker = notes.find(SUPERSEDES)
        if not row.get("generated_by") or target is None or marker < 0:
            continue
        if SUPERSEDES not in str(target["notes"]):
            target["notes"] = f"{target['notes']}{notes[marker:]}"


def anchor_positions(rows: list[JSONObject]) -> tuple[dict[str, int], dict[str, int]]:
    """Last row index for each source header and for each package directory."""
    by_source, by_package = {}, {}
    for position, row in enumerate(rows):
        source = str(row.get("source", ""))
        by_source[source] = position
        by_package[source.rsplit("/", 1)[0]] = position
    return by_source, by_package


def merged_rows(hand_rows: list[JSONObject], generated: list[JSONObject]) -> list[JSONObject]:
    """Insert generated rows after the last hand row of their header (or package).

    Example: `merged_rows([a_hxx_row], [a_hxx_generated])` keeps them adjacent.
    """
    by_source, by_package = anchor_positions(hand_rows)
    slots: dict[int, list[JSONObject]] = defaultdict(list)
    ordered = sorted(generated, key=lambda row: (str(row.get("source")), str(row.get("upstream_id"))))
    for row in ordered:
        source = str(row.get("source", ""))
        anchor = by_source.get(source, by_package.get(source.rsplit("/", 1)[0], len(hand_rows) - 1))
        slots[anchor].append(row)
    merged = list(slots.get(-1, []))
    for position, row in enumerate(hand_rows):
        merged.append(row)
        merged.extend(slots.get(position, []))
    return merged
