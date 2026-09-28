"""Valid B-spline curve definitions shared by the BSplCLib and BSplSLib corpora.

Usage: definition = curve(rng, degree=3, periodic=False) then encode(definition).
Every curve respects the native preconditions (increasing knots, end and interior
multiplicities in range, pole count equal to BSplCLib::NbPoles) so that requests
never reach the undefined pointer arithmetic of the native code.
"""
from __future__ import annotations
from dataclasses import dataclass, field
import random


@dataclass
class Curve:
    """A curve request: kind 1 (flat reals of `dim`), 2 (gp_Pnt2d) or 3 (gp_Pnt)."""
    kind: int
    dim: int
    rational: bool
    degree: int
    periodic: bool
    knots: list[float]
    mults: list[int]
    poles: list[float]
    weights: list[float] = field(default_factory=list)
    knot_lower: int = 1
    pole_lower: int = 1

    def count(self) -> int:
        """Number of poles; Curve(...).count()."""
        return len(self.poles)//self.width()

    def width(self) -> int:
        """Values per pole in the request; 2 for kind 2."""
        return self.dim if self.kind == 1 else self.kind


def nb_poles(degree: int, periodic: bool, mults: list[int]) -> int:
    """BSplCLib::NbPoles for valid multiplicities; nb_poles(3, False, [4, 4]) == 4."""
    return sum(mults[:-1]) if periodic else sum(mults)-degree-1


def knot_vector(rng: random.Random, count: int, style: str) -> list[float]:
    """Increasing knots: 'uniform', 'random', 'tiny' spans or 'huge' offsets."""
    start = rng.choice([0.0, -1.0, rng.uniform(-5, 5)])
    if style == 'uniform':
        step = rng.choice([1.0, 0.25, 0.1])
        return [start+i*step for i in range(count)]
    scale = {'random': 1.0, 'tiny': 1e-7, 'huge': 1e3}[style]
    knots = [start*scale]
    for _ in range(count-1):
        knots.append(knots[-1]+rng.uniform(0.05, 2.0)*scale)
    return knots


def multiplicities(rng: random.Random, degree: int, count: int, periodic: bool, clamped: bool) -> list[int]:
    """End and interior multiplicities; periodic ends are equal and <= degree."""
    interior = [rng.randint(1, max(degree, 1)) for _ in range(count-2)]
    if periodic:
        end = rng.randint(1, max(degree, 1))
        return [end]+interior+[end]
    first = degree+1 if clamped else rng.randint(1, degree+1)
    last = degree+1 if clamped else rng.randint(1, degree+1)
    return [first]+interior+[last]


def curve(rng: random.Random, degree: int, periodic: bool, kind: int = 3, dim: int = 3,
          rational: bool = False, clamped: bool = True, style: str = 'random') -> Curve:
    """A valid curve with at least Degree+1 poles; curve(rng, 3, False)."""
    count = rng.randint(2, 6)
    while True:
        mults = multiplicities(rng, degree, count, periodic, clamped)
        poles = nb_poles(degree, periodic, mults)
        if poles >= degree+1 and (not periodic or sum(mults[1:]) >= degree+1):
            break
        count += 1
    width = dim if kind == 1 else kind
    values = [rng.uniform(-10, 10) for _ in range(poles*width)]
    weights = [rng.uniform(0.3, 3.0) for _ in range(poles)] if rational else []
    if rational and rng.random() < 0.2:
        weights = [weights[0]]*poles
    return Curve(kind, dim, rational, degree, periodic, knot_vector(rng, count, style), mults, values, weights)


def flat_knots(definition: Curve) -> list[float]:
    """BSplCLib::KnotSequence(Knots, Mults, Degree, Periodic) of a valid curve."""
    knots, mults, degree = definition.knots, definition.mults, definition.degree
    flat = [k for k, m in zip(knots, mults) for _ in range(m)]
    if not definition.periodic:
        return flat
    period, m1 = knots[-1]-knots[0], degree+1-mults[0]
    before = [k-period for k, m in zip(knots[:-1], mults[:-1]) for _ in range(m)][-m1:] if m1 else []
    after = [k+period for k, m in zip(knots[1:], mults[1:]) for _ in range(m)][:m1]
    return before+flat+after


def array(lower: int, values: list) -> list:
    """Encode an array as 'lower count values'; array(1, [2.0]) == [1, 1, 2.0]."""
    return [lower, len(values)]+list(values)


def encode(definition: Curve, knots: list[float] | None = None) -> list:
    """Serialize for the probes' Curve reader, optionally replacing the knots."""
    poles = [definition.pole_lower, definition.count()]+definition.poles
    if definition.kind == 1:
        poles = [definition.pole_lower, len(definition.poles)]+definition.poles
    used = definition.knots if knots is None else knots
    header = [definition.kind, definition.dim, int(definition.rational), definition.degree, int(definition.periodic)]
    weights = definition.weights if definition.rational else []
    return (header+array(definition.knot_lower, used)+array(definition.knot_lower, definition.mults)
            + poles+array(definition.pole_lower, weights))
