#!/usr/bin/env python3
"""Shared driver of the per-toolkit numeric oracles (TKG2d, TKG3d, ...).

A toolkit verifier names its native probe source, the OCCT libraries it links, its
Rust twin and its request corpus; `verify_numeric` builds both probes, runs them on
the corpus, compares every output token with the TKMath comparator (finite,
infinity and signed-zero bits exact, NaN by classification, exception types and
DumpJson text) and writes `report.json` only on success.

Usage from a verifier:
    verify_numeric(NumericOracle('TKG3d', 'tools/oracle/tkg3d_probe.cxx',
                                 ['TKG3d', 'TKG2d', 'TKMath', 'TKernel'],
                                 'rustycad-tkg3d', corpus))
"""
from __future__ import annotations
import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Callable
from porting import ROOT, snapshot
from verify_tkmath_numeric import compare, reject_unknown_operation, run, selected_paths


@dataclass(frozen=True)
class NumericOracle:
    """One toolkit's differential oracle: probe sources, libraries and corpus."""

    toolkit: str
    probe_source: str
    libraries: list[str]
    rust_package: str
    corpus: Callable[[], str]
    # Table generators (tools/generate_*.py) whose output the probes compare; each
    # runs with `--check` first so a drifted table fails before any comparison.
    generators: tuple[str, ...] = ()

    @property
    def evidence(self) -> Path:
        """Directory of the evidence files; `target/oracle-tkg3d/numeric`."""
        return ROOT/'target'/f'oracle-{self.toolkit.lower()}'/'numeric'

    @property
    def rust_probe(self) -> Path:
        """The Rust twin executable (`numeric_probe` example of the toolkit crate)."""
        return ROOT/'target/debug/examples/numeric_probe'


def build_probes(oracle: NumericOracle, build: Path, output: Path) -> Path:
    """Compile the native oracle and the Rust twin; build_probes(oracle, build, out) -> native path."""
    libraries = [next(build.rglob('lib'+name+'.so')) for name in oracle.libraries]
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/oracle.probe_source), *map(str, libraries), '-Wl,-rpath,'+str(libraries[0].parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', oracle.rust_package, '--example', 'numeric_probe'],
        output/'rust-build.log')
    return executable


def check_generators(oracle: NumericOracle, source: Path, output: Path) -> None:
    """Fail when a generated table differs from the OCCT sources; check_generators(oracle, source, out)."""
    for generator in oracle.generators:
        log = output/(Path(generator).stem+'-check.log')
        run([sys.executable, str(ROOT/generator), '--check', '--source', str(source)], log)


def verify_numeric(oracle: NumericOracle) -> None:
    """Build both probes, compare every request, and write evidence only on success."""
    source, build = selected_paths()
    inventory = snapshot(source)
    output = oracle.evidence
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    check_generators(oracle, source, output)
    executable = build_probes(oracle, build, output)
    reject_unknown_operation(executable)
    reject_unknown_operation(oracle.rust_probe)
    requests = oracle.corpus()
    (output/'inputs.txt').write_text(requests)
    native = run([str(executable)], output/'native.txt', requests)
    rust = run([str(oracle.rust_probe)], output/'rust.txt', requests)
    compare(native, rust, requests)
    report = {'revision': inventory['revision'], 'toolkit': oracle.toolkit,
              'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(native.encode()).hexdigest(),
              'comparator': 'finite, infinity and signed-zero bits exact; NaN classification; exception types; DumpJson text',
              'nan_representation_difference_cases': sum(a != b for a, b in zip(native.splitlines(), rust.splitlines()))}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"{oracle.toolkit}: {report['cases_passed']} numeric comparisons (exact finite bits, NaN classification) passed")
