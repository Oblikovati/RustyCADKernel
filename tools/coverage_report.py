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

from coverage_profile_policy import coverage_profile_environment
from coverage_profile_observation import (
    ObservedCoverageProfile,
    observe_compiler_profile,
)
from porting import ROOT
from probe_shards import SHARD_COUNT, ProbeCommand, ShardRunner, run_shard, run_sharded
from verify_tkmath_numeric import compare

Runner = Callable[[list[str], str | None, dict[str, str] | None], str]


@dataclass(frozen=True)
class ProbeCorpus:
    """One Rust probe with the corpus and native output its verifier saved."""

    package: str
    example: str
    evidence: str
    native_name: str = "native.txt"
    # TKernel's verifier requires identical text; the numeric verifiers compare NaN by class.
    exact: bool = False
    # The TKernel probe creates fixed-name files and directories (OSD requests), so two
    # processes collide; its verifier runs it as one process too.
    shard_count: int = SHARD_COUNT

    def inputs(self) -> Path:
        """The saved request corpus.

        Example: corpus.inputs().
        """
        return ROOT / self.evidence / "inputs.txt"

    def native(self) -> Path:
        """The saved native OCCT output.

        Example: corpus.native().
        """
        return ROOT / self.evidence / self.native_name


PROBE_CORPORA: tuple[ProbeCorpus, ...] = (
    ProbeCorpus(
        "rustycad-tkernel",
        "oracle_probe",
        "target/oracle-tkernel",
        "occt-output.txt",
        True,
        1,
    ),
    ProbeCorpus("rustycad-tkmath", "numeric_probe", "target/oracle-tkmath/numeric"),
    ProbeCorpus("rustycad-tkg2d", "numeric_probe", "target/oracle-tkg2d/numeric"),
    ProbeCorpus("rustycad-tkg3d", "numeric_probe", "target/oracle-tkg3d/numeric"),
    ProbeCorpus(
        "rustycad-tkgeombase", "numeric_probe", "target/oracle-tkgeombase/numeric"
    ),
    ProbeCorpus("rustycad-tkbrep", "numeric_probe", "target/oracle-tkbrep/numeric"),
    ProbeCorpus(
        "rustycad-tkgeomalgo", "numeric_probe", "target/oracle-tkgeomalgo/numeric"
    ),
    ProbeCorpus(
        "rustycad-tktopalgo", "numeric_probe", "target/oracle-tktopalgo/numeric"
    ),
    ProbeCorpus("rustycad-tkhelix", "numeric_probe", "target/oracle-tkhelix/numeric"),
    ProbeCorpus(
        "rustycad-tkshhealing", "numeric_probe", "target/oracle-tkshhealing/numeric"
    ),
    ProbeCorpus("rustycad-tkprim", "numeric_probe", "target/oracle-tkprim/numeric"),
    ProbeCorpus("rustycad-tkhlr", "numeric_probe", "target/oracle-tkhlr/numeric"),
    ProbeCorpus("rustycad-tkfillet", "numeric_probe", "target/oracle-tkfillet/numeric"),
    ProbeCorpus(
        "rustycad-tkservice", "numeric_probe", "target/oracle-tkservice/numeric"
    ),
    ProbeCorpus(
        "rustycad-tkexpress", "numeric_probe", "target/oracle-tkexpress/numeric"
    ),
    ProbeCorpus("rustycad-tkbool", "numeric_probe", "target/oracle-tkbool/numeric"),
    ProbeCorpus("rustycad-tkv3d", "numeric_probe", "target/oracle-tkv3d/numeric"),
)


def verifier_environment() -> dict[str, str]:
    """The environment the verifiers run probes in (C locale, pinned cargo first), so a
    replay sees the same locale variables the native output was produced under;
    verifier_environment()['LC_ALL'] == 'C'."""
    environment = dict(os.environ, LC_ALL="C", LANG="C")
    environment["PATH"] = (
        str(Path.home() / ".cargo/bin") + os.pathsep + environment["PATH"]
    )
    return coverage_profile_environment(environment)


def run_command(
    args: list[str], stdin: str | None, environment: dict[str, str] | None = None
) -> str:
    """Run a command in the repository and fail with its stderr; run_command(['ls'], None)."""
    result = subprocess.run(
        args,
        cwd=ROOT,
        input=stdin,
        text=True,
        capture_output=True,
        env=verifier_environment() if environment is None else environment,
    )
    if result.returncode:
        raise RuntimeError(
            f"Command failed ({result.returncode}): {' '.join(args)}\n{result.stderr[-2000:]}"
        )
    return result.stdout


class ProbeCoverageCommands:
    """Forward actual tool reads through the injected existing probe runner."""

    def __init__(self, run: Runner) -> None:
        """Keep the caller-owned subprocess seam.

        Example: ProbeCoverageCommands(run_command).
        """
        self.run = run

    def capture(self, root: Path, argv: list[str], environment: dict[str, str]) -> str:
        """Preserve one captured environment for all subprocesses.

        Example: commands.capture(root, argv, environment).
        """
        return self.run(argv, None, environment)


def profile_pattern(run: Runner = run_command) -> str:
    """Read the actual bounded probe collector; profile_pattern().endswith('-%4m.profraw')."""
    observation = observe_compiler_profile(
        ROOT, verifier_environment(), ProbeCoverageCommands(run)
    )
    return str(observation.profile_path)


def build_probe(corpus: ProbeCorpus, run: Runner) -> Path:
    """Return the actual instrumented example path; build_probe(PROBE_CORPORA[1], run_command).

    Empty input makes the instrumented build run a no-op. --no-report preserves
    the earlier test profiles; cargo-llvm-cov refuses --no-clean alongside it.
    """
    binary, _ = instrumented_probe_configuration(corpus, run)
    return binary


