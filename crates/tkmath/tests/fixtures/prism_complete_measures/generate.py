"""ADR0031 independent midpoint oracle; offline, no Martensite or OCCT imports."""
from fractions import Fraction
from pathlib import Path
import json
import math
import struct


def polynomial_numerator(numerator: int, denominator: int) -> int:
    """Return D^4*p(n/D), using the independently derived power polynomial."""
    n, d = numerator, denominator
    return 18*n**4 - 36*n**3*d + 26*n*n*d*d - 8*n*d**3 + d**4


def node_bracket(index: int, count: int, scale: int) -> tuple[int, int]:
    """Enclose sqrt(p(midpoint)) on an integer grid; verify both squares exactly."""
    denominator = (2*count)**4
    numerator = polynomial_numerator(2*index + 1, 2*count)*scale*scale
    lower = math.isqrt(numerator // denominator)
    assert lower*lower*denominator <= numerator < (lower+1)**2*denominator
    return lower, lower+1


def midpoint_bounds(count: int, scale: int) -> tuple[Fraction, Fraction]:
    """Include the analytic |f''|<=16128 remainder and every node's sqrt bracket."""
    lower, upper = 0, 0
    for index in range(count):
        first, last = node_bracket(index, count, scale)
        lower, upper = lower+first, upper+last
    error = Fraction(672, count*count)
    return Fraction(24*lower, scale*count)-error, Fraction(24*upper, scale*count)+error


def outward(value: Fraction, upward: bool) -> float:
    """Use Python's rational conversion and check the direction with exact fractions."""
    rounded = float(value)
    if (Fraction(rounded) < value) if upward else (Fraction(rounded) > value):
        rounded = math.nextafter(rounded, math.inf if upward else -math.inf)
    assert Fraction(rounded) >= value if upward else Fraction(rounded) <= value
    return rounded


def encode(bounds: tuple[Fraction, Fraction]) -> dict[str, object]:
    """Retain exact endpoints alongside outward binary64 fixture bits."""
    rounded = [outward(bounds[0], False), outward(bounds[1], True)]
    return {"exact": [str(value) for value in bounds], "decimal": rounded,
            "bits": [struct.unpack("Q", struct.pack("d", value))[0] for value in rounded]}


def generate() -> dict[str, object]:
    """Compute the one fixed representative; no production result is an input."""
    count, scale = 2**17, 2**80
    side = midpoint_bounds(count, scale)
    total = tuple(endpoint + Fraction(24, 5) for endpoint in side)
    return {"method": "exact-node-midpoint-with-analytic-remainder", "count": count,
            "sqrt_grid": scale, "second_derivative_bound": 16128,
            "side": encode(side), "total": encode(total)}


if __name__ == "__main__":
    result = generate()
    Path(__file__).with_name("closed_cubic.json").write_text(json.dumps(result, indent=2)+"\n")
    rows = [name+"\t"+"\t".join(map(str, result[name]["bits"])) for name in ["side", "total"]]
    Path(__file__).with_name("closed_cubic.tsv").write_text("\n".join(rows)+"\n")
    print(json.dumps(result))
