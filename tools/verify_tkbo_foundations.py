#!/usr/bin/env python3
"""Compare complete IntTools foundation types with unchanged pinned TKBO.

Usage: python3 tools/verify_tkbo_foundations.py --source OCCT --native-build BUILD
"""
from itertools import product
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
    requests += marked_requests()
    requests += bopds_requests()
    requests += intersection_requests()
    requests += [f'localize {count} {minimum} {steps} {depth}'
                 for count, minimum, steps, depth in product([-1, 0, 4], [-0.0, 0.1], [0, 1, 101, 420], [-2, 0, 3])]
    return '\n'.join(requests) + '\n'


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


ORACLE = NumericOracle('TKBO-foundations', 'tools/oracle/tkbo_foundations_probe.cxx',
                       ['TKBO', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase',
                        'TKG3d', 'TKG2d', 'TKMath', 'TKernel'], 'rustycad-tkbo', corpus)

if __name__ == '__main__':
    ORACLE.evidence.mkdir(parents=True, exist_ok=True)
    (ORACLE.evidence/'report.json').unlink(missing_ok=True)
    verify_missing_isvalid(selected_paths()[1], ORACLE.evidence)
    verify_numeric(ORACLE)
