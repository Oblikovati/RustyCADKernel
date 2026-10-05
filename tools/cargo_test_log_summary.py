#!/usr/bin/env python3
"""Aggregate every Cargo/libtest result group without mistaking the last doctest group for the run."""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import gzip
import json
from pathlib import Path
import re
import sys

CARGO_RESULT_GROUP = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;",
    re.MULTILINE,
)
ANSI_CONTROL_SEQUENCE = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")


@dataclass(frozen=True)
class CargoTestLogSummary:
    """Totals from libtest summaries; command success requires the real process exit code."""

    passed: int
    failed: int
    ignored: int
    result_groups: int
    failed_result_groups: int
    command_exit_code: int
    command_succeeded: bool


def summarize_cargo_test_log(
    contents: str, command_exit_code: int
) -> CargoTestLogSummary:
    """Aggregate all groups; e.g. `summarize_cargo_test_log(log_text, 0).passed`.

    Counts describe the supplied log, not API or original-declaration completeness.
    A successful prefix alone cannot prove that a Cargo command completed.
    """
    groups = list(CARGO_RESULT_GROUP.finditer(ANSI_CONTROL_SEQUENCE.sub("", contents)))
    if not groups:
        raise ValueError(
            "Cargo log contains 0 result groups; expected at least one libtest summary"
        )
    return _summarize_cargo_groups(groups, command_exit_code)


def _summarize_cargo_groups(
    groups: list[re.Match[str]], command_exit_code: int
) -> CargoTestLogSummary:
    passed = sum(int(group[2]) for group in groups)
    failed = sum(int(group[3]) for group in groups)
    ignored = sum(int(group[4]) for group in groups)
    failed_groups = sum(group[1] == "FAILED" for group in groups)
    return CargoTestLogSummary(
        passed,
        failed,
        ignored,
        len(groups),
        failed_groups,
        command_exit_code,
        command_exit_code == 0 and failed == 0 and failed_groups == 0,
    )


def read_cargo_test_log(log_path: Path) -> str:
    """Read a plain or archived log; e.g. `read_cargo_test_log(Path('tests.log.gz'))`."""
    if log_path.suffix == ".gz":
        with gzip.open(log_path, "rt", encoding="utf-8", errors="replace") as stream:
            return stream.read()
    return log_path.read_text(encoding="utf-8", errors="replace")


def cargo_log_arguments() -> tuple[Path, int]:
    """Parse the input and real exit status; e.g. `log_path, status = cargo_log_arguments()`."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument(
        "--exit-code", type=int, required=True, help="Actual Cargo process exit code"
    )
    arguments = parser.parse_args()
    return arguments.log, arguments.exit_code


def main() -> int:
    """Print a JSON receipt; e.g. `python3 tools/cargo_test_log_summary.py tests.log --exit-code 0`."""
    log_path, exit_code = cargo_log_arguments()
    try:
        summary = summarize_cargo_test_log(read_cargo_test_log(log_path), exit_code)
    except (OSError, ValueError) as error:
        print(f"Cannot summarize Cargo log {log_path}: {error}", file=sys.stderr)
        return 2
    print(json.dumps(asdict(summary), indent=2))
    return 0 if summary.command_succeeded else 1


if __name__ == "__main__":
    raise SystemExit(main())
