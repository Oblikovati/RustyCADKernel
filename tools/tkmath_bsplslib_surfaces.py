"""Valid B-spline surface definitions for the BSplSLib corpus.

Usage: definition = surface(rng, 3, 2) then encode_surface(definition).
A surface is two BSplCLib curve definitions (knots, multiplicities, degree,
periodicity per direction) plus a pole grid of NbPoles(U) x NbPoles(V) points and
a weight grid of the same shape, so every request respects the native
preconditions of the pole walks.
"""
from __future__ import annotations
from dataclasses import dataclass
import random
from tkmath_bsplclib_curves import Curve, array, curve, flat_knots
from tkmath_bsplclib_eval_corpus import last_u_knot_index


@dataclass
class Surface:
    """Directions, flags, a row-major pole grid and a weight grid."""
    u: Curve
    v: Curve
    u_rational: bool
    v_rational: bool
    has_weights: bool
    poles: list[list[list[float]]]
    weights: list[list[float]]
    row_lower: int = 1
    col_lower: int = 1

    def rows(self) -> int:
        """Number of U poles; surface(rng, 1, 1).rows() >= 2."""
        return len(self.poles)

    def cols(self) -> int:
        """Number of V poles; surface(rng, 1, 1).cols() >= 2."""
        return len(self.poles[0])


def grid(row_lower: int, col_lower: int, rows: list[list]) -> list:
    """Encode a grid as 'lowerRow rows lowerCol cols values' in row-major order."""
    values: list = []
    for row in rows:
        for cell in row:
            values += cell if isinstance(cell, list) else [cell]
    return [row_lower, len(rows), col_lower, len(rows[0])]+values


def point_grid(rng: random.Random, rows: int, cols: int) -> list[list[list[float]]]:
    """Random points in [-10, 10]^3; point_grid(rng, 2, 3)[1][2] has three coordinates."""
    return [[[rng.uniform(-10, 10) for _ in range(3)] for _ in range(cols)] for _ in range(rows)]


def weight_grid(rng: random.Random, rows: int, cols: int, constant: bool = False) -> list[list[float]]:
    """Positive weights, or one repeated weight (a locally polynomial surface)."""
    if constant:
        value = rng.choice([1.0, 2.5])
        return [[value]*cols for _ in range(rows)]
    return [[rng.uniform(0.3, 3.0) for _ in range(cols)] for _ in range(rows)]


def first_u_knot_index(degree: int, mults: list[int]) -> int:
    """BSplCLib::FirstUKnotIndex for 1-based multiplicities; first_u_knot_index(3, [4, 4]) == 1."""
    index, sigma = 1, mults[0]
    while sigma <= degree:
        index += 1
        sigma += mults[index-1]
    return index


def direction(rng: random.Random, degree: int, periodic: bool, clamped: bool, style: str) -> Curve:
    """One direction with at least one valid span.

    Unclamped ends can leave FirstUKnotIndex >= LastUKnotIndex; the native local
    knot walk then leaves entries of its stack buffer unwritten (undefined reads).
    """
    while True:
        definition = curve(rng, degree, periodic, clamped=clamped, style=style)
        if periodic or first_u_knot_index(degree, definition.mults) < last_u_knot_index(degree, definition.mults):
            return definition


def surface(rng: random.Random, u_degree: int, v_degree: int, u_periodic: bool = False,
            v_periodic: bool = False, rational: bool = False, clamped: bool = True,
            style: str = 'random') -> Surface:
    """A valid surface; rational ones flag both directions; surface(rng, 3, 2)."""
    u = direction(rng, u_degree, u_periodic, clamped, style)
    v = direction(rng, v_degree, v_periodic, clamped, style)
    rows, cols = u.count(), v.count()
    weights = weight_grid(rng, rows, cols, rational and rng.random() < 0.2)
    return Surface(u, v, rational, rational, rational, point_grid(rng, rows, cols), weights)


def encode_surface(definition: Surface, flat: bool = False) -> list:
    """Serialize for the probes' surface reader; `flat` sends flat knots without mults."""
    u, v = definition.u, definition.v
    u_knots = flat_knots(u) if flat else u.knots
    v_knots = flat_knots(v) if flat else v.knots
    header = [u.degree, v.degree, int(u.periodic), int(v.periodic), int(definition.u_rational),
              int(definition.v_rational), int(not flat)]
    return (header+array(1, u_knots)+array(1, u.mults)+array(1, v_knots)+array(1, v.mults)
            + grid(definition.row_lower, definition.col_lower, definition.poles)
            + [int(definition.has_weights)]
            + grid(definition.row_lower, definition.col_lower, definition.weights))


def parameter(rng: random.Random, knots: list[float], outside: bool = True) -> float:
    """Inside, on a knot, next to a knot or (optionally) outside the knot range."""
    span = knots[-1]-knots[0]
    choices = [rng.uniform(knots[0], knots[-1]), rng.choice(knots), rng.choice(knots)*(1+1e-16)]
    if outside:
        choices.append(rng.uniform(knots[0]-span, knots[-1]+span))
    return rng.choice(choices)
