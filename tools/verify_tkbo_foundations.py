#!/usr/bin/env python3
"""Compare complete IntTools foundation types with unchanged pinned TKBO.

Usage: python3 tools/verify_tkbo_foundations.py --source OCCT --native-build BUILD
"""
from itertools import product
from tkbo_algorithms_corpus import algorithm_requests
from tkbo_edge_corpus import edge_full_requests
from toolkit_numeric import NumericOracle, verify_numeric
from tkbo_upstream_gaps import verify_missing_isvalid
from verify_tkmath_numeric import selected_paths


def corpus() -> str:
    """Exercise constructors, all mutators, states, subdivisions and native hashes."""
    requests = [f'range {a} {b} {b} {a}' for a, b in product(
        [-1e100, -10.0, -0.0, 0.0, 0.125, 9.0, 1e100], repeat=2)]
    requests += [f'root -0.0 -1 0.5 2 -3.0 {a} {b} -2 3 -4 5'
                 for a, b in product(range(4), repeat=2)]
    requests += [f'curve {depth} {depth+1} {a} {b} {samples} {index} {index+1}'
                 for depth, a, b, samples, index in product(
                     [-2, 0, 1, 2, 7, 64, 1024], [-2.0, 0.0], [-3.0, 7.0],
                     [-3, 0, 1, 2, 10], [-3, 0, 1, 7])]
    requests += [f'surface {index} {depth} {index+1} {depth+1} 3 -2 4 2 -5 6'
                 for index, depth in product([-3, 0, 1, 7], [-2, 0, 1, 2, 7, 64])]
    requests += [f"face_state {a} {b} {c} {tol} {fuzzy} {n}" for a, b, c, tol, fuzzy, n in product([0,1], [0,1], [0,1], [-1,1e-7], [-1,0,1e-7,1e-3,99], [0,3])]
    requests += [f"face_perform {a} {b} {x} {y} {z} {parallel} {split}" for (a, b), x, y, z, parallel, split in product([(0, 0), (0, 1), (1, 0), (0, 3), (3, 0), (1, 3), (3, 1)], [0, 1], [0, 1], [0, 1], [0, 1], [0, 1])]
    requests += algorithm_requests()
    requests += edge_full_requests()
    requests += shrunk_requests()
    requests += marked_requests()
    requests += bopds_requests()
    requests += intersection_requests()
    requests += [f'localize {count} {minimum} {steps} {depth}'
                 for count, minimum, steps, depth in product([-1, 0, 4], [-0.0, 0.1], [0, 1, 101, 420], [-2, 0, 3])]
    requests += surface_grid_requests()
    requests += topol_requests()
    requests += fclass_requests()
    requests += context_requests()
    requests += context_compute_requests()
    requests += tools_requests()
    requests += wline_requests()
    requests += [f"wline_decompose {a} {b} {pattern} {v} {shift} {mode} {axis}" for a, b, pattern, v, shift, mode, axis in product([1, 3, 4], [0, 1, 3, 4], range(12, 15), [0, 1], [-1e-6, 1e-6], [1, 2, 4], [0, 1])]
    requests += [f"wline_decompose {a} {b} {pattern} {v} {shift} {avoid}" for a, b, pattern, v, shift, avoid in product([1, 3, 4], [0, 1, 3, 4], range(12), [0, 1.5707963267948966], [0, 6.283185307179586], range(5))]
    requests += [f"tools_topology {t} {offset} {tol} {ori}" for t, offset, tol, ori in product(
        [0, 1e-7, 5e-7, 1e-6, 1, 2], [0, 1e-7, 1], [1e-7, 0.1], range(4))]
    requests += [f"tools_surface {a} {b} {lo} {hi} {tol} {parallel}" for a, b, (lo, hi), tol, parallel in product(
        [0, 0.1], [0, 0.2], [(0, 1), (-2, 3), (1, 1)], [1e-9, 0.1], [0, 1])]
    requests += [f"tools_curves {kind} {a} {b} {tol} {pc} {other}"
                 for kind, (a, b), tol, pc, other in product(range(7), [(-2, 3), (0, 6.283185307179586)], [0, 1e-7], [0, 1], [0, 1, 6])]
    return '\n'.join(requests) + '\n'


def shrunk_requests() -> list[str]:
    """Retain range history, boxes, partial failures and endpoint tolerance policies."""
    return ['shrunk '+' '.join(map(str,[kind,*bounds,edge_tol,v_tol,v_tol*2,*second]))
            for kind,bounds,edge_tol,v_tol,second in product([0,1,2,3,4,6],
                [(0,1),(-2,3),(0,6.283185307179586)],[1e-7,1e-4,0.2],
                [1e-7,0.1,10],[(0,0),(0,1),(1,0),(-2,4)])]


