"""Requests for Standard.hxx memory services and Standard_MMgrOpt (operation prefix `services_`).

Excluded inputs (native undefined behavior, never probed):
- Standard_MMgrOpt without memory mapping and a request within 15 of SIZE_MAX: the header
  size wraps to 0, `calloc(0)` succeeds and the header is written past the block.
- A threshold below the rounded cell size once a small-block pool exists: Purge (also run
  by the destructor) reads free lists past the `calloc`'ed array.
- Freeing or reallocating a block twice, or an address the manager did not return.
- Allocation sizes a machine could satisfy only by overcommitting terabytes; the huge
  requests below exceed every address space.
"""
import random

SIZE_MAX = 2**64 - 1
HUGE = [SIZE_MAX, SIZE_MAX - 7, 2**63, 2**62]


def memory_requests(rng: random.Random) -> list[str]:
    """Standard.hxx and Standard_MMgrOpt requests; memory_requests(random.Random(1))."""
    return standard_requests() + mmgr_fixed_requests() + [mmgr_random_script(rng) for _ in range(80)]


def standard_requests() -> list[str]:
    """Allocate / AllocateOptimal / Reallocate / AllocateAligned / Purge / Free(T*&)."""
    sizes = [0, 1, 7, 8, 15, 16, 17, 255, 4096, 100000] + HUGE
    requests = [f"services_standard_allocate {size}" for size in sizes]
    requests += [f"services_standard_allocate_optimal {size}" for size in sizes]
    for old in ["-", "0", "1", "4", "32"]:
        requests += [f"services_standard_reallocate {old} {new}" for new in [0, 1, 3, 4, 40, 4096] + HUGE]
    for alignment in [0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 64, 4096, 65536, 2**62, 2**63]:
        requests += [f"services_standard_aligned {size} {alignment}" for size in [0, 1, 24, 5000, SIZE_MAX]]
    return requests + ["services_standard_misc"]


def mmgr_fixed_requests() -> list[str]:
    """Hand-written Standard_MMgrOpt scripts covering each allocation path and quirk."""
    lines = [
        # pool offsets, LIFO reuse by rounded size, Free(NULL), Reallocate keep/move
        "1 1 200 1000 40000 a10 a10 a16 f0 a8 a10 n R24 r5,20 r6,24 r6,40 p",
        "0 1 200 1000 40000 a10 a10 a16 f0 a8 a10 n R24 r5,20 r6,24 r6,40 p",
        # medium blocks: newest-first reuse, Purge count
        "1 1 200 1000 40000 a300 a300 f0 f1 a300 a300 p a500 f2 f3 p a300",
        "1 0 200 1000 40000 a300 a301 a302 a303 f0 f1 f2 f3 a300 a304 p",
        # large blocks, SIZE_MAX wrapping to a zero-size pool block, mmap(0) failing
        "1 1 200 1000 40000 a50000 f0 a50000 a18446744073709551615 a18446744073709551609 a18446744073709551608",
        "1 1 200 1000 40000 a18446744073709551600 a39999 a40000 a39993 a39992",
        # a failed allocation purges before raising
        "1 0 200 1000 40000 a300 f0 a4611686018427387904 a300",
        "1 1 200 1000 40000 a300 f0 a4611686018427387904 a300",
        "0 0 200 1000 40000 a300 f0 a9223372036854775808 a300 a300",
        # threshold 0: only zero-size blocks recycle; every other block is mapped
        "0 1 200 5 0 a8 a1 f1 a8 f0 f2 a16",
        "1 0 200 5 0 a8 a1 f1 a8 f0 f2 a16",
        # cell size 0 and 16, small thresholds
        "1 1 0 100 16 a0 a0 a1 a8 a9 f2 a8 p",
        "1 1 16 100 17 a16 a17 a24 f1 f2 a24 p",
        # exhausting a pool: the tail is recycled; a fully free pool is never released
        "1 1 16 100 40000 m18000,16 a8 a16 F0,17000 p a16",
        "1 1 16 100 40000 m17066,16 m10,16 F0,17066 p a16 a8",
        "0 0 200 100 40000 m2000,200 a200 a8 F0,1000 p a200",
        # Reallocate to exactly the rounded size, across the threshold, and from NULL
        "1 1 200 1000 40000 a1 r0,8 r1,9 r2,100000 r3,100008 R0 R50000 f5",
        "0 1 200 1000 40000 a1 r0,8 r1,9 r2,100000 r3,100008 R0 R50000 f5",
    ]
    return [f"services_mmgr {line}" for line in lines]


def mmgr_random_script(rng: random.Random) -> str:
    """A valid random script: frees only live blocks and keeps the threshold above the cell."""
    cell = rng.choice([0, 16, 40, 200, 256])
    threshold = rng.choice([cell + 16 + 1, 1000, 4096, 40000])
    header = f"{rng.randint(0, 1)} {rng.randint(0, 1)} {cell} {rng.choice([5, 100, 150])} {threshold}"
    live: list[int] = []
    count = 0
    tokens = []
    for _ in range(rng.randint(5, 40)):
        token, count = random_token(rng, live, count, cell, threshold)
        tokens.append(token)
    return f"services_mmgr {header} {' '.join(tokens)}"


def random_token(rng: random.Random, live: list[int], count: int, cell: int, threshold: int) -> tuple[str, int]:
    """One script token and the new allocation count; `live` tracks freeable indices."""
    choice = rng.random()
    if live and choice < 0.25:
        return f"f{live.pop(rng.randrange(len(live)))}", count
    if live and choice < 0.4:
        index = live.pop(rng.randrange(len(live)))
        live.append(count)
        return f"r{index},{random_size(rng, cell, threshold)}", count + 1
    if choice < 0.45:
        return rng.choice(["p", "n"]), count
    if choice < 0.5:
        live.append(count)
        return f"R{random_size(rng, cell, threshold)}", count + 1
    live.append(count)
    return f"a{random_size(rng, cell, threshold)}", count + 1


def random_size(rng: random.Random, cell: int, threshold: int) -> int:
    """A size from one of the manager's three classes (small, medium, large)."""
    kind = rng.random()
    if kind < 0.5:
        return rng.randint(0, max(cell, 1))
    if kind < 0.85:
        return rng.randint(cell, threshold)
    return rng.randint(threshold, threshold + 70000)
