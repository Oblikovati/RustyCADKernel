"""Request lines for the NCollection allocators probe family.

Incremental and accumulating sequences are seeded random walks over sizes that straddle the
block thresholds (minimum block size, the 16-byte used-heap threshold, the 4-byte Acc
alignment, oversize dedicated blocks); frees only target live allocations so no request
depends on where either heap happens to reuse memory.
"""
from __future__ import annotations

import random

INC_SIZES = [0, 1, 3, 8, 15, 16, 17, 100, 1000, 2032, 2033, 2047, 2048, 4096, 12288, 20000, 100000]
ACC_SIZES = [0, 1, 2, 3, 4, 5, 8, 13, 16, 31, 64, 100, 257, 1000, 24600, 30000]


def incremental_cases(rng: random.Random) -> list[str]:
    """Allocation walks with rewinds (r0) and releases (r1) for several block sizes."""
    lines = ["ncoll_allocators_inc 2048 12 a2048 a2048 a2048 a2048 a2048 a2048 a2048 a2033 a16 r0 a8 a2048",
             "ncoll_allocators_inc 0 6 a12288 a1 r1 a4096 a4096 a4096",
             "ncoll_allocators_inc 100 4 a0 a0 a12288 a0"]
    for block_size in [0, 1, 2047, 2048, 4096, 12288, 50000, 1000000]:
        for _ in range(6):
            steps = [rng.choice(["r0", "r1"]) if rng.random() < 0.04 else f"a{rng.choice(INC_SIZES)}"
                     for _ in range(rng.randrange(1, 160))]
            lines.append(f"ncoll_allocators_inc {block_size} {len(steps)} {' '.join(steps)}")
    growth = [f"a{2048 * 8 ** min(i // 5, 3)}" for i in range(40)]
    lines.append(f"ncoll_allocators_inc 2048 {len(growth)} {' '.join(growth)}")
    return lines


def accumulating_walk(rng: random.Random, count: int) -> list[str]:
    """Allocations and frees of live allocations only, plus foreign frees."""
    steps: list[str] = []
    live: list[int] = []
    allocated = 0
    for _ in range(count):
        roll = rng.random()
        if roll < 0.05:
            steps.append("x")
        elif roll < 0.45 and live:
            steps.append(f"f{live.pop(rng.randrange(len(live)))}")
        else:
            steps.append(f"a{rng.choice(ACC_SIZES)}")
            live.append(allocated)
            allocated += 1
    return steps


def accumulating_cases(rng: random.Random) -> list[str]:
    """Hand-picked Acc edge cases and random walks across block sizes."""
    lines = ["ncoll_allocators_acc 64 4 a8 a8 f0 f0",
             "ncoll_allocators_acc 64 5 a8 a8 f0 f0 f1",
             "ncoll_allocators_acc 64 6 a40 a40 a24 a20 f3 f1",
             "ncoll_allocators_acc 64 4 a100 a8 f0 f1",
             "ncoll_allocators_acc 1 5 a0 a4 a5 f2 x",
             "ncoll_allocators_acc 24600 2 x a0"]
    lookup = [f"a{60}" for _ in range(20)] + ["a4"]
    lines.append(f"ncoll_allocators_acc 64 {len(lookup)} {' '.join(lookup)}")
    for block_size in [3, 4, 64, 100, 1024, 24600]:
        for _ in range(8):
            steps = accumulating_walk(rng, rng.randrange(1, 200))
            lines.append(f"ncoll_allocators_acc {block_size} {len(steps)} {' '.join(steps)}")
    return lines + ["ncoll_allocators_acc_extension"]


def aligned_cases() -> list[str]:
    """_mm_malloc alignment rules: zero, non powers of two, pointer-size promotion, pages."""
    alignments = [0, 1, 2, 3, 4, 5, 8, 12, 16, 24, 32, 64, 128, 256, 4096, 65536]
    return [f"ncoll_allocators_aligned {alignment} {size}"
            for alignment in alignments for size in [0, 1, 100, 4097]]


def buffer_cases() -> list[str]:
    """Buffer construction, reallocation and release on every allocator kind."""
    lines = ["ncoll_allocators_buffer null 0 0"]
    for name in ["common", "heap", "inc", "acc", "a16", "a3", "a1"]:
        lines += [f"ncoll_allocators_buffer {name} {size} {resize}"
                  for size, resize in [(0, 0), (16, 8), (0, 32), (5, 0), (100, 100000)]]
    return lines


def cases() -> list[str]:
    """Deterministic allocators requests: normal, degenerate and boundary inputs."""
    rng = random.Random(20020412)
    return (incremental_cases(rng) + accumulating_cases(rng) + aligned_cases() + buffer_cases()
            + ["ncoll_allocators_occ_eq", "ncoll_allocators_instance_identity"])
