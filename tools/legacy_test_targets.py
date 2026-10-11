#!/usr/bin/env python3
"""Check that every Rust test target named by a legacy GTest mapping holds only mapped tests.

The toolkit verifiers require a mapped target to run exactly its mapped tests, so a target that
mixes upstream translations with port-only behaviour tests fails the gate (for example
rustycad-tkgeomalgo/geom_fill_filling_behavior once it received an upstream test). This check
finds every such target across all toolkits in one pass, before a verifier run.
Usage: `python3 tools/legacy_test_targets.py [--toolkit TKBRep]`.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from api_inventory.inputs import JSONObject
from legacy_pending_dependency import runnable_rows
from tkernel_mapping import mapped_runtime_sets
from test_target_selection import compiled_case_target, original_case_names

ROOT = Path(__file__).resolve().parents[1]
TargetKey = tuple[str, str]


class RustTestLister(Protocol):
    """Lists the test names that one Rust test target compiles to."""

    def list_tests(self, targets: list[TargetKey]) -> dict[TargetKey, set[str]]:
        """Map each (package, target) to its test names; lister.list_tests([('p', 't')])."""
        ...


@dataclass(frozen=True)
class TargetMismatch:
    """A mapped target whose compiled tests differ from its mapped tests."""

    package: str
    target: str
    unmapped: tuple[str, ...]
    missing: tuple[str, ...]


def mapped_target_tests(rows: list[JSONObject]) -> dict[TargetKey, set[str]]:
    """Mapped Rust test names per target, pending rows excluded; mapped_target_tests(rows)."""
    _, rust = mapped_runtime_sets(runnable_rows(rows))
    targets: dict[TargetKey, set[str]] = {}
    for package, target, name in rust:
        targets.setdefault((package, target), set()).add(name)
    return targets


def target_mismatches(mapped: dict[TargetKey, set[str]], lister: RustTestLister) -> list[TargetMismatch]:
    """Every target whose listed tests differ from the mapped ones; target_mismatches(m, lister)."""
    listed = lister.list_tests(sorted(mapped))
    mismatches: list[TargetMismatch] = []
    for key in sorted(mapped):
        compiled = listed.get(key)
        if compiled is None:
            raise ValueError(f"No test listing for {key!r}; expected one per mapped target")
        if compiled != mapped[key]:
            unmapped = tuple(sorted(compiled - mapped[key]))
            mismatches.append(TargetMismatch(*key, unmapped, tuple(sorted(mapped[key] - compiled))))
    return mismatches


def terse_test_names(listing: str) -> set[str]:
    """Test names of a libtest `--list --format=terse` output; terse_test_names('a: test\\n') == {'a'}."""
    return {line[: -len(": test")] for line in listing.splitlines() if line.endswith(": test")}


def built_executables(messages: str) -> dict[str, Path]:
    """Test executables by target name from cargo's JSON messages; built_executables(stdout)."""
    artifacts = [json.loads(line) for line in messages.splitlines() if line.startswith("{")]
    return {
        artifact["target"]["name"]: Path(artifact["executable"])
        for artifact in artifacts
        if artifact.get("reason") == "compiler-artifact"
        and artifact.get("executable")
        and "test" in artifact["target"]["kind"]
    }


class CargoTestLister:
    """Builds the test targets with cargo and lists each executable's tests."""

    def __init__(self, root: Path, environment: dict[str, str]) -> None:
        self.root = root
        self.environment = environment

    def list_tests(self, targets: list[TargetKey]) -> dict[TargetKey, set[str]]:
        """Build each package's mapped targets once, then list them; lister.list_tests(keys)."""
        listed: dict[TargetKey, set[str]] = {}
        for package in sorted({package for package, _ in targets}):
            names = [target for owner, target in targets if owner == package]
            compiled = {target: compiled_case_target(self.root, package, target) for target in names}
            binaries = self.build(package, sorted(set(compiled.values())))
            cases = {binary: terse_test_names(self.output([str(executable), '--list', '--format=terse']))
                     for binary, executable in binaries.items()}
            for target, binary in compiled.items():
                selected = cases[binary] if target == binary else {
                    name for name in cases[binary] if name.startswith(f'{target}::')}
                listed[(package, target)] = original_case_names(selected, target)
        return listed

    def build(self, package: str, targets: list[str]) -> dict[str, Path]:
        """Compile the named integration tests of one package; lister.build('p', ['t'])."""
        command = ["cargo", "test", "--locked", "--no-run", "--message-format=json", "-p", package]
        for target in targets:
            command += ["--test", target]
        return built_executables(self.output(command))

    def output(self, command: list[str]) -> str:
        """Run a command in the repository; raise with its tail when it fails."""
        result = subprocess.run(command, cwd=self.root, env=self.environment, capture_output=True, text=True)
        if result.returncode:
            raise RuntimeError(f"{' '.join(command)} exited {result.returncode}: {result.stderr[-2000:]}")
        return result.stdout


def selected_ledger_rows(ledger: Path, toolkits: list[str]) -> list[JSONObject]:
    """Ledger test rows, limited to `toolkits` when given; selected_ledger_rows(path, ['TKBRep'])."""
    rows: list[JSONObject] = json.loads(ledger.read_text())["tests"]
    return [row for row in rows if not toolkits or row["toolkit"] in toolkits]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--toolkit", action="append", default=[], help="Limit the check to this toolkit")
    parser.add_argument("--ledger", type=Path, default=ROOT / "porting/parity.json")
    args = parser.parse_args()
    environment = dict(os.environ)
    environment["PATH"] = str(Path.home() / ".cargo/bin") + os.pathsep + environment["PATH"]
    mapped = mapped_target_tests(selected_ledger_rows(args.ledger, args.toolkit))
    mismatches = target_mismatches(mapped, CargoTestLister(ROOT, environment))
    for mismatch in mismatches:
        print(f"{mismatch.package}/{mismatch.target}: unmapped={list(mismatch.unmapped)} missing={list(mismatch.missing)}")
    print(f"legacy test targets: {len(mapped) - len(mismatches)}/{len(mapped)} hold exactly their mapped tests")
    return 1 if mismatches else 0


if __name__ == "__main__":
    sys.exit(main())
