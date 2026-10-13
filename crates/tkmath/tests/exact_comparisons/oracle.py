# SPDX-License-Identifier: GPL-2.0-only
"""Emit independent Fraction comparisons of supplied binary64 bits; run python3 oracle.py."""

from fractions import Fraction
from random import Random
from struct import pack, unpack
import math


def hexadecimal(value: float) -> str:
    return pack(">d", value).hex()


def expected(points: list[tuple[float, float]]) -> tuple[int, int]:
    a, b, c, d = [tuple(Fraction(value) for value in point) for point in points]
    distance = sum((x - y) ** 2 for x, y in zip(a, b)) - sum(
        (x - y) ** 2 for x, y in zip(a, c)
    )
    projection = sum((v - u) * (x - y) for u, v, x, y in zip(a, b, c, d))
    return (
        int(distance > 0) - int(distance < 0),
        int(projection > 0) - int(projection < 0),
    )


def ordinary_cases() -> list[list[tuple[float, float]]]:
    rows: list[list[tuple[float, float]]] = [
        [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (0.0, 0.0)],
        [(0.0, 0.0), (2.0, 0.0), (1.0, 0.0), (2.0, 0.0)],
        [(0.0, 0.0), (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0)],
        [(3.0, 4.0), (3.0, 4.0), (3.0, 4.0), (0.0, 0.0)],
        [(-3.0, -4.0), (0.0, 0.0), (-6.0, -8.0), (3.0, 4.0)],
        [(1.0, 2.0), (4.0, 6.0), (4.0, 2.0), (0.0, 5.0)],
        [(0.0, 0.0), (0.0, -2.0), (1.0, 1.0), (-1.0, -1.0)],
        [(-0.0, 0.0), (0.0, -0.0), (1.0, 0.0), (0.0, 1.0)],
    ]
    return rows


def scaled_cases() -> list[list[tuple[float, float]]]:
    rows: list[list[tuple[float, float]]] = []
    for exponent in [-1074, -1022, -600, -100, 0, 100, 600, 1023]:
        scale = math.ldexp(1.0, exponent)
        after = math.nextafter(scale, math.inf)
        rows.extend(
            [
                [(0.0, 0.0), (scale, 0.0), (after, 0.0), (-scale, 0.0)],
                [(-scale, -scale), (scale, scale), (scale, -scale), (-scale, scale)],
                [(0.0, 0.0), (scale, scale), (scale, -scale), (0.0, 0.0)],
                [(scale, scale), (scale, scale), (-scale, 0.0), (0.0, -scale)],
            ]
        )
    return rows


def random_coordinate(random: Random) -> float:
    bits = (
        random.getrandbits(1) << 63
        | random.randrange(2047) << 52
        | random.getrandbits(52)
    )
    return unpack(">d", pack(">Q", bits))[0]


def corpus() -> list[list[tuple[float, float]]]:
    rows = ordinary_cases() + scaled_cases()
    random = Random(3563)
    for _ in range(256):
        coordinates = [random_coordinate(random) for _ in range(8)]
        rows.append(list(zip(coordinates[::2], coordinates[1::2])))
    return rows


def main() -> None:
    print("ax,ay,bx,by,cx,cy,dx,dy,distance,projection")
    for points in corpus():
        signs = expected(points)
        print(
            ",".join(
                [hexadecimal(value) for point in points for value in point]
                + [str(sign) for sign in signs]
            )
        )


if __name__ == "__main__":
    main()
