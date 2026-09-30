#!/usr/bin/env python3
"""Private EdgeEdge helpers compared with the unchanged pinned translation unit."""
import hashlib
import json
from itertools import product
from pathlib import Path
from porting import ROOT
from toolkit_numeric import NumericOracle, verify_numeric
from verify_tkmath_numeric import selected_paths


class EdgeHelperOracle(NumericOracle):
    @property
    def rust_example(self) -> str:
        """Distinct Cargo identity prevents public-probe artifact reuse."""
        return 'rustycad-tkbo-edge-private-probe'


def requests() -> str:
    """Compare repeated range additions, box distance and geometric step sizing."""
    # Negative requested counts can natively allocate tens of millions of ranges;
    # retain those branches at bounded resolution so this is a fast regression corpus.
    lines=['edge_math '+' '.join(map(str,[a,b,res,count,gap,*point]))
           for (a,b),res,count,gap,point in product([(-2,3),(1,1),(2,-1),(0,1e-12)],
               [-1,0,1e-7,0.2,10],[-2,0,1,2,7],[0,0.1],[(0,0,0),(-3,5,10)])
           if not (count < 0 and res > 0 and (b-a)/res > 10000)]
    lines+=['edge_geometry '+' '.join(map(str,[kind,a,b,tol,res,offset]))
            for kind,(a,b),tol,res,offset in product(range(15),[(0,1),(0,0),(-2,3),(0,6.283185307179586)],
                [-1,0,1e-7,0.1,10],[-1,0,1e-7],[-2,0.5])]
    lines+=['edge_distance '+' '.join(map(str,[source,target,a,b,threshold,eps,maximum]))
            for source,target,(a,b),threshold,eps,maximum in product([0,1,3,4,7,9,11],
                [0,1,3,7,9,11],[(0,1),(0,0),(1,0),(-1,1)],[0.1,10],[1e-7],[0,1])]
    lines+=['edge_parameters '+' '.join(map(str,[kind,a,b,center,radius,gap,tol,res,eps]))
            for kind,(a,b),center,radius,gap,tol,res,eps in product([0,1,3,4,7,9,11],
                [(0,1),(0,0),(1,0),(-2,3)],[-3,0,1,10],[0.1,2],[0,0.01],[0,0.1],[0.05],[1e-7])]
    lines+=['edge_pairs '+' '.join(map(str,[source,target,a,b,c,d,tol,res]))
            for source,target,(a,b),(c,d),tol,res in product([0,1,3,4,7,9,11],
                [0,1,3,7,9,11],[(0,1),(0,0),(1,0),(-1,1)],[(0,1),(-2,3)],[0,1e-7,0.01,10],[1e-7,0.001,10])]
    lines+=['edge_lines '+' '.join(map(str,[kind,x,y,z,a,b,tol1,tol2,shared]))
            for kind,x,y,z,(a,b),(tol1,tol2),shared in product(range(6),[-1,0,0.5],[0,0.5],[0,0.05],
                [(-1,1),(0,1),(1,0),(0,0)],[(1e-7,1e-7),(0.01,0.05),(0.5,0.2)],[0,1])]
    return '\n'.join(lines)+'\n' 


def main() -> None:
    """Retain native source hash and build command before exact comparison."""
    source,build=selected_paths();output=ROOT/'target/oracle-tkbo-edge-helpers/numeric'
    output.mkdir(parents=True,exist_ok=True)
    original=source/'src/ModelingAlgorithms/TKBO/IntTools/IntTools_EdgeEdge.cxx'
    wrapper=output/'native-translation-unit.cxx'
    wrapper.write_text('#include '+json.dumps(str(original))+'\n#include '+json.dumps(str(ROOT/'tools/oracle/tkbo_edge_private_probe.cxx'))+'\n')
    libraries=['TKBO','TKPrim','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']
    linked=[str(next(build.rglob('lib'+name+'.so'))) for name in libraries]
    command=['c++','-std=c++17','-O2','-I'+str(build/'include/opencascade'),str(wrapper),*linked,'-Wl,-rpath,'+str(Path(linked[0]).parent),'-o',str(output/'native-probe')]
    (output/'native-source.json').write_text(json.dumps({'source':str(original),'source_sha256':hashlib.sha256(original.read_bytes()).hexdigest(),'native_command':command,'scope':'private helpers; no complete API or legacy declaration claim'},indent=2)+'\n')
    verify_numeric(EdgeHelperOracle('TKBO-edge-helpers',str(wrapper),libraries,'rustycad-tkbo',requests))


if __name__=='__main__':
    main()
