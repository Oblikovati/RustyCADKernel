"""Deterministic requests for the Graphic3d driver model (tools/oracle/tkservice_probe_driver.cxx).

`sv_scene` runs a seeded script on a null driver: structures with group boxes (small, large,
degenerate), display and erase, priorities (with the invalid one), Z layers (with an unknown
one), infinite and mutable states, connections (with the cycle check), visibility,
highlight, transform persistence (zoom, rotate, 2D, trihedron), translations, removal,
texts, point arrays, group removal, clearing, visuals, computed mode, activation and the
frustum culling of the layers; then it compares every structure, the view bounds and the
layers (their structures by priority, boxes and zoom persistence coefficient).
"""
from __future__ import annotations
import random

from tkservice_corpus_camera import DEFAULT, camera, numbers

BOXES = [[0, 0, 0, 1, 1, 1], [-5, -5, -5, 5, 5, 5], [10, 20, 30, 11, 21, 31], [0, 0, 0, 0, 0, 0],
         [-1e6, -1e6, -1e6, 1e6, 1e6, 1e6], [100, -50, 0, 400, 50, 10], [-3, 2, 7, -2, 2.5, 9]]
LAYERS = [0, 0, -2, -3, -4, -5, 7]
MODES = [0, 2, 8, 10, 64, 32]


class Script:
    """A scene script that keeps the removed structures out of the operations."""

    def __init__(self, rng: random.Random, with_activation: bool) -> None:
        self.rng = rng
        # Activate, Deactivate and SetComputedMode iterate pointer-hashed maps natively, so
        # their order is only deterministic with a single structure.
        self.with_activation = with_activation
        self.ops: list[list[float]] = []
        self.count = 0
        self.removed: set[int] = set()
        # Structures whose "always rendered" state may have changed: natively a displayed one
        # can stay in a layer BVH set after its erasure (Graphic3d_Layer::Remove picks the
        # container from the current state), so removing it leaves a dangling pointer.
        self.tainted: set[int] = set()

    def alive(self) -> int | None:
        choices = [i for i in range(self.count) if i not in self.removed]
        return self.rng.choice(choices) if choices else None

    def point(self) -> list[float]:
        return [self.rng.choice([0, 1, -2, 3.5, 50, -120]) for _ in range(3)]

    def create(self) -> None:
        self.ops.append([1] + self.rng.choice(BOXES))
        self.count += 1

    def structure_op(self, s: int) -> None:
        rng = self.rng
        op = rng.choice([2, 2, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18, 20, 21, 22, 23])
        if op == 14 and s in self.tainted:
            op = 3
        if op in (6, 12, 15, 17, 20, 21):
            self.tainted.add(s)
        args: list[float] = []
        if op == 4:
            args = [rng.randrange(-1, 11)]
        elif op == 5:
            args = [rng.choice(LAYERS)]
        elif op in (6, 9, 15, 20, 21):
            args = [rng.randrange(2)]
        elif op in (7, 8):
            other = self.alive()
            if other is None:
                return
            # Always with the cycle check: a cycle makes the native Network recursion endless.
            args = [other, rng.randrange(2), 1] if op == 7 else [other]
        elif op == 12:
            args = [rng.choice(MODES)] + self.point()
        elif op == 13:
            args = self.point()
        elif op == 17:
            args = self.point() + [rng.randrange(2)]
        elif op == 18:
            count = rng.randrange(0, 4)
            args = [count] + [v for _ in range(count) for v in self.point()]
        elif op == 22:
            args = [rng.randrange(1, 4)]
        elif op == 23:
            args = [rng.randrange(4)]
        if op == 14:
            self.ops.append([3, s])
            self.removed.add(s)
        self.ops.append([op, s] + args)

    def step(self) -> None:
        roll = self.rng.random()
        s = self.alive()
        if (roll < 0.15 and not self.with_activation) or s is None:
            if self.count == 0 or not self.with_activation:
                self.create()
        elif roll < 0.22:
            self.ops.append([19, self.rng.randrange(3)])
        elif roll < 0.25 and self.with_activation:
            self.ops.append([self.rng.choice([24, 25]), self.rng.randrange(2)])
        else:
            self.structure_op(s)


def append_driver_requests(requests: list[str]) -> None:
    """Append seeded scene scripts on the null driver."""
    rng = random.Random(20261014)
    for index in range(180):
        cam = DEFAULT if index < 20 else camera(rng)
        single = index >= 150
        script = Script(rng, single)
        for _ in range(1 if single else rng.randrange(2, 5)):
            script.create()
        for _ in range(rng.randrange(5, 40)):
            script.step()
        script.ops.append([19, 1])
        flat = [v for op in script.ops for v in op]
        size = rng.choice([[800, 600], [640, 480], [300, 900]])
        requests.append('sv_scene ' + numbers(cam + size + [len(script.ops)] + flat))
