"""Produce an input-bound inventory of unresolved OCCT header capture gaps."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
from collections import Counter
from tools.api_inventory.context_retries import verify_retry_source
from tools.api_inventory.inputs import JSONValue, JSONObject, object_rows, text_field


def sha256(path: pathlib.Path) -> str:
    """Hash an input file; e.g. `sha256(pathlib.Path("capture.json"))`."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_revision(source: pathlib.Path) -> str:
    """Read the pinned OCCT commit; e.g. `source_revision(Path("../OCCT"))`."""
    result = subprocess.run(
        ["git", "-C", str(source), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def load_json(path: pathlib.Path) -> JSONObject:
    """Load a JSON object and reject other top-level shapes."""
    value: JSONValue = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected JSON object")
    return value


def classify_unconfigured(toolkit: str) -> tuple[str, str]:
    """Explain known absent toolkit configurations without treating them as exclusions."""
    if toolkit.startswith("TKIVtk"):
        return (
            "optional_dependency_unconfigured",
            "Enable USE_VTK and capture with VTK dependencies.",
        )
    if toolkit.startswith("TKD3DHost"):
        return (
            "platform_unconfigured",
            "Capture with a supported Windows and Direct3D toolchain.",
        )
    return (
        "configuration_unresolved",
        "Add a configuration that builds this owning toolkit.",
    )


def classify_failed(path: str) -> tuple[str, str]:
    """Classify observed failures using explicit path-specific evidence."""
    if "/BVH/" in path:
        return (
            "extractor_unhandled_decl_kind",
            "Support UnresolvedUsingValue, rebuild, and recapture.",
        )
    if path.endswith("RWGltf_GltfJsonParser.hxx"):
        return (
            "external_dependency_parse_error",
            "Resolve the RapidJSON 1.1.0 Clang incompatibility and recapture.",
        )
    return (
        "parse_failure_unclassified",
        "Inspect diagnostics and capture with a verified source context.",
    )


def context_gap(path: str) -> tuple[str, str]:
    """Describe why a header has no source translation-unit include context."""
    if path.endswith("NCollection_Haft.h"):
        return (
            "language_mode_unavailable",
            "Capture with the actual MSVC C++/CLI configuration if in scope.",
        )
    if path.endswith("OpenGl_GLESExtensions.hxx"):
        return (
            "optional_platform_feature_unconfigured",
            "Enable GLES and verify an active source include.",
        )
    return (
        "no_source_include_consumer",
        "Keep unresolved until a real consuming translation unit is identified.",
    )


def _capture_gaps(capture: JSONObject, retries: JSONObject) -> list[JSONObject]:
    retry_rows = object_rows(retries.get("retry_results"), "retry results")
    failed = {text_field(row, "path"): row for row in retry_rows}
    gaps: list[JSONObject] = []
    for candidate in object_rows(capture.get("header_candidates"), "header candidates"):
        path = text_field(candidate, "path")
        status = text_field(candidate, "status")
        if status == "missing_compile_configuration":
            gaps.append(_unconfigured_gap(candidate))
        elif (
            path in failed
            and text_field(failed[path], "status") != "parsed_with_context"
        ):
            gaps.append(_retry_gap(path, failed[path], candidate))
    return sorted(gaps, key=lambda item: text_field(item, "path"))


def _unconfigured_gap(candidate: JSONObject) -> JSONObject:
    owners = object_rows(candidate.get("owners"), "header owners")
    toolkit = text_field(owners[0], "toolkit")
    category, remedy = classify_unconfigured(toolkit)
    return {
        "path": text_field(candidate, "path"),
        "toolkit": toolkit,
        "category": category,
        "evidence": "header manifest marks missing_compile_configuration",
        "recommended_recovery": remedy,
        "capture_status": "unresolved",
    }


def _retry_gap(path: str, row: JSONObject, candidate: JSONObject) -> JSONObject:
    owners = object_rows(candidate.get("owners"), "header owners")
    if not owners:
        raise ValueError(f"header {path!r}: expected at least one manifest owner")
    status = text_field(row, "status")
    if status == "no_source_include_context":
        category, remedy = context_gap(path)
    elif row.get("returncode") == 0 and row.get("target_header_seen") is False:
        category = "platform_condition_excludes_header"
        remedy = "Capture with the platform configuration that activates the guarded include."
    else:
        category, remedy = classify_failed(path)
    return {
        "path": path,
        "toolkit": text_field(owners[0], "toolkit"),
        "owners": owners,
        "category": category,
        "evidence": row.get("context_evidence", row.get("stderr", "")),
        "retry_status": status,
        "artifact_sha256": row.get("output_sha256"),
        "recommended_recovery": remedy,
        "capture_status": "unresolved",
    }


def verify_retry_manifest(
    capture: JSONObject, retry: JSONObject, capture_path: pathlib.Path
) -> None:
    if retry.get("original_capture_sha256") != sha256(capture_path):
        raise ValueError("retry manifest: expected hash of supplied original capture")
    if retry.get("original_fingerprint") != capture.get("fingerprint"):
        raise ValueError("retry manifest: expected original capture fingerprint")
    inputs = capture.get("inputs")
    if not isinstance(inputs, dict):
        raise ValueError("capture inputs: expected object")
    for key in ("extractor_sha256", "database_sha256"):
        if retry.get(key) != inputs.get(key):
            raise ValueError(f"retry {key}: expected original capture configuration")


def successful_retries(retry: JSONObject, source: pathlib.Path) -> list[JSONObject]:
    rows = object_rows(retry.get("retry_results"), "retry results")
    successes = [
        row for row in rows if text_field(row, "status") == "parsed_with_context"
    ]
    for row in successes:
        verify_retry_source(row, source)
    return successes


def gap_categories(gaps: list[JSONObject]) -> dict[str, int]:
    return dict(sorted(Counter(text_field(gap, "category") for gap in gaps).items()))


def retry_context_counts(successes: list[JSONObject]) -> JSONObject:
    direct = sum(
        "compile_commands" in text_field(row, "context_evidence") for row in successes
    )
    transitive = sum(
        "transitive source include graph" in text_field(row, "context_evidence")
        for row in successes
    )
    return {
        "direct_source_context": direct,
        "transitive_source_context": transitive,
        "total_verified": len(successes),
    }


def report_inputs(
    capture_path: pathlib.Path,
    retry_path: pathlib.Path,
    capture: JSONObject,
    retry: JSONObject,
) -> JSONObject:
    return {
        "capture_sha256": sha256(capture_path),
        "retry_sha256": sha256(retry_path),
        "capture_fingerprint": text_field(capture, "fingerprint"),
        "retry_extractor_sha256": text_field(retry, "extractor_sha256"),
        "retry_database_sha256": text_field(retry, "database_sha256"),
        "cmake_cache_sha256": text_field(
            capture.get("inputs", {}), "cmake_cache_sha256"
        ),
    }


def merged_capture_evidence(
    path: pathlib.Path | None, retry_path: pathlib.Path
) -> JSONObject | None:
    if path is None:
        return None
    manifest = load_json(path)
    retry_hash = sha256(retry_path)
    if manifest.get("context_retry_sha256") != retry_hash:
        raise ValueError("merged manifest: expected this context retry manifest hash")
    counts = manifest.get("headers_by_status")
    if not isinstance(counts, dict):
        raise ValueError("merged manifest headers_by_status: expected object")
    retries = object_rows(load_json(retry_path).get("retry_results"), "retry results")
    passed = sum(text_field(row, "status") == "parsed_with_context" for row in retries)
    open_rows = len(retries) - passed
    if (
        counts.get("parsed_with_context") != passed
        or counts.get("parse_failed") != open_rows
    ):
        raise ValueError(
            "merged manifest: expected counts matching context retry outcomes"
        )
    return {
        "manifest_sha256": sha256(path),
        "context_retry_sha256": retry_hash,
        "headers_by_status": counts,
        "coverage_proof": manifest.get("coverage_proof") is True,
    }


def assemble_report(
    capture_path: pathlib.Path,
    retry_path: pathlib.Path,
    source: pathlib.Path,
    capture: JSONObject,
    retry: JSONObject,
    gaps: list[JSONObject],
    successes: list[JSONObject],
    merged_manifest: JSONObject | None,
) -> JSONObject:
    return {
        "schema": "occt-capture-gap-audit-v1",
        "occt_revision": source_revision(source),
        "configuration_matrix_complete": capture.get("configuration_matrix_complete")
        is True,
        "inputs": report_inputs(capture_path, retry_path, capture, retry),
        "merged_capture": merged_manifest,
        "verified_context_retries": retry_context_counts(successes),
        "verified_context_paths": sorted(text_field(row, "path") for row in successes),
        "unresolved_gap_count": len(gaps),
        "unresolved_by_category": gap_categories(gaps),
        "unresolved_gaps": gaps,
        "coverage_proof": False,
        "declaration_inventory_complete": False,
        "note": "Supplemental retries are reported separately and do not rewrite capture evidence.",
    }


def build_report(
    capture_path: pathlib.Path,
    retry_path: pathlib.Path,
    source: pathlib.Path,
    merged_manifest_path: pathlib.Path | None = None,
) -> JSONObject:
    """Build conservative gap counts and evidence; e.g. `build_report(capture, retry, source)`."""
    capture, retry = load_json(capture_path), load_json(retry_path)
    verify_retry_manifest(capture, retry, capture_path)
    gaps = _capture_gaps(capture, retry)
    successful = successful_retries(retry, source)
    merged_manifest = merged_capture_evidence(merged_manifest_path, retry_path)
    return assemble_report(
        capture_path,
        retry_path,
        source,
        capture,
        retry,
        gaps,
        successful,
        merged_manifest,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=pathlib.Path, required=True)
    parser.add_argument("--retries", type=pathlib.Path, required=True)
    parser.add_argument("--source", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument("--merged-manifest", type=pathlib.Path)
    args = parser.parse_args()
    report = build_report(args.capture, args.retries, args.source, args.merged_manifest)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
