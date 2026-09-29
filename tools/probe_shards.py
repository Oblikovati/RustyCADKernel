#!/usr/bin/env python3
"""Run a line-oriented probe over its request corpus in parallel shards.

Every oracle probe answers one output line per request line, and a probe process
is single-threaded. A corpus of tens of thousands of requests therefore used one
core for the whole run. This module deals the requests round-robin to SHARD_COUNT
probe processes, runs them on one worker per core, and puts the answers back in
corpus order.

Round-robin, not contiguous blocks: corpora are appended package by package, so the
slow requests of one package (a ProjLib polar approximation takes over a minute in a
debug build) sit next to each other and would all land in one block.

Some OCCT code keeps static state between calls (the FEmTool criterion and the
AdvApp2Var common blocks), so an answer can depend on the requests the same process
ran before it; the Rust twin keeps the same state, so both sides still agree. The
shard layout is therefore a fixed constant, never the core count: every machine
deals the same requests to the same shards and saves the same evidence.

Usage:
    output = run_sharded(ProbeCommand(['target/debug/examples/numeric_probe'], env), requests)
"""
from __future__ import annotations
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


# Fixed, so the answers of stateful requests do not depend on the machine; 64 keeps
# every core of a 32-core host busy with two shards.
SHARD_COUNT = 64


@dataclass(frozen=True)
class ProbeCommand:
    """One probe executable with the working directory and environment it runs in."""
    args: list[str]
    environment: dict[str, str]
    cwd: Path | None = None


@dataclass(frozen=True)
class ShardResult:
    """The stdout and stderr of one shard; ShardResult('OK 1\\n', '')."""
    stdout: str
    stderr: str


ShardRunner = Callable[[ProbeCommand, str], ShardResult]


def default_workers() -> int:
    """The number of probe processes to run at once: one per core; default_workers() >= 1."""
    return max(1, os.cpu_count() or 1)


def split_requests(requests: str, shards: int) -> list[list[str]]:
    """Deal the request lines round-robin to at most `shards` non-empty shards.

    Line i goes to shard i % shards; split_requests('a\\nb\\nc\\n', 2) == [['a\\n', 'c\\n'], ['b\\n']].
    """
    if shards < 1:
        raise ValueError(f'Shard count must be at least 1, got {shards}')
    lines: list[str] = requests.splitlines(keepends=True)
    return [lines[index::shards] for index in range(min(shards, len(lines)))]


def interleave_answers(shards: list[list[str]], outputs: list[str]) -> str:
    """Put the answers of round-robin shards back in corpus order.

    Each shard must answer one line per request, or the order cannot be restored;
    interleave_answers([['a\\n', 'c\\n'], ['b\\n']], ['A\\nC\\n', 'B\\n']) == 'A\\nB\\nC\\n'.
    """
    answers: list[list[str]] = [output.splitlines(keepends=True) for output in outputs]
    for shard, answer in zip(shards, answers):
        if len(answer) != len(shard):
            raise RuntimeError(f'Probe answered {len(answer)} lines for {len(shard)} requests in the shard '
                               f'that starts with {shard[0]!r}; expected one line per request')
    joined: list[str] = [answers[index % len(shards)][index // len(shards)]
                         for index in range(sum(map(len, shards)))]
    return ''.join(joined)


def run_shard(command: ProbeCommand, requests: str) -> ShardResult:
    """Run the probe on one shard and fail with its stderr; run_shard(command, 'op 1\\n')."""
    result = subprocess.run(command.args, input=requests, text=True, capture_output=True,
                            cwd=command.cwd, env=command.environment, check=False)
    if result.returncode:
        first: str = requests.splitlines()[0] if requests else '<empty>'
        raise RuntimeError(f'Probe failed ({result.returncode}) on the shard that starts with {first!r}: '
                           f"{' '.join(command.args)}\n{result.stderr[-2000:]}")
    return ShardResult(result.stdout, result.stderr)


def run_sharded(command: ProbeCommand, requests: str, workers: int | None = None,
                runner: ShardRunner = run_shard) -> ShardResult:
    """Run the probe over the whole corpus in parallel shards and join the answers in order.

    run_sharded(ProbeCommand(['probe'], env), 'op 1\\nop 2\\n').stdout == 'OK 1\\nOK 2\\n'
    """
    shards: list[list[str]] = split_requests(requests, SHARD_COUNT)
    if not shards:
        return runner(command, '')
    with ThreadPoolExecutor(max_workers=workers or default_workers()) as pool:
        results: list[ShardResult] = list(pool.map(lambda shard: runner(command, ''.join(shard)), shards))
    stdout: str = interleave_answers(shards, [result.stdout for result in results])
    return ShardResult(stdout, ''.join(result.stderr for result in results))
