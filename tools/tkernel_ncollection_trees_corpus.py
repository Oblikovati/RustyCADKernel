"""Request lines for the NCollection trees probe family.

UBTree/EBTree shapes (insertion, selection with stop, Gemmate, Kill, removal), UBTreeFiller
orders in both randomization modes (sizes crossing the 624-word mt19937 twist), CellFilter
scripts including the native Reset quirks, and KDTree Array1 builds and moves.
"""
from __future__ import annotations

import random

BoxItem = tuple[int, list[float]]


def fmt(values: list[float]) -> str:
    """Space-joined values with round-trip precision; `fmt([0.5, 1])` -> `'0.5 1'`."""
    return " ".join(repr(float(v)) if isinstance(v, float) else str(v) for v in values)


def random_box(rng: random.Random, spread: float, size: float) -> list[float]:
    """A box with its minimum corner in `[-spread, spread]^2` and extents up to `size`."""
    x, y = rng.uniform(-spread, spread), rng.uniform(-spread, spread)
    return [x, y, x + rng.uniform(0.0, size), y + rng.uniform(0.0, size)]


def item_sets(rng: random.Random) -> list[list[BoxItem]]:
    """Normal, degenerate (points, duplicates, nested, huge) and random item lists."""
    sets: list[list[BoxItem]] = [
        [],
        [(1, [0.0, 0.0, 1.0, 1.0])],
        [(1, [0.0, 0.0, 1.0, 1.0]), (2, [5.0, 5.0, 6.0, 6.0])],
        [(i, [0.0, 0.0, 0.0, 0.0]) for i in range(1, 6)],
        [(i, [float(i), float(i), float(i), float(i)]) for i in range(1, 9)],
        [(i, [-float(i), -float(i), float(i), float(i)]) for i in range(1, 7)],
        [(i, [1e300 * (i % 2), -1e300, 1e300, 1e-300 * i]) for i in range(1, 5)],
        [(7, [0.0, 0.0, 1.0, 1.0]), (7, [2.0, 2.0, 3.0, 3.0]), (8, [0.5, 0.5, 2.5, 2.5])],
    ]
    for count in [3, 10, 33, 100]:
        sets.append([(i, random_box(rng, 100.0, 10.0)) for i in range(1, count + 1)])
    return sets


def encode(items: list[BoxItem]) -> str:
    """`<n> {obj xmin ymin xmax ymax}`."""
    return " ".join([str(len(items))] + [f"{obj} {fmt(box)}" for obj, box in items])


def ubtree_cases(rng: random.Random) -> list[str]:
    """Add/select lines, plus Kill and Gemmate on reachable nodes."""
    lines: list[str] = []
    queries = [([-1e9, -1e9, 1e9, 1e9], 0), ([0.0, 0.0, 0.5, 0.5], 0), ([-50.0, -50.0, 50.0, 50.0], 2),
               ([200.0, 200.0, 300.0, 300.0], 1)]
    for items in item_sets(rng):
        for query, stop in queries:
            lines.append(f"ncoll_trees_ubtree {encode(items)} {fmt(query)} {stop}")
        if len(items) < 2:
            continue
        # Paths stop at a leaf in both probes; Kill on a leaf is a native no-op.
        for path, index in [([], 0), ([], 1), ([0], 0), ([1], 1), ([0, 0], 1), ([1, 1, 1], 0)]:
            lines.append(f"ncoll_trees_kill {encode(items)} {len(path)} {fmt(path)} {index}".replace("  ", " "))
        for path in [[], [0], [1], [0, 1]]:
            extra = f"{fmt([-1.0, -1.0, 9.0, 9.0])} 99 {fmt([8.0, 8.0, 9.0, 9.0])}"
            lines.append(f"ncoll_trees_gemmate {encode(items)} {len(path)} {fmt(path)} {extra}".replace("  ", " "))
    return lines


