# Micro-edge classification

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_micro_edge.cxx` with installed OCCT headers and link TKBO,
TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Output: `tkbo_micro_edge.expected`.

The 192 exact boolean cases combine six fixtures (empty edge, four line lengths
from 1e-8 to 1, and degenerated edge), two tolerance scales, both splittability
options, present/null contexts and all edge orientations. Edges are built with
explicit endpoint vertices so a construction API does not reject the shortest
line before the predicate runs.

The source obtains endpoint parameters, sorts the parameter pair without
swapping vertices, and delegates to IntTools_ShrunkRange. It retains early
success for degenerated/non-geometric edges. The API remains partial pending
curved and located edges, missing endpoints, pcurve-only geometry and wider
ShrunkRange behavior. No legacy declaration is mapped by these fixtures.
