"""Present exact legacy declaration accounting without granting runtime proof."""

from __future__ import annotations

from collections.abc import Mapping

from api_inventory.inputs import JSONObject, JSONValue
from publish_legacy_test_inventory import (
    gtest_records,
    gtest_summary,
    toolkit_summary,
)

LEGACY_ANOMALY_FIELDS = (
    "duplicate_mappings",
    "duplicate_declarations",
    "unknown_parity_mappings",
)


def legacy_status_lines(inventory: JSONObject, ledger: JSONObject) -> list[str]:
    """Classify current mappings; e.g. `legacy_status_lines(inventory, ledger)`."""
    declarations, anomalies = gtest_records(inventory, ledger)
    totals = gtest_summary(declarations, anomalies)
    lines = [_legacy_mapping_line("GoogleTest total", totals, len(anomalies))]
    for toolkit, counts in toolkit_summary(inventory, declarations, anomalies).items():
        anomaly_count = sum(counts[field] for field in LEGACY_ANOMALY_FIELDS)
        if counts["declared"] or anomaly_count:
            lines.append(_legacy_mapping_line(toolkit, counts, anomaly_count))
    lines.append(
        "GoogleTest counts describe declaration mappings only; runtime parity and "
        "full API/DRAW/configuration completion remain unproven. "
        "Mapping anomalies include duplicate declarations/mappings and unknown ledger identities."
    )
    return lines


def _legacy_mapping_line(
    label: str, counts: Mapping[str, JSONValue], anomaly_count: int
) -> str:
    return (
        f"{label}: {counts['mapped']}/{counts['declared']} GoogleTest declarations mapped; "
        f"{counts['unmapped']} unmapped; {counts['pending_dependency']} pending dependencies; "
        f"{counts['not_applicable_to_rust']} not applicable to Rust; "
        f"{anomaly_count} mapping anomalies"
    )
