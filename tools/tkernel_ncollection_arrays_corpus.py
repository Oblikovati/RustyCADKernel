"""Request lines for the NCollection arrays probe family."""
from __future__ import annotations

import random


def bounds_cases() -> list[str]:
    """Array1/Array2 bound arithmetic: origins of every sign, empty ranges, int wrap-around."""
    lines = [f"ncoll_arrays_array1 {lower} {upper} {new}" for lower, upper, new in [
        (1, 5, 10), (0, 0, 0), (-3, 3, -1), (1, 0, 7), (5, 4, -5), (-2147483647, -2147483640, 2147483647),
        (2147483640, 2147483646, -2147483648)]]
    lines += [f"ncoll_arrays_range {lower} {upper} {size}" for lower, upper, size in [
        (1, 5, 5), (3, 2, 0), (5, 0, 1), (0, -1, 7), (-2147483648, 2147483647, 3),
        (2147483647, -2147483648, 4294967296), (10, 9, 18446744073709551615)]]
    lines += [f"ncoll_arrays_layout {rl} {ru} {cl} {cu}" for rl, ru, cl, cu in [
        (1, 2, 1, 3), (0, 1, 0, 2), (-2, 2, -5, 5), (1, 0, 1, 3), (1, 3, 1, 0), (5, 4, 7, 6),
        (100, 101, -100, -99), (46340, 46341, 1, 46341)]]
    return lines


def storage_cases() -> list[str]:
    """AliasedArray shapes and the LinearVector growth rule, ending on an out-of-range throw."""
    lines = [f"ncoll_arrays_aliased {stride} {count}" for stride, count in [
        (4, 3), (1, 1), (16, 0), (8, 5), (3, 7)]]
    lines += ["ncoll_arrays_linear " + ops for ops in [
        "a a a a a", "r5 a a R2 a c a C a", "R3 i0 i4 i2 e0 e3", "r0 R0 c C", "a a a a a a a a a a e10",
        "i1", "a r1 r3 r2 R9 R1 a a a a"]]
    return lines


def sparse_cases() -> list[str]:
    """Block-table growth from the increment, block freeing, clear, assign and exchange."""
    lines = ["ncoll_arrays_sparse " + ops for ops in [
        "4 s0 s16 s100 u100 u100 c s3", "1 s0 s1 s2 s7 u1 s1", "8 s1000 s5 u5 u1000 s63 s64",
        "3 u0 s2 s2 u2 u2"]]
    rng = random.Random(8131)
    for increment in [2, 5, 16]:
        ops = " ".join(rng.choice("su") + str(rng.randrange(200)) for _ in range(40))
        lines.append(f"ncoll_arrays_sparse {increment} {ops}")
    lines += ["ncoll_arrays_sparse_assign " + spec for spec in [
        "2 1 9 3 1 0", "4 3 1 2 30 4 4 1 2 3 99", "4 0 4 2 0 50", "7 2 0 70 7 3 1 2 100"]]
    return lines


def dynamic_cases() -> list[str]:
    """DynamicArray appends across blocks, gap-filling SetValue, inserts and erasure."""
    return ["ncoll_arrays_dynamic " + ops for ops in [
        "2 a1 a2 a3 a4 a5", "4 s6,9 a7 b0,5 f7,8", "1 a1 a2 e e e a3", "3 a1 a2 c a3 s4,4",
        "256 a1 b0,2 f1,3 b2,4", "2 a1 b1,2", "2 a1 f-1,2", "2 s-1,3", "8 a1 a2 a3 f2,9"]]


def cases() -> list[str]:
    """Deterministic arrays requests: normal, degenerate and boundary inputs."""
    return bounds_cases() + storage_cases() + sparse_cases() + dynamic_cases()
