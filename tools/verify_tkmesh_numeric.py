#!/usr/bin/env python3
"""Compare the TKMesh discrete model foundations against unchanged OCCT.

Usage: python3 tools/verify_tkmesh_numeric.py --source OCCT --native-build BUILD
This foundation corpus does not claim coverage of the triangulation algorithms.
"""
from itertools import product
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """Every curve size, endpoint mode, orientation, and status mask; corpus()."""
    rows = [f'mesh_curve {count} {keep}' for count in (0, 1, 2, 3, 8, 33)
            for keep in (0, 1)]
    rows += [f'mesh_pcurve {count} {keep} {orientation}'
             for count in (0, 1, 2, 3, 8, 33) for keep in (0, 1)
             for orientation in range(4)]
    rows += [f'mesh_model {mask} {unset} {orientation}' for mask in range(512)
             for unset in (0, 1, 8, 16, 24, 256, -1) for orientation in range(4)]
    rows += [f'mesh_tools {outcome}' for outcome in (-1, 0, 1)]
    rows += [f'mesh_explorer {orientation} {duplicates}' for orientation in range(4)
             for duplicates in (0, 1)]
    rows += ['mesh_pipeline ' + ' '.join(map(str, (builder, *stages, clean, cancel)))
             for builder in (-4, -3, -2, -1, 0, 1)
             for stages in product((-2, -1, 0, 1), repeat=5)
             for clean, cancel in product((0, 1), repeat=2)]
    rows += ['mesh_shape_tool ' + ' '.join(map(str, values))
             for values in product(range(5), range(3), range(3), range(8), range(4), range(2))]
    rows += [f'mesh_shape_polygons {located} {orient}' for located, orient in product(range(2), range(4))]
    rows += ['mesh_wire_order ' + ' '.join(map(str, values))
             for values in product(range(4), range(16), range(3), range(2), range(2), range(2), range(2), range(2))]
    rows += ['mesh_outer_wire ' + ' '.join(map(str, values))
             for values in product(range(4), range(16), range(3), range(4), range(3))]
    rows += ['mesh_model_builder ' + ' '.join(map(str, values))
             for values in product(range(10), range(2), range(3), range(3), range(4))]
    rows += ['mesh_deflection ' + ' '.join(map(str, v)) for v in product(range(8), range(2), range(2), range(3), range(3), range(3))]
    rows += ['mesh_consistent ' + ' '.join(map(str, v)) for v in product(range(9), range(9), range(2), range(4))]
    rows += ['mesh_post_processor ' + ' '.join(map(str, v)) for v in product(range(-1,4), (0,4,128,132), (0,4,128,132), (0,2,3), (1,2), (0,1), (0,1))]
    rows += ['mesh_edge_parameter ' + ' '.join(map(str, v)) for v in product((4,5,6), range(6), range(2), range(4), range(2), range(2), range(3))]
    rows += ['mesh_tessellation_extractor ' + ' '.join(map(str, v)) for v in product((1,2,3), range(2), range(-2,3), range(2))]
    rows += ['mesh_curve_tessellator ' + ' '.join(map(str, v)) for v in product((4,5,6), range(4), range(2), range(4), range(2), range(2), range(2), range(2))]
    rows += ['mesh_curve_refinement ' + ' '.join(map(str, v)) for v in product(range(2), range(2), range(2), range(2), range(4), range(2), range(2))]
    rows += ['mesh_edge_discret ' + ' '.join(map(str, v)) for v in product((0,2,3,4,5,6,8,9,10,11), range(2), range(2), range(2), range(2), range(4))]
    rows += ['mesh_edge_tessellate ' + ' '.join(map(str, v)) for v in product((4,6), range(3), range(2), range(2), range(2), range(2))]
    rows += ['mesh_edge_callback']
    rows += ['mesh_range ' + ' '.join(map(str, v)) for v in product(range(3), range(6), range(2), range(2))]
    rows += ['mesh_pre ' + ' '.join(map(str, v)) for v in product((-1,0,1,2), (0,64), (0,4,64,128), range(3), range(3), range(3), range(2), range(2))]
    rows += ['mesh_pre_cone ' + ' '.join(map(str, v)) for v in product(range(2), range(3), range(2), range(2), range(6))]
    rows += ['mesh_geom_intersection ' + ' '.join(map(str, v)) for v in product(range(15), range(15), range(2), range(2))]
    rows += ['mesh_geom_surface ' + ' '.join(map(str, v)) for v in product(range(3), range(4), range(3), range(6), range(5), range(3))]
    rows += ['mesh_geom_sampling ' + ' '.join(map(str, v)) for v in product(range(2), range(2), range(3), range(2))]
    rows += ['mesh_geom_cells ' + ' '.join(map(str, v)) for v in product(range(13), range(6), range(4))]
    rows += ['mesh_range_overflow']
    rows += ['mesh_editable_tree ' + ' '.join(map(str, v)) for v in product((1,2,8,33,700), range(2), range(2))]
    rows += ['mesh_face_checker ' + ' '.join(map(str, v)) for v in product(range(-1,8), range(2), range(3), range(4), range(2), range(2), range(2))]
    rows += ['mesh_face_checker 8 0 0 0 0 0 0', 'mesh_face_segment']
    rows += ['mesh_healer ' + ' '.join(map(str, v)) for v in product((0,2,3,4,5,6,8,9), range(6), range(2), range(2), range(2))]
    rows += ['mesh_healer ' + ' '.join(map(str, v)) for v in product(range(10,19), range(2), range(2), range(2), range(2))]
    rows += ['mesh_healer ' + ' '.join(map(str, v)) for v in product((4,5), range(6,12), range(2), range(2), range(2))]
    rows += ['mesh_element_vertex ' + ' '.join(map(str, v)) for v in product(range(8), (-1,0,7), range(7), range(7), range(4))]
    rows += ['mesh_element_edge ' + ' '.join(map(str, v)) for v in product(range(9), range(9), range(7), range(7))]
    rows += ['mesh_element_triangle ' + ' '.join(map(str, v)) for v in product(range(6), range(6), range(8), range(7), range(7))]
    rows += ['mesh_element_pair ' + ' '.join(map(str, v)) for v in product(range(512), (-3,-1,0,7), (-1,0,1,2,3))]
    rows += ['mesh_cell_failure ' + ' '.join(map(str, v)) for v in product(range(5), range(2), range(2))]
    return '\n'.join(rows) + '\n'


ORACLE = NumericOracle('TKMesh', 'tools/oracle/tkmesh_probe.cxx',
    ['TKMesh', 'TKPrim', 'TKShHealing', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase',
     'TKG3d', 'TKG2d', 'TKMath', 'TKernel'], 'rustycad-tkmesh', corpus)

if __name__ == '__main__':
    verify_numeric(ORACLE)
