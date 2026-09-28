#!/usr/bin/env python3
"""Compare the TKG2d (Geom2d, Geom2dEval, Geom2dHash) port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkg2d_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkg2d_probe.cxx against libTKG2d and the Rust twin
crates/tkg2d/examples/numeric_probe.rs, runs both on tools/tkg2d_corpus.py and
requires identical bits (NaN compared by classification), as the TKMath verifier does.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
from porting import ROOT, snapshot
from tkg2d_corpus import append_tkg2d_requests
from tkg2d_eval_corpus import append_tkg2d_eval_requests
from verify_tkmath_numeric import compare, reject_unknown_operation, run, selected_paths

RUST_PROBE = ROOT/'target/debug/examples/numeric_probe'


def corpus() -> str:
    """The whole TKG2d request corpus, one request per line."""
    requests: list[str] = []
    append_tkg2d_requests(requests)
    append_tkg2d_eval_requests(requests)
    return '\n'.join(requests)+'\n'


def build_probes(build: Path, output: Path) -> Path:
    """Compile the native oracle and the Rust twin; build_probes(build, out) -> native path."""
    libraries = [next(build.rglob('lib'+name+'.so')) for name in ['TKG2d', 'TKMath', 'TKernel']]
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/tkg2d_probe.cxx'), *map(str, libraries), '-Wl,-rpath,'+str(libraries[0].parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkg2d', '--example', 'numeric_probe'], output/'rust-build.log')
    return executable


def main() -> None:
    """Build both probes, compare every request, and write evidence only on success."""
    source, build = selected_paths()
    inventory = snapshot(source)
    output = ROOT/'target/oracle-tkg2d/numeric'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    executable = build_probes(build, output)
    reject_unknown_operation(executable)
    reject_unknown_operation(RUST_PROBE)
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    oracle = run([str(executable)], output/'native.txt', requests)
    rust = run([str(RUST_PROBE)], output/'rust.txt', requests)
    compare(oracle, rust, requests)
    report = {'revision': inventory['revision'], 'toolkit': 'TKG2d', 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(oracle.encode()).hexdigest(),
              'comparator': 'finite, infinity and signed-zero bits exact; NaN classification; exception types; DumpJson text',
              'nan_representation_difference_cases': sum(a != b for a, b in zip(oracle.splitlines(), rust.splitlines()))}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"TKG2d: {report['cases_passed']} numeric comparisons (exact finite bits, NaN classification) passed")


if __name__ == '__main__':
    main()
