"""Differential scripts for the Message progress classes, appended to verify_tkernel's corpus.

Each request is `msgprog <allowed_polls> <start|null> <tokens...>`, interpreted by
`tools/oracle/tkernel_message_progress_probe.cxx` and its Rust twin (the grammar is in
the probe header). The fixed scripts cover every constructor name kind, nested finite
and infinite scopes, ranges dropped unused, copied, assigned and closed, reused ranges,
forced shows, zero and fractional maxima, breaks at every poll, the deprecated sentry
(valid and refused parameters) and the lazy scope; seeded random scripts add nested
Next() sequences with irregular steps.
"""
from __future__ import annotations

import random

FIXED = [
    "-1 start",
    "-1 null r S L 3 0 [ v n 1 [ r ] v ]",
    "-1 start r",
    "-1 start x r",
    "-1 start S L 4 0 [ v n 1 [ ] n 1 [ S S 2 0 [ v n 1 [ ] v ] ] v ]",
    "-1 start S Z 2 0 [ v n 1 [ S N 3 0 [ v ] ] ]",
    "-1 start S E 2 0 [ v ] ",
    "-1 start S N 0 0 [ v n 1 [ r ] v ]",
    "-1 start S L 2.5 0 [ n 1 [ ] v n 1 [ ] v n 1 [ ] v ]",
    "-1 start S L 2 1 [ n 1 [ ] v n 1 [ ] v n 5 [ ] v c v ]",
    "-1 start S L 1 1 [ n 1 [ S L 3 1 [ n 2 [ ] v ] ] v ]",
    "-1 start S L 3 0 [ n 0 [ r ] n -1 [ r ] n 0.5 [ r ] v ]",
    "-1 start k [ r ] r",
    "-1 start k [ k [ r x r ] r ] r",
    "-1 start S L 4 0 [ a 1 2 v ]",
    "-1 start S L 4 0 [ a 3 1 v ]",
    "-1 start S L 2 0 [ w n 1 [ ] w c w v ]",
    "-1 start S L 2 0 [ c n 1 [ r ] v ]",
    "-1 start S L 3 0 [ ] S S 3 0 [ v n 1 [ ] v ] r",
    "-1 start S L 2 0 [ n 1 [ S L 2 0 [ n 1 [ ] c v ] r ] v ]",
    "-1 start S L 2 0 [ n 1 [ x S L 2 0 [ v ] ] ]",
    "0 start r S L 2 0 [ b n 1 [ r ] ]",
    "1 start S L 2 0 [ b b n 1 [ r ] ]",
    "2 start S L 5 0 [ b n 1 [ S L 2 0 [ b b ] ] ]",
    "-1 start y 0 10 1 0 0 [ v n 3 [ ] v ]",
    "-1 start y 1 10 1 0 0 [ v ]",
    "-1 start y 0 10 2 0 0 [ v ]",
    "-1 start y 0 10 1 0 0.5 [ v ] r",
    "-1 start y 0 4 1 1 0 [ n 1 [ ] v ]",
    "-1 start z 10 3 0 7",
    "-1 start z 10 1 0 10",
    "-1 start z 4 10 0 3",
    "-1 start z 5 2 1 9",
    "0 start z 10 2 0 1",
    "0 start z 10 2 0 2",
    "1 start z 10 2 0 6",
    "-1 null k [ r ] y 0 3 1 0 0 [ v b ] z 3 1 0 3",
    "-1 null y 2 3 1 0 0 [ v ]",
]


def random_scope(rng: random.Random, depth: int) -> str:
    """One random scope statement with nested Next() bodies."""
    kind = rng.choice("LSZEN")
    maximum = rng.choice([1, 2, 3, 4, 7, 10, 0.5, 2.5, 100])
    infinite = int(rng.random() < 0.25)
    body = []
    for _ in range(rng.randint(0, 5)):
        step = rng.choice([1, 1, 1, 2, 0.5, 0.25, 3, 0, -1, 1e-3])
        inner = random_scope(rng, depth + 1) if depth < 3 and rng.random() < 0.4 else ""
        tail = rng.choice(["", "r", "x", "k [ r ]"])
        body.append(f"n {step} [ {inner} {tail} ]")
        body.append(rng.choice(["", "v", "b", "w"]))
    if rng.random() < 0.2:
        body.append("c v")
    return f"S {kind} {maximum} {infinite} [ {' '.join(body)} ]"


def cases() -> list[str]:
    """Every msgprog request, fixed scripts first then 200 seeded random scripts.

    >>> len(cases()) == len(FIXED) + 200
    True
    """
    rng = random.Random(20200505)
    lines = [f"msgprog {script}" for script in FIXED]
    for _ in range(200):
        allowed = rng.choice([-1, -1, -1, 0, 1, 2, 3, 5, 8])
        lines.append(f"msgprog {allowed} start {random_scope(rng, 0)}")
    return [" ".join(line.split()) for line in lines]
