"""Create portable plans from previously validated OCCT context probes."""

from __future__ import annotations
import json
import pathlib
from .context_replay import (
    CompileContext,
    compile_contexts,
    context_flags_hash,
    git_revision,
    object_field,
    read_object,
    target_declaration_count,
)
from .context_replay import object_rows, text_field
from .inputs import JSONObject, digest


def capture_inputs(
    capture: JSONObject, database: pathlib.Path, extractor: pathlib.Path
) -> JSONObject:
    inputs = object_field(capture, "inputs")
    expected = {
        "database_sha256": digest(database),
        "extractor_sha256": digest(extractor),
    }
    for name, actual in expected.items():
        if inputs.get(name) != actual:
            raise ValueError(
                f"capture {name}: expected {actual}, got {inputs.get(name)!r}"
            )
    return inputs


def make_plan(
    retry_path: pathlib.Path,
    capture_path: pathlib.Path,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
) -> JSONObject:
    retry, capture = read_object(retry_path), read_object(capture_path)
    verify_retry_parent(
        retry, capture, retry_path, capture_path, source, database, extractor
    )
    contexts = compile_contexts(database, source)
    rows = planned_retry_rows(retry, source, contexts)
    return plan_document(
        retry_path, capture_path, source, database, extractor, capture, rows
    )


def plan_document(
    retry_path: pathlib.Path,
    capture_path: pathlib.Path,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
    capture: JSONObject,
    rows: list[JSONObject],
) -> JSONObject:
    if not rows:
        raise ValueError(f"{retry_path}: expected a verified transitive target context")
    return {
        **plan_identity(capture_path, source, database, extractor, capture, retry_path),
        "rows": sorted(rows, key=lambda row: text_field(row, "path")),
        "coverage_proof": False,
    }


def plan_identity(
    capture_path: pathlib.Path,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
    capture: JSONObject,
    retry_path: pathlib.Path,
) -> JSONObject:
    return {
        "schema": "occt-header-context-replay-plan-v1",
        "occt_revision": git_revision(source),
        "original_capture_sha256": digest(capture_path),
        "original_fingerprint": text_field(capture, "fingerprint"),
        "database_sha256": digest(database),
        "original_extractor_sha256": digest(extractor),
        "plan_source_retry_sha256": digest(retry_path),
    }


def planned_retry_rows(
    retry: JSONObject, source: pathlib.Path, contexts: dict[str, CompileContext]
) -> list[JSONObject]:
    rows = object_rows(retry.get("retry_results"), "retry results")
    selected = [
        row
        for row in rows
        if row.get("status") in {"parsed_with_context", "parse_failed"}
        and row.get("target_header_seen") is True
        and "transitive source include graph" in str(row.get("context_evidence", ""))
    ]
    return [make_plan_row(row, source, contexts) for row in selected]


def verify_retry_parent(
    retry: JSONObject,
    capture: JSONObject,
    retry_path: pathlib.Path,
    capture_path: pathlib.Path,
    source: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
) -> None:
    verify_retry_capture_link(retry, capture, retry_path, capture_path)
    verify_capture_source(capture, source)
    verify_capture_tool_inputs(
        retry, object_field(capture, "inputs"), retry_path, database, extractor
    )


def verify_retry_capture_link(
    retry: JSONObject,
    capture: JSONObject,
    retry_path: pathlib.Path,
    capture_path: pathlib.Path,
) -> None:
    if retry.get("original_capture_sha256") != digest(capture_path):
        raise ValueError(f"{retry_path}: expected supplied original capture hash")
    if retry.get("original_fingerprint") != capture.get("fingerprint"):
        raise ValueError(f"{retry_path}: expected original capture fingerprint")


