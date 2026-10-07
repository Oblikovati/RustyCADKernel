"""Deterministic requests for Graphic3d_FrameStats (tools/oracle/tkservice_probe_stats.cxx).

`sv_stats` fills a data frame and the maxima with counters around the k/M/G and KiB/MiB/GiB
thresholds, timers in milliseconds, seconds, minutes, hours and below zero, and rates with
and without immediate redraws, then compares FormatStats as text and as a dictionary for
random counter flags in both line formats, and the FlushTimers, FillMax and Reset results.
"""
from __future__ import annotations
import random

from tkservice_corpus_camera import numbers

COUNTS = [0, 1, 7, 999, 1000, 1049, 1023, 1024, 999999, 1000000, 1048576, 123456789,
          999999999, 1000000000, 1073741824, 5000000000]
TIMES = [0.0, 0.0005, 0.016, 0.04996, 0.9996, 1.0, 1.5, 59.99, 61.25, 3599.9999, 3725.5,
         90000.0, -0.5, -1.5, -3700.0]
RATES = [-1.0, 0.0, 59.94, 1234.56, 0.05]
FLAGS = [0x001, 0x003, 0x00F, 0x3FF, 0xFFF, 0x1FFF, 0x400, 0xC00, 0x200, 0x0F0, 0x00C, 0x004, 0x008]


def frame(rng: random.Random, culled: bool, immediate: bool) -> list[float]:
    """The 27 counters, 5 timers and 4 rates of a data frame."""
    counters = [rng.choice(COUNTS) for _ in range(27)]
    if not culled:
        counters[5], counters[6] = counters[0], counters[1]
    if not immediate:
        counters[16:27] = [0] * 11
    rates = [rng.choice(RATES), rng.choice(RATES), rng.choice(RATES) if immediate else -1.0,
             rng.choice(RATES) if immediate else -1.0]
    return counters + [rng.choice(TIMES) for _ in range(5)] + rates


def append_stats_requests(requests: list[str]) -> None:
    """Append seeded statistics frames with their maxima and counter flags."""
    rng = random.Random(20261013)
    for index in range(120):
        flags = FLAGS[index % len(FLAGS)] if index < 2 * len(FLAGS) else rng.randrange(0, 0x2000)
        long_line = index % 2
        current = frame(rng, rng.random() < 0.5, rng.random() < 0.5)
        maxima = frame(rng, True, True) if rng.random() < 0.7 else [0.0] * 36
        requests.append('sv_stats ' + numbers([long_line, flags] + current + maxima + [rng.choice([1, 3, 10])]))
