"""Rerun validated OCCT TU contexts into fresh, input-bound artifacts."""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
from dataclasses import dataclass

from .context_replay import (
    CompileContext,
    compile_contexts,
    context_flags_hash,
    digest,
    git_revision,
    inspect_target_artifact,
    object_rows,
    read_object,
    text_field,
)
from .context_replay_plan import capture_inputs, verify_capture_source
from .inputs import JSONObject


@dataclass(frozen=True)
class ReplayTarget:
    row: JSONObject
    original: JSONObject | None
    header: pathlib.Path
    source_file: pathlib.Path
    context: CompileContext


def replay_plan(
    plan_path: pathlib.Path,
    capture_path: pathlib.Path,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
    output: pathlib.Path,
) -> JSONObject:
    plan, capture = read_object(plan_path), read_object(capture_path)
    validate_replay_inputs(plan, capture, source, database, extractor, output)
    results = execute_plan_rows(
        plan, plan_path, capture_path, capture, source, database, extractor, output
    )
    return replay_manifest(
        plan_path, capture_path, database, extractor, capture, results
    )


def execute_plan_rows(
    plan: JSONObject,
    plan_path: pathlib.Path,
    capture_path: pathlib.Path,
    capture: JSONObject,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
    output: pathlib.Path,
) -> list[JSONObject]:
    contexts = compile_contexts(database, source)
    originals = capture_result_index(capture)
    return replay_rows(
        plan,
        digest(plan_path),
        originals,
        capture_path,
        contexts,
        source,
        extractor,
        output,
    )


def capture_result_index(capture: JSONObject) -> dict[str, JSONObject]:
    rows = object_rows(capture.get("results"), "capture results")
    return {text_field(row, "path"): row for row in rows}


def replay_rows(
    plan: JSONObject,
    plan_sha256: str,
    originals: dict[str, JSONObject],
    capture_path: pathlib.Path,
    contexts: dict[str, CompileContext],
    source: pathlib.Path,
    extractor: pathlib.Path,
    output: pathlib.Path,
) -> list[JSONObject]:
    results = []
    for row in object_rows(plan.get("rows"), "plan rows"):
        original = originals.get(text_field(row, "path"))
        if original and original.get("status") == "parsed":
            verify_standalone_parse(original, row, capture_path, source)
            continue
        results.append(
            run_plan_row(
                row, plan, plan_sha256, originals, contexts, source, extractor, output
            )
        )
    return results


def verify_standalone_parse(
    original: JSONObject,
    row: JSONObject,
    capture_path: pathlib.Path,
    source: pathlib.Path,
) -> None:
    if original.get("returncode") != 0:
        raise ValueError(f"{row.get('path')}: parsed capture must have returncode 0")
    artifact_path = pathlib.Path(text_field(original, "artifact"))
    if not artifact_path.is_absolute():
        artifact_path = capture_path.parent / artifact_path
    if digest(artifact_path) != original.get("output_sha256"):
        raise ValueError(f"capture artifact {artifact_path}: expected recorded SHA256")
    header = source / text_field(row, "path")
    declarations, seen = inspect_target_artifact(artifact_path, header)
    if not seen:
        raise ValueError(f"capture artifact {artifact_path}: expected target header")
    if original.get("target_declarations") not in (None, declarations):
        raise ValueError(
            f"capture artifact {artifact_path}: expected recorded declaration count"
        )


def validate_replay_inputs(
    plan: JSONObject,
    capture: JSONObject,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
    output: pathlib.Path,
) -> None:
    verify_plan_source(plan, source)
    verify_capture_binding(plan, capture, source, database, extractor)
    verify_fresh_output(output)


def verify_plan_source(plan: JSONObject, source: pathlib.Path) -> None:
    if text_field(plan, "schema") != "occt-header-context-replay-plan-v1":
        raise ValueError(f"{plan.get('schema')!r}: expected OCCT context replay plan")
    if text_field(plan, "occt_revision") != git_revision(source):
        raise ValueError(
            f"OCCT source {source}: expected plan revision {plan.get('occt_revision')!r}"
        )