def ebtree_cases(rng: random.Random) -> list[str]:
    """Unique adds (duplicates refused) and removals in several orders."""
    lines: list[str] = []
    for items in item_sets(rng):
        objects = [obj for obj, _ in items]
        orders = [[], objects[:1], list(reversed(objects)), objects[::2] + [12345]]
        shuffled = objects[:]
        rng.shuffle(shuffled)
        orders.append(shuffled[: len(shuffled) // 2])
        for order in orders:
            removal = " ".join([str(len(order))] + [str(obj) for obj in order])
            lines.append(f"ncoll_trees_ebtree {encode(items)} {removal} {fmt([-1e9, -1e9, 1e9, 1e9])} 0")
    return lines


def filler_cases(rng: random.Random) -> list[str]:
    """Both randomization modes; 700 items redraw across the 624-word twist."""
    lines: list[str] = []
    for count in [1, 2, 3, 7, 64, 700]:
        items = [(i, random_box(rng, 1000.0, 5.0)) for i in range(1, count + 1)]
        for full in [1, 0]:
            lines.append(f"ncoll_trees_filler {full} {encode(items)}")
    return lines


def cell_cases(rng: random.Random) -> list[str]:
    """Scripts over the policy (-1) and explicit dimensions, purges, ranges and Resets."""
    scripts = [
        (-1, 1.0, "a 1 0.5 0.5 0 a 2 0.6 0.4 0 a 3 1.5 0.5 0 i -1 0.1 0.1 0 i 2 0.9 0.9 0 i -1 0.2 0.2 0"),
        (-1, 1.0, "A 5 0 0 0 2.5 1.5 0 I -1 0 0 0 3 3 0 R 5 1 0 0 2 1 0 I -1 0 0 0 3 3 0"),
        (-1, 2.0, "A 1 3 3 0 1 1 0 I -1 -5 -5 0 5 5 0 A 2 -3 -3 0 -1 -1 0 I 2 -4 -4 0 0 0 0 I -1 -4 -4 0 0 0 0"),
        (-1, 1.0, "a 1 3e9 0 0 a 2 -3e9 0 0 i -1 3e9 0 0 i -1 -3e9 0 0 i -1 1 0 0 i -1 -1 0 0"),
        (-1, 0.5, "a 1 1 1 0 s 1 i -1 1 1 0 a 1 1 1 0 i -1 1 1 0"),
        (-1, 1.0, "a 1 0.5 0.5 0 S 1 3 1 1 1 i -1 0.5 0.5 0 a 2 0.5 0.5 0 s 2 i -1 0.5 0.5 0"),
        (-1, 1.0, "S 0 1 4 a 3 5 5 0 s 1 i -1 5 5 0 a 4 0.5 9 9 i -1 0.9 0 0"),
        (-1, 1.0, "S 0 3 1 2 3 a 1 0.5 0.5 5 A 2 0 0 0 1 1 9 i -1 0.5 0.5 5 I -1 0 0 0 1 1 5"),
        (3, 1.0, "a 1 0.5 0.5 0.5 a 2 0.5 0.5 1.5 A 3 0 0 0 1 1 2 I -1 0 0 0 0 0 3"),
        (1, 1.0, "a 1 0.5 7 0 a 2 0.5 -7 0 i -1 0.5 0 0 A 3 -2 0 0 2 5 0 I -1 0 9 0 0 9 0"),
        (0, 1.0, "a 1 5 5 0 a 2 -5 -5 0 i -1 0 0 0 r 1 9 9 0 i -1 1 1 0"),
        (-1, 1.0, "A 1 3 0 0 1 0 0 I -1 0 0 0 5 0 0"),
    ]
    lines = [f"ncoll_trees_cell {dim} {size} {count_steps(script)} {script}"
             for dim, size, script in scripts]
    for _ in range(12):
        steps: list[str] = []
        for _ in range(20):
            kind = rng.choice("aARriI")
            point = [round(rng.uniform(-4, 4), 3) for _ in range(3)]
            other = [p + round(rng.uniform(0, 3), 3) for p in point]
            if kind in "ar":
                steps.append(f"{kind} {rng.randint(1, 6)} {fmt(point)}")
            elif kind in "AR":
                steps.append(f"{kind} {rng.randint(1, 6)} {fmt(point)} {fmt(other)}")
            elif kind == "i":
                steps.append(f"i {rng.randint(-1, 6)} {fmt(point)}")
            else:
                steps.append(f"I {rng.randint(-1, 6)} {fmt(point)} {fmt(other)}")
        lines.append(f"ncoll_trees_cell -1 {rng.choice([0.5, 1.0, 1.7])} {len(steps)} {' '.join(steps)}")
    return lines


def count_steps(script: str) -> int:
    """Number of steps in a cell script (tokens that are step letters)."""
    return sum(token in {"a", "A", "r", "R", "i", "I", "s", "S"} for token in script.split())


def kd_cases(rng: random.Random) -> list[str]:
    """Array1 builds with shifted bounds, extra and missing radii, and moves."""
    lines: list[str] = []
    for count, lower, radii_lower, radii_count in [(0, 1, 1, 0), (1, 1, 1, 1), (5, 0, 3, 5), (5, -2, 1, 7),
                                                   (5, 1, 1, 4), (40, 10, -5, 40), (3, 1, 1, 0)]:
        points = [[round(rng.uniform(-9, 9), 3), round(rng.uniform(-9, 9), 3)] for _ in range(count)]
        radii = [round(rng.uniform(0, 3), 3) for _ in range(radii_count)]
        flat = " ".join(fmt(point) for point in points)
        lines.append(f"ncoll_trees_kd_array {lower} {count} {flat} {radii_lower} {radii_count} {fmt(radii)}"
                     .replace("  ", " ").strip())
    return lines


def cases() -> list[str]:
    """Deterministic trees requests: normal, degenerate and boundary inputs."""
    rng = random.Random(20020730)
    return ubtree_cases(rng) + ebtree_cases(rng) + filler_cases(rng) + cell_cases(rng) + kd_cases(rng)
