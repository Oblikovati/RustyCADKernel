"""Run isolated Clang header jobs, preserving diagnostics and reusable evidence."""

from __future__ import annotations

import concurrent.futures
import hashlib
import json
import pathlib
import subprocess
from dataclasses import dataclass

from .inputs import HeaderJob, JSONObject, digest


@dataclass(frozen=True)
class CaptureContext:
    source: pathlib.Path
    extractor: pathlib.Path
    output: pathlib.Path
    fingerprint: str
    timeout: int


def job_fingerprint(job: HeaderJob, context: CaptureContext) -> str:
    """Bind a job to flags and source snapshot; e.g. `job_fingerprint(job, context)`."""
    encoded = json.dumps(
        [context.fingerprint, job.path, job.input_path, job.directory, job.flags]
    ).encode()
    return hashlib.sha256(encoded).hexdigest()


def job_paths(
    job: HeaderJob, context: CaptureContext
) -> tuple[pathlib.Path, pathlib.Path]:
    """Choose stable cache paths; e.g. `job_paths(job, context)`."""
    stem = job_fingerprint(job, context)
    folder = context.output / "headers"
    folder.mkdir(parents=True, exist_ok=True)
    return folder / f"{stem}.jsonl", folder / f"{stem}.status.json"


def extractor_command(job: HeaderJob, context: CaptureContext) -> list[str]:
    """Use the owning toolkit flags; e.g. `extractor_command(job, context)`."""
    return [
        str(context.extractor),
        "--source-root",
        str(context.source),
        "--main-header",
        job.input_path or str(context.source / job.path),
        "--",
        *job.flags,
    ]


def cached_result(output: pathlib.Path, status: pathlib.Path) -> JSONObject | None:
    """Reuse evidence only if its digest still matches; e.g. `cached_result(raw, status)`."""
    if not status.is_file() or not output.is_file():
        return None
    result: JSONObject = json.loads(status.read_text())
    if result.get("output_sha256") != digest(output):
        return None
    return result


def run_extractor(
    job: HeaderJob, context: CaptureContext, output: pathlib.Path
) -> JSONObject:
    """Keep stdout in an artifact and stderr in the status record."""
    with output.open("w") as stream:
        result = subprocess.run(
            extractor_command(job, context),
            cwd=job.directory,
            stdout=stream,
            stderr=subprocess.PIPE,
            text=True,
            timeout=context.timeout,
        )
    return {
        "returncode": result.returncode,
        "stderr": result.stderr,
        "status": "parsed" if result.returncode == 0 else "parse_failed",
    }


def execute_header(
    job: HeaderJob, context: CaptureContext, output: pathlib.Path
) -> JSONObject:
    """Capture compiler output and failures, including timeouts."""
    try:
        return run_extractor(job, context, output)
    except subprocess.TimeoutExpired as failure:
        stderr = failure.stderr
        if isinstance(stderr, bytes):
            stderr = stderr.decode(errors="replace")
        return {"returncode": None, "stderr": stderr or "", "status": "timeout"}
    except OSError as failure:
        return {
            "returncode": None,
            "stderr": str(failure),
            "status": "execution_failed",
        }


def capture_header(job: HeaderJob, context: CaptureContext) -> JSONObject:
    """Execute or reuse one bound header probe; e.g. `capture_header(job, context)`."""
    output, status = job_paths(job, context)
    previous = cached_result(output, status)
    if previous is not None:
        return previous
    result = execute_header(job, context, output)
    result.update(
        {
            "path": job.path,
            "toolkit": job.toolkit,
            "artifact": str(output.relative_to(context.output)),
            "output_sha256": digest(output),
        }
    )
    status.write_text(json.dumps(result, sort_keys=True) + "\n")
    return result


def run_header_jobs(
    jobs: list[HeaderJob], context: CaptureContext, workers: int
) -> list[JSONObject]:
    """Bound parallelism and report progress; e.g. `run_header_jobs(jobs, context, 8)`."""
    results: list[JSONObject] = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        pending = {pool.submit(capture_header, job, context): job for job in jobs}
        for future in concurrent.futures.as_completed(pending):
            result = future.result()
            results.append(result)
            if len(results) % 100 == 0 or len(results) == len(jobs):
                failed = sum(row["status"] != "parsed" for row in results)
                print(
                    f"Captured {len(results)}/{len(jobs)} headers; {failed} failed probes",
                    flush=True,
                )
    return sorted(results, key=lambda row: str(row["path"]))
