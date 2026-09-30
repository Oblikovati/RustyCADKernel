#!/usr/bin/env python3
"""Compare all HLRBRep_BCurveTool members with unchanged pinned TKHLR."""
from itertools import product
from toolkit_numeric import NumericOracle, verify_numeric
from tktopalgo_corpus_breplib_records import numbers


class BCurveOracle(NumericOracle):
    """Independent target for the exact-HLR curve dependency."""
    @property
    def rust_example(self) -> str:
        """Unique executable identity; avoids shared example collisions."""
        return 'hlrbrep_bcurve_probe'


def corpus() -> str:
    """Conics, rational/nonrational splines, locations, span reversal and indexed output."""
    return '\n'.join('bcurve '+numbers([kind,shift,t,lo,hi,*poles,*weights])
        for kind,shift,t,(lo,hi),poles,weights in product(range(11),[0,2],[-1,0,0.5,1,2],
            [(0,1),(0,0),(1,0),(0,0.01),(0,100)],[(1,4),(2,3),(0,2),(3,5)],[(1,4),(2,3),(0,2),(3,5)]))+'\n'


if __name__ == '__main__':
    verify_numeric(BCurveOracle('TKHLR-bcurve','tools/oracle/tkhlr_bcurve_probe.cxx',
        ['TKHLR','TKTopAlgo','TKGeomAlgo','TKBRep','TKGeomBase','TKG3d','TKG2d','TKMath','TKernel'],
        'rustycad-tkhlr',corpus))
