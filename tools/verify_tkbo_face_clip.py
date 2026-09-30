#!/usr/bin/env python3
"""Compare FaceFace's private plane intersection helpers from unchanged pinned source.

Usage: python3 tools/verify_tkbo_face_clip.py --source OCCT --native-build BUILD
The native source is included verbatim into an isolated executable because the
shared library hides its private helpers. No installed source or native build is changed.
"""
import json
import hashlib
from itertools import product
from pathlib import Path
from porting import ROOT
from toolkit_numeric import NumericOracle, verify_numeric
from verify_tkmath_numeric import selected_paths


def clip_requests() -> str:
    """Exercise corners, zero-width bounds, infinite bounds and tolerance suppression."""
    bounds = [(-2,3,-4,5), (0,0,0,1), (0,1,0,0), (-2e100,2e100,-2e100,2e100)]
    points = [(-2,-4),(-2,5),(3,5),(3,-4),(0,0),(9,10),(0,1e-7),(0,1e-8)]
    directions = [(1,0),(-1,0),(0,1),(1,1),(1,-1),(1,1e-10)]
    clipped = '\n'.join(f"face_clip {a} {b} {c} {d} {x} {y} {dx} {dy} {tol}"
        for (a,b,c,d), (x,y), (dx,dy), tol in product(bounds, points, directions, [0,1e-7,1,10]))+'\n'
    return clipped + plane_requests() + policy_requests() + boundary_requests()


def plane_requests() -> str:
    """Exercise plane fast paths, pcurve flag asymmetry and endpoint/tolerance bits."""
    normals = [(0,0,1),(1,0,0),(1,1,1),(1e-7,0,1),(0,0,-1)]
    return '\n'.join(f"face_planes {z} {dx} {dy} {dz} {size} {t1} {t2} {a} {b}"
        for z, (dx,dy,dz), size, t1, t2, a, b in product(
            [0,1e-7,1,100], normals, [0,1,10], [0,1e-7], [1e-7,1], [0,1], [0,1]))+'\n'


def policy_requests() -> str:
    """Exercise quadric parameters, surface ordering, radius equality and sphere poles."""
    requests = [f"face_policy {a} {b} {r1} {r2} {tol} {x} {y} {z}"
        for a, b, (r1,r2), tol, (x,y,z) in product(range(9), range(9),
            [(3,3),(3,3.00000005),(3,4)], [0,1e-7,0.1], [(0,0,0),(1,2,3)])]
    requests += [f"face_pcurves {r1} {r2} {x} {y} {axis} {reverse}"
        for r1, r2, x, y, axis, reverse in product([0,1,2,10], [0,0.5,1],
            [0,0.8,1,1.2,2,9,10,11,100], [0,1], [0,1], [0,1])]
    return '\n'.join(requests)+'\n'


def boundary_requests() -> str:
    """Compare finite/infinite enlargement and periodic seams on genuine BRep faces."""
    intervals = [(0,1,-1,1),(-100,100,-100,100),(-2e100,2e100,-2e100,2e100),(-1e-8,1e-8,0,0)]
    return '\n'.join(f"face_bounds {kind} {radius} {tol} {explicit} {u1} {u2} {v1} {v2}"
        for kind, radius, tol, explicit, (u1,u2,v1,v2) in product(
            range(30), [3,4], [-1,0,1e-7,1], [0,1], intervals))+'\n'


def main() -> None:
    """Compile unchanged OCCT with the probe and run the exact shared comparator."""
    source, build = selected_paths()
    output = ROOT/'target/oracle-tkbo-face-clip/numeric'
    output.mkdir(parents=True, exist_ok=True)
    wrapper = output/'native-translation-unit.cxx'
    original = source/'src/ModelingAlgorithms/TKBO/IntTools/IntTools_FaceFace.cxx'
    probe = ROOT/'tools/oracle/tkbo_foundations_probe.cxx'
    wrapper.write_text('#define TKBO_FACEFACE_PRIVATE_PROBE\n#include '+json.dumps(str(original))+'\n#include '+json.dumps(str(probe))+'\n')
    libraries = ['TKBO','TKPrim','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']
    record_source(original, build, output, libraries)
    verify_numeric(NumericOracle('TKBO-face-clip', str(wrapper), libraries, 'rustycad-tkbo', clip_requests))


def record_source(original: Path, build: Path, output: Path, libraries: list[str]) -> None:
    """Retain the verbatim source hash and actual native build command ingredients."""
    linked = [str(next(build.rglob('lib'+name+'.so'))) for name in libraries]
    command = ['c++', '-std=c++17', '-O2', '-I'+str(build/'include/opencascade'),
               str(output/'native-translation-unit.cxx'), *linked,
               '-Wl,-rpath,'+str(Path(linked[0]).parent), '-o', str(output/'native-probe')]
    manifest = {'source': str(original), 'source_sha256': hashlib.sha256(original.read_bytes()).hexdigest(),
                'native_command': command, 'scope': 'private FaceFace plane and surface-policy helpers; no legacy declarations'}
    (output/'native-source.json').write_text(json.dumps(manifest, indent=2)+'\n')


if __name__ == '__main__':
    main()
