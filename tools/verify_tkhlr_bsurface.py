#!/usr/bin/env python3
"""Compare every executable BSurfaceTool declaration with unchanged native TKHLR."""
from itertools import product
from toolkit_numeric import NumericOracle, verify_numeric
from tktopalgo_corpus_breplib_records import numbers
from tkhlr_bsurface_gap import verify_missing_axis
from verify_tkmath_numeric import selected_paths

class BSurfaceOracle(NumericOracle):
    """Independent exact-HLR surface target."""
    @property
    def rust_example(self) -> str:
        """Avoid shared executable collisions."""
        return 'hlrbrep_bsurface_probe'

def corpus() -> str:
    """Real analytic, swept and rational faces, signed spans and out-of-range parameters."""
    return '\n'.join('bsurface '+numbers([kind,shift,u,v,first,last])
        for kind,shift,(u,v),(first,last) in product(range(12),[0,2],
        [(-1,-1),(0,0),(.3,.7),(1,1),(3,2)],[(0,1),(1,0),(0,0),(0,100),(100,0)]))+'\n'

if __name__ == '__main__':
    oracle=BSurfaceOracle('TKHLR-bsurface','tools/oracle/tkhlr_bsurface_probe.cxx',
        ['TKHLR','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel'],
        'rustycad-tkhlr',corpus)
    _,build=selected_paths()
    verify_missing_axis(build,oracle.evidence)
    verify_numeric(oracle)