def marked_requests() -> list[str]:
    """Exercise duplicate boundaries, every insertion overload, and endpoint tie rules."""
    requests = []
    for bounds, mode, first, last, hint in product(
            ['2 0 2', '3 0 1 2', '4 0 1 1 2'], range(4),
            [-1, 0, 0.25, 1, 1.5, 2], [-1, 0, 0.75, 1, 2, 3], [0, 1, 2, 7]):
        requests.append(f'marked {bounds} 0 {mode} {first} {last} 3 {hint} '
                        '2 0.5 1.5 7 1 1 1 1 9 0')
    requests += ['marked 2 0 2 0 2 1e-16 1.9999999999999998 8 1',
                 'marked 3 2 1 0 0 0 1.5 0.5 2 1']
    return requests


def bopds_requests() -> list[str]:
    """Compare all scalar BOPDS records, native hashes, and formatted dumps."""
    requests = [f'bop_range {a} {b} {b} {a} {query}'
                for a, b, query in product([-3, -1, 0, 1, 9], repeat=3)]
    requests += [f'bop_pave {a} {parameter} {b} {other}'
                 for a, b, parameter, other in product([-1, 0, 2], [-1, 0, 2],
                     [-1e20, -1.2345, -0.0, 0.0, 1.2345, 99.0],
                     [-1.2345, -0.0, 0.0, 1.2345])]
    requests += [f'bop_pair {a} {b} {c} {d}' for a, b, c, d in product([-1, 0, 2], repeat=4)]
    requests += [f'bop_point {a} {b} {c} {b} {c} {c} {a} -7'
                 for a, b, c in product([-3.0, -0.0, 0.0, 5.25], repeat=3)]
    requests += [
        'bop_pave_block 8 3 1 0 2 1 4 0.25 5 0.75 4 0.5 0.5 1e-9 1 -2 -2 -2 2 2 2 0.1 0.9 1',
        'bop_pave_block 3 3 7 1 8 2 1 1.5 2 0.5 3 1.25 0.5000000001 1e-8 0 -1 -2 -3 4 5 6 -0.5 2 0',
        'bop_pave_block -1 9 5 0 6 1 7 0.8 8 0.2 9 0.5 0.5 0.0 1 0 0 0 1 1 1 0.25 0.75 1',
    ]
    requests += [
        'bop_common_block 9 20 3 30 5 50 41 42 5 99 60 43 44 70 0.125',
        'bop_common_block 1 11 1 12 0 13 8 8 1 8 -7 8 9 -9 -0.0',
        'bop_couple_paves 3 8 12 19 -0.0',
        'bop_couple_paves -4 0 -1 77 1e-7',
    ]
    requests += [f'bop_shape_info {shape} {reference} {flag} -1 -2 -3 4 5 6 3 1 3 {gap}'
                 for shape, reference, flag, gap in product(
                     range(9), [-1, 4], [-1, 7], [-0.0, 0.25])]
    requests.append('bop_face_info')
    requests.append('bop_data_structure')
    requests.append('bop_data_structure_interferences')
    requests.append('bop_data_structure_common_block')
    requests.append('bopds_curve')
    requests.append('bopds_interf')
    requests.append('bopds_interf_common')
    requests.append('bopds_interf_ff')
    requests.append('bopds_interference_arrays')
    requests.append('bopds_solid_interferences')
    requests += ['bop_data_structure_init 0.004', 'bop_data_structure_init -0.002']
    requests += ['bop_data_structure_edges 0.0001']
    requests += ['bop_data_structure_box 0.0001']
    requests += [f'bop_data_structure_infinite 0.0001 {kind}' for kind in range(5)]
    requests += [f'bop_data_structure_seam 0.0001 {kind}' for kind in range(4)]
    return requests


def intersection_requests() -> list[str]:
    """Compare record copies and evaluation inside/outside native curve bounds."""
    requests = [f'points {a} {b} {c} {b} {a} {flag}'
                for a, b, c, flag in product([-0.0, 0.0, 2.5], [-3, 0, 4], [0, 1], [0, 1])]
    requests += [f'common {a} {b} {c} {b} {a} {c}'
                 for a, b, c in product([-0.0, 0.0, 2.5], [-3, 0, 4], [0, 1])]
    requests += [f'intersection_curve {kind} {parameter} {tol} {tol*3}'
                 for kind, parameter, tol in product(range(4), [-10, 0, 0.25, 1, 2, 3, 4, 5, 10], [0, 1e-7, -1])]
    return requests


