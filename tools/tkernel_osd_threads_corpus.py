"""Request lines for the OSD threads probe family (OSD_Thread, OSD_ThreadPool, OSD_Parallel).

Covers pool sizes for automatic, zero, one and several participants with and without Init
and a changed default launch size; launcher sizes for negative, zero, capped and oversized
maximums on idle and busy pools; Perform coverage over empty, reversed, single, negative
and large ranges; single failures on one- and many-thread launchers (with the kept
self-thread failure); For/ForEach coverage including the forced and single-item paths;
the processor count and backend switch; and OSD_Thread run/wait/copy/timed-wait results.
"""
from __future__ import annotations

POOL_SIZES = [-1, 0, 1, 2, 3, 5, 8]


def pool_cases() -> list[str]:
    """Pool construction, SetNbDefaultThreadsToLaunch and Init combinations."""
    lines = [f"osd_pool {nb} 0 -99" for nb in POOL_SIZES]
    lines += [f"osd_pool {nb} {init} -99" for nb in (1, 4) for init in (-1, 1, 3, 6)]
    lines += [f"osd_pool 4 {init} {default}" for init in (0, 2) for default in (-3, 0, 1, 9)]
    return lines + ["osd_default_pool"]


def launcher_cases() -> list[str]:
    """Two launchers per pool: the second one sees the workers the first one locked."""
    maximums = [-1, 0, 1, 2, 3, 4, 10]
    return [f"osd_launcher {nb} {first} {second}" for nb in (1, 3, 5)
            for first in maximums for second in (-1, 0, 2, 10)]


def perform_cases() -> list[str]:
    """Perform coverage and failures over ranges of every shape."""
    ranges = [(0, 0), (0, 1), (5, 3), (-7, 7), (0, 1000), (100, 2600), (-2147483648, -2147483600)]
    lines = [f"osd_perform {nb} {mx} {b} {e}" for nb in (1, 2, 4) for mx in (-1, 0, 3)
             for b, e in ranges]
    lines += [f"osd_perform_fail {nb} {mx} 0 50 {at}" for nb in (1, 4) for mx in (-1, 1)
              for at in (0, 17, 49, 50)]
    return lines


def loop_cases() -> list[str]:
    """OSD_Parallel::For and ForEach coverage, forced and pooled."""
    lines = [f"osd_for {b} {e} {force}" for b, e in [(0, 0), (0, 1), (3, 4), (0, 2), (-50, 50),
                                                       (0, 10000), (10, 5)] for force in (0, 1)
             if not (force and e < b)]
    lines += [f"osd_foreach {count} {nb} {force}" for count in (0, 1, 2, 7, 5000)
              for nb in (-1, 1, 2, 64) for force in (0, 1)]
    return lines + ["osd_nproc"]


def thread_cases() -> list[str]:
    """OSD_Thread results, copies and timed waits."""
    lines = [f"osd_thread {value}" for value in (0, 1, 21, 1000000)]
    lines += [f"osd_thread_timed {ms}" for ms in (-5, 0, 10)]
    return lines + [f"osd_thread_copy {value}" for value in (0, 5, 123)]


def cases() -> list[str]:
    """Every threads request line, in a fixed order.

    >>> len([c for c in cases() if c.startswith("osd_launcher")])
    84
    """
    return pool_cases() + launcher_cases() + perform_cases() + loop_cases() + thread_cases()
