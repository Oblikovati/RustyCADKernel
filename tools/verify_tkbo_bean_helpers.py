#!/usr/bin/env python3
"""Private BeanFace flags/grouping/sampling compared with the unchanged pinned source."""
from itertools import product
from porting import ROOT
from native_private_oracle import private_translation_unit
from tkbo_bean_private_corpus import grid_requests
from toolkit_numeric import NumericOracle,verify_numeric
from verify_tkmath_numeric import selected_paths


class BeanHelperOracle(NumericOracle):
    @property
    def rust_example(self) -> str:
        """Keep Cargo artifacts independent of every other TKBO probe."""
        return 'rustycad-tkbo-bean-private-probe'


def requests() -> str:
    """Exercise insertion histories, stable grouping and three independent sample limits."""
    rows=['bean_empty '+' '.join(map(str,[*bounds,flag,first,last]))
          for bounds,flag,first,last in product([(0,0.5,1),(1,0,-1),(0,0,1)],[-1,0,1,2,3],[-1,0,0.25,0.5,1,2],[0,0.5,1])]
    rows+=['bean_group '+' '.join(map(str,[curves,surfaces,groups]))
           for curves,surfaces,groups in product([0,1,2,10,33],[0,1,3,10,35],[1,2,7])]
    rows+=['bean_sampling '+' '.join(map(str,[depth,depth+1,depth-1,samples,samples+1,samples-1,minimum,minimum*2,minimum/2,difference,difference*2,-difference]))
           for depth,samples,minimum,difference in product([-2,-1,0,1,2,3,10,1024],[-2,0,1,2,3,10,32],[-1,0,0.1],[0,1,10,1000])]
    rows += ['bean_box '+' '.join(map(str,[kind,count,*bounds,deflection,tolerance,seed]))
             for kind,count,bounds,deflection,tolerance,seed in product(range(5),[1,3,7],[(0,1,0,1),(-1,2,-1,2),(1,0,1,0),(0,0,0,0)],[-0.1,0,0.01],[-0.25,0,0.5],[0,1])]
    rows += grid_requests()
    return '\n'.join(rows)+'\n'


def main() -> None:
    """Build and compare private helpers without changing native access or implementation."""
    source,build=selected_paths();output=ROOT/'target/oracle-tkbo-bean-helpers/numeric'
    libraries=['TKBO','TKPrim','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel']
    original=source/'src/ModelingAlgorithms/TKBO/IntTools/IntTools_BeanFaceIntersector.cxx'
    wrapper=private_translation_unit(original,ROOT/'tools/oracle/tkbo_bean_private_probe.cxx',output,build,libraries)
    verify_numeric(BeanHelperOracle('TKBO-bean-helpers',str(wrapper),libraries,'rustycad-tkbo',requests))


if __name__=='__main__':
    main()
