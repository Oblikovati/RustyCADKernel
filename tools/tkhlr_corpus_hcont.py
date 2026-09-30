"""Contour sampling's shared state and all arc operations."""
from itertools import product
from sys import float_info
from tkgeomalgo_corpus_intpatch_tools import ARCS
from tktopalgo_corpus_breplib_records import numbers


def append_hcont_requests(requests: list[str]) -> None:
    """Reset native statics per request before testing state changes or failures."""
    baseline = [0, 1, 2, 9, 18, 2, 2, 1, 1, 3, 4, 0]
    infinite = float_info.max
    bounds = [(1, 2, 9, 18), (9, 18, 1, 2), (-infinite, -infinite, infinite, infinite),
              (-infinite, 2, 9, infinite), (1, -infinite, infinite, 18),
              (-2e100, -2e100, 2e100, 2e100), (2, 3, 2, 3)]
    counts = [(2, 2, 1, 1, 3, 4), (4, 8, 3, 3, 5, 7), (0, 0, 0, 0, 2, 2)]
    for kind, extent, count, failure in product(range(11), bounds, counts, range(12)):
        requests.append('ct_sampling '+numbers([*baseline, kind, *extent, *count, failure]))
    for arc, point, tolerance in product(ARCS, [(0,0),(0.2,0.7),(3,-2)], [0,1e-7,0.1]):
        requests.append('ct_hcont_arc '+numbers([*arc,*point,tolerance]))
