"""Contap boundary records with original handle and copied limit behavior."""
from itertools import product
from tkgeomalgo_corpus_intpatch_tools import ARCS
from tktopalgo_corpus_breplib_records import numbers


def append_search_record_requests(requests: list[str]) -> None:
    """Every record overload with varied parameters and all arc representations."""
    for arc, parameter, tolerance in product(ARCS, [-1,0,0.3,2], [-1e-7,0,1e-7]):
        requests.append('ct_search_records '+numbers([*arc,1,2,3,tolerance,parameter]))
