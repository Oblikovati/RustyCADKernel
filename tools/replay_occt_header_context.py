#!/usr/bin/env python3
"""Replay pinned real-TU contexts into a fresh OCCT capture output directory."""

from __future__ import annotations
import argparse
import json
import pathlib
from api_inventory.context_replay_plan import make_plan
from api_inventory.context_replay_run import replay_plan


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="action", required=True)
    add_plan_arguments(commands.add_parser("plan"))
    add_run_arguments(commands.add_parser("run"))
    return parser.parse_args()


def add_plan_arguments(parser: argparse.ArgumentParser) -> None:
    for name in ("retries", "capture", "source", "database", "extractor", "output"):
        parser.add_argument(f"--{name}", type=pathlib.Path, required=True)


def add_run_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--plan", type=pathlib.Path, required=True)
    for name in ("capture", "source", "database", "extractor", "output"):
        parser.add_argument(f"--{name}", type=pathlib.Path, required=True)


def write_plan(args: argparse.Namespace) -> None:
    result = make_plan(
        args.retries, args.capture, args.source.resolve(), args.database, args.extractor
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")


def write_replay(args: argparse.Namespace) -> None:
    result = replay_plan(
        args.plan,
        args.capture,
        args.source.resolve(),
        args.database,
        args.extractor.resolve(),
        args.output,
    )
    args.output.mkdir(parents=True, exist_ok=True)
    manifest_path = args.output / "context-replay-manifest.json"
    if manifest_path.exists():
        raise FileExistsError(f"{manifest_path}: expected fresh output path")
    manifest_path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(
        f"Context replay: {result['parsed_with_context']}/{result['retry_count']} passed"
    )


def main() -> None:
    args = parse_args()
    (write_plan if args.action == "plan" else write_replay)(args)


if __name__ == "__main__":
    main()
