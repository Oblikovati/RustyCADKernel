#!/usr/bin/env python3
"""Compare Martensite shader material records bit-for-bit with pinned OCCT."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from dataclasses import dataclass

from vulkan_build import vulkan_build_environment

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class MaterialRecord:
    """One complete native shader record; MaterialRecord('default', bits)."""

    case: str
    bits: tuple[int, ...]


def expected_cases() -> set[str]:
    """Require every named case, including highlighted variants; expected_cases()."""
    cases = {"default"}
    for rank in range(26):
        for flags in range(8):
            names = [str(bool(flags & (1 << bit))).lower() for bit in (2, 1, 0)]
            key = "-".join([str(rank), *names])
            cases.update((key, key + "-highlight"))
    return cases


def parse_records(output: str) -> list[MaterialRecord]:
    """Validate unique complete records before comparing; parse_records(probe_stdout)."""
    records = []
    for line in output.splitlines():
        value = json.loads(line)
        case, bits = value.get("case"), value.get("bits")
        if not isinstance(case, str) or not isinstance(bits, list) or len(bits) != 56:
            raise ValueError(
                f"received {value!r}; expected case name and 56 float bit patterns"
            )
        if any(type(bit) is not int or not 0 <= bit <= 0xFFFFFFFF for bit in bits):
            raise ValueError(
                f"received {bits!r}; expected unsigned 32-bit float patterns"
            )
        records.append(MaterialRecord(case, tuple(bits)))
    cases = {record.case for record in records}
    if cases != expected_cases() or len(cases) != len(records):
        raise ValueError(
            f"received {len(records)} records/{len(cases)} cases; expected all 417 unique cases"
        )
    return records


def compare_records(native: str, rust: str) -> list[str]:
    """Return exact mismatches without numerical tolerances; compare_records(native, rust)."""
    expected = {record.case: record.bits for record in parse_records(native)}
    actual = {record.case: record.bits for record in parse_records(rust)}
    return [case for case in sorted(expected) if expected[case] != actual[case]]


def run_probe(command: list[str], environment: dict[str, str]) -> str:
    """Run a checked local compiler or probe; run_probe(command, environment)."""
    return subprocess.run(
        command,
        cwd=ROOT,
        env=environment,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    ).stdout


def build_native(build: Path, output: Path, environment: dict[str, str]) -> Path:
    """Link OCCT only into the test oracle; build_native(build, output, environment)."""
    library = build / "lin64/gcc/libi"
    executable = output / "native-material"
    run_probe(
        [
            "c++",
            "-std=c++17",
            "-O2",
            f"-I{build / 'include/opencascade'}",
            str(ROOT / "tools/oracle/martensite_material_reference.cxx"),
            f"-L{library}",
            "-lTKOpenGl",
            "-lTKService",
            "-lTKernel",
            f"-Wl,-rpath,{library}",
            "-o",
            str(executable),
        ],
        environment,
    )
    return executable


def verify(build: Path, output: Path) -> list[str]:
    """Build both probes and retain exact raw records; verify(build, output)."""
    output.mkdir(parents=True, exist_ok=True)
    environment = vulkan_build_environment(ROOT, dict(os.environ))
    native = run_probe([str(build_native(build, output, environment))], environment)
    run_probe(
        [
            "cargo",
            "build",
            "-p",
            "martensite-vulkan",
            "--example",
            "material_probe",
            "--locked",
        ],
        environment,
    )
    executable = Path(environment["CARGO_TARGET_DIR"]) / "debug/examples/material_probe"
    rust = run_probe([str(executable)], environment)
    (output / "native.jsonl").write_text(native)
    (output / "rust.jsonl").write_text(rust)
    return compare_records(native, rust)


def main() -> None:
    """Command-line native comparison; python3 tools/verify_renderer_material.py --help."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--occt-build", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=ROOT / "target/material-proof")
    args = parser.parse_args()
    output = args.output.resolve()
    failures = verify(args.occt_build.resolve(), output)
    hashes = {
        name: hashlib.sha256((output / name).read_bytes()).hexdigest()
        for name in ("native.jsonl", "rust.jsonl")
    }
    report = {
        "cases": 417,
        "float_patterns": 23352,
        "failures": failures,
        "sha256": hashes,
    }
    (output / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))
    raise SystemExit(bool(failures))


if __name__ == "__main__":
    main()
