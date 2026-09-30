#!/usr/bin/env python3
"""Private Contap_Contour helpers compared with the unchanged pinned source."""
import hashlib
import json
from itertools import product
from pathlib import Path
from porting import ROOT
from toolkit_numeric import NumericOracle, verify_numeric
from verify_tkmath_numeric import selected_paths
from tktopalgo_corpus_breplib_records import numbers


class ContourHelperOracle(NumericOracle):
    """Use a separate executable so unfinished helpers do not expand the public probe."""
    @property
    def rust_example(self) -> str:
        """Unique Cargo identity for the private helper translation."""
        return 'rustycad-tkhlr-contour-private-probe'


def requests() -> str:
    """Cover signed transitions, periodic recadrage and zero-radius projection."""
    lines: list[str] = []
    for kind, radius, uv, tangent, direction, circle in product(range(6), [0,2],
            [(-20,15),(0,0),(0.3,0.5),(20,-15)], [(1,0,0),(0,0,1),(0,0,0)],
            [(1,0,0),(1,2,3)], [0,2]):
        surface=[kind,0,0,0,0,0,1,1,0,0,0,radius,0.4,-2,2,-1,1]
        lines.append('contour_math '+numbers([*surface,*uv,*tangent,*direction,circle]))
    append_walking_internal_requests(lines)
    append_analytic_requests(lines)
    append_attachment_requests(lines)
    append_section_requests(lines)
    return '\n'.join(lines)+'\n'


def append_section_requests(lines: list[str]) -> None:
    """Exercise all line kinds, vertex partitions, tiny spans and duplicate points."""
    for surface_kind, kind, scenario, duplicate, offset in product(range(6), range(4),
            range(6), [0,1], [-3,0,3]):
        if kind == 1 and surface_kind not in (2,3): continue
        if kind == 2 and surface_kind not in (1,2,3): continue
        if kind == 0 and scenario == 5: continue
        surface=[surface_kind,0,0,0,0,0,1,1,0,0,0,2,0.4,-2,2,-1,1]
        lines.append('contour_sections '+numbers([*surface,kind,scenario,duplicate,offset]))


def append_attachment_requests(lines: list[str]) -> None:
    """Compare boundary attachment and interior filtering on real search outputs."""
    for kind,mode,direction,radius in product(range(6),range(3),[(1,0,0),(0,0,1)], [0,2]):
        surface=[kind,0,0,0,0,0,1,1,0,0,0,radius,0.4,-2,2,-1,1]
        fields=[*surface,mode,*direction]
        if kind < 4:
            lines.append('contour_analytic '+numbers(fields))
        lines.append('contour_attachment '+numbers(fields))
        for seeded,tolerance in product([0,1],[-1,0,1e-7]):
            lines.append('contour_segments '+numbers([*fields,seeded,tolerance]))
        for scenario in range(4):
            lines.append('contour_internal '+numbers([*fields,scenario]))


def append_analytic_requests(lines: list[str]) -> None:
    """Vary trimmed domains and translated frames for repeated elementary Perform."""
    bounds=[(-2,2,-1,1),(0,6.283185307179586,-1.5,1.5),(-0.1,0.1,-0.1,0.1)]
    for kind,mode,direction,radius,limits,origin in product(range(4),range(3),
            [(1,2,3),(-1,0,1),(0,1,0),(0,0,-1)],[0.5,3],bounds,[(0,0,0),(1,-2,3)]):
        surface=[kind,*origin,0,0,1,1,0,0,0,radius,0.4,*limits]
        lines.append('contour_analytic '+numbers([*surface,mode,*direction]))


def append_walking_internal_requests(lines: list[str]) -> None:
    """Observe mutation of empty, singleton and sampled surface polylines."""
    for kind,count,path,seed,mode,direction in product(range(6),[0,1,3,7],range(3),[0,1],range(3),[(1,0,0),(0,0,1),(1,2,3)]):
        surface=[kind,0,0,0,0,0,1,1,0,0,0,2,0.4,-2,2,-1,1]
        lines.append('contour_walking_internal '+numbers([*surface,mode,*direction,count,path,seed]))


def source_wrapper(source: Path, build: Path, output: Path, libraries: list[str]) -> Path:
    """Retain the original file hash and exact native compilation command."""
    original=source/'src/ModelingAlgorithms/TKHLR/Contap/Contap_Contour.cxx'
    wrapper=output/'native-translation-unit.cxx'
    probe=ROOT/'tools/oracle/tkhlr_contour_private_probe.cxx'
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
    source,build=selected_paths();output=ROOT/'target/oracle-tkhlr-contour-helpers/numeric'
    output.mkdir(parents=True,exist_ok=True)
    libraries=['TKHLR','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']
    wrapper=source_wrapper(source,build,output,libraries)
    verify_numeric(ContourHelperOracle('TKHLR-contour-helpers',str(wrapper),libraries,'rustycad-tkhlr',requests))


if __name__ == '__main__':
    main()
