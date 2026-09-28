"""Deterministic MathPoly Laguerre and std::complex requests appended after the corpus."""
import random

SIZE = 21


def expanded(real_roots: list[float], pairs: list[tuple[float, float]], lead: float) -> list[float]:
    """Ascending coefficients of lead * prod(x - r) * prod(x^2 - 2a x + a^2 + b^2)."""
    poly = [lead]
    for root in real_roots:
        poly = [0.0] + poly
        for i in range(len(poly) - 1):
            poly[i] -= root * poly[i + 1]
    for re, im in pairs:
        b, c = -2.0 * re, re * re + im * im
        poly = [0.0, 0.0] + poly
        for i in range(len(poly) - 2):
            poly[i] += c * poly[i + 2] + b * poly[i + 1]
    return poly


def structured(rng: random.Random, case: int) -> list[float]:
    """Known-root polynomials: distinct, repeated, clustered, pure imaginary and mixed."""
    kind = case % 6
    count = rng.randint(1, 8)
    if kind == 0:
        return expanded([float(k + 1) for k in range(rng.randint(1, 12))], [], 1.0)
    if kind == 1:
        root = rng.choice([1.0, -2.0, 0.5, 3.0])
        return expanded([root] * rng.randint(2, 5) + [rng.uniform(-4, 4)], [], rng.choice([1.0, -2.5]))
    if kind == 2:
        pairs = [(rng.choice([0.0, 1.0, -0.5]), rng.choice([1.0, 0.01, 1e-7, 2.0])) for _ in range(rng.randint(1, 4))]
        return expanded([rng.uniform(-3, 3) for _ in range(rng.randint(0, 3))], pairs, 1.0)
    if kind == 3:
        pair = (rng.choice([0.0, 1.0]), rng.choice([1.0, 0.5]))
        return expanded([1.0] * rng.randint(0, 2), [pair] * rng.randint(2, 3), 1.0)
    if kind == 4:
        return expanded([1.0 + k * rng.choice([1e-3, 1e-6, 1e-9]) for k in range(count)], [], 1.0)
    return expanded([rng.uniform(-10, 10) for _ in range(count)],
                    [(rng.uniform(-2, 2), rng.uniform(0.1, 2)) for _ in range(rng.randint(0, 3))], rng.uniform(0.5, 3))


def coefficients(rng: random.Random, case: int) -> list[float]:
    """Random or structured coefficients, optionally globally scaled, padded to 21 slots."""
    if case % 3 == 0:
        poly = [rng.uniform(-10, 10) for _ in range(rng.randint(2, SIZE))]
        for index in range(len(poly) - 1):
            if rng.random() < 0.2:
                poly[index] = 0.0
    else:
        poly = structured(rng, case)
    scale = rng.choice([1.0, 1.0, 1.0, 1e-300, 1e-150, 1e150, 1e300, 0.001])
    return [x * scale for x in poly][:SIZE] + [0.0] * (SIZE - min(len(poly), SIZE))


def laguerre_request(rng: random.Random, case: int) -> str:
    """One `modern_laguerre` line: degree, tolerance, iterations, mode, 21 coefficients."""
    values = coefficients(rng, case)
    degree = max((i for i, x in enumerate(values) if x != 0.0), default=0)
    if case % 17 == 0:
        degree = rng.choice([-1, 0, 21, degree + 1, degree - 1])
    tolerance = rng.choice([1e-12, 1e-12, 1e-11, 1e-8, 1e-14, 0.0, -1e-12, 1e-3])
    iterations = rng.choice([100, 100, 0, 1, 3, 20, 1000])
    mode = rng.choice([0] * 12 + [1, 2, 3, 4, 4, 5, 6, 7])
    return 'modern_laguerre ' + ' '.join(map(str, [degree, tolerance, iterations, mode] + values))


def complex_request(rng: random.Random, case: int) -> str:
    """One `modern_complex` line covering ordinary, huge, tiny, subnormal and zero parts."""
    magnitude = [1.0, 1e300, 1e-300, 5e-324, 1e154, 1e-160, 4.49423283715579e307, 0.0]
    parts = [rng.uniform(-4, 4) * rng.choice(magnitude) for _ in range(4)]  # stays finite
    return 'modern_complex ' + ' '.join(map(str, parts + [case % 9 if case % 9 <= 4 else 0]))


def append_laguerre_requests(requests: list[str]) -> None:
    """Append 512 Laguerre and 512 complex-arithmetic requests; append_laguerre_requests(requests)."""
    rng = random.Random(20260928)
    for case in range(512):
        requests.append(laguerre_request(rng, case))
    for case in range(512):
        requests.append(complex_request(rng, case))