def verify_capture_binding(
    plan: JSONObject,
    capture: JSONObject,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
) -> None:
    capture_inputs(capture, database, extractor)
    verify_capture_source(capture, source)
    if digest(database) != plan.get("database_sha256"):
        raise ValueError(
            f"compile database {database}: expected plan hash {plan.get('database_sha256')!r}"
        )


def verify_fresh_output(output: pathlib.Path) -> None:
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError(
            f"output {output}: expected an empty directory for new artifacts"
        )


def run_plan_row(
    row: JSONObject,
    plan: JSONObject,
    plan_sha256: str,
    originals: dict[str, JSONObject],
    contexts: dict[str, CompileContext],
    source: pathlib.Path,
    extractor: pathlib.Path,
    output: pathlib.Path,
) -> JSONObject:
    target = resolve_replay_target(row, originals, contexts, source)
    verify_plan_row(row, target.header, target.source_file, target.context)
    artifact = allocate_artifact(output, replay_key(plan, row, extractor))
    process = execute_extractor(
        extractor, source, target.header, target.context, artifact
    )
    return replay_result(target, process, artifact, plan_sha256)


def resolve_replay_target(
    row: JSONObject,
    originals: dict[str, JSONObject],
    contexts: dict[str, CompileContext],
    source: pathlib.Path,
) -> ReplayTarget:
    header_path, tu_path = text_field(row, "path"), text_field(row, "compile_source")
    context = contexts.get(tu_path)
    if context is None:
        raise ValueError(
            f"compile source {source / tu_path}: expected plan TU in database"
        )
    return ReplayTarget(
        row, originals.get(header_path), source / header_path, source / tu_path, context
    )


def verify_plan_row(
    row: JSONObject,
    header: pathlib.Path,
    source_file: pathlib.Path,
    context: CompileContext,
) -> None:
    verify_planned_source_hashes(row, header, source_file)
    verify_planned_compile_flags(row, source_file, context)


def verify_planned_source_hashes(
    row: JSONObject, header: pathlib.Path, source_file: pathlib.Path
) -> None:
    if digest(header) != row.get("header_sha256"):
        raise ValueError(f"header {header}: expected replay-plan SHA256")
    if digest(source_file) != row.get("compile_source_sha256"):
        raise ValueError(f"compile source {source_file}: expected replay-plan SHA256")


def verify_planned_compile_flags(
    row: JSONObject, source_file: pathlib.Path, context: CompileContext
) -> None:
    if context_flags_hash(context.flags) != row.get("compile_flags_sha256"):
        raise ValueError(f"compile source {source_file}: expected replay-plan flags")
    if context.toolkit != text_field(row, "compile_toolkit"):
        raise ValueError(
            f"compile source {source_file}: expected planned compile toolkit"
        )


def allocate_artifact(output: pathlib.Path, key: str) -> pathlib.Path:
    artifact = output / "artifacts" / f"{key}.jsonl"
    artifact.parent.mkdir(parents=True, exist_ok=True)
    return artifact


def replay_key(plan: JSONObject, row: JSONObject, extractor: pathlib.Path) -> str:
    fields = [
        text_field(plan, "original_fingerprint"),
        text_field(row, "path"),
        text_field(row, "compile_source"),
        digest(extractor),
        text_field(row, "compile_flags_sha256"),
    ]
    return hashlib.sha256(
        json.dumps(fields, separators=(",", ":")).encode()
    ).hexdigest()


def execute_extractor(
    extractor: pathlib.Path,
    source: pathlib.Path,
    header: pathlib.Path,
    context: CompileContext,
    artifact: pathlib.Path,
) -> subprocess.CompletedProcess[str]:
    with artifact.open("x") as stream:
        return subprocess.run(
            extractor_command(extractor, source, header, context),
            cwd=context.directory,
            stdout=stream,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )


def extractor_command(
    extractor: pathlib.Path,
    source: pathlib.Path,
    header: pathlib.Path,
    context: CompileContext,
) -> list[str]:
    return [
        str(extractor),
        "--source-root",
        str(source / "src"),
        "--main-header",
        str(context.source_file),
        "--emit-header",
        str(header),
        "--",
        *context.flags,
    ]


