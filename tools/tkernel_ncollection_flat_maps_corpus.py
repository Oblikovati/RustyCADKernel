"""Request lines for the NCollection flat maps probe family.

Scripts drive NCollection_FlatMap/FlatDataMap (Robin Hood displacement, growth, backward-shift
removal), NCollection_OrderedMap/OrderedDataMap (signed bucket counts, insertion order, the
swapping move assignment, Reset) and NCollection_PackedMap (signed constructor/ReSize, the
deprecated set algebra) on int keys.
"""
from __future__ import annotations

import random


def cases() -> list[str]:
    """Deterministic flat-map requests: normal, degenerate and boundary inputs.

    >>> all(line.startswith("ncoll_flat_maps_") for line in cases())
    True
    """
    return flat_set_cases() + flat_data_cases() + ordered_cases() + packed_cases() + algebra_cases()


def script(rng: random.Random, commands: str, size: int, span: int, reserve: str = "R") -> str:
    """`size` random `command key` pairs drawn from `commands` over keys in [-span, span].

    The flat maps' `Reserve(size_t)` gets non-negative counts only: a negative int cast to
    size_t asks for an unallocatable table, which has no defined native result. Commands
    outside `reserve` (the ordered/packed signed `ReSize(int)`) keep negative arguments.
    """
    pairs = []
    for _ in range(size):
        name, key = rng.choice(commands), rng.randint(-span, span)
        pairs.append(f"{name} {abs(key) if name in reserve else key}")
    return " ".join(pairs)


def flat_set_cases() -> list[str]:
    fixed = [
        "",
        "a 0 a 1 e 8",  # emplaced returns the displaced key, not the inserted one
        "a 0 a 1 d 8",
        "a 0 a 1 t 8 T 8",
        "a 7 a 15 a 23 a 31 r 15 q 23 q 15",  # wraparound cluster and backward shift
        "R 0 R 13 R 14 a 5 c 0 a 6 c 1",
        "a -1 a -9 a 2147483647 a -2147483648 q -1 r -9",
        "a 3 a 11 a 19 m 0 a 4",
        "E 5 E 5 T 5 r 5 r 5",
    ]
    rng = random.Random(8101)
    fixed += [script(rng, "aaaddeetTrqRc", 60, 40) for _ in range(40)]
    fixed += [script(rng, "aaar", 300, 5000) for _ in range(8)]
    return [f"ncoll_flat_maps_set {line}".rstrip() for line in fixed]


def flat_data_cases() -> list[str]:
    fixed = [
        "",
        "b 0 1 b 1 2 B 8 3 T 8 4 e 16 5 E 16 6",
        "f 5 b 5 50 f 5 u 5 f 5 q 5",
        "t 1 10 t 1 11 T 1 12 b 1 13 q 1",
        "b 7 1 b 15 2 b 23 3 u 7 q 15 q 23",
        "R 100 b 1 1 c 0 b 2 2 c 1 b 3 3",
    ]
    rng = random.Random(8102)
    for _ in range(40):
        pairs = []
        for _ in range(50):
            name = rng.choice("bbBtTeEufqRc")
            key = abs(rng.randint(-30, 30)) if name == "R" else rng.randint(-30, 30)
            pairs.append(f"{name} {key} {rng.randint(-99, 99)}" if name in "bBtTeE" else f"{name} {key}")
        fixed.append(" ".join(pairs))
    return [f"ncoll_flat_maps_data {line}".rstrip() for line in fixed]


def ordered_cases() -> list[str]:
    keys = [
        "0",
        "1",
        "-1",  # negative signed bucket count
        "0 a 5 a 3 a 9 N 0",
        "3 a 5 a 3 a 9 r 3 a 3 e 5 t 9 d 7",
        "1 R -1 R 200 N 0 a 1",
        "1 a 1 a 2 c 0 N 0 a 3 c 1 N 0 C 0 a 4",
        "1 a 10 a 20 a 30 m 0 a 40",
        "1 r 5 q 5",
    ]
    data = [
        "0",
        "-7",
        "1 b 1 10 b 2 20 B 1 11 T 2 22 e 3 30 E 3 33 t 4 40",
        "1 f 9 F 9 b 9 90 f 9 F 9 u 9 u 9",
        "1 b 1 1 b 2 2 n 77 b 3 3 m 0",
        "1 R -2 b 5 5 c 1 C 0",
    ]
    rng = random.Random(8103)
    for _ in range(30):
        keys.append(f"{rng.randint(0, 12)} " + script(rng, "aaaderqtRcN", 60, 150, reserve=""))
        pairs = []
        for _ in range(60):
            name = rng.choice("bbBtTeEufFnRc")
            key = rng.randint(-40, 40) if name != "R" else rng.randint(-3, 300)
            pairs.append(f"{name} {key} {rng.randint(-9, 9)}" if name in "bBtTeE" else f"{name} {key}")
        data.append(f"{rng.randint(0, 12)} " + " ".join(pairs))
    return ([f"ncoll_flat_maps_ordered {line}" for line in keys]
            + [f"ncoll_flat_maps_ordered_data {line}" for line in data])


def packed_cases() -> list[str]:
    fixed = [
        "1",
        "0",
        "-5",  # the signed constructor clamps to one bucket
        "1 a 0 a 31 a 32 a -1 a -32 h 31 r 31 h 31",
        "1 R -1 R 500 a 7 c 0 a 8",
        "1 a 1 a 2 m 0 a 3",
        "1 a 2147483647 a -2147483648 r -2147483648",
    ]
    rng = random.Random(8104)
    fixed += [f"{rng.randint(-2, 200)} " + script(rng, "aaaarhRc", 80, 3000, reserve="") for _ in range(30)]
    return [f"ncoll_flat_maps_packed {line}" for line in fixed]


def algebra_cases() -> list[str]:
    rng = random.Random(8105)
    sets: list[tuple[list[int], list[int]]] = [([], []), ([1, 2, 3], []), ([], [4, 5]), ([1, 2, 3], [1, 2, 3]),
                                              ([1, 2, 3], [2, 3, 4]), ([0, 64, 128], [64]),
                                              ([-1, -33], [-1, 31])]
    for _ in range(20):
        left = rng.sample(range(-300, 300), rng.randint(0, 60))
        right = rng.sample(range(-300, 300), rng.randint(0, 60))
        sets.append((left, right))
    lines = []
    for width in ["32", "64"]:
        for left, right in sets:
            if width == "64":
                left = [value * 4294967296 + 7 for value in left]
                right = [value * 4294967296 + 7 for value in right]
            fields = [len(left), *left, len(right), *right]
            lines.append(f"ncoll_flat_maps_algebra{width} " + " ".join(map(str, fields)))
    return lines
