"""Account source-disabled OCCT assertions without counting them as passing ports."""

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
import re
import xml.etree.ElementTree as ET


@dataclass(frozen=True)
class SourceDisabledCase:
    """Pinned disabled assertion and its separate current-behavior regression."""

    identity: str
    reason: str
    expected_failure: str
    rust_test: str
    rust_reference: str


def required_disabled_text(row: Mapping[str, object], field: str) -> str:
    """Require nonblank provenance; required_disabled_text(row, 'native_disabled_reason')."""
    value = row.get(field)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(
            f"{field}={value!r}; expected nonblank disabled-case provenance"
        )
    return value


def source_disabled_row(row: Mapping[str, object]) -> SourceDisabledCase:
    """Validate one explicit preservation decision; source_disabled_row(ledger_row)."""
    identity = validate_disabled_provenance(row)
    case = SourceDisabledCase(
        identity,
        required_disabled_text(row, "native_disabled_reason"),
        required_disabled_text(row, "native_disabled_expected_failure"),
        required_disabled_text(row, "rust_test"),
        required_disabled_text(row, "rust_reference"),
    )
    if case.rust_test == case.rust_reference or "native_expected_failure" in row:
        raise ValueError(
            f"{identity!r}; expected separate compatibility test and disabled assertion"
        )
    return case


def validate_disabled_provenance(row: Mapping[str, object]) -> str:
    """Require the pinned source and preservation decision; validate_disabled_provenance(row)."""
    identity = required_disabled_text(row, "upstream_id")
    if not is_source_disabled_name(identity):
        raise ValueError(
            f"{identity!r}; expected a source DISABLED_ suite or case name"
        )
    if row.get("behavior_decision") != "preserve_current" or "runtime_cases" in row:
        raise ValueError(
            f"{identity!r}; expected preserve_current without runtime expansions"
        )
    digest = required_disabled_text(row, "source_sha256")
    if not re.fullmatch("[0-9a-f]{64}", digest):
        raise ValueError(f"{digest!r}; expected a pinned source SHA256")
    return identity


def source_disabled_rows(
    rows: Sequence[Mapping[str, object]],
) -> dict[str, SourceDisabledCase]:
    """Reject unsupported statuses and unclassified DISABLED_ names; source_disabled_rows(rows)."""
    disabled: dict[str, SourceDisabledCase] = {}
    for row in rows:
        identity = required_disabled_text(row, "upstream_id")
        if row.get("status") == "upstream_disabled":
            case = source_disabled_row(row)
            if identity in disabled:
                raise ValueError(
                    f"{identity!r}; expected one disabled declaration mapping"
                )
            disabled[identity] = case
            continue
        if row.get("status") != "translated" or is_source_disabled_name(identity):
            raise ValueError(
                f'{identity!r} status={row.get("status")!r}; expected translated active or upstream_disabled'
            )
    return disabled


def is_source_disabled_name(identity: str) -> bool:
    """Recognize Google's suite/case disabling prefix; is_source_disabled_name('S.DISABLED_C')."""
    parts = identity.split(".")
    return any(part.startswith("DISABLED_") for part in parts)


def native_case_identity(case: ET.Element) -> str:
    """Recover the full XML identity; native_case_identity(testcase)."""
    suite = case.attrib.get("classname", "")
    name = case.attrib.get("name", "")
    return suite + "." + name


def active_native_cases(
    cases: Sequence[ET.Element], disabled: Mapping[str, SourceDisabledCase]
) -> list[ET.Element]:
    """Require the precise source-disabled suppression; active_native_cases(xml_cases, disabled)."""
    active: list[ET.Element] = []
    observed: set[str] = set()
    for case in cases:
        identity = native_case_identity(case)
        if identity not in disabled:
            active.append(case)
            continue
        require_default_suppression(case, identity, observed)
        observed.add(identity)
    if observed != set(disabled):
        raise ValueError(
            f"{sorted(observed)!r}; expected all source-disabled cases {sorted(disabled)!r}"
        )
    return active


def require_default_suppression(
    case: ET.Element, identity: str, observed: set[str]
) -> None:
    """Reject an original that was run, skipped, or duplicated; require_default_suppression(case, name, seen)."""
    if (
        identity in observed
        or case.attrib.get("status") != "notrun"
        or case.attrib.get("result") != "suppressed"
        or len(case)
    ):
        raise ValueError(
            f"{identity!r}; expected one suppressed/notrun source-disabled case without failures/skips"
        )


def forced_disabled_failure(
    case: ET.Element, expected: SourceDisabledCase
) -> dict[str, object]:
    """Require the pinned failure when enabled; forced_disabled_failure(xml_case, provenance)."""
    messages = require_original_failure(case, expected)
    return {
        "upstream_id": expected.identity,
        "reason": expected.reason,
        "forced_failure_messages": messages,
        "current_behavior_rust_test": expected.rust_test,
        "rust_reference": expected.rust_reference,
        "legacy_assertions_passed": False,
    }


def require_original_failure(
    case: ET.Element, expected: SourceDisabledCase
) -> list[str]:
    """Require execution and the known failing assertion; require_original_failure(case, expected)."""
    require_executed_original(case, expected.identity)
    messages = [
        failure.attrib.get("message", "") for failure in case.findall("failure")
    ]
    if not messages or not all(
        expected.expected_failure in message for message in messages
    ):
        raise ValueError(
            f"{expected.identity!r} failures={messages!r}; expected original failure {expected.expected_failure!r}"
        )
    return messages


def require_executed_original(case: ET.Element, identity: str) -> None:
    """Reject suppressed or skipped forced originals; require_executed_original(case, name)."""
    if (
        case.attrib.get("status") != "run"
        or case.attrib.get("result") != "completed"
        or case.find("skipped") is not None
    ):
        raise ValueError(
            f"{identity!r}; expected forced disabled case to run without skips"
        )


def forced_disabled_observations(
    cases: Sequence[ET.Element], disabled: Mapping[str, SourceDisabledCase]
) -> list[dict[str, object]]:
    """Reject missing, extra, duplicate, or newly passing originals; forced_disabled_observations(cases, disabled)."""
    identities = [native_case_identity(case) for case in cases]
    if len(identities) != len(set(identities)) or set(identities) != set(disabled):
        raise ValueError(
            f"{identities!r}; expected exactly the forced source-disabled cases {sorted(disabled)!r}"
        )
    return [
        forced_disabled_failure(case, disabled[native_case_identity(case)])
        for case in cases
    ]
