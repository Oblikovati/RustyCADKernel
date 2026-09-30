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
    return '\n'.join(rows) + '\n'


ORACLE = NumericOracle('TKMesh', 'tools/oracle/tkmesh_probe.cxx',
    ['TKMesh', 'TKPrim', 'TKShHealing', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase',
     'TKG3d', 'TKG2d', 'TKMath', 'TKernel'], 'rustycad-tkmesh', corpus)

if __name__ == '__main__':
    verify_numeric(ORACLE)
