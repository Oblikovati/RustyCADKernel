#!/usr/bin/env python3
"""Compile real OCCT access controls without promoting private type names."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path

from api_inventory.inputs import JSONObject, digest, verify_checkout

CONTROLS = {
    "positive": "",
    "sparse_private_name": "REJECT_SPARSE_PRIVATE_NAME",
    "interface_private_name": "REJECT_INTERFACE_PRIVATE_NAME",
    "protected_type": "REJECT_DIRECT_PROTECTED_TYPE",
    "protected_producer": "REJECT_DIRECT_PROTECTED_PRODUCER",
    "abstract_construction": "REJECT_INFERRED_ABSTRACT_CONSTRUCTION",
    "private_storage": "REJECT_PRIVATE_ITERATOR_STORAGE",
}
EXPECTED_STDOUT = "sparse 3 8 46\ninterface 2\n"


def access_diagnostic(control: str, stderr: str) -> bool:
    """Require the intended rejection category; e.g. `access_diagnostic('protected_type', stderr)`."""
    category = "private"
    if control.startswith("protected_"):
        category = "protected"
    if control == "abstract_construction":
        category = "abstract"
    return category in stderr.lower()


def require_probe_result(
    control: str, result: subprocess.CompletedProcess[str]
) -> None:
    """Reject unrelated compile failures; e.g. `require_probe_result('positive', result)`."""
    if control == "positive" and result.returncode == 0:
        return
    if (
        control != "positive"
        and result.returncode != 0
        and access_diagnostic(control, result.stderr)
    ):
        return
    raise ValueError(
        f"probe {control}: expected intended access outcome, got {result.returncode}: {result.stderr}"
    )


def compile_access_control(
    command: list[str], control: str, output: Path
) -> JSONObject:
    """Record compile and execution evidence; e.g. `compile_access_control(argv, 'positive', executable)`."""
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    require_probe_result(control, result)
    row: JSONObject = {
        "control": control,
        "command": command,
        "exit_code": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "expected_outcome_observed": True,
    }
    if control != "positive":
        return row
    row["execution"] = execute_positive_probe(output)
    return row


def execute_positive_probe(output: Path) -> JSONObject:
    """Check all observable access assertions; e.g. `execute_positive_probe(executable)`."""
    run = subprocess.run([str(output)], capture_output=True, text=True, check=True)
    if run.stdout != EXPECTED_STDOUT:
        raise ValueError(
            f"positive stdout {run.stdout!r}: expected {EXPECTED_STDOUT!r}"
        )
    return {
        "exit_code": run.returncode,
        "stdout": run.stdout,
        "stderr": run.stderr,
    }


def native_probe_report(
    source: Path, fixture: Path, compiler: str, output: Path
) -> JSONObject:
    """Exercise inferred records and six access boundaries; e.g. `native_probe_report(occt, fixture, 'c++', target)`."""
    rows = run_native_controls(source, fixture, compiler, output / "effective-access")
    resolved = Path(shutil.which(compiler) or compiler).resolve()
    return {
        "fixture_sha256": digest(fixture),
        "compiler": str(resolved),
        "compiler_sha256": digest(resolved),
        "compiler_version": subprocess.check_output([compiler, "--version"], text=True),
        "controls": rows,
        "callability_domain": "C++17 Linux x86_64; explicit inferred-type expressions in fixture only",
        "coverage_credit": False,
        "api_denominator_complete": False,
    }


def run_native_controls(
    source: Path, fixture: Path, compiler: str, executable: Path
) -> list[JSONObject]:
    """Compile seven independent controls; e.g. `run_native_controls(occt, fixture, 'c++', executable)`."""
    includes = sorted(
        {
            header.parent
            for header in (source / "src/FoundationClasses/TKernel").rglob("*.hxx")
        }
    )
    base = [compiler, "-std=c++17", "-UNDEBUG", *[f"-I{path}" for path in includes]]
    rows: list[JSONObject] = []
    for control, macro in CONTROLS.items():
        command = (
            base
            + ([f"-D{macro}"] if macro else [])
            + [str(fixture), "-o", str(executable)]
        )
        rows.append(compile_access_control(command, control, executable))
    return rows


def main() -> None:
    """Run pinned source access controls; e.g. `--source ../OCCT --output target/access`."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--compiler", default="c++")
    args = parser.parse_args()
    verify_checkout(args.source, "3d097a0328e71b826377d4814ab05ec3c3d23871")
    args.output.mkdir(parents=True, exist_ok=True)
    fixture = (
        Path(__file__).resolve().parent / "oracle/tkernel_returned_type_obligations.cxx"
    )
    report = native_probe_report(
        args.source.resolve(), fixture, args.compiler, args.output.resolve()
    )
    (args.output / "native-access.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n"
    )
    print(json.dumps({"native_access_controls": len(CONTROLS), "passed": True}))


if __name__ == "__main__":
    main()
