#!/usr/bin/env python3
"""Capture explicit OCCT declarations using the pinned Clang AST extractor."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
from dataclasses import dataclass

from api_inventory.capture import CaptureContext, run_header_jobs
from api_inventory.dependencies import (
    compiler_library_fingerprints,
    external_include_fingerprints,
    sdk_fingerprints,
)
from api_inventory.inputs import (
    HeaderJob,
    JSONObject,
    digest,
    read_object,
    select_header_jobs,
    toolkit_commands,
    verify_sources,
)

ROOT = pathlib.Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "porting/api-header-manifest.json"


@dataclass
class CapturePlan:
    manifest: JSONObject
    inputs: JSONObject
    jobs: list[HeaderJob]
    coverage: list[JSONObject]
    context: CaptureContext


def arguments() -> argparse.Namespace:
    """Read capture inputs; e.g. `capture_occt_inventory.py --help`."""
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "database", "extractor"):
        parser.add_argument(f"--{name}", type=pathlib.Path, required=True)
    parser.add_argument(
        "--output", type=pathlib.Path, default=ROOT / "target/api-proof/occt-inventory"
    )
    parser.add_argument(
        "--llvm-root",
        type=pathlib.Path,
        default=pathlib.Path.home() / ".cache/rustycad-api-proof/llvm-23.1.2",
    )
    for name, default in (("jobs", 8), ("timeout", 120), ("limit", 0)):
        parser.add_argument(f"--{name}", type=int, default=default)
    parsed = parser.parse_args()
    if parsed.jobs < 1 or parsed.timeout < 1 or parsed.limit < 0:
        parser.error("jobs and timeout must be positive; limit must be nonnegative")
    return parsed


def base_inputs(args: argparse.Namespace) -> JSONObject:
    """Bind the build database, generated header and source inventory."""
    generated = args.database.parent / "include/opencascade/Standard_Version.hxx"
    paths = {
        "manifest": MANIFEST,
        "database": args.database,
        "extractor": args.extractor,
        "toolchain_config": ROOT / "porting/api-proof-toolchain.json",
        "cmake_cache": args.database.parent / "CMakeCache.txt",
        "generated_version": generated,
    }
    inputs: JSONObject = {f"{key}_sha256": digest(path) for key, path in paths.items()}
    inputs["file_inputs"] = {key: str(path.resolve()) for key, path in paths.items()}
    inputs["source"] = str(args.source.resolve())
    inputs["generated_version_path"] = str(generated.resolve())
    return inputs


def capture_inputs(args: argparse.Namespace) -> JSONObject:
    """Bind evidence to configuration and tools; e.g. `capture_inputs(args)`."""
    inputs = base_inputs(args)
    inputs["sdk_headers"] = sdk_fingerprints(args.llvm_root)
    inputs["compiler_libraries"] = compiler_library_fingerprints(args.llvm_root)
    inputs["external_headers"] = external_include_fingerprints(
        args.database, args.source.resolve()
    )
    scripts = sorted((ROOT / "tools/api_inventory").glob("*.py")) + [
        pathlib.Path(__file__)
    ]
    inputs["collector_sha256"] = hashlib.sha256(
        "".join(digest(path) for path in scripts).encode()
    ).hexdigest()
    return inputs


def add_generated_header(
    jobs: list[HeaderJob], coverage: list[JSONObject], args: argparse.Namespace
) -> None:
    """Probe the configured version header, retaining its distinct generated identity."""
    native = args.database.parent / "include/opencascade/Standard_Version.hxx"
    template = next(job for job in jobs if job.toolkit == "TKernel")
    identity = "@generated/Standard_Version.hxx"
    jobs.append(
        HeaderJob(identity, "TKernel", template.directory, template.flags, str(native))
    )
    owner: JSONObject = {
        "module": "FoundationClasses",
        "toolkit": "TKernel",
        "package": "Standard",
    }
    coverage.append({"path": identity, "status": "pending", "owners": [owner]})


def make_capture_plan(args: argparse.Namespace) -> CapturePlan:
    """Validate immutable inputs before starting compiler processes."""
    manifest = read_object(MANIFEST)
    verify_sources(args.source.resolve(), manifest)
    inputs = capture_inputs(args)
    fingerprint = hashlib.sha256(
        json.dumps(inputs, sort_keys=True).encode()
    ).hexdigest()
    commands = toolkit_commands(args.database, args.source.resolve())
    jobs, coverage = select_header_jobs(manifest, commands)
    add_generated_header(jobs, coverage, args)
    context = CaptureContext(
        args.source.resolve(),
        args.extractor.resolve(),
        args.output.resolve(),
        fingerprint,
        args.timeout,
    )
    return CapturePlan(manifest, inputs, jobs, coverage, context)


def capture_record(plan: CapturePlan, results: list[JSONObject]) -> JSONObject:
    """Record unfinished headers alongside successful jobs."""
    return {
        "schema": "occt-header-capture-v1",
        "inputs": plan.inputs,
        "fingerprint": plan.context.fingerprint,
        "planned_headers": len(plan.jobs),
        "captured_headers": len(results),
        "header_candidates": plan.coverage,
        "results": results,
        "configuration_matrix_complete": False,
        "coverage_proof": False,
    }


def run_capture(args: argparse.Namespace) -> None:
    """Collect headers without converting parser failures into exclusions."""
    plan = make_capture_plan(args)
    selected = plan.jobs[: args.limit] if args.limit else plan.jobs
    results = run_header_jobs(selected, plan.context, args.jobs)
    verify_sources(args.source.resolve(), plan.manifest)
    if plan.inputs != capture_inputs(args):
        raise ValueError(
            "capture inputs changed: expected identical source/compiler/configuration fingerprints"
        )
    record = capture_record(plan, results)
    destination = plan.context.output / "capture-manifest.json"
    destination.write_text(json.dumps(record, indent=2) + "\n")


if __name__ == "__main__":
    run_capture(arguments())
