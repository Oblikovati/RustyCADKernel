#!/usr/bin/env python3
"""Report staged inferred-type reachability without altering the published API denominator."""

from __future__ import annotations

import argparse
import gzip
import json
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path

from api_inventory.inputs import (
    JSONObject,
    digest,
    object_rows,
    read_object,
    text_field,
)
from api_inventory.merge import write_jsonl_gzip

COMPARISON_FIELDS = (
    "kind",
    "usr",
    "signature",
    "identity_signature",
    "template_signature",
    "api_visible",
    "exclusion_reason",
    "qualified_name",
    "canonical_type",
    "access",
    "ancestor_access",
    "parent_usr",
    "parameter_types",
    "variadic",
    "default_arguments",
)


def read_table(root: Path, name: str) -> list[JSONObject]:
    """Require the staged manifest's exact artifact bytes; e.g. `read_table(root, 'declarations')`."""
    manifest = read_object(root / "occt-manifest.json")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, dict) or not isinstance(artifacts.get(name), dict):
        raise TypeError(f"artifact {name!r}: expected manifest artifact object")
    artifact = artifacts[name]
    path = root / text_field(artifact, "path")
    if digest(path) != artifact.get("sha256"):
        raise ValueError(f"artifact {path}: expected SHA256 {artifact.get('sha256')!r}")
    with gzip.open(path, "rt") as stream:
        rows = [json.loads(line) for line in stream]
    if type(artifact.get("rows")) is not int or len(rows) != artifact.get("rows"):
        raise ValueError(
            f"artifact {path}: expected {artifact.get('rows')!r} rows, got {len(rows)}"
        )
    return object_rows(rows, name)


def declaration_comparison(
    current: list[JSONObject], prior: list[JSONObject], sources: set[str]
) -> list[JSONObject]:
    """Keep additions, omissions and changed fields explicit instead of rewriting old records."""
    fresh = {text_field(row, "id"): row for row in current}
    old = {
        text_field(row, "id"): row
        for row in prior
        if text_field(row, "id") in fresh or declaration_has_site(row, sources)
    }
    return [
        compared_declaration(identifier, fresh.get(identifier), old.get(identifier))
        for identifier in sorted(fresh.keys() | old.keys())
    ]


def declaration_has_site(row: JSONObject, sources: set[str]) -> bool:
    """Use every observed declaration site, since first-source attribution can change."""
    if row.get("source") in sources:
        return True
    return any(
        site.get("source") in sources
        for site in object_rows(row.get("sites", []), "declaration sites")
    )


def semantic_projection(row: JSONObject | None) -> JSONObject | None:
    """Select comparable native fields; e.g. `semantic_projection(declaration)`."""
    if row is None:
        return None
    return {name: row.get(name) for name in COMPARISON_FIELDS}


def record_comparison_state(
    current: JSONObject | None, prior: JSONObject | None, changed: list[str]
) -> str:
    """Distinguish missing observations from changed native fields."""
    if prior is None:
        return "only_current"
    if current is None:
        return "only_published"
    return "changed" if changed else "unchanged"


def changed_native_fields(
    current: JSONObject | None, prior: JSONObject | None
) -> list[str]:
    """List semantic differences while keeping absent rows in their separate category."""
    return (
        []
        if current is None or prior is None
        else [
            name for name in COMPARISON_FIELDS if current.get(name) != prior.get(name)
        ]
    )


def comparison_observations(
    current: JSONObject | None, prior: JSONObject | None
) -> JSONObject:
    """Keep visibility, semantic projections and contextual exclusions together."""
    changed = changed_native_fields(current, prior)
    context_only = (
        current is not None
        and current.get("exclusion_reason") == "implementation_only_no_header_site"
    )
    return {
        "state": record_comparison_state(current, prior, changed),
        "changed_fields": changed,
        "implementation_context_only": context_only,
        "current_api_visible": current.get("api_visible") if current else None,
        "published_api_visible": prior.get("api_visible") if prior else None,
        "current": semantic_projection(current),
        "published": semantic_projection(prior),
    }


def compared_declaration(
    identifier: str, current: JSONObject | None, prior: JSONObject | None
) -> JSONObject:
    """Compare identity-bearing fields independently of site observations and artifact locations."""
    row = current if current is not None else prior
    if row is None:
        raise ValueError(
            f"declaration {identifier!r}: expected current or published record, got neither"
        )
    return {
        "id": identifier,
        "qualified_name": row.get("qualified_name"),
        "source": row.get("source"),
        "kind": row.get("kind"),
        **comparison_observations(current, prior),
    }


@dataclass
class ReachedMember:
    """Accumulate exact incoming exposures without merging overload identities."""

    declaration: JSONObject
    access: set[str] = field(default_factory=set)
    producers: set[str] = field(default_factory=set)
    seeds: set[str] = field(default_factory=set)
    signatures: set[str] = field(default_factory=set)
    sources: set[str] = field(default_factory=set)
    observations: int = 0


def nested_declaration(row: JSONObject, key: str) -> JSONObject:
    """Require a previously bound native declaration object; e.g. `nested_declaration(edge, 'seed_declaration')`."""
    value = row.get(key)
    if not isinstance(value, dict) or not isinstance(value.get("id"), str):
        raise TypeError(f"{key}: expected bound declaration object, got {value!r}")
    return value


