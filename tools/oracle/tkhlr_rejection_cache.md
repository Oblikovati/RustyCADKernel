# HLR private rejection cache

Source: `HLRBRep_Data.cxx` at pinned OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe compiles the native private nested class body verbatim, changing only
its qualified declaration name to `RejectionCache` for standalone access. Generate
the temporary header from the pinned checkout:

```python
from pathlib import Path
source = Path('../OCCT/src/ModelingAlgorithms/TKHLR/HLRBRep/HLRBRep_Data.cxx').read_text()
start = source.index('class HLRBRep_Data::RejectionCache')
end = source.index('\n};', start) + 3
Path('/tmp/tkhlr_native_rejection_cache.hxx').write_text(
    source[start:end].replace('HLRBRep_Data::RejectionCache', 'RejectionCache', 1))
```

Compile `tkhlr_rejection_cache.cxx` with C++17, `-O0 -g0`, the native installed
OCCT include directory and `/tmp` include directory, linking TKGeomAlgo, TKMath,
TKernel and setting the native library rpath. Its stdout is the adjacent expected
file. The Rust unit test replays that output exactly.

Covered behavior: symmetric rejection bits at edges 31/32/33 and 64/65, descending
row ordering, duplicate insertion before existing entries, absent-first-parameter
short circuit, paired parameters, and transition filtering based on the first
transition's kind and both positions. In particular the second transition need
not be In/Out. Supplemental Rust tests cover empty allocation and endpoint exclusion.

This is a prerequisite for HLRBRep_Data, not a public API parity claim. The full
HLR legacy tests require the remaining exact hidden-line pipeline, including edge
and face structures, projected intersections and result extraction. None of those
legacy declarations is mapped by this private-cache replay. Invalid edge indices
are outside the native defined domain; this probe makes no safety parity claim.
