"""Concrete EdgeEdge API histories, including repeated execution and located edges."""
from itertools import product


def edge_full_requests() -> list[str]:
    """Preserve all construction variants and retained error/result histories."""
    rows = ['edge_full '+' '.join(map(str,[a,b,first,last,fuzzy,quick,0,constructor,0]))
            for a,b,(first,last),fuzzy,quick,constructor in product([0,1,3,6,7,9,11],
                [0,1,3,6,7,9],[(0,1),(0,0)],[0,0.01],[0,1],range(3))]
    rows += ['edge_full '+' '.join(map(str,[a,b,0,1,0.01,quick,error,constructor,0]))
             for a,b,quick,error,constructor in product([0,1,3],[0,1,3],[0,1],[1,2,3],range(3))]
    rows += ['edge_full '+' '.join(map(str,[a,b,0,1,0.01,quick,0,constructor,placement]))
             for a,b,quick,constructor,placement in product([0,1,3,7,9,11],[0,1,3,7],
                 [0,1],[0,2],[-0.125,0.125])]
    return rows
