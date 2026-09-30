#!/usr/bin/env python3
"""Private Contap_Contour ComputeTangency compared with the unchanged pinned source."""
import hashlib
import json
from itertools import product
from pathlib import Path
from porting import ROOT
from toolkit_numeric import NumericOracle, verify_numeric
from verify_tkmath_numeric import selected_paths
from tktopalgo_corpus_breplib_records import numbers


class TangencyOracle(NumericOracle):
    """Use a separate executable so unfinished helpers do not expand the public probe."""
    @property
    def rust_example(self) -> str:
        """Unique Cargo identity for the private helper translation."""
        return 'contour_tangency_probe'


def requests() -> str:
    """Real search roots, corner multiplicity, restrictions and seeded numbering."""
    lines: list[str] = []
    for kind, mode, direction, radius, bounds, seed in product(range(6), range(3),
            [(1,0,0),(0,0,1),(1,2,3)], [0,2],
            [(-2,2,-1,1),(0,6.283185307179586,-1.5707963267948966,1.5707963267948966),(0,1,0,1)],range(4)):
        surface=[kind,0,0,0,0,0,1,1,0,0,0,radius,0.4,*bounds]
        lines.append('tangency '+numbers([*surface,mode,*direction,seed,0]))
    for kind, mode, direction, bounds, orientation, seed in product([1,2,3,5],range(3),
            [(1,0,0),(0,1,0),(0,0,1),(1,2,3),(0,1,1),(0,1,-1),(1,1,0)],
            [(0,1,0,1),(0,1.5707963267948966,0,1),(-1.5707963267948966,1.5707963267948966,-1,1)],range(1,10),[0,2,3]):
        surface=[kind,0,0,0,0,0,1,1,0,0,0,2,0.4,*bounds]
        lines.append('tangency '+numbers([*surface,mode,*direction,seed,orientation]))
    return '\n'.join(lines)+'\n'


def source_wrapper(source: Path, build: Path, output: Path, libraries: list[str]) -> Path:
    """Retain the original file hash and exact native compilation command."""
    original=source/'src/ModelingAlgorithms/TKHLR/Contap/Contap_Contour.cxx'
    wrapper=output/'native-translation-unit.cxx'
    probe=ROOT/'tools/oracle/tkhlr_contour_tangency_probe.cxx'
    wrapper.write_text('#include '+json.dumps(str(original))+'\n#include '+json.dumps(str(probe))+'\n')
    linked=[str(next(build.rglob('lib'+name+'.so'))) for name in libraries]
    command=['c++','-std=c++17','-O2','-I'+str(build/'include/opencascade'),str(wrapper),*linked,
             '-Wl,-rpath,'+str(Path(linked[0]).parent),'-o',str(output/'native-probe')]
    manifest={'source':str(original),'source_sha256':hashlib.sha256(original.read_bytes()).hexdigest(),
              'native_command':command,'scope':'private contour helpers; no API or legacy declaration claims'}
    (output/'native-source.json').write_text(json.dumps(manifest,indent=2)+'\n')
    return wrapper


def main() -> None:
    """Build the isolated native translation unit and run the exact comparator."""
    source,build=selected_paths();output=ROOT/'target/oracle-tkhlr-contour-tangency/numeric'
    output.mkdir(parents=True,exist_ok=True)
    libraries=['TKHLR','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']
    wrapper=source_wrapper(source,build,output,libraries)
    verify_numeric(TangencyOracle('TKHLR-contour-tangency',str(wrapper),libraries,'rustycad-tkhlr',requests))


if __name__ == '__main__':
    main()
