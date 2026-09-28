#!/usr/bin/env python3
"""Compare implemented TKMath numeric and storage behavior with the pinned kernel.

Usage: python3 tools/verify_tkmath_numeric.py (after building the geometry dependency closure).
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import re
import subprocess
from porting import ROOT, snapshot
from tkmath_system_corpus import append_system_requests
from tkmath_poly_corpus import append_laguerre_requests
from tkmath_cslib_corpus import append_cslib_requests
from tkmath_plib_corpus import append_plib_requests
from tkmath_elementary_corpus import append_elementary_requests
from tkmath_bvh_corpus import append_bvh_requests
from tkmath_bvh_query_corpus import append_bvh_query_requests
from tkmath_bvh_tool_classes_corpus import append_bvh_tool_classes_requests
from tkmath_bsplclib_corpus import append_bsplclib_requests
from tkmath_bsplslib_corpus import append_bsplslib_requests
from tkmath_gp_planar_corpus import append_gp_planar_requests
from tkmath_mathpoly_utils_corpus import append_mathpoly_utils_requests
from tkmath_ncollection_vec_corpus import append_ncollection_vec_requests
from tkmath_gp_transform_corpus import append_gp_transform_requests
from tkmath_convert_corpus import append_convert_requests
from tkmath_gp_core_corpus import append_gp_core_requests
from tkmath_bnd_corpus import append_bnd_requests


def run(command: list[str], log: Path, requests: str | None = None) -> str:
    """Capture a checked probe invocation; run(command, log, input_text)."""
    environment = dict(os.environ, LC_ALL='C', LANG='C')
    environment['PATH'] = str(Path.home()/'.cargo/bin')+os.pathsep+environment['PATH']
    result = subprocess.run(command, input=requests, text=True, capture_output=True,
                            cwd=ROOT, env=environment, check=False)
    log.write_text(result.stdout+result.stderr)
    result.check_returncode()
    return result.stdout


def corpus() -> str:
    """Cover scalar algebra and range branches within defined native integer-cast domains."""
    rng = random.Random(20160607)
    requests = []
    for _ in range(256):
        values = [rng.uniform(-100,100) for _ in range(7)]
        requests.append('xyz '+' '.join(map(str,values)))
        bounds = sorted(values[:2]); other = sorted(values[2:4])
        requests.append('range '+' '.join(map(str,bounds+other+[values[4],rng.choice([0,1,4,-4,20])])) )
    for first,last in [(0,-1),(3,15),(5,5),(-10,-2),(1e-160,2e-160)]:
        for value in [first,last,0,1,3,5,20]:
            for period in [0,4,-4]:
                requests.append(f'range {first} {last} 4 9 {value} {period}')
    requests += ['xyz 0 0 0 1 2 3 2','xyz 1e-160 0 0 0 1e-160 0 2',
                 'xyz -0.0 0.0 -0.0 0.0 -0.0 0.0 -2']
    for _ in range(256):
        entries = ' '.join(str(rng.uniform(-5,5)) for _ in range(9))
        requests.append(f'matrix {entries} {rng.uniform(-4,4)} {rng.randint(-5,5)}')
    requests += ['matrix 0 0 0 0 0 0 0 0 0 0 -1',
                 'matrix 1 0 0 0 1 0 0 0 1 2 0',
                 'matrix 1 2 3 2 4 6 0 0 0 1 -2']
    for _ in range(512):
        coordinates = [rng.uniform(-20,20) for _ in range(9)]
        parameters = [rng.uniform(-4,4) for _ in range(3)]+[rng.randint(-6,6)]
        requests.append('point '+' '.join(map(str,coordinates+parameters)))
    for _ in range(256):
        requests.append('direction '+' '.join(str(rng.uniform(-10,10)) for _ in range(10)))
    requests += ['direction 0 0 1 0 1 0 0 0 0 0',
                 'direction 1 0 0 1 0 0 0 0 0 0',
                 'direction 1 0 0 -1 0 0 0 0 0 0']
    for _ in range(256):
        requests.append('frame '+' '.join(str(rng.uniform(-10,10)) for _ in range(10)))
    for _ in range(256):
        requests.append('planar '+' '.join(str(rng.uniform(-10,10)) for _ in range(9)))
    requests += ['planar 1 0 0 0 -1 0 0 0 1e-12', 'planar 1 0 0 0 0 -1 0 0 1e-12']
    for index in range(512):
        values = [rng.uniform(-10,10) for _ in range(15)]
        if index % 4 != 0:
            values[6:9] = [[1,0,0],[0,1,0],[0,0,1]][index % 4 - 1]
        requests.append('bounds '+' '.join(map(str,values+[index % 66])))
    for _ in range(256):
        requests.append('line_plane '+' '.join(str(rng.uniform(-10,10)) for _ in range(12)))
    for index in range(512):
        values = [rng.uniform(-10,10) for _ in range(12)]
        if index % 3 != 0: values[6:8] = [1,0] if index % 3 == 1 else [0,1]
        requests.append('bounds2 '+' '.join(map(str,values+[index % 18])))
    for _ in range(256):
        requests.append('sphere '+' '.join(str(rng.uniform(-10,10)) for _ in range(12)))
    for index in range(512):
        values = [rng.uniform(-10,10) for _ in range(12)]
        if index % 4 != 0:
            for coordinate in [2,3,6,7]: values[coordinate] = abs(values[coordinate])
        requests.append('compact2 '+' '.join(map(str,values)))
    for index in range(512):
        values = [rng.uniform(-10,10) for _ in range(18)]
        if index % 4 != 0:
            for coordinate in [3,4,5,9,10,11]: values[coordinate] = abs(values[coordinate])
            values[6:9] = [[1,0,0],[0,1,0],[0,0,1]][index % 4 - 1]
        requests.append('compact3 '+' '.join(map(str,values)))
    for index in range(256):
        values = [rng.uniform(-10,10) for _ in range(14)]
        values[12] = rng.randint(-6,6)
        requests.append('scale '+' '.join(map(str,values)))
    for scale in [0,-0.0,1,-1,2,-2]:
        requests.append(f'scale 1 2 3 0 0 0 0 0 1 0.5 {scale} 2 4 3')
    for index in range(520):
        values = [rng.uniform(-10,10) for _ in range(10)]+[index % 26]
        requests.append('quaternion '+' '.join(map(str,values)))
    for order in range(26):
        for quaternion in ['0 0 0 1','1 0 0 0','0 1 0 0','0 0 1 0','0 0 0 0']:
            requests.append(f'quaternion {quaternion} 0 0 0 1 0 0 {order}')
    for index in range(128):
        rows,cols = rng.randint(1,10),rng.randint(1,10)
        requests.append(f'table {rows} {cols} {rng.randint(-10,10)} {rng.randint(-10,10)} {rng.uniform(-100,100)}')
    for _ in range(128):
        requests.append('nodes '+' '.join(str(rng.uniform(-1e10,1e10)) for _ in range(6)))
    requests += ['nodes -0.0 0.0 -0.0 0.0 -0.0 0.0', 'nodes 1e100 -1e100 1e-100 -1e-100 1.0000000596046448 -1.0000000596046448']
    for index in range(256):
        values = [rng.uniform(-10,10) for _ in range(4)]
        requests.append(f'numeric_vector {rng.randint(1,40)} {rng.choice([0,1,2,5])} '+' '.join(map(str,values)))
    requests += ['numeric_vector 4 1 0 0 0 0', 'numeric_vector 5 1 1e-160 1e-160 1e-160 2.220446049250313e-16', 'numeric_vector 8 0 -0.0 0.0 -0.0 -2']
    for index in range(256):
        n = rng.randint(1,8)
        entries = [rng.uniform(-10,10) for _ in range(n*n+n)]
        if index % 5 == 0 and n > 1: entries[n:2*n] = entries[:n]
        requests.append(f'numeric_matrix {n} {rng.randint(-3,3)} {rng.randint(-3,3)} {rng.choice([1e-20,1e-3,1.0,-1.0])} {rng.uniform(-5,5)} '+' '.join(map(str,entries)))
    requests += ['numeric_matrix 2 1 1 1e-20 0 0 0 0 0 1 2', 'numeric_matrix 2 1 1 -1 2 0 0 0 0 1 2', 'numeric_matrix 2 1 1 1e-20 2 1e-21 0 0 1e-19 1 2', 'numeric_matrix 2 1 1 1e-20 2 1e300 1e300 -1e300 -1e300 1 2']
    for index in range(128):
        rows = rng.randint(1,8); cols = rng.randint(1,rows)
        entries = [rng.uniform(-10,10) for _ in range(rows*cols+rows)]
        if index % 5 == 0 and cols > 1:
            for row in range(rows): entries[row*cols+1] = entries[row*cols]
        requests.append(f'least_squares {rows} {cols} {rng.randint(-3,3)} {rng.randint(-3,3)} {rng.choice([1e-20,1e-3,1.0,-1.0])} '+' '.join(map(str,entries)))
    requests += ['least_squares 1 1 1 1 1e-20 0 1', 'least_squares 2 2 1 1 -1 0 0 0 0 1 2', 'least_squares 2 2 1 1 1e-20 1e-21 0 0 1e-19 1 2']
    for case in range(256):
        rows, cols = rng.randint(1,8), rng.randint(1,8)
        entries = [rng.uniform(-10,10) for _ in range(rows*cols)]
        if case%4 == 0 and cols>1:
            for row in range(rows): entries[row*cols+1] = entries[row*cols]
        if case%11 == 0: entries = [0.0]*(rows*cols)
        entries += [rng.uniform(-10,10) for _ in range(rows)]
        requests.append(f'svd {rows} {cols} {rng.randint(-3,3)} {rng.randint(-3,3)} {rng.choice([0.0,1e-6,0.2,2.0])} 1e-12 '+' '.join(map(str,entries)))
    for case in range(256):
        n = rng.randint(1,16)
        scale = rng.choice([1e-100,1e-10,1.0,1e10,1e100])
        diagonal = [rng.uniform(-10,10)*scale for _ in range(n)]
        subdiagonal = [rng.uniform(-10,10)*scale for _ in range(n)]
        if case%4 == 0: subdiagonal = [0.0]*n
        if case%7 == 0: diagonal = [scale]*n
        requests.append(f'eigen {n} {rng.randint(-5,5)} '+' '.join(map(str,diagonal+subdiagonal)))
    requests += [f'kronrod {order}' for order in list(range(1,33))+[63,64,125]]
    for case in range(256):
        coefficients = [rng.uniform(-3,3) for _ in range(4)]
        if case%3 == 0: coefficients[0] = 0.0
        if case%5 == 0: coefficients[1] = 0.0
        first, second = rng.uniform(-10,10), rng.uniform(-10,10)
        requests.append(f'root {first} {second} {rng.choice([1e-15,1e-10,1e-3,0.0])} {rng.choice([0,1,5,100])} {rng.choice([1e-15,1e-12,0.0])} {case%2} '+' '.join(map(str,coefficients)))
    for case in range(256):
        first, second = rng.uniform(-10,10), rng.uniform(-10,10)
        center, scale, offset = rng.uniform(-100,100), rng.uniform(0.1,10), rng.uniform(-10,10)
        requests.append(f'minimum {first} {second} {center} {scale} {offset} {case%4} -10 10 {rng.choice([0,0,0,1,2,3,4,5])}')
    for case in range(128):
        n = rng.randint(1,5)
        coefficients = [rng.uniform(-3,3) for _ in range(n*n)]
        if case%7 == 0: coefficients = [0.0]*(n*n)
        quadratic = [rng.uniform(-2,2) if case%2 else 0.0 for _ in range(n)]
        rhs_start = [rng.uniform(-3,3) for _ in range(2*n)]
        requests.append(f'newton_set {n} 1e-8 1e-8 {rng.choice([0,1,10,50])} {rng.choice([0.0,1.0,100.0])} {rng.choice([0,0,0,1,2,4])} '+' '.join(map(str,coefficients+quadratic+rhs_start)))
    for case in range(256):
        n,m = rng.randint(1,4),rng.randint(1,4)
        coefficients = [rng.uniform(-3,3) for _ in range(n*m)]
        if case%7 == 0: coefficients = [0.0]*(n*m)
        quadratic = [rng.uniform(-2,2) if case%2 else 0.0 for _ in range(m)]
        rhs_start = [rng.uniform(-3,3) for _ in range(m+n)]
        requests.append(f'function_set {n} {m} 1e-8 {rng.choice([0,1,10,30])} {rng.choice([0.0,1.0,100.0])} {rng.choice([0,0,0,1,2,4])} {int(case%5==0)} '+' '.join(map(str,coefficients+quadratic+rhs_start)))
    for case in range(256):
        n = rng.randint(1,5)
        centers = [rng.uniform(-3,3) for _ in range(n)]
        weights = [rng.uniform(0.1,5) for _ in range(n)]
        start = [rng.uniform(-5,5) for _ in range(n)]
        if case%11 == 0: start = centers[:]
        width = rng.choice([-1.0,0.0,1.0,10.0])
        requests.append(f'optimization {n} {case%2} {rng.choice([0.0,-10.0,10.0])} {rng.choice([0,0,1,2,3,10])} {rng.choice([0,1,5,100])} {rng.choice([1e-6,1e-10])} 1e-12 {width} {case%2} '+' '.join(map(str,centers+weights+start)))
    for case in range(256):
        columns = rng.randint(1,6); rows = rng.randint(1,columns)
        equalities = rng.randint(0,rows); inequalities = rows-equalities
        coefficients = [rng.uniform(-3,3) for _ in range(rows*columns)]
        if case%13 == 0: coefficients = [0.0]*len(coefficients)
        rhs_start = [rng.uniform(-3,3) for _ in range(rows+columns)]
        requests.append(f'uzawa {rows} {columns} {equalities} {inequalities} {rng.choice([1e-3,1e-6,1e-10])} {rng.choice([1e-3,1e-6,1e-10])} {rng.choice([0,1,2,10,500])} {case%2} '+' '.join(map(str,coefficients+rhs_start)))
    requests.append('uzawa 1 1 0 1 100 0 500 1 1 -10 0')
    for case in range(512):
        degree = 1+case%4
        if case<256:
            coefficients = [rng.uniform(-30,30) for _ in range(degree+1)]
            coefficients[0] *= rng.choice([0.0,1e-30,1e-15,1.0,1e15])
        else:
            roots = [rng.uniform(-30,30) for _ in range(degree)]
            if case%7==0: roots = [roots[0]]*degree
            coefficients = [rng.choice([1.0,1e-20,1e20])]
            for root in roots:
                expanded = [0.0]*(len(coefficients)+1)
                for index,coefficient in enumerate(coefficients):
                    expanded[index] += coefficient; expanded[index+1] -= coefficient*root
                coefficients = expanded
        requests.append(f'polynomial {degree} '+' '.join(map(str,coefficients)))
    for degree in range(1,5):
        requests.append(f'polynomial {degree} '+' '.join(['0']*(degree+1)))
    requests += ['polynomial 3 1 1e60 1 1', 'polynomial 4 1 0 -5 0 4',
                 'polynomial 4 1e-15 1e-13 1 -2 1', 'polynomial 3 1 0 -3 0',
                 'polynomial 3 1 0 0 0', 'polynomial 2 1 -0.0 0.0']
    for case in range(512):
        coefficients = [rng.uniform(-5,5) for _ in range(5)]
        if case%8==0: coefficients = [0.0]*5
        if case%8==1: coefficients[0]=coefficients[2]=coefficients[4]=0.0
        if case%8==2: coefficients[0]=coefficients[3]=coefficients[4]=0.0
        if case%8==3: coefficients[0]=coefficients[1]=0.0
        if case%8==4: coefficients = [value*1e-12 for value in coefficients]
        lower = 0.0 if case%3==0 else rng.uniform(-20,20)
        upper = 6.283185307179586 if case%3==0 else lower+rng.uniform(-1,20)
        requests.append(f'trig {case%3} '+' '.join(map(str,coefficients+[lower,upper])))
    requests += ['trig 2 0 1 0 -2 0 0 6.283185307179586',
                 'trig 2 0 1 -2 0 0 0 6.283185307179586',
                 'trig 2 0 1 0 2 0 0 6.283185307179586',
                 'trig 2 0 1 2 0 0 0 6.283185307179586',
                 'trig 2 1 0 1 0 0 -1.7976931348623157e308 1.7976931348623157e308',
                 'trig 2 1 0 0 0 -0.5 1 1.7976931348623157e308',
                 'trig 2 1 0 0 0 -0.5 -1.7976931348623157e308 1',
                 'trig 2 1e60 1 -1e60 1 1 0 6.283185307179586']
    for case in range(256):
        mode = case % 5
        center = rng.uniform(-1, 1)
        offset = [0.0, 1e-12, -1e-12, 0.25, -0.25][(case//5)%5]
        scale = [-2.0, -0.01, 0.0, 0.125, 1.0, 3.0][case%6]
        lower, upper = (-4.0, 5.0) if case%2 else (5.0, -4.0)
        samples = [0, 5, 10, 23][case%4]
        fail = [0, 1, 17, 25, 59][(case//7)%5]
        epsx = [0.0, 1e-12, 1e-3][case%3]
        epsf = [0.0, 1e-10, 1e-3][(case//3)%3]
        null = 1e-6 if case%19 == 0 else 0.0
        target = 0.0 if case%3 else offset
        requests.append('sampled '+' '.join(map(str,[mode,fail,center,offset,scale,lower,upper,samples,epsx,epsf,null,target])))
    requests += ['sampled 1 0 0 0 1 -2 2 20 0 0 0 0',
                 'sampled 1 0 0.731 0 -1 -2 2 20 1e-10 1e-10 0 0',
                 'sampled 1 0 0.731 -4 1 -2 2 20 1e-10 1e-10 0 -4',
                 'sampled 0 0 2 0 1 2 2 0 0 0 0 0']
    for case in range(192):
        mode = [0,1,2,3,4,5,6][case%7]
        center = rng.uniform(-0.5,0.5)
        offset = [0.0,0.0,1e-9,0.1][case%4]
        scale = [-1.0,0.125,1.0,2.0][(case//4)%4]
        lower, upper = [(-4.0,5.0),(-0.25,4.0),(-4.0,0.25),(-0.25,0.25)][case%4]
        samples = [3,11,21,31][(case//3)%4]
        fail = [0,0,0,1,23,59][(case//5)%6]
        eps = [1e-8,1e-6,1e-3][case%3]
        requests.append('all_roots '+' '.join(map(str,[mode,fail,center,offset,scale,lower,upper,samples,eps,eps,eps*.01])))
    requests += ['all_roots 5 0 0 0 1 -4 4 31 1e-6 1e-6 1e-6',
                 'all_roots 6 0 0 0 1 -4 4 41 1e-6 1e-6 1e-6',
                 'all_roots 4 0 0 0 1 0 1 11 1e-6 1e-6 1e-6']
    for order in range(1,130):
        requests.append(f'quadrature {order}')
    for case in range(128):
        mode = [0,1,2,3,4,5,6][case%7]
        center = rng.uniform(-.5,.5)
        offset = [0.0,0.125,1.0,-2.0][case%4]
        scale = [-1.0,0.125,1.0,2.0][(case//4)%4]
        lower,upper = [(-4.0,5.0),(0.0,1.0),(1.0,0.0),(1.0,1.0)][case%4]
        order = [3,4,7,15,31,61,125,129][(case//3)%8]
        fail = [0,0,1,5,17,53][(case//5)%6]
        tolerance = [1e-6,1e-8,1e-10][case%3]
        limit = [1,2,10,30][(case//7)%4]
        requests.append('integration '+' '.join(map(str,[mode,fail,center,offset,scale,lower,upper,order,tolerance,limit])))
    for seed in [0,1,2,2147483647,2147483648,4294967295]:
        requests.append(f'bullard {seed}')
    for case in range(128):
        dimension = 1+case%3
        mode = (case//3)%3
        count = [1,3,7,12][case%4]
        limit = [0,1,3,17,25,40][(case//4)%6]
        lower,upper = rng.uniform(-2,-.5),rng.uniform(.5,2)
        step = [0.25,0.5,1.0,3.0][(case//5)%4]
        requests.append('pso '+' '.join(map(str,[dimension,mode,count,limit,lower,upper,step])))
    for case in range(64):
        dimension = 1+case%2
        shape = (case//2)%3
        failure = [0,0,1,2,3][(case//6)%5]
        capability = case%3
        continuity = (case//3)%3
        locked,single = case%2,(case//4)%2
        shift = rng.uniform(-.3,.3)
        lower,upper = -1.0,2.0
        constant = [0.1,9.0,50.0][case%3]
        tolerance = [.1,.2,.3][case%3]
        requests.append('global '+' '.join(map(str,[dimension,shape,failure,capability,continuity,locked,single,shift,lower,upper,constant,tolerance,1e-7])))
    requests += ['global 2 2 0 0 0 1 0 0 -1 2 9 .03 1e-7',
                 'global 1 0 0 2 2 0 1 0 1 1.0000000001 9 .1 1e-7']
    for _ in range(256):
        length = rng.randrange(1, 33)
        values = [rng.uniform(-1, 1) * 10.0**rng.randrange(-300, 301) for _ in range(2*length)]
        requests.append('utils_core '+str(length)+' '+' '.join(map(str,values)))
    requests += ['utils_core 3 1e308 1e308 1 -1e308 1e308 1',
                 'utils_core 3 1e16 1 -1e16 1 1 1', 'utils_core 1 0 0']
    for index in range(128):
        values = [index%5, rng.uniform(-10,10), rng.uniform(-10,10),
                  rng.uniform(-10,10), index%12, -5 if index%2 else 0, 5 if index%2 else 0]
        requests.append('utils_bracket '+' '.join(map(str,values)))
    for index in range(128):
        values = [rng.uniform(-30,30),rng.uniform(-20,20),rng.uniform(-5,5),
                  rng.uniform(0.1,100),1e-8, index%32, index%15]
        requests.append('utils_line '+' '.join(map(str,values)))
    for index in range(256):
        values = [index%7,rng.uniform(-5,5),rng.uniform(-5,5),rng.uniform(-10,0),rng.uniform(0,10),
                  1e-12 if index%19 else 0,1e-10 if index%3 else 0,index%40,index%11]
        requests.append('modern_root '+' '.join(map(str,values)))
    for index in range(256):
        values = [rng.uniform(-10,10) for _ in range(5)]+[index%20]
        if index%5 == 0: values[1] = index%6-3
        if index%7 == 0: values[0] = 0
        requests.append('scalar_functor '+' '.join(map(str,values)))
    for index in range(256):
        values = [rng.uniform(-10,10) for _ in range(6)]+[index%20]
        requests.append('vector_functor '+' '.join(map(str,values)))
    for case in range(128):
        values = [case % 5, rng.choice([0,0,0,1,2,3,5,10]), rng.choice([0,1,3,20]),
                  rng.uniform(-3,3), rng.uniform(-5,5), rng.uniform(-5,5),
                  rng.choice([0.0,1e-6,1e-10]), rng.choice([0.0,1e-8,1e-5])]
        requests.append('modern_opt '+' '.join(map(str,values)))
    requests += [line.replace('modern_root ', 'scalar_opt ', 1)
                 for line in requests if line.startswith('modern_root ')]
    requests += [line.replace('modern_opt ', 'modern_frpr ', 1)
                 for line in requests if line.startswith('modern_opt ')]
    for case in range(256):
        n = rng.randint(1,12)
        scale = rng.choice([1e-300,1e-100,1.0,1e100,1e300])
        entries = [rng.uniform(-10,10)*scale for _ in range(2*n)]
        if case%5 == 0: entries[n:] = [0.0]*n
        requests.append(f'modern_eigen {n} {rng.randint(-3,3)} {rng.choice([0,1,30])} {case%17} '+' '.join(map(str,entries)))
    for case in range(256):
        n = rng.randint(1,8)
        scale = rng.choice([1e-300,1e-100,1.0,1e100,1e300])
        entries = [rng.uniform(-10,10)*scale for _ in range(n*n+n)]
        if case%5 == 0 and n>1: entries[n:2*n] = entries[:n]
        if case%11 == 0: entries[:n] = [0.0]*n
        requests.append(f'modern_lu {n} {rng.randint(-3,3)} {case%19} {rng.choice([0.0,1e-15,0.1,-1.0])} '+' '.join(map(str,entries)))
    for case in range(256):
        values = [case % 10, rng.choice([0,0,0,1,2,3,5,10]), rng.choice([0,1,3,20]),
                  rng.uniform(-3,3), rng.uniform(-5,5), rng.uniform(-5,5),
                  rng.choice([0.0,1e-6,1e-10]), rng.choice([0.0,1e-6,1e-5])]
        requests.append('modern_newton '+' '.join(map(str,values)))
    requests += [line.replace('modern_opt ', 'modern_lbfgs ', 1)
                 for line in requests if line.startswith('modern_opt ')]
    for case in range(256):
        n = rng.randint(1, 7)
        m = n if case % 2 == 0 else rng.randint(1, 9)
        scale = rng.choice([1.0, 1e-300, 1e-150, 1e150, 1e300])
        entries = [rng.uniform(-5, 5)*scale for _ in range(m*n+m)]
        if case % 13 == 0: entries[:m*n] = [0.0]*(m*n)
        if case % 17 == 0 and n > 1:
            for row in range(m): entries[row*n+1] = entries[row*n]*2
        requests.append(f'modern_factors {m} {n} {case%19} {rng.choice([0.0,1e-15,0.1,-1.0])} {rng.choice([0.0,0.01,1e300,-1.0])} '+' '.join(map(str,entries)))
    for case in range(256):
        scale = rng.choice([1.0, 1e-300, 1e-150, 1e150, 1e300])
        coefficients = [rng.uniform(-10, 10)*scale for _ in range(5)]
        if case % 7 == 0:
            coefficients[:case%5+1] = [0.0]*(case%5+1)
        if case % 11 == 0:
            coefficients = [x*scale for x in [1.0,-8.0,22.0,-24.0,9.0]]
        requests.append('modern_poly '+' '.join(map(str,coefficients+[case%29])))
    for case in range(256):
        scale = rng.choice([1.0, 1e-300, 1e-150, 1e150, 1e300])
        coefficients = [rng.uniform(-5, 5)*scale for _ in range(5)]
        if case % 6 == 0: coefficients[:2] = [0.0,0.0]
        if case % 6 == 1: coefficients[:3] = [0.0,0.0,0.0]
        if case % 6 == 2: coefficients[:2] = [0.0,0.0]; coefficients[3] = 0.0
        if case % 6 in [3,4]:
            coefficients[0] = coefficients[4] = 0.0
            coefficients[2 if case%6==3 else 3] = 0.0
        bounds = rng.choice([[-2e100,2e100],[-2e100,1.0],[1.0,2e100],[1e17,1e17+100],[3.0,-3.0],[-10.0,10.0],[0.0,6.283185307179586],sorted([rng.uniform(-100,100),rng.uniform(-100,100)])])
        requests.append('modern_trig '+' '.join(map(str,coefficients+bounds+[rng.choice([1.5e-12,0.0,-1.0,0.1]),case%29,rng.choice([6.283185307179586,0.0,-1.0,1e-300]),rng.choice([0.0,1e-12,-1.0,1e300])])))
    for case in range(256):
        mode = case%11
        bounds = [0.99,1.02] if mode==1 else rng.choice([[-5.0,5.0],[0.0,7.0],[0.0,1.0],[1.0,0.0]])
        samples = rng.choice([1,2,10,30])
        xtol = rng.choice([1e-10,1e-8,0.0])
        ftol = rng.choice([0.0,1e-12,1e-8])
        ntol = rng.choice([0.0,1e-12,0.1])
        limit = rng.choice([1,4,100])
        fail = rng.choice([0,0,3,25,70])
        offset = rng.choice([0.0,0.0,1.0,-0.5])
        requests.append('modern_multiple '+' '.join(map(str,[mode,rng.uniform(0.1,2.0)]+bounds+[samples,xtol,ftol,ntol,limit,fail,offset])))
    for case in range(256):
        bounds = rng.choice([[0.0,1.0],[-3.0,2.0],[1.0,-1.0],[2.0,2.0],[-1e308,1e308]])
        orders = rng.choice([[3,7],[15,61],[2,2],[0,7],[7,3],[100,100],[1,61]])
        requests.append('modern_integ '+' '.join(map(str,[case%8]+bounds+orders+[
            rng.choice([0,1,3,10]),rng.choice([1e-10,1e-18,0.0]),rng.choice([0,0,1,5,24,50]),rng.choice([0,1,3,7,61,62,65,80])])))
    for case in range(256):
        bounds = rng.choice([[0.0,1.0],[-3.0,2.0],[1.0,-1.0],[2.0,2.0],[-1e308,1e308]])
        requests.append('modern_kronrod '+' '.join(map(str,[case%8]+bounds+[
            rng.choice([0,1,2,7,15,30,31]),case%2,rng.choice([0,1,3,10]),
            rng.choice([1e-10,1e-18,0.0]),rng.choice([0,0,1,5,24,50]),rng.choice([0,7,11,30,31])])))
    for case in range(256):
        bounds = rng.choice([[0.0,1.0],[-3.0,2.0],[1.0,-1.0],[2.0,2.0],[-1e308,1e308]])
        requests.append('modern_double_exp '+' '.join(map(str,[case%10]+bounds+[
            rng.choice([0,1,3,8]),rng.choice([0,1,3,10]),rng.choice([0.0,0.5,0.1,1e-300,1e300]),
            rng.choice([1e-10,1e-18,0.0]),rng.choice([1,20,100,10000]),rng.choice([0,0,1,5,24,50]),case%6,rng.choice([0.0,0.5,3.0])])))
    for case in range(256):
        bounds = rng.choice([[0.0,1.0],[-3.0,2.0],[1.0,-1.0],[2.0,2.0],[-1e308,1e308]])
        requests.append('modern_tensor '+' '.join(map(str,[rng.choice([1,2,3,4]),case%4,
            rng.choice([0,0,1,5,24]),rng.choice([-1,0,1,3])]+bounds+[rng.choice([0,1,2,7]),rng.choice([0,1,7,61,62])])))
    append_system_requests(rng, requests)
    append_laguerre_requests(requests)
    append_cslib_requests(requests)
    append_plib_requests(requests)
    append_elementary_requests(requests)
    append_bvh_requests(requests)
    append_bvh_query_requests(requests)
    append_bsplclib_requests(requests)
    append_bsplslib_requests(requests)
    append_gp_planar_requests(requests)
    append_mathpoly_utils_requests(requests)
    append_ncollection_vec_requests(requests)
    append_gp_transform_requests(requests)
    append_convert_requests(requests)
    append_gp_core_requests(requests)
    append_bnd_requests(requests)
    append_bvh_tool_classes_requests(requests)
    return '\n'.join(requests)+'\n'           


def canonical_nan_tokens(text: str) -> str:
    """Compare NaN classification while retaining exact finite/infinity/signed-zero bits."""
    def normalize(match: re.Match[str]) -> str:
        bits = int(match.group())
        if bits <= 0xffffffffffffffff and bits & 0x7ff0000000000000 == 0x7ff0000000000000 and bits & 0xfffffffffffff:
            return 'NaN'
        return match.group()
    return re.sub(r'\b[0-9]{16,20}\b', normalize, text)


def compare(oracle: str, rust: str, requests: str) -> None:
    """Require exact result count and bits; compare(native, translated, requests)."""
    if len(oracle.splitlines()) != len(requests.splitlines()):
        raise RuntimeError('Incomplete native geometry result')
    if canonical_nan_tokens(oracle) == canonical_nan_tokens(rust):
        return
    for request, expected, actual in zip(requests.splitlines(), oracle.splitlines(), rust.splitlines()):
        if canonical_nan_tokens(expected) != canonical_nan_tokens(actual):
            raise RuntimeError(f'Geometry mismatch {request!r}: OCCT {expected}; Rust {actual}')
    raise RuntimeError('Numeric result counts differ')


def selected_paths() -> tuple[Path, Path]:
    """Resolve the pinned OCCT checkout and native build; selected_paths() -> (source, build)."""
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, default=ROOT/'../OCCT')
    parser.add_argument('--native-build', type=Path, default=ROOT/'target/occt-tkernel-full')
    arguments = parser.parse_args()
    return arguments.source.resolve(), arguments.native_build.resolve()


def main() -> None:
    """Build probes, compare all cases, and write evidence only on success."""
    source, build = selected_paths()
    inventory = snapshot(source)
    output = ROOT/'target/oracle-tkmath/numeric'
    output.mkdir(parents=True, exist_ok=True)
    (output/'report.json').unlink(missing_ok=True)
    libraries = [next(build.rglob('lib'+name+'.so')) for name in ['TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']]
    library = libraries[-1]
    executable = output/'native-probe'
    run(['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
         str(ROOT/'tools/oracle/tkmath_probe.cxx'), *map(str,libraries), '-Wl,-rpath,'+str(library.parent),
         '-o', str(executable)], output/'compile.log')
    run(['cargo', 'build', '--locked', '-p', 'rustycad-tkmath', '--example', 'numeric_probe'], output/'rust-build.log')
    verify_probes(output, executable, inventory['revision'])


def reject_unknown_operation(executable: Path) -> None:
    """Ensure stale binaries cannot silently accept unknown operations."""
    result = subprocess.run([str(executable)], input='unknown_operation 0\n',
                            text=True, capture_output=True, check=False)
    if result.returncode == 0:
        raise RuntimeError(f'Probe accepts unknown operation: {executable}')


def verify_probes(output: Path, executable: Path, revision: str) -> None:
    """Execute both probes on identical persisted input; verify_probes(out, executable, sha)."""
    reject_unknown_operation(executable)
    reject_unknown_operation(ROOT/'target/debug/examples/numeric_probe')
    requests = corpus()
    (output/'inputs.txt').write_text(requests)
    oracle = run([str(executable)], output/'native.txt', requests)
    rust = run([str(ROOT/'target/debug/examples/numeric_probe')], output/'rust.txt', requests)
    compare(oracle, rust, requests)
    report = {'revision': revision, 'cases_passed': len(requests.splitlines()),
              'input_sha256': hashlib.sha256(requests.encode()).hexdigest(),
              'output_sha256': hashlib.sha256(oracle.encode()).hexdigest(),
              'comparator': 'finite, infinity and signed-zero bits exact; NaN classification; raw payload/sign differences retained',
              'nan_representation_difference_cases': sum(a != b for a,b in zip(oracle.splitlines(),rust.splitlines()))}
    (output/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"TKMath: {report['cases_passed']} numeric comparisons (exact finite bits, NaN classification) passed")


if __name__ == '__main__':
    main()
