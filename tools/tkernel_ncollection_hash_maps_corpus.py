"""Request lines for the NCollection hash maps probe family.

Scripts drive NCollection_DataMap, Map, IndexedMap, IndexedDataMap and DoubleMap of int
keys; keys that differ by 101 or 1009 share a bucket, so the traversal printed after each
script checks the exact chain order through inserts, removals, resizes and clears.
"""
from __future__ import annotations

import random

SIZE_MAX = 2**64 - 1
PRIMES = [0, 1, 100, 101, 102, 1008, 1009, 2038431744, 2038431745, 2038431746, SIZE_MAX - 1, SIZE_MAX]
HASH_TYPES = {
    "bool": [0, 1, 7], "char": [-128, -1, 0, 65, 127], "schar": [-5, 5], "uchar": [0, 200, 255],
    "wchar": [-1, 0, 66000], "char16": [0, 65535], "char32": [0, 1114111, 4294967295],
    "short": [-32768, -1, 12], "ushort": [65535], "int": [-2147483648, -1, 0, 2147483647],
    "uint": [4294967295], "long": [-9223372036854775808, -1, 5], "ulong": [-1, 3],
    "llong": [-2, 9223372036854775807], "ullong": [0, -1],
}
DOUBLES = ["0", "-0", "1", "-1.5", "3.141592653589793", "1e-310", "1e308", "-2.5e-5"]

FIXED_SCRIPTS = [
    ("datamap", "0"),
    ("datamap", "0 b 1 10 b 102 20 b 203 30 i 102 u 102 i 102 f 102 f 1 F 203 F 9 q 1 q 9 x 203 x 5"),
    ("datamap", "1 b 1 1 t 1 2 s 1 3 T 1 4 T 5 5 e 1 6 E 1 7 y 1 8 Y 1 9 e 2 2 E 3 3 y 4 4 Y 6 6"),
    ("datamap", "7 b -1 1 b -102 2 b 100 3 r 500 r 1 c 0 b 7 7 c 1 b 8 8 A"),
    ("map", "0 a 1 a 102 a 203 A 102 e 102 E 304 y 1 Y 405 m 102 m 9 i 1 x 203 x 9 C"),
    ("map", "3 a 5 c 1 a 6 r 2000 a 2015 c 0 a 1"),
    ("indexed", "0 a 1 a 102 a 203 A 102 e 102 y 405 f 0 f 5 f 2 n 203 n 9 x 1 x 9"),
    ("indexed", "0 a 1 a 2 a 3 s 2 3 s 2 9 s 0 9 s 4 9 w 1 3 w 1 1 w 0 1 R 1 R 9 l m 9 m 2 l l l C"),
    ("indexed", "5 a 1 a 102 r 1500 a 1010 c 0 a 4 c 1 a 5"),
    ("indexed_data", "0 a 1 10 a 1 11 b 1 12 b 2 20 t 2 21 t 3 30 B 3 31 T 3 32 T 4 40 e 4 41 y 4 42 y 5 50"),
    ("indexed_data", "0 a 1 1 a 102 2 a 203 3 s 1 203 9 s 1 304 9 s 9 1 1 w 1 3 f 1 f 0 F 102 F 9 G 1 G 9 q 1 q 9"),
    ("indexed_data", "0 F 1 a 1 1 R 1 l R 0 a 2 2 a 3 3 m 2 m 9 C r 3000 c 0 a 4 4"),
    ("double", "0 b 1 5 b 2 106 q 1 5 q 2 106 q 1 106 q 3 5 b 102 7 q 102 7 q 1 5 b 1 9 b 9 5"),
    ("double", "0 t 1 5 t 1 6 t 6 5 i 1 I 5 i 9 f 1 f 9 g 5 g 9 F 1 F 9 G 5 G 9 u 1 u 1 t 1 5 U 5 U 5"),
    ("double", "0 b 1 1 b 102 102 b 203 203 b 304 405 q 203 203 q 304 405 u 102 q 203 203 C r 1500 c 0 b 1 2"),
]


