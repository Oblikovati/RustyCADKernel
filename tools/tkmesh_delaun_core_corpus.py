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

def cleanup_rows() -> list[str]:
    """Exercise native free-triangle cleanup with each existing fixture topology."""
    return [f'mesh_delaun_cleanup {kind} {flags}' for kind in range(6)
            for flags in (0,1,2,7,31)]

def intersection_rows() -> list[str]:
    """Compare candidate constraint links against polygon edge bounds and flags."""
    return [f'mesh_delaun_intersections {kind} {flags}' for kind in range(6)
            for flags in (0,1,2,7,31)]

def next_link_rows() -> list[str]:
    """Choose boundary-walk neighbors while mutating skipped, leprous, and dead maps."""
    return [f'mesh_delaun_next_link {kind} {flags}' for kind in range(6)
            for flags in (0,1,7)]

def left_polygon_rows() -> list[str]:
    """Recover loops on empty, populated, and interior-victim meshes."""
    return [f'mesh_delaun_left_polygon {kind} {start} {forward}'
            for kind in range(5) for start in range(1,5) for forward in (0,1)]

def cleanup_victim_rows() -> list[str]:
    """Delete a populated polygon cavity containing an interior victim vertex."""
    return ['mesh_delaun_cleanup_victim']

def internal_constraint_rows() -> list[str]:
    """Recover both missing sides of one fixed boundary edge."""
    return ['mesh_delaun_internal_constraint']

def corpus() -> str:
    """Compare every supported nonempty core path with the immutable native library."""
    return '\n'.join(core_rows()+polygon_rows()+polygon_mesh_rows()+super_mesh_rows()+super_fan_rows()+use_edge_rows()+sort_rows()+cleanup_rows()+intersection_rows()+next_link_rows()+left_polygon_rows()+cleanup_victim_rows()+internal_constraint_rows())+'\n'


def partition() -> tuple[list[str],list[str]]:
    """Validate every manifested native crash remains in the complete candidate corpus."""
    import json
    from porting import ROOT
    from native_crash_accounting import partition_crash_requests
    manifest=json.loads((ROOT/'tools/oracle/tkmesh_delaun_core_crashes.json').read_text())
    return partition_crash_requests(core_rows()+polygon_rows()+polygon_mesh_rows()+super_mesh_rows()+super_fan_rows()+use_edge_rows()+sort_rows()+cleanup_rows()+intersection_rows()+next_link_rows()+left_polygon_rows()+cleanup_victim_rows()+internal_constraint_rows(),frozenset(manifest['input_sha256']))
