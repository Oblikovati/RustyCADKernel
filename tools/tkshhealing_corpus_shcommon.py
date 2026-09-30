"""The geometry prelude of the ShapeCustom, ShapeUpgrade and ShapeProcess probe requests
(tools/oracle/tkshhealing_probe_shcommon.cxx documents the records).

`Prelude` records surfaces and locations the TKTopAlgo shape program ops cannot describe
(indirect frames, trimmed and offset surfaces of them, mirror locations) and fixes the
table sizes of the program that follows, so the program's own indices stay right.
"""
from __future__ import annotations
from tktopalgo_shape_program import ShapeProgram, num

PLANE, CYLINDER, SPHERE, CONE, TORUS = 0, 1, 2, 3, 4
Vec = tuple[float, float, float]


def translation(v: Vec) -> list[float | int]:
    return [0, *v]


def rotation(p: Vec, d: Vec, angle: float) -> list[float | int]:
    return [1, *p, *d, angle]


def scale(p: Vec, factor: float) -> list[float | int]:
    return [2, *p, factor]


def mirror_point(p: Vec) -> list[float | int]:
    return [3, *p]


def mirror_axis(p: Vec, d: Vec) -> list[float | int]:
    return [4, *p, *d]


def mirror_plane(p: Vec, n: Vec) -> list[float | int]:
    return [5, *p, *n]


def product(first: list[float | int], second: list[float | int]) -> list[float | int]:
    return [6, *first, *second]


class Prelude:
    """The prelude records and the counts of what they add."""

    def __init__(self) -> None:
        self.records: list[list[float | int]] = []
        self.surfaces = 0
        self.locations = 1

    def elementary(self, kind: int, p: Vec, n: Vec, x: Vec, indirect: bool, *params: float) -> int:
        """An elementary surface on gp_Ax3(P, N, X), Y-reversed when indirect."""
        self.records.append([0, kind, *p, *n, *x, indirect, *params])
        self.surfaces += 1
        return self.surfaces-1

    def trimmed(self, surface: int, u1: float, u2: float, v1: float, v2: float) -> int:
        self.records.append([1, surface, u1, u2, v1, v2])
        self.surfaces += 1
        return self.surfaces-1

    def offset(self, surface: int, distance: float) -> int:
        self.records.append([2, surface, distance])
        self.surfaces += 1
        return self.surfaces-1

    def location(self, trsf: list[float | int]) -> int:
        self.records.append([3, *trsf])
        self.locations += 1
        return self.locations-1

    def program(self) -> ShapeProgram:
        """A shape program whose table indices continue after the prelude."""
        program = ShapeProgram()
        program.sizes['surface'] = self.surfaces
        program.sizes['location'] = self.locations
        return program

    def tokens(self) -> str:
        return ' '.join([str(len(self.records)), *[num(v) for record in self.records for v in record]])
