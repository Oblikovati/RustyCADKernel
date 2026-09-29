"""Deterministic requests for the HatchGen package (tools/oracle/tkgeomalgo_probe_hatchgen.cxx).

IntRes2d points are written as ga_read_ipoint reads them: `u1 u2 x y` then both transitions
`position type tangent situation opposite` (type 0 In, 1 Out, 2 Touch, 3 Undecided).
`hgen_element` builds the point on element of every pair of hatching and element transitions
(the TOUCH hatching side reads IsOpposite of the element side, which raises for a non-touch
element), changes one field of a copy and compares both ways within a confusion; parameters
closer than, equal to and farther than the confusion, huge and signed-zero ones are covered.
`hgen_hatching` adds identical, near and different points on element to a point on hatching
and orders it against another one; `hgen_domain` runs the domain constructors and setters.
"""
from __future__ import annotations
import random
from tkgeomalgo_corpus_intcurve import fmt

# The probes read decimal numbers only (no nan or inf token); huge and signed-zero values
# stand for the extreme cases.
HUGE = 1.0e308
# (type, situation, opposite) of the transitions: In, Out, the Touch cases, Undecided.
HATCHING_KINDS = [(0, 0, 0), (1, 0, 0), (2, 0, 0), (2, 1, 0), (2, 2, 0), (3, 0, 0)]
ELEMENT_KINDS = [(0, 0, 0), (1, 0, 0), (2, 0, 0), (2, 0, 1), (2, 1, 1), (3, 0, 0)]


def ipoint(u1, u2, first, second, x=0.25, y=-0.5):
    """An IntRes2d point: parameters, value and the (position, kind) of both transitions."""
    (pos1, (t1, s1, o1)), (pos2, (t2, s2, o2)) = first, second
    return [u1, u2, x, y, pos1, t1, 1 if t1 == 2 else 0, s1, o1, pos2, t2, 1 if t2 == 2 else 0, s2, o2]


# One field change of a point on element: (field, value, type), field as in ga_hgen_set.
CHANGES = [(0, 0.0, -1), (1, 3.0, -1), (2, 0.5 + 1e-10, -1), (2, 0.5 + 1e-6, -1), (2, HUGE, -1), (3, 0.0, -1),
           (3, 3.0, -1), (4, 2.0, -1), (5, 3.0, -1), (6, 1.0, -1), (7, 1.0, -1), (0, 0.0, 0), (0, 0.0, 2),
           (0, 0.0, 3), (2, 0.5, 1)]


def append_element_requests(requests: list[str]) -> None:
    """Every hatching/element transition pair, with one field change each."""
    k = 0
    for pos1 in range(3):
        for kind1 in HATCHING_KINDS:
            for pos2 in range(3):
                for kind2 in ELEMENT_KINDS:
                    field, value, kind = CHANGES[k % len(CHANGES)]
                    conf = (1e-9, 1e-7, 0.0)[k % 3]
                    point = ipoint(0.25, 0.5, (pos1, kind1), (pos2, kind2))
                    requests.append('hgen_element ' + fmt([*point, k % 5, conf, field, value, kind]))
                    k += 1
    requests.append('hgen_element ' + fmt([*ipoint(-0.0, 0.0, (1, (0, 0, 0)), (1, (1, 0, 0))), 1, 0.0, 2, 0.0, -1]))
    requests.append('hgen_element ' + fmt([*ipoint(1e300, -1e-300, (0, (2, 1, 0)), (2, (2, 0, 1))), 2, 1e300,
                                           2, -1e300, -1]))


def element(u2, first, second, index, field=0, value=0.0):
    return [*ipoint(0.75, u2, first, second), index, field, value]


def append_hatching_requests(requests: list[str]) -> None:
    """Points on hatching with duplicate, near and distinct points on element."""
    rng = random.Random(9187)
    head = (0, (0, 0, 0))
    for conf in (0.0, 1e-9, 1e-3):
        dup = element(0.5, (1, (0, 0, 0)), (1, (1, 0, 0)), 1)
        near = element(0.5 + 1e-6, (1, (0, 0, 0)), (1, (1, 0, 0)), 1)
        other = element(0.5, (1, (1, 0, 0)), (0, (1, 0, 0)), 2, 6, 1.0)
        raising = element(0.5, (1, (2, 0, 0)), (1, (0, 0, 0)), 3)
        for elements, rem in (([], 1), ([dup], 1), ([dup, dup], 2), ([dup, near, other], 0), ([near, dup, raising], 3),
                              ([other, dup, near, dup], 2)):
            for u1, u_other in ((0.75, 0.75 + 1e-10), (0.75, 1.0), (0.75, 0.5), (-HUGE, HUGE)):
                values = [*ipoint(u1, 0.5, head, (1, (0, 0, 0))), len(elements)]
                for e in elements:
                    values.extend(e)
                values += [conf, *ipoint(u_other, 0.1, (2, (1, 0, 0)), (0, (0, 0, 0))), rem]
                requests.append('hgen_hatching ' + fmt(values))
    for _ in range(20):
        n = rng.randrange(6)
        values = [*ipoint(rng.uniform(-1, 1), rng.uniform(-1, 1), (rng.randrange(3), (0, 0, 0)),
                          (rng.randrange(3), (1, 0, 0))), n]
        for _ in range(n):
            values.extend(element(rng.choice((0.5, 0.5 + 1e-8, 0.25)), (rng.randrange(3), rng.choice(HATCHING_KINDS[:2])),
                                  (rng.randrange(3), rng.choice(ELEMENT_KINDS[:3])), rng.randrange(3),
                                  rng.randrange(8), rng.choice((0.0, 1.0, 2.0))))
        values += [1e-7, *ipoint(rng.uniform(-1, 1), 0.0, head, head), rng.randrange(-1, n + 2)]
        requests.append('hgen_hatching ' + fmt(values))


def append_domain_requests(requests: list[str]) -> None:
    """The domain constructors, every setter sequence and the dump indices."""
    p1 = ipoint(-1.5, 0.2, (0, (0, 0, 0)), (1, (1, 0, 0)))
    p2 = ipoint(2.5, 0.7, (2, (1, 0, 0)), (0, (2, 1, 1)))
    scripts = [[], [0], [1], [2], [3], [4], [5], [0, 1], [0, 3, 2], [0, 5, 4], [2, 5], [4, 3, 1, 0], [1, 2, 4, 3, 5],
               [0, 1, 2, 3, 4, 5, 0]]
    for k, ops in enumerate(scripts):
        requests.append('hgen_domain ' + fmt([*p1, *p2, len(ops), *ops, (k % 4) - 1]))


def append_hatchgen_requests(requests: list[str]) -> None:
    """Every HatchGen request family in a fixed order."""
    append_element_requests(requests)
    append_hatching_requests(requests)
    append_domain_requests(requests)
