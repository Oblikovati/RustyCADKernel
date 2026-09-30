"""Real contour roots and segments on bounded domains."""
from itertools import product
from tktopalgo_corpus_breplib_records import numbers


def append_boundary_requests(requests: list[str]) -> None:
    """All result getters and repeated operations on six surface classes."""
    for surface_kind, mode, kind, recheck, direction in product(range(6),range(3),range(3),range(2),[(1,2,3),(1,0,0)]):
        surface=[surface_kind,0,0,0,0,0,1,1,0,0,0,2,0.4,-2,2,-1,1]
        requests.append('ct_boundary '+numbers([*surface,mode,kind,recheck,*direction,4,-2,3,0.3]))
