#!/usr/bin/env python3
"""Retry failed OCCT header probes inside real translation-unit include contexts."""
from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import pathlib
import re
import shlex
import subprocess
from dataclasses import dataclass
from typing import Any

from api_inventory.inputs import compiler_flags, digest


INCLUDE = re.compile(r"^\s*#\s*include\s*([<\"])([^>\"]+)[>\"]", re.MULTILINE)


@dataclass(frozen=True)
class CompileContext:
    source_file: pathlib.Path
    directory: pathlib.Path
    flags: tuple[str, ...]


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=pathlib.Path, required=True)
    parser.add_argument("--database", type=pathlib.Path, required=True)
    parser.add_argument("--extractor", type=pathlib.Path, required=True)
    parser.add_argument("--capture", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--timeout", type=int, default=120)
    return parser.parse_args()


def command_arguments(row: dict[str, Any]) -> list[str]:
    arguments = row.get("arguments")
    if isinstance(arguments, list):
        return [str(value) for value in arguments]
    command = row.get("command")
    if not isinstance(command, str):
        raise ValueError(f"compile command {row.get('file')!r}: expected arguments or command")
    return shlex.split(command)


def compile_contexts(database: pathlib.Path) -> list[CompileContext]:
    rows = json.loads(database.read_text())
    contexts = []
    for row in rows:
        directory = pathlib.Path(row["directory"]).resolve()
        source_file = pathlib.Path(row["file"])
        if not source_file.is_absolute():
            source_file = directory / source_file
        contexts.append(CompileContext(source_file.resolve(), directory,
                                       tuple(compiler_flags(command_arguments(row)))))
    return contexts


def include_directives(source_file: pathlib.Path) -> list[tuple[str, str]]:
    text = source_file.read_text(errors="replace")
    return [(match.group(1), match.group(2)) for match in INCLUDE.finditer(text)]


def include_roots(flags: tuple[str, ...]) -> tuple[list[pathlib.Path], list[pathlib.Path], list[pathlib.Path]]:
    quote: list[pathlib.Path] = []
    normal: list[pathlib.Path] = []
    system: list[pathlib.Path] = []
    groups = {"-iquote": quote, "-I": normal, "-isystem": system, "-idirafter": system}
    index = 0
    while index < len(flags):
        flag = flags[index]
        matched = next((option for option in groups if flag.startswith(option) and flag != option), None)
        if matched:
            groups[matched].append(pathlib.Path(flag[len(matched):]).resolve())
        elif flag in groups and index + 1 < len(flags):
            index += 1
            groups[flag].append(pathlib.Path(flags[index]).resolve())
        index += 1
    return quote, normal, system


def resolves_to_header(
    source_file: pathlib.Path, flags: tuple[str, ...], header: pathlib.Path,
) -> tuple[bool, str | None]:
    quote, normal, system = include_roots(flags)
    target = header.resolve()
    for delimiter, include_name in include_directives(source_file):
        roots = ([source_file.parent, *quote, *normal, *system]
                 if delimiter == '"' else [*normal, *system])
        candidate = next(((root / include_name).resolve() for root in roots
                          if (root / include_name).is_file()), None)
        if candidate == target:
            return True, include_name
    return False, None


def matching_contexts(
    header: pathlib.Path, toolkit: str,
    include_index: dict[pathlib.Path, list[tuple[CompileContext, str]]],
    source_root: pathlib.Path,
) -> list[tuple[CompileContext, str]]:
    matches = []
    for context, include_name in include_index.get(header.resolve(), []):
        try:
            relative = context.source_file.relative_to(source_root)
        except ValueError:
            continue
        if len(relative.parts) > 1 and relative.parts[1] == toolkit:
            matches.append((context, include_name))
    return sorted(matches, key=lambda item: (
        item[0].source_file.stem != header.stem, str(item[0].source_file)
    ))


def index_source_includes(
    contexts: list[CompileContext], source_root: pathlib.Path,
) -> dict[pathlib.Path, list[tuple[CompileContext, str]]]:
    index: dict[pathlib.Path, list[tuple[CompileContext, str]]] = {}
    for context in contexts:
        if context.source_file.suffix not in {".cxx", ".cpp", ".cc"}:
            continue
        for delimiter, include_name in include_directives(context.source_file):
            target = resolve_include(context, delimiter, include_name)
            for source_header in source_headers_reached(target, context, source_root):
                index.setdefault(source_header, []).append((context, include_name))
    return index


def resolve_include(context: CompileContext, delimiter: str, include_name: str) -> pathlib.Path | None:
    return resolve_file_include(context.source_file, context.flags, delimiter, include_name)


def resolve_file_include(
    including_file: pathlib.Path, flags: tuple[str, ...], delimiter: str, include_name: str,
) -> pathlib.Path | None:
    quote, normal, system = include_roots(flags)
    roots = ([including_file.parent, *quote, *normal, *system]
             if delimiter == '"' else [*normal, *system])
    return next(((root / include_name).resolve() for root in roots
                 if (root / include_name).is_file()), None)


def source_headers_reached(
    candidate: pathlib.Path | None, context: CompileContext, source_root: pathlib.Path,
    seen: frozenset[pathlib.Path] = frozenset(),
) -> set[pathlib.Path]:
    if candidate is None or candidate in seen:
        return set()
    if candidate.is_relative_to(source_root):
        return {candidate}
    if candidate.stat().st_size > 8192 or len(seen) >= 3:
        return set()
    next_seen = seen | {candidate}
    return nested_source_headers(candidate, context, source_root, next_seen)


def nested_source_headers(
    wrapper: pathlib.Path, context: CompileContext, source_root: pathlib.Path,
    seen: frozenset[pathlib.Path],
) -> set[pathlib.Path]:
    reached: set[pathlib.Path] = set()
    for delimiter, include_name in include_directives(wrapper):
        nested = resolve_file_include(wrapper, context.flags, delimiter, include_name)
        reached.update(source_headers_reached(nested, context, source_root, seen))
    return reached


def retry_key(
    original_fingerprint: str, header: pathlib.Path, context: CompileContext, extractor: pathlib.Path,
) -> str:
    fields = [original_fingerprint, str(header.resolve()), digest(header),
              str(context.source_file), digest(context.source_file), list(context.flags), digest(extractor)]
    return hashlib.sha256(json.dumps(fields, sort_keys=True).encode()).hexdigest()


def artifact_reference(output: pathlib.Path, manifest_directory: pathlib.Path) -> str:
    """Store retry artifacts relative to their manifest; e.g. `artifacts/abc.jsonl`."""
    return output.resolve().relative_to(manifest_directory.resolve()).as_posix()


def run_probe(
    header: pathlib.Path, context: CompileContext, include_name: str, source_root: pathlib.Path,
    extractor: pathlib.Path, output: pathlib.Path, timeout: int,
) -> dict[str, Any]:
    command = [str(extractor), "--source-root", str(source_root), "--main-header",
               str(context.source_file), "--emit-header", str(header), "--", *context.flags]
    with output.open("w") as stream:
        result = subprocess.run(command, cwd=context.directory, stdout=stream,
                                stderr=subprocess.PIPE, text=True, timeout=timeout)
    target_declarations, target_seen = inspect_target_artifact(output, header)
    return {
        "status": "parsed_with_context" if result.returncode == 0 and target_seen else "parse_failed",
        "returncode": result.returncode, "stderr": result.stderr,
        "compile_source": str(context.source_file), "compile_source_sha256": digest(context.source_file),
        "compile_directory": str(context.directory), "compile_flags": list(context.flags),
        "compile_flags_sha256": hashlib.sha256(json.dumps(context.flags).encode()).hexdigest(),
        "include_name": include_name, "target_header_seen": target_seen,
        "target_declarations": target_declarations,
        "artifact": artifact_reference(output, output.parent.parent),
        "output_sha256": digest(output),
    }


def inspect_target_artifact(output: pathlib.Path, header: pathlib.Path) -> tuple[int, bool]:
    declaration_count = 0
    target_seen = False
    target = str(header.resolve())
    for line in output.read_text().splitlines():
        row = json.loads(line)
        if row.get("event") == "header_seen" and pathlib.Path(row.get("source", "")).resolve() == header.resolve():
            target_seen = True
        if row.get("event") == "declaration" and row.get("source") == target:
            declaration_count += 1
    return declaration_count, target_seen


def retry_one(
    failure: dict[str, Any], manifest: dict[str, Any], source_root: pathlib.Path,
    include_index: dict[pathlib.Path, list[tuple[CompileContext, str]]],
    extractor: pathlib.Path, output_root: pathlib.Path, timeout: int,
) -> dict[str, Any]:
    header = source_root.parent / failure["path"]
    options = matching_contexts(header, failure["toolkit"], include_index, source_root)
    if not options:
        return {"path": failure["path"], "status": "no_source_include_context"}
    context, include_name = options[0]
    key = retry_key(manifest["fingerprint"], header, context, extractor)
    output = output_root / "artifacts" / f"{key}.jsonl"
    output.parent.mkdir(parents=True, exist_ok=True)
    record = run_probe(header, context, include_name, source_root, extractor, output, timeout)
    record.update({"path": failure["path"], "toolkit": failure["toolkit"],
                   "original_status": failure["status"], "original_artifact": failure["artifact"],
                   "header_sha256": digest(header),
                   "context_evidence": "compile_commands source include resolves through configured headers to this OCCT file"})
    return record


def run_capture(args: argparse.Namespace) -> None:
    source_root = args.source.resolve() / "src"
    original = json.loads(args.capture.read_text())
    contexts = compile_contexts(args.database)
    include_index = index_source_includes(contexts, source_root)
    failures = [row for row in original["results"] if row["status"] == "parse_failed"]
    args.output.mkdir(parents=True, exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = [pool.submit(retry_one, row, original, source_root, include_index,
                               args.extractor.resolve(), args.output, args.timeout)
                   for row in failures]
        results = [future.result() for future in concurrent.futures.as_completed(futures)]
    results.sort(key=lambda row: row["path"])
    record = retry_manifest(args, original, results)
    destination = args.output / "context-retry-manifest.json"
    destination.write_text(json.dumps(record, indent=2) + "\n")
    print_retry_summary(record)


def retry_manifest(
    args: argparse.Namespace, original: dict[str, Any], results: list[dict[str, Any]],
) -> dict[str, Any]:
    record = {"schema": "occt-context-retry-v1", "artifact_path_base": "manifest_directory",
              "original_capture": str(args.capture.resolve()),
              "original_capture_sha256": digest(args.capture), "original_fingerprint": original["fingerprint"],
              "database_sha256": digest(args.database), "extractor_sha256": digest(args.extractor),
              "retry_results": results, "retry_count": len(results),
              "parsed_with_context": sum(row["status"] == "parsed_with_context" for row in results),
              "still_failed": sum(row["status"] == "parse_failed" for row in results),
              "no_source_include_context": sum(row["status"] == "no_source_include_context" for row in results),
              "coverage_proof": False}
    return record


def print_retry_summary(record: dict[str, Any]) -> None:
    print(f"Context retries: {record['retry_count']}; parsed: {record['parsed_with_context']}; "
          f"failed: {record['still_failed']}; no source context: {record['no_source_include_context']}")


if __name__ == "__main__":
    run_capture(parse_arguments())
