# `BOPAlgo_PaveFiller::PerformVV`

Pinned native source: `src/ModelingAlgorithms/TKBO/BOPAlgo/BOPAlgo_PaveFiller_1.cxx`, revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe runs the protected stage through a derived test class after native `Init`. Two compound arguments hold vertices 0.5 µm apart, each with 0.1 µm tolerance, and fuzzy value 1 µm. The expected result records the appended same-domain vertex, VV interference, source-to-representative map, and merged point/tolerance. The Rust regression calls the public stage after the same initialization path.

Build with the repository's pinned OCCT build and link TKBO, TKPrim, TKTopAlgo, TKShHealing, TKGeomAlgo, TKBRep, TKGeomBase, TKG3d, TKG2d, TKMath, and TKernel. This fixture covers one two-vertex component; warning ordering, pre-existing SD chains, cancellation, and multi-component rank diagnostics remain outside this stage's proven behavior.