def verify_capture_tool_inputs(
    retry: JSONObject,
    inputs: JSONObject,
    retry_path: pathlib.Path,
    database: pathlib.Path,
    extractor: pathlib.Path,
) -> None:
    for key, actual in (
        ("database_sha256", digest(database)),
        ("extractor_sha256", digest(extractor)),
    ):
        if inputs.get(key) != actual or retry.get(key) != actual:
            raise ValueError(
                f"{retry_path} {key}: expected current capture input {actual}"
            )


def make_plan_row(
    row: JSONObject, source: pathlib.Path, contexts: dict[str, CompileContext]
) -> JSONObject:
    header = source / text_field(row, "path")
    source_file = pathlib.Path(text_field(row, "compile_source")).resolve()
    relative_tu = source_file.relative_to(source).as_posix()
    context = contexts.get(relative_tu)
    if context is None:
        raise ValueError(
            f"compile source {relative_tu}: expected a compilation database entry"
        )
    verify_context_row(row, header, source_file, context)
    artifact = pathlib.Path(text_field(row, "artifact"))
    declarations = verify_prior_artifact(row, artifact, header)
    return plan_row_document(
        row, source, source_file, header, context, artifact, declarations
    )


def plan_row_document(
    row: JSONObject,
    source: pathlib.Path,
    source_file: pathlib.Path,
    header: pathlib.Path,
    context: CompileContext,
    artifact: pathlib.Path,
    declarations: int,
) -> JSONObject:
    return {
        "path": text_field(row, "path"),
        "toolkit": text_field(row, "toolkit"),
        "compile_toolkit": context.toolkit,
        "compile_source": source_file.relative_to(source).as_posix(),
        **source_hash_fields(source_file, header, context),
        "include_name": text_field(row, "include_name"),
        "prior_artifact_sha256": digest(artifact),
        "prior_target_declarations": declarations,
        "prior_status": text_field(row, "status"),
        "prior_returncode": row.get("returncode"),
        "prior_target_header_seen": row.get("target_header_seen"),
        "context_kind": "transitive_source_include",
    }


def source_hash_fields(
    source_file: pathlib.Path, header: pathlib.Path, context: CompileContext
) -> JSONObject:
    return {
        "compile_source_sha256": digest(source_file),
        "header_sha256": digest(header),
        "compile_flags_sha256": context_flags_hash(context.flags),
    }


def verify_prior_artifact(
    row: JSONObject, artifact: pathlib.Path, header: pathlib.Path
) -> int:
    if digest(artifact) != row.get("output_sha256"):
        raise ValueError(f"retry artifact {artifact}: expected recorded output hash")
    declarations = target_declaration_count(artifact, header)
    if row.get("target_header_seen") is not True or declarations != row.get(
        "target_declarations"
    ):
        raise ValueError(
            f"retry artifact {artifact}: expected observed target and declaration count"
        )
    return declarations


def verify_context_row(
    row: JSONObject,
    header: pathlib.Path,
    source_file: pathlib.Path,
    context: CompileContext,
) -> None:
    if context.source_file != source_file or digest(header) != row.get("header_sha256"):
        raise ValueError(f"context {header}: expected pinned header and TU hashes")
    if digest(source_file) != row.get("compile_source_sha256"):
        raise ValueError(f"compile source {source_file}: expected recorded source hash")
    if context_flags_hash(context.flags) != row.get("compile_flags_sha256"):
        raise ValueError(
            f"compile source {source_file}: expected recorded compiler flags"
        )


def verify_capture_source(capture: JSONObject, source: pathlib.Path) -> None:
    inputs = object_field(capture, "inputs")
    files = object_field(inputs, "file_inputs")
    manifest_path = pathlib.Path(text_field(files, "manifest"))
    if digest(manifest_path) != inputs.get("manifest_sha256"):
        raise ValueError(
            f"header manifest {manifest_path}: expected capture input hash"
        )
    revision = text_field(read_object(manifest_path), "revision")
    if revision != git_revision(source):
        raise ValueError(
            f"header manifest {manifest_path}: expected current OCCT revision"
        )
