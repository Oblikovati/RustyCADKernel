"""Request lines for the NCollection lists probe family.

Each `ncoll_lists_list` / `ncoll_lists_seq` line is `<count> <values...>` followed by a
command script (see tools/oracle/tkernel_ncollection_lists_probe.cxx for the command set);
`ncoll_lists_hseq` takes two value lists.
"""
from __future__ import annotations

import random

LIST_SCRIPTS = [
    "0 rf rm ib 1 ia 1 fl ct 0 rx 0 nx",
    "0 ap 5 fl aa 6 ib 4 ia 7 rm rm rm",
    "3 10 20 30 nx ib 15 ia 25 rm nx rm fl",
    "3 10 20 30 aa 40 ib 35 rm rm it rm rm rm rm rf",
    "2 1 4 la 2 2 3 fl nx nx nx la 1 5 lb 1 9",
    "2 1 4 nx lb 2 2 3 it lb 1 0 al 2 5 6 pl 1 -1 al 0 pl 0",
    "4 1 2 3 2 rx 2 rx 2 rx 2 ct 3 ct 2 rv fl nx nx rm",
    "1 7 rv rf rf pp 8 pp 9 rv nx ia 1 ib 2 rm",
    "0 lb 0 la 0 la 2 1 2 nx la 1 3",
]

SEQ_SCRIPTS = [
    "0 fl vl 0 vl 1 at 0 rm 1 rr 1 1 ex 1 1 sp 1 ri ic 5 ec 6 pv",
    "3 1 2 3 vl 0 vl 3 vl 4 at 2 at 3 sv 2 20 sv 4 1 fl",
    "3 1 2 3 ib 0 9 ib 1 0 ib 5 9 ia 5 9 ia 4 4 ea 6 9 ea 0 -1 eb 0 9 eb 1 -2",
    "5 1 2 3 4 5 rr 0 1 rr 2 6 rr 3 2 rr 2 3 rm 0 rm 4 rm 1",
    "5 1 2 3 4 5 ex 1 5 ex 5 1 ex 2 3 ex 0 2 ex 2 6 ex 3 3",
    "4 1 2 3 4 sp 0 sp 5 sp 3 sp 1",
    "4 1 2 3 4 nx nx pv pv pv i1 pv pv nx nx i0 ri ri nx ri ri ri",
    "3 1 2 3 i1 ic 9 nx ec 8 i0 nx ic 7 rv pv pv nx",
    "2 1 2 qa 3 1 9 qa 3 0 qb 0 0 qb 0 1 5 qa 0 2 7 8 qb 3 1 6 sa 2 3 4 sq 1 0 sq 0 sa 0",
    "0 i1 pv ri nx ec 1 ic 2 fl",
]


def random_sequence_script(rng: random.Random) -> str:
    """A seeded mix of index commands whose indices straddle the valid 1..n range."""
    size = rng.randrange(0, 6)
    values = [rng.randrange(-50, 50) for _ in range(size)]
    commands: list[str] = []
    for _ in range(12):
        name = rng.choice(["vl", "at", "rm", "ib", "ia", "ea", "eb", "sv", "ex", "rr", "nx", "pv", "ic", "ri"])
        index = rng.randrange(0, size + 3)
        if name in ("vl", "at", "rm"):
            commands.append(f"{name} {index}")
        elif name in ("ex", "rr"):
            commands.append(f"{name} {index} {rng.randrange(0, size + 3)}")
        elif name in ("nx", "pv", "ri"):
            commands.append(name)
        elif name == "ic":
            commands.append(f"ic {rng.randrange(-9, 9)}")
        else:
            commands.append(f"{name} {index} {rng.randrange(-9, 9)}")
    return " ".join([str(size), *map(str, values), *commands])


def random_list_script(rng: random.Random) -> str:
    """A seeded mix of cursor commands, including edits at the end position."""
    size = rng.randrange(0, 5)
    values = [rng.randrange(-50, 50) for _ in range(size)]
    commands: list[str] = []
    for _ in range(12):
        name = rng.choice(["ap", "pp", "aa", "ib", "ia", "rx", "ct", "rf", "rm", "nx", "it", "fl"])
        if name in ("rf", "rm", "nx", "it", "fl"):
            commands.append(name)
        else:
            commands.append(f"{name} {rng.randrange(-5, 5)}")
    return " ".join([str(size), *map(str, values), *commands])


def cases() -> list[str]:
    """Deterministic lists requests: normal, degenerate and boundary inputs."""
    rng = random.Random(4242)
    lines = [f"ncoll_lists_list {script}" for script in LIST_SCRIPTS]
    lines += [f"ncoll_lists_seq {script}" for script in SEQ_SCRIPTS]
    lines += [f"ncoll_lists_list {random_list_script(rng)}" for _ in range(40)]
    lines += [f"ncoll_lists_seq {random_sequence_script(rng)}" for _ in range(60)]
    lines += ["ncoll_lists_hseq 0 0", "ncoll_lists_hseq 2 1 2 3 3 4 5", "ncoll_lists_hseq 0 1 9"]
    return lines