def replay_result(
    target: ReplayTarget,
    process: subprocess.CompletedProcess[str],
    artifact: pathlib.Path,
    plan_sha256: str,
) -> JSONObject:
    count, seen = inspect_target_artifact(artifact, target.header)
    return replay_result_document(target, process, artifact, plan_sha256, count, seen)


def replay_result_document(
    target: ReplayTarget,
    process: subprocess.CompletedProcess[str],
    artifact: pathlib.Path,
    plan_sha256: str,
    declarations: int,
    target_seen: bool,
) -> JSONObject:
    status = (
        "parsed_with_context"
        if process.returncode == 0 and target_seen
        else "parse_failed"
    )
    return {
        **result_status_fields(target, process, status, plan_sha256),
        **compile_context_fields(target),
        **result_artifact_fields(target, artifact, declarations, target_seen),
    }


def result_status_fields(
    target: ReplayTarget,
    process: subprocess.CompletedProcess[str],
    status: str,
    plan_sha256: str,
) -> JSONObject:
    original_status = (
        text_field(target.original, "status")
        if target.original
        else "not_in_capture_results"
    )
    return {
        "path": text_field(target.row, "path"),
        "toolkit": text_field(target.row, "toolkit"),
        "status": status,
        "returncode": process.returncode,
        "stderr": process.stderr,
        "original_status": original_status,
        **plan_reference_fields(plan_sha256),
    }


def plan_reference_fields(plan_sha256: str) -> JSONObject:
    return {
        "context_evidence": "extractor rerun in validated transitive source-TU context",
        "replay_plan_sha256": plan_sha256,
    }


def result_artifact_fields(
    target: ReplayTarget, artifact: pathlib.Path, declarations: int, seen: bool
) -> JSONObject:
    return {
        "include_name": text_field(target.row, "include_name"),
        "target_header_seen": seen,
        "target_declarations": declarations,
        "artifact": str(artifact),
        "output_sha256": digest(artifact),
    }


def compile_context_fields(target: ReplayTarget) -> JSONObject:
    return {
        "compile_source": str(target.source_file),
        "compile_source_sha256": digest(target.source_file),
        "compile_directory": str(target.context.directory),
        "compile_flags": list(target.context.flags),
        "compile_flags_sha256": context_flags_hash(target.context.flags),
        "header_sha256": digest(target.header),
    }


def replay_manifest(
    plan_path: pathlib.Path,
    capture_path: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
    capture: JSONObject,
    results: list[JSONObject],
) -> JSONObject:
    already_parsed = standalone_parsed_rows(capture, plan_path)
    return {
        "schema": "occt-context-retry-v1",
        **retry_input_fields(plan_path, capture_path, database, extractor, capture),
        "retry_results": results,
        "already_parsed": already_parsed,
        "already_parsed_count": len(already_parsed),
        "retry_count": len(results),
        "parsed_with_context": status_count(results, "parsed_with_context"),
        "still_failed": status_count(results, "parse_failed"),
        "coverage_proof": False,
    }


def standalone_parsed_rows(
    capture: JSONObject, plan_path: pathlib.Path
) -> list[JSONObject]:
    plan = read_object(plan_path)
    parsed = {
        text_field(row, "path"): row
        for row in object_rows(capture.get("results"), "capture results")
        if row.get("status") == "parsed"
    }
    return [
        {
            "path": path,
            "status": "already_parsed_in_standalone_capture",
            "capture_artifact": parsed[path].get("artifact"),
            "capture_output_sha256": parsed[path].get("output_sha256"),
        }
        for path in sorted(
            text_field(row, "path")
            for row in object_rows(plan.get("rows"), "plan rows")
            if text_field(row, "path") in parsed
        )
    ]


def retry_input_fields(
    plan_path: pathlib.Path,
    capture_path: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
    capture: JSONObject,
) -> JSONObject:
    return {
        "original_capture": str(capture_path.resolve()),
        "original_capture_sha256": digest(capture_path),
        "original_fingerprint": text_field(capture, "fingerprint"),
        "database_sha256": digest(database),
        "extractor_sha256": digest(extractor),
        "replay_plan_sha256": digest(plan_path),
    }


def status_count(results: list[JSONObject], expected: str) -> int:
    return sum(text_field(row, "status") == expected for row in results)