def surface_grid_requests() -> list[str]:
    """Exercise shared arrays, strict/reversed/empty frames and native map copy order."""
    frames = [(-10, 10), (1, 3), (0, 0), (5, -5), (-3, -1)]
    requests = [f'surface_grid {u} {v} {du} {dv} {a} {b} {c} {d}'
                for u, v, du, dv, (a, b), (c, d) in product([1, 3], [1, 4], [-1, 0, 1], [-1, 0, 1], frames, frames)]
    requests += [f'surface_cache {count}' for count in [0, 1, 101, 420, 1001]]
    return requests


def topol_requests() -> list[str]:
    """Every sampling surface kind with small/large radii, infinite domains and spline poles."""
    requests = [f'topol {kind} {radius} 0 6.0 -1 1'
                for kind, radius in product(range(7), [0.005, 1, 100])]
    requests += [f'topol {kind} {radius} -2e100 2e100 -2e100 2e100'
                 for kind, radius in product([0, 1, 2, 5], [0.005, 1, 100])]
    requests += [f'topol {kind} {poles} 0 1 0 1' for kind, poles in product([7, 8], [3, 9, 12])]
    return requests


def fclass_requests() -> list[str]:
    """Real faces including circular holes, exact boundaries and shifted periodic UVs."""
    points = [(0, 0), (1, 1), (0.25, -1), (5.5, 1), (0.25-1e-8, 0), (3, 0), (-3, 0), (10, 10), (-10, -10)]
    return [f'fclass {kind} {radius} {tol} {u} {v} {tol*3} {periodic}'
            for kind, radius, tol, (u, v), periodic in product(range(14), [0.1, 2], [1e-7, 1e-3], points, [0, 1])]


def wline_requests() -> list[str]:
    """Exercise real surfaces, poles, seam endpoints and native early index bypass."""
    return [f'wline {first} {last} {radius} {u} {v} {x} {y}'
            for first, last, radius, (u,v), (x,y) in product(
                range(5), range(5), [1e-6,2],
                [(0,0),(1,0),(0,1.5707963267948966),(1,1.5707963267948966),
                 (6.283185307179586,0),(0,-1.5707963267948966)],
                [(1,0),(0,1.5707963267948966)])]


def tools_requests() -> list[str]:
    """Cover signed plane statuses, range boundaries, strictness and obtuse angles."""
    return [f'tools {a} {b} {c} {d} {t} {tol} {angle} {x} {y} {z}'
            for (a,b), (c,d), t, tol, angle, (x,y,z) in product(
                [(-1,1),(1,-1),(0,0)], [(-2,2),(0,1)], [-1,0,1,2],
                [0,1e-7,0.5], [0,0.5,1.5707963267948966,2.5],
                [(1,0,0),(-1,0,0),(1,1e-4,0),(0,0,1),(1,1,1)])]


def context_compute_requests() -> list[str]:
    """Exercise exact status codes, vertex bounds, distances and asymmetric blocks."""
    points = [(0, 0, -1), (0, 0, -1e-7), (0, 0, -1e-6), (0, 0, 0), (0, 0, 1e-7),
              (0, 0, 1), (0, 0, 2), (0, 0, 2.0000001), (0, 0, 3),
              (1, 1, 0), (3, 1, 0), (1e-7, 0, 1), (2, 0, 0), (4, 5, 6), (4, 5, 6.0000002), (0, 0, 2.0000002), (0, 0, 3.9999998)]
    return [f'context_compute {edge} {x} {y} {z} {tol} {fuzz} {curve} {a} {b}'
            for edge, (x, y, z), tol, fuzz, curve, (a, b) in product(
                range(3), points, [1e-7, 1e-5, 0.1], [0, 1e-4], range(4), [(0, 1), (-1, 4)])]


def context_requests() -> list[str]:
    """Cache identity and visible mutations, native projector lifetimes, and box classifiers."""
    points = [(0, 0, 0), (1, 1, 0), (1, 1, 1e-7), (10, -3, 4), (1, 2, 3)]
    return [f'context {kind} {radius} {x} {y} {z} {tol}'
            for kind, radius, (x, y, z), tol in product(range(14), [0.1, 2], points, [1e-7, 1e-5, 0.1])]


ORACLE = NumericOracle('TKBO-foundations', 'tools/oracle/tkbo_foundations_probe.cxx',
                       ['TKBO', 'TKPrim', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase',
                        'TKG3d', 'TKG2d', 'TKMath', 'TKernel'], 'rustycad-tkbo', corpus)

if __name__ == '__main__':
    ORACLE.evidence.mkdir(parents=True, exist_ok=True)
    (ORACLE.evidence/'report.json').unlink(missing_ok=True)
    verify_missing_isvalid(selected_paths()[1], ORACLE.evidence)
    verify_numeric(ORACLE)
    from tkbo_undefined_radius import verify_undefined_radius
    verify_undefined_radius(ORACLE, selected_paths()[0])
