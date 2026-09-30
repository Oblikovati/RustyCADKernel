"""Interior-search seeds, repeated performs and unset-result exceptions."""
from itertools import product
from tktopalgo_corpus_breplib_records import numbers


def append_search_inside_requests(requests: list[str]) -> None:
    """All search overloads for elementary and polynomial contour equations."""
    for surface_kind, mode, kind, uv in product(range(6),range(4),range(3),[(0.3,0.4),(-3,3)]):
        surface=[surface_kind,0,0,0,0,0,1,1,0,0,0,2,0.4,-2,2,-1,1]
        requests.append('ct_inside '+numbers([*surface,mode,kind,*uv,1,2,3,4,-2,3,0.3]))
