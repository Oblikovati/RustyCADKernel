"""Arc contour callbacks and saved-point sequences."""
from itertools import product
from tkgeomalgo_corpus_intpatch_tools import ARCS
from tktopalgo_corpus_breplib_records import numbers


def append_arc_function_requests(requests: list[str]) -> None:
    """Cover every arc representation, surface type and contour mode."""
    for arc, kind, radius, parameter in product(ARCS, range(6), [0, 2], [-0.2, 0.4, 1.5]):
        surface = [kind, 0,0,0, 0,0,1, 1,0,0, 0, radius,0.4, -4,4,-4,4]
        requests.append('ct_arc_function '+numbers([*surface,*arc,parameter,0.3,1,2,3,4,-2,3]))