def collect_members(edges: list[JSONObject]) -> dict[str, ReachedMember]:
    """Deduplicate by exact native ID while retaining every access/producer/seed observation."""
    members: dict[str, ReachedMember] = {}
    for edge in edges:
        producer = text_field(nested_declaration(edge, "producer_declaration"), "id")
        seed = text_field(nested_declaration(edge, "seed_declaration"), "id")
        for row in object_rows(edge.get("members", []), "reached members"):
            declaration = nested_declaration(row, "declaration")
            entry = members.setdefault(
                text_field(declaration, "id"), ReachedMember(declaration)
            )
            entry.access.add(text_field(row, "access_basis"))
            entry.producers.add(producer)
            entry.seeds.add(seed)
            entry.signatures.add(text_field(declaration, "signature"))
            entry.sources.add(text_field(declaration, "source"))
            entry.observations += 1
    return members


def member_observations(member: ReachedMember) -> JSONObject:
    """Retain all source, signature and access observations of one exact member ID."""
    return {
        "source_observations": sorted(member.sources),
        "signature_observations": sorted(member.signatures),
        "access_basis_observations": sorted(member.access),
        "producer_ids": sorted(member.producers),
        "seed_ids": sorted(member.seeds),
        "observations": member.observations,
    }


def member_primary_link(original: JSONObject | None) -> JSONObject:
    """Link a primary declaration without granting callable or API parity credit."""
    return {
        "primary_declaration_present": original is not None,
        "primary_api_visible": original.get("api_visible") if original else None,
        "primary_exclusion_reason": original.get("exclusion_reason")
        if original
        else None,
        "coverage_credit": False,
        "callability_proven": False,
    }


def member_rows(
    edges: list[JSONObject], declarations: list[JSONObject]
) -> list[JSONObject]:
    """Link reachable members to primary declarations while withholding all API credit."""
    primary = {text_field(row, "id"): row for row in declarations}
    rows: list[JSONObject] = []
    for identifier, member in sorted(collect_members(edges).items()):
        rows.append(
            {
                "id": identifier,
                "qualified_name": member.declaration.get("qualified_name"),
                "kind": member.declaration.get("kind"),
                "signature": member.declaration.get("signature"),
                **member_observations(member),
                **member_primary_link(primary.get(identifier)),
            }
        )
    return rows


def edge_metrics(edges: list[JSONObject]) -> JSONObject:
    """Count distinct identities separately from repeated capture observations."""
    targets = {
        text_field(nested_declaration(row, "target_declaration"), "id")
        for row in edges
        if row.get("status") == "private_record_members_recorded"
    }
    return {
        "returned_edge_observations": len(edges),
        "distinct_returned_edges": len({text_field(row, "id") for row in edges}),
        "edge_statuses": dict(Counter(text_field(row, "status") for row in edges)),
        "distinct_reached_private_record_types": len(targets),
    }


def comparison_metrics(comparison: list[JSONObject]) -> JSONObject:
    """Retain missing API IDs and changed fields as explicit review cases."""
    changed = [row for row in comparison if row["state"] == "changed"]
    return {
        "primary_id_comparison": dict(
            Counter(text_field(row, "state") for row in comparison)
        ),
        "missing_published_api_ids": [
            row["id"]
            for row in comparison
            if row["state"] == "only_published" and row["published_api_visible"] is True
        ],
        "changed_primary_fields": [row["id"] for row in changed],
        "implementation_context_visibility_changes": [
            row["id"] for row in changed if row["implementation_context_only"]
        ],
    }


def report_summary(
    edges: list[JSONObject], members: list[JSONObject], comparison: list[JSONObject]
) -> JSONObject:
    """Separate observed edges, distinct IDs and unresolved visibility domains."""
    access = Counter(
        str(access) for row in members for access in row["access_basis_observations"]
    )
    return {
        "schema": "rustycad.returned-type-scope-report.v1",
        "coverage_credit": False,
        "api_denominator_complete": False,
        "distinct_reached_members": len(members),
        "member_access_observations": dict(access),
        **edge_metrics(edges),
        **comparison_metrics(comparison),
    }


def write_comparison_tables(
    output: Path, comparison: list[JSONObject], members: list[JSONObject]
) -> JSONObject:
    """Write deterministic evidence tables; e.g. `write_comparison_tables(output, comparison, members)`."""
    output.mkdir(parents=True, exist_ok=True)
    return {
        name: write_jsonl_gzip(output / f"{name}.jsonl.gz", rows)
        for name, rows in [
            ("primary-comparison", comparison),
            ("reached-members", members),
        ]
    }


def write_report(inventory: Path, published: Path, output: Path) -> JSONObject:
    """Write staged evidence only; e.g. `write_report(staged, published, output)`."""
    current, prior = (
        read_table(inventory, "declarations"),
        read_table(published, "declarations"),
    )
    headers = read_table(inventory, "headers")
    edges = read_table(inventory, "returned-type-reachability")
    comparison = declaration_comparison(
        current, prior, {text_field(row, "path") for row in headers}
    )
    members = member_rows(edges, current)
    summary = {
        **report_summary(edges, members, comparison),
        "artifacts": write_comparison_tables(output, comparison, members),
        "staged_manifest_sha256": digest(inventory / "occt-manifest.json"),
        "published_manifest_sha256": digest(published / "occt-manifest.json"),
    }
    write_report_document(output, summary)
    return summary


def write_report_document(output: Path, summary: JSONObject) -> None:
    """Serialize the deterministic report document; e.g. `write_report_document(output, summary)`."""
    (output / "report.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n"
    )


def main() -> None:
    """Read staged and published inventories, then write a separate comparison report."""
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["inventory", "published", "output"]:
        parser.add_argument(f"--{name}", type=Path, required=True)
    args = parser.parse_args()
    print(
        json.dumps(
            write_report(args.inventory, args.published, args.output), sort_keys=True
        )
    )


if __name__ == "__main__":
    main()
