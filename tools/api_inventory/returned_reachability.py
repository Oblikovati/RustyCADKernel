"""Validate supplemental inferred-type edges without promoting declaration visibility."""

from __future__ import annotations

import hashlib
import json
from collections.abc import Callable

from .inputs import JSONObject, JSONValue

POLICY = "inferred_private_record_v1"


def bind_declaration(value: JSONValue, identity: Callable[[JSONObject], str]) -> str:
    """Attach the existing inventory identity; e.g. `bind_declaration(row, declaration_identity)`."""
    if not isinstance(value, dict) or value.get("event") != "declaration":
        raise ValueError(f"expected nested declaration object, got {value!r}")
    identifier = identity(value)
    value["id"] = identifier
    return identifier


def bind_members(event: JSONObject, identity: Callable[[JSONObject], str]) -> None:
    """Validate access classifications and bind member IDs without adding primary rows."""
    members = event.get("members", [])
    if not isinstance(members, list):
        raise TypeError(f"expected member list, got {members!r}")
    accesses = {
        "inferred_type_public_member",
        "requires_derived_inferred_type",
        "protected_member_of_final_type",
        "private_member_inaccessible",
    }
    for member in members:
        if not isinstance(member, dict) or member.get("access_basis") not in accesses:
            raise ValueError(
                f"expected member object with supported access basis, got {member!r}"
            )
        bind_declaration(member.get("declaration"), identity)


def validate_edge_status(event: JSONObject) -> None:
    """Withhold malformed or newly promoted edges; e.g. `validate_edge_status(edge)`."""
    status = event.get("status")
    resolved = {
        "private_record_members_recorded",
        "cycle_or_repeated_target",
        "target_definition_unavailable",
    }
    if status not in resolved | {"unsupported_template_or_dependent_type"}:
        raise ValueError(f"expected supported returned-type status, got {status!r}")
    if status in resolved and not isinstance(event.get("target_declaration"), dict):
        raise ValueError(
            f"expected target declaration for status {status!r}, got {event.get('target_declaration')!r}"
        )
    validate_member_capture(event)


def validate_member_capture(event: JSONObject) -> None:
    """Reject enumerated unresolved surfaces and promoted credit; e.g. `validate_member_capture(edge)`."""
    status = event.get("status")
    if (
        event.get("coverage_credit") is not False
        or event.get("reachability_complete") is not False
    ):
        raise ValueError(f"expected withheld coverage and closure, got {event!r}")
    if status == "private_record_members_recorded" and not isinstance(
        event.get("members"), list
    ):
        raise ValueError(
            f"expected member list for status {status!r}, got {event.get('members')!r}"
        )
    if status != "private_record_members_recorded" and "members" in event:
        raise ValueError(
            f"expected no enumerated members for status {status!r}, got {event['members']!r}"
        )


def bind_returned_edge(
    event: JSONObject, identity: Callable[[JSONObject], str]
) -> None:
    """Bind seed, producer, target and member IDs using the primary declaration algorithm."""
    validate_edge_status(event)
    keys = ["seed_declaration", "producer_declaration"]
    if "target_declaration" in event:
        keys.append("target_declaration")
    identities = [bind_declaration(event.get(key), identity) for key in keys]
    bind_members(event, identity)
    identity_fields: list[JSONValue] = [
        *identities,
        event.get("type_wrappers"),
        event.get("status"),
    ]
    event["id"] = hashlib.sha256(
        json.dumps(identity_fields, sort_keys=True).encode()
    ).hexdigest()


def returned_capture_gap(
    summary: JSONObject, observed: int, header: str
) -> JSONObject | None:
    """Reject missing policies or truncated edge streams; e.g. `returned_capture_gap(summary, 3, header)`."""
    emitted = summary.get("returned_type_reachability_events")
    if summary.get("returned_type_reachability_policy") != POLICY:
        return {
            "reason": "returned_type_reachability_capture_unavailable",
            "header": header,
            "observed_policy": summary.get("returned_type_reachability_policy"),
        }
    if type(emitted) is not int or emitted != observed:
        return {
            "reason": "returned_type_reachability_count_mismatch",
            "header": header,
            "summary_count": emitted,
            "observed_count": observed,
        }
    return None
