"""Real contour walks from known geometric roots."""
from itertools import product
from math import pi
from tktopalgo_corpus_breplib_records import numbers


def append_walking_requests(requests: list[str]) -> None:
    """Open/closed starts, reversal, hole filling and unperformed queries."""
    for kind, mode, fill, reverse, seeds in product(range(6),range(3),range(2),range(2),range(3)):
        surface=[kind,0,0,0,0,0,1,1,0,0,0,2,0.4,-2,2,-1,1]
        u,v=(0.0,0.0) if kind in (0,5) else (pi/2,0.0)
        paths=[1,u,v,0,int(kind==0),0] if seeds==1 else [0]
        loops=[1,u,v] if seeds==2 else [0]
        requests.append('ct_walking '+numbers([*surface,1,0,0,1e-7,0.01,0.1,fill,reverse,mode,*paths,*loops]))
