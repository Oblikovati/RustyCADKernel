"""Account upstream GTests that skip themselves (GTEST_SKIP) in the native kernel.

An unchanged OCCT test may end in GTEST_SKIP for a configuration the kernel itself does not
complete (TKOffset's Bug24909_2_ClosedCircularSpine skips when the closed-spine sweep throws).
Such a declaration is an `upstream_skipped` ledger row with `behavior_decision:
preserve_current` and the exact skip message in `native_skip_reason`; its Rust translation
takes the same branch and passes. The native case must still skip with that message, so the
approval lapses (and the gate fails) once the kernel completes the configuration; every other
native skip remains a failure of the gate.

Example: native_skip_observations(cases, approved_native_skips(rows)).
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
import xml.etree.ElementTree as ET

UPSTREAM_SKIPPED = "upstream_skipped"


def approved_native_skips(rows: Sequence[Mapping[str, object]]) -> dict[str, str]:
    """The validated skip approvals by identity; approved_native_skips(rows) == {'S.C': 'why'}."""
    approved: dict[str, str] = {}
    for row in rows:
        if row.get("status") != UPSTREAM_SKIPPED:
            continue
        identity = str(row.get("upstream_id"))
        reason = row.get("native_skip_reason")
        if not isinstance(reason, str) or not reason.strip():
            raise ValueError(f"{identity!r} native_skip_reason={reason!r}; expected the native skip message")
        if row.get("behavior_decision") != "preserve_current":
            raise ValueError(f"{identity!r}; expected behavior_decision preserve_current for a native skip")
        approved[identity] = reason
    return approved


def skip_reason(case: ET.Element) -> str | None:
    """The last line of the skip message of a native case, or None when it ran to the end;
    skip_reason(case) == 'TODO OCC24909: ...'."""
    marker = case.find("skipped")
    if marker is None:
        return None
    lines = marker.attrib.get("message", "").strip().splitlines()
    return lines[-1] if lines else ""


def native_skip_observations(cases: Sequence[ET.Element], approved: Mapping[str, str]) -> tuple[list[ET.Element], list[dict[str, str]]]:
    """Split the approved native skips from the other cases; each approved case must skip with
    its recorded message and no other case may skip; returns (other cases, observations)."""
    others: list[ET.Element] = []
    observed: list[dict[str, str]] = []
    for case in cases:
        identity = case.attrib["classname"] + "." + case.attrib["name"]
        reason = skip_reason(case)
        if reason is None:
            if identity in approved:
                raise ValueError(f"Approved native skip now runs to the end; review required: {identity}")
            others.append(case)
            continue
        if approved.get(identity) != reason:
            raise ValueError(f"Unexpected native skip: {identity}: {reason!r}; expected an approved skip message")
        observed.append({"upstream_id": identity, "reason": reason})
    if {row["upstream_id"] for row in observed} != set(approved):
        raise ValueError(f"Missing approved native skip observation: {sorted(set(approved) - {row['upstream_id'] for row in observed})}")
    return others, observed
