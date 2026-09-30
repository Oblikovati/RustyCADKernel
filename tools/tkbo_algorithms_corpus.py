"""IntTools utilities preserve caller state and native unstable root ordering."""
from itertools import product


def algorithm_candidates() -> list[str]:
    """Root histories and bounded geometric subdivision requests."""
    rows = ['algorithms_roots '+' '.join(map(str,[count,seed,scale,tolerance,null]))
            for count,seed,scale,tolerance,null in product([0,1,2,3,4,16,17,33,65],
                [0,1,7],[-0.1,0,5],[-1,0,0.2,10],[-1,0,0.001,0.5])]
    rows += ['algorithms_geometry '+' '.join(map(str,[kind,first,last,count,deflection,0,degenerate,*point]))
             for kind,(first,last),count,deflection,degenerate,point in product(range(15),
                 [(0,1),(1,0),(0,0)],[-2,0,1,3],[0.01,0.5],[0,1],[(0,0,0),(1,0.5,2)])]
    return rows


def undefined_radius_request(row: str) -> bool:
    """Identify calls reaching gce_MakeCirc's uninitialized coincident-point status."""
    fields=row.split()
    if fields[0]!='algorithms_geometry':
        return False
    kind,first,last,_,_,_,degenerate,*_=map(float,fields[1:])
    return first==last and degenerate==0 and int(kind) not in (0,1,2,6)


def algorithm_partition() -> tuple[list[str],list[str]]:
    """Retain every candidate while keeping native undefined status out of exact counts."""
    candidates=algorithm_candidates()
    return ([row for row in candidates if not undefined_radius_request(row)],
            [row for row in candidates if undefined_radius_request(row)])


def algorithm_requests() -> list[str]:
    """Defined-domain requests; the required companion gate replays the remaining rows."""
    return algorithm_partition()[0]
