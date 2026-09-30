#!/usr/bin/env python3
"""Line coverage of the unit tests plus the checked differential probe corpora.

`cargo coverage` counts only `cargo test` runs. Most TKG3d and TKGeomBase code is
exercised by the numeric probes instead, which the verifiers compare with native OCCT.
This tool runs the tests under coverage, then each Rust probe over the corpus that its
verifier last used (in parallel shards, one process per core), and fails unless the probe output still equals the saved native
output (with the verifiers' own comparator). Covered probe lines are therefore always
lines whose results were checked against OCCT. Run the verifiers first: they write the
corpus and the native output this tool replays.

Usage: python3 tools/coverage_report.py [--minimum 80]
"""
from __future__ import annotations
import argparse
import os
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from porting import ROOT
from probe_shards import SHARD_COUNT, ProbeCommand, ShardRunner, run_shard, run_sharded
from verify_tkmath_numeric import compare

Runner = Callable[[list[str], str | None], str]


@dataclass(frozen=True)
class ProbeCorpus:
    """One Rust probe with the corpus and native output its verifier saved."""
    package: str
    example: str
    evidence: str
    native_name: str = 'native.txt'
    # TKernel's verifier requires identical text; the numeric verifiers compare NaN by class.
    exact: bool = False
    # The TKernel probe creates fixed-name files and directories (OSD requests), so two
    # processes collide; its verifier runs it as one process too.
    shard_count: int = SHARD_COUNT

    def inputs(self) -> Path:
        """The saved request corpus; corpus.inputs()."""
        return ROOT/self.evidence/'inputs.txt'

    def native(self) -> Path:
        """The saved native OCCT output; corpus.native()."""
        return ROOT/self.evidence/self.native_name


PROBE_CORPORA: tuple[ProbeCorpus, ...] = (
    ProbeCorpus('rustycad-tkernel', 'oracle_probe', 'target/oracle-tkernel', 'occt-output.txt', True, 1),
    ProbeCorpus('rustycad-tkmath', 'numeric_probe', 'target/oracle-tkmath/numeric'),
    ProbeCorpus('rustycad-tkg2d', 'numeric_probe', 'target/oracle-tkg2d/numeric'),
    ProbeCorpus('rustycad-tkg3d', 'numeric_probe', 'target/oracle-tkg3d/numeric'),
    ProbeCorpus('rustycad-tkgeombase', 'numeric_probe', 'target/oracle-tkgeombase/numeric'),
    ProbeCorpus('rustycad-tkbrep', 'numeric_probe', 'target/oracle-tkbrep/numeric'),
    ProbeCorpus('rustycad-tkgeomalgo', 'numeric_probe', 'target/oracle-tkgeomalgo/numeric'),
    ProbeCorpus('rustycad-tktopalgo', 'numeric_probe', 'target/oracle-tktopalgo/numeric'),
    ProbeCorpus('rustycad-tkhelix', 'numeric_probe', 'target/oracle-tkhelix/numeric'),
    ProbeCorpus('rustycad-tkshhealing', 'numeric_probe', 'target/oracle-tkshhealing/numeric'),
    ProbeCorpus('rustycad-tkprim', 'numeric_probe', 'target/oracle-tkprim/numeric'),
    ProbeCorpus('rustycad-tkhlr', 'numeric_probe', 'target/oracle-tkhlr/numeric'),
)


def verifier_environment() -> dict[str, str]:
    """The environment the verifiers run probes in (C locale, pinned cargo first), so a
    replay sees the same locale variables the native output was produced under;
    verifier_environment()['LC_ALL'] == 'C'."""
    environment = dict(os.environ, LC_ALL='C', LANG='C')
    environment['PATH'] = str(Path.home()/'.cargo/bin') + os.pathsep + environment['PATH']
    return environment


def run_command(args: list[str], stdin: str | None) -> str:
    """Run a command in the repository and fail with its stderr; run_command(['ls'], None)."""
    result = subprocess.run(args, cwd=ROOT, input=stdin, text=True, capture_output=True,
                            env=verifier_environment())
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}): {' '.join(args)}\n{result.stderr[-2000:]}")
    return result.stdout


def profile_pattern() -> str:
    """Where a probe process writes its raw profile, so `cargo llvm-cov report` merges it.

    The pattern is the one `cargo llvm-cov run` (cargo-llvm-cov 0.9) gives its child:
    `<target>/llvm-cov-target/<workspace directory>-%p-%m.profraw`; %p keeps the files
    of parallel shards apart. profile_pattern().endswith('-%p-%m.profraw')
    """
    return str(ROOT/'target/llvm-cov-target'/f'{ROOT.name}-%p-%m.profraw')