def instrumented_probe_configuration(
    corpus: ProbeCorpus, run: Runner
) -> tuple[Path, ObservedCoverageProfile]:
    """Capture one collector for the build and all shards; instrumented_probe_configuration(corpus, run_command)."""
    observation = observe_compiler_profile(
        ROOT, verifier_environment(), ProbeCoverageCommands(run)
    )
    run(probe_coverage_build_arguments(corpus), "", observation.environment)
    binary = observation.directory / "debug/examples" / corpus.example
    return binary, observation


def probe_coverage_build_arguments(corpus: ProbeCorpus) -> list[str]:
    """Keep existing example instrumentation arguments; probe_coverage_build_arguments(PROBE_CORPORA[1])."""
    return [
        "cargo",
        "llvm-cov",
        "run",
        "--no-report",
        "--locked",
        "-p",
        corpus.package,
        "--example",
        corpus.example,
    ]


def probe_coverage_requests(corpus: ProbeCorpus) -> str:
    """Preserve missing-evidence failure before compilation; probe_coverage_requests(corpus)."""
    for path in (corpus.inputs(), corpus.native()):
        if not path.is_file():
            raise FileNotFoundError(
                f"{path} is missing; run the {corpus.package} verifier first"
            )
    return corpus.inputs().read_text()


def replay_probe(
    corpus: ProbeCorpus, run: Runner, shard_runner: ShardRunner = run_shard
) -> int:
    """Run one probe under coverage in parallel shards and check it against the saved native output.

    Returns the number of requests; replay_probe(PROBE_CORPORA[1], run_command).
    """
    requests = probe_coverage_requests(corpus)
    binary, observation = instrumented_probe_configuration(corpus, run)
    environment = dict(
        observation.environment, LLVM_PROFILE_FILE=str(observation.profile_path)
    )
    command = ProbeCommand([str(binary)], environment, ROOT)
    rust: str = run_sharded(
        command, requests, runner=shard_runner, shard_count=corpus.shard_count
    ).stdout
    check_output(corpus, corpus.native().read_text(), rust, requests)
    return len(requests.splitlines())


def check_output(corpus: ProbeCorpus, native: str, rust: str, requests: str) -> None:
    """Apply the corpus verifier's own comparison; check_output(corpus, occt, rust, reqs)."""
    if not corpus.exact:
        compare(native, rust, requests)
        return
    if native == rust:
        return
    for request, expected, actual in zip(
        requests.splitlines(), native.splitlines(), rust.splitlines()
    ):
        if expected != actual:
            raise RuntimeError(
                f"{corpus.package} mismatch {request!r}: OCCT {expected}; Rust {actual}"
            )
    raise RuntimeError(
        f"{corpus.package}: output line counts differ from the native output"
    )


def crate_totals(lcov: str) -> dict[str, tuple[int, int]]:
    """Sum found and hit lines per crate of an lcov report; crate_totals(text)['tkmath']."""
    totals: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    crate: str = "other"
    for line in lcov.splitlines():
        if line.startswith("SF:"):
            path: str = line[3:]
            crate = (
                path.split("/crates/")[1].split("/")[0]
                if "/crates/" in path
                else "other"
            )
        elif line.startswith("LF:"):
            totals[crate][0] += int(line[3:])
        elif line.startswith("LH:"):
            totals[crate][1] += int(line[3:])
    return {name: (found, hit) for name, (found, hit) in totals.items()}


def percent(found: int, hit: int) -> float:
    """Covered share of the found lines, 100 when there are none.

    Example: percent(4, 3) == 75.0.
    """
    return 100.0 if found == 0 else 100.0 * hit / found


def main() -> None:
    """Measure, print per-crate coverage, and fail below the minimum."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--minimum", type=float, default=80.0)
    args = parser.parse_args()
    lcov_path = run_coverage_corpora(run_command)
    print_coverage_totals(lcov_path.read_text(), args.minimum)


def run_coverage_corpora(run: Runner) -> Path:
    """Accumulate every existing checked corpus; run_coverage_corpora(run_command)."""
    run(["cargo", "llvm-cov", "clean", "--workspace"], None)
    run(workspace_corpus_test_arguments(), None)
    for corpus in PROBE_CORPORA:
        print(
            f"{corpus.package}: {replay_probe(corpus, run)} probe requests match native OCCT"
        )
    lcov_path: Path = ROOT / "target/lcov.info"
    run(
        ["cargo", "llvm-cov", "report", "--lcov", "--output-path", str(lcov_path)], None
    )
    return lcov_path


def workspace_corpus_test_arguments() -> list[str]:
    """Keep the legacy corpus test graph unchanged; workspace_corpus_test_arguments()."""
    return [
        "cargo",
        "llvm-cov",
        "--no-report",
        "--workspace",
        "--all-features",
        "--locked",
    ]


def print_coverage_totals(lcov: str, minimum: float) -> None:
    """Preserve corpus totals and threshold failure; print_coverage_totals(text, 80.)."""
    totals = crate_totals(lcov)
    for name, (found, hit) in sorted(totals.items()):
        print(f"  {name}: {percent(found, hit):.1f}% ({hit}/{found})")
    found, hit = sum(f for f, _ in totals.values()), sum(h for _, h in totals.values())
    print(f"total line coverage: {percent(found, hit):.1f}% ({hit}/{found})")
    if percent(found, hit) < minimum:
        sys.exit(f"coverage {percent(found, hit):.1f}% is below the {minimum}% minimum")


if __name__ == "__main__":
    main()
