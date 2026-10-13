# SPDX-License-Identifier: GPL-2.0-only
"""Generate exact orientation fixtures: python3 generate.py > oracle.csv."""

from fractions import Fraction
import math
import random
import struct

Point = tuple[float, float]
Triangle = tuple[Point, Point, Point]


def rational_sign(points: Triangle) -> int:
    """Use a homogeneous 3x3 determinant; e.g. rational_sign(unit_triangle)."""
    rows = tuple((Fraction(x), Fraction(y)) for x, y in points)
    positive = sum(rows[i][0] * rows[(i + 1) % 3][1] for i in range(3))
    negative = sum(rows[i][1] * rows[(i + 1) % 3][0] for i in range(3))
    determinant = positive - negative
    return (determinant > 0) - (determinant < 0)


def binary_word(value: float) -> str:
    """Preserve binary64 input bits; e.g. binary_word(-0.0)."""
    return struct.pack('>d', value).hex()


def random_finite(generator: random.Random) -> float:
    """Draw finite bit patterns; e.g. random_finite(random.Random(10))."""
    bits = generator.getrandbits(64)
    while (bits >> 52) & 0x7ff == 0x7ff:
        bits = generator.getrandbits(64)
    return struct.unpack('>d', bits.to_bytes(8, 'big'))[0]


def corpus_triangles() -> list[Triangle]:
    """Produce fixed and generated cases; e.g. len(corpus_triangles())."""
    tiny = math.ulp(0.0)
    largest = float.fromhex('0x1.fffffffffffffp+1023')
    epsilon = math.ulp(1.0)
    cases: list[Triangle] = [
        ((0., 0.), (1., 0.), (0., 1.)),
        ((0., 0.), (1e-200, 0.), (0., 1e-200)),
        ((0., 0.), (tiny, 0.), (0., tiny)),
        ((-largest, -largest), (largest, -largest), (0., largest)),
        ((0., 0.), (1. + epsilon, 1.), (1., 1. - epsilon)),
        ((-0., 0.), (0., -0.), (-0., -0.)),
        ((tiny, largest), (tiny, largest), (-largest, tiny)),
    ]
    generator = random.Random(0x2081)
    cases.extend(near_collinear(generator) for _ in range(256))
    cases.extend(tuple((random_finite(generator), random_finite(generator))
                       for _ in range(3)) for _ in range(128))
    return cases


def near_collinear(generator: random.Random) -> Triangle:
    """Exercise rounded construction; e.g. near_collinear(random.Random(10))."""
    a = (generator.uniform(-1e5, 1e5), generator.uniform(-1e5, 1e5))
    b = (generator.uniform(-1e5, 1e5), generator.uniform(-1e5, 1e5))
    fraction = generator.random()
    c = (a[0] + fraction * (b[0] - a[0]), a[1] + fraction * (b[1] - a[1]))
    return a, b, c


if __name__ == '__main__':
    print('ax,ay,bx,by,cx,cy,sign')
    for triangle in corpus_triangles():
        words = [binary_word(coordinate) for point in triangle for coordinate in point]
        print(','.join(words + [str(rational_sign(triangle))]))
