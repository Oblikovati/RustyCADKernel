#!/usr/bin/env python3
"""Compare complete IntTools foundation types with unchanged pinned TKBO.

Usage: python3 tools/verify_tkbo_foundations.py --source OCCT --native-build BUILD
"""
from itertools import product
from toolkit_numeric import NumericOracle, verify_numeric


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


ORACLE = NumericOracle('TKBO-foundations', 'tools/oracle/tkbo_foundations_probe.cxx',
                       ['TKBO', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase',
                        'TKG3d', 'TKG2d', 'TKMath', 'TKernel'], 'rustycad-tkbo', corpus)

if __name__ == '__main__':
    verify_numeric(ORACLE)
