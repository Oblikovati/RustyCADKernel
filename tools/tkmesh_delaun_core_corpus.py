"""Populated-mesh Delaun dependency core; no insertion/constraint pipeline claims."""
from itertools import product

def core_rows() -> list[str]:
    """Constraints, orientation, degenerate edges, circle initialization and actual deletion."""
    return ['mesh_delaun_core '+' '.join(map(str,v)) for v in product(range(6),(0,1,2,7,31),(-1,4),(1,7),(0,1),(0,1,2,5,15))]

def polygon_rows() -> list[str]:
    """Private winding/area/intersection with explicit closure and both orientations."""
    return ['mesh_delaun_polygon '+' '.join(map(str,v)) for v in product(range(6),range(3),range(7),range(2),range(4))]

def polygon_mesh_rows() -> list[str]:
    """Insert and decompose convex/concave polygon links in both orientations."""
    return [f'mesh_delaun_polygon_mesh {kind} {reverse} {frontier}'
            for kind in range(2) for reverse in range(2) for frontier in range(2)]

def super_mesh_rows() -> list[str]:
    """Enclosing triangle coordinates for square, offset, and anisotropic bounds."""
    return ['mesh_delaun_super_mesh '+' '.join(map(str,bounds)) for bounds in
            ((0,0,2,2),(-2,1,3,5),(0,0,1,4),(2,-3,7,0))]

def super_fan_rows() -> list[str]:
    """Initial fan creation for distinct points strictly inside the supertriangle."""
    return ['mesh_delaun_super_fan '+' '.join(map(str,point)) for point in
            ((1,1),(0,0),(2,-1),(1,2))]

def use_edge_rows() -> list[str]:
    """Probe the exported UseEdge method across signed integer boundary values."""
    return [f'mesh_delaun_use_edge {index}' for index in
            (-2147483648,-100,-1,0,1,100,2147483647)]

def sort_rows() -> list[str]:
    """Sort insertion vertices including equal projections and duplicate points."""
    points = [(), ((0,0),), ((2,0),(0,1)), ((0,2),(1,1),(2,0)),
              ((3,0),(0,3),(2,1),(1,2)), ((0,0),(0,0),(1,0),(0,1)),
              ((-2,4),(4,-2),(0,0),(1,1),(2,0)), ((5,1),(-3,4),(2,-2),(0,0),(4,-4),(1,1))]
    return ['mesh_delaun_sort '+str(len(row))+(' '+' '.join(f'{x} {y}' for x,y in row) if row else '')
            for row in points]

def corpus() -> str:
    """Compare every supported nonempty core path with the immutable native library."""
    return '\n'.join(core_rows()+polygon_rows()+polygon_mesh_rows()+super_mesh_rows()+super_fan_rows()+use_edge_rows()+sort_rows())+'\n'


def partition() -> tuple[list[str],list[str]]:
    """Validate every manifested native crash remains in the complete candidate corpus."""
    import json
    from porting import ROOT
    from native_crash_accounting import partition_crash_requests
    manifest=json.loads((ROOT/'tools/oracle/tkmesh_delaun_core_crashes.json').read_text())
    return partition_crash_requests(core_rows()+polygon_rows()+polygon_mesh_rows()+super_mesh_rows()+super_fan_rows()+use_edge_rows()+sort_rows(),frozenset(manifest['input_sha256']))