def build_probe(corpus: ProbeCorpus, run: Runner) -> Path:
    """Build the instrumented probe of one corpus and return its path.

    `cargo llvm-cov run` builds the example with the coverage flags; the empty input
    makes the run itself a no-op. `--no-report` also skips the clean step, so the test
    profiles stay (cargo-llvm-cov refuses an explicit `--no-clean` beside it).
    build_probe(PROBE_CORPORA[1], run_command)
    """
    run(['cargo', 'llvm-cov', 'run', '--no-report', '--locked', '-p', corpus.package,
         '--example', corpus.example], '')
    return ROOT/'target/llvm-cov-target/debug/examples'/corpus.example


def replay_probe(corpus: ProbeCorpus, run: Runner, shard_runner: ShardRunner = run_shard) -> int:
    """Run one probe under coverage in parallel shards and check it against the saved native output.

    Returns the number of requests; replay_probe(PROBE_CORPORA[1], run_command).
    """
    for path in (corpus.inputs(), corpus.native()):
        if not path.is_file():
            raise FileNotFoundError(f'{path} is missing; run the {corpus.package} verifier first')
    requests: str = corpus.inputs().read_text()
    environment: dict[str, str] = dict(verifier_environment(), LLVM_PROFILE_FILE=profile_pattern())
    command = ProbeCommand([str(build_probe(corpus, run))], environment, ROOT)
    rust: str = run_sharded(command, requests, runner=shard_runner, shard_count=corpus.shard_count).stdout
    check_output(corpus, corpus.native().read_text(), rust, requests)
    return len(requests.splitlines())


def check_output(corpus: ProbeCorpus, native: str, rust: str, requests: str) -> None:
    """Apply the corpus verifier's own comparison; check_output(corpus, occt, rust, reqs)."""
    if not corpus.exact:
        compare(native, rust, requests)
        return
    if native == rust:
        return
    for request, expected, actual in zip(requests.splitlines(), native.splitlines(), rust.splitlines()):
        if expected != actual:
            raise RuntimeError(f'{corpus.package} mismatch {request!r}: OCCT {expected}; Rust {actual}')
    raise RuntimeError(f'{corpus.package}: output line counts differ from the native output')


def crate_totals(lcov: str) -> dict[str, tuple[int, int]]:
    """Sum found and hit lines per crate of an lcov report; crate_totals(text)['tkmath']."""
    totals: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    crate: str = 'other'
    for line in lcov.splitlines():
        if line.startswith('SF:'):
            path: str = line[3:]
            crate = path.split('/crates/')[1].split('/')[0] if '/crates/' in path else 'other'
        elif line.startswith('LF:'):
            totals[crate][0] += int(line[3:])
        elif line.startswith('LH:'):
            totals[crate][1] += int(line[3:])
    return {name: (found, hit) for name, (found, hit) in totals.items()}


def percent(found: int, hit: int) -> float:
    """Covered share of the found lines, 100 when there are none; percent(4, 3) == 75.0."""
    return 100.0 if found == 0 else 100.0 * hit / found


def main() -> None:
    """Measure, print per-crate coverage, and fail below the minimum."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--minimum', type=float, default=80.0)
    args = parser.parse_args()
    run_command(['cargo', 'llvm-cov', 'clean', '--workspace'], None)
    run_command(['cargo', 'llvm-cov', '--no-report', '--workspace', '--all-features', '--locked'], None)
    for corpus in PROBE_CORPORA:
        print(f'{corpus.package}: {replay_probe(corpus, run_command)} probe requests match native OCCT')
    lcov_path: Path = ROOT/'target/lcov.info'
    run_command(['cargo', 'llvm-cov', 'report', '--lcov', '--output-path', str(lcov_path)], None)
    totals = crate_totals(lcov_path.read_text())
    for name, (found, hit) in sorted(totals.items()):
        print(f'  {name}: {percent(found, hit):.1f}% ({hit}/{found})')
    found, hit = sum(f for f, _ in totals.values()), sum(h for _, h in totals.values())
    print(f'total line coverage: {percent(found, hit):.1f}% ({hit}/{found})')
    if percent(found, hit) < args.minimum:
        sys.exit(f'coverage {percent(found, hit):.1f}% is below the {args.minimum}% minimum')


if __name__ == '__main__':
    main()
