"""Populated-mesh Delaun dependency core; no insertion/constraint pipeline claims."""
from itertools import product

def core_rows() -> list[str]:
    """Constraints, orientation, degenerate edges, circle initialization and actual deletion."""
    return ['mesh_delaun_core '+' '.join(map(str,v)) for v in product(range(6),(0,1,2,7,31),(-1,4),(1,7),(0,1),(0,1,2,5,15))]

def polygon_rows() -> list[str]:
    """Private winding/area/intersection with explicit closure and both orientations."""
    return ['mesh_delaun_polygon '+' '.join(map(str,v)) for v in product(range(6),range(3),range(7),range(2),range(4))]

def corpus() -> str:
    """Compare every supported nonempty core path with the immutable native library."""
    return '\n'.join(core_rows()+polygon_rows())+'\n'


def partition() -> tuple[list[str],list[str]]:
    """Validate every manifested native crash remains in the complete candidate corpus."""
    import json
    from porting import ROOT
    from native_crash_accounting import partition_crash_requests
    manifest=json.loads((ROOT/'tools/oracle/tkmesh_delaun_core_crashes.json').read_text())
    return partition_crash_requests(core_rows()+polygon_rows(),frozenset(manifest['input_sha256']))
