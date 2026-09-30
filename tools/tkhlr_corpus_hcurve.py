"""Curve-tool calls across the shared rational/conic/wrapper catalog."""
from itertools import product
from tkgeomalgo_corpus_intpatch_tools import ARCS
from tktopalgo_corpus_breplib_records import numbers


def append_hcurve_requests(requests: list[str]) -> None:
    """Exercise every forwarding call and Contap's original sample count."""
    for arc, parameter, bounds in product(ARCS, [-1,0,0.3,1,3], [(0,1),(1,0),(0,0),(0,1e-9),(-10,10)]):
        requests.append('ct_hcurve '+numbers([*arc,parameter,*bounds]))