def algo_cases() -> list[str]:
    """Every MapAlgo function on overlapping, disjoint, empty and colliding operand sets."""
    operands = [([], [1, 2, 3], [3, 4]), ([9, 8], [1, 102, 203], [102, 5]), ([], [], [1]),
                ([7], [1, 2], []), ([], [1, 2], [2, 1]), ([5], [4, 105, 206, 307], [307, 206, 105, 4])]
    names = ["union", "intersection", "subtraction", "difference", "unite", "intersect", "subtract",
             "differ", "has_intersection", "contains", "is_equal"]
    lines = []
    for name in names:
        for result, left, right in operands:
            fields = [len(result), *result, len(left), *left, len(right), *right]
            lines.append(f"ncoll_hash_maps_algo {name} " + " ".join(map(str, fields)))
    return lines


ARGUMENTS = {
    "datamap": {**dict.fromkeys("btsTeEyY", "kv"), **dict.fromkeys("uifFqx", "k"), "r": "n", "c": "b"},
    "map": {**dict.fromkeys("aAeEyYmix", "k"), "r": "n", "c": "b"},
    "indexed": {**dict.fromkeys("aAeymnx", "k"), "f": "i", "R": "i", "w": "ii", "s": "ik", "l": "",
                "r": "n", "c": "b"},
    "indexed_data": {**dict.fromkeys("abtBTey", "kv"), **dict.fromkeys("mFGq", "k"), "f": "i", "R": "i",
                     "w": "ii", "s": "ikv", "l": "", "r": "n", "c": "b"},
    "double": {**dict.fromkeys("btq", "kv"), **dict.fromkeys("uUiIfgFG", "k"), "r": "n", "c": "b"},
}
# Insertions dominate so that the maps grow past the first prime step (101 buckets).
RANDOM_CODES = {
    "datamap": "bbbbbbttsTeEyYuuifFqxrc", "map": "aaaaaaAeEyYmmixrc",
    "indexed": "aaaaaaAeymnxfRwslrc", "indexed_data": "aaaaabbtBTeyRmfFGqwslrc",
    "double": "bbbbbbttuUqqiIfgFGrc",
}


def argument(rng: random.Random, spec: str) -> str:
    """One script argument: a collision-prone key, a value, an index, a size or a flag."""
    if spec == "k":
        return str(rng.choice([rng.randrange(-300, 300), 101 * rng.randrange(-4, 5) + 7, rng.randrange(2000)]))
    if spec == "v":
        return str(rng.randrange(-300, 300))
    if spec == "i":
        return str(rng.randrange(0, 60))
    if spec == "n":
        return str(rng.choice([0, 50, 150, 1500]))
    return str(rng.choice([0, 0, 0, 1]))


def random_script(rng: random.Random, kind: str) -> str:
    """A seeded script of 40-260 operations on one map kind."""
    fields = [str(rng.choice([0, 1, 5, 101]))]
    for _ in range(rng.randrange(40, 260)):
        code = rng.choice(RANDOM_CODES[kind])
        fields += [code] + [argument(rng, spec) for spec in ARGUMENTS[kind][code]]
    return f"ncoll_hash_maps_{kind} " + " ".join(fields)


def cases() -> list[str]:
    """Deterministic hash maps requests: normal, degenerate and boundary inputs."""
    lines = [f"ncoll_hash_maps_prime {value}" for value in PRIMES]
    lines += [f"ncoll_hash_maps_hash {kind} {value}" for kind, values in HASH_TYPES.items() for value in values]
    lines += [f"ncoll_hash_maps_hash double {value}" for value in DOUBLES]
    lines += [f"ncoll_hash_maps_{kind} {script}" for kind, script in FIXED_SCRIPTS]
    lines += algo_cases()
    rng = random.Random(0x0CC7)
    for kind in RANDOM_CODES:
        lines += [random_script(rng, kind) for _ in range(24)]
    return lines
