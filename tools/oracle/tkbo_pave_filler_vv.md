# `BOPAlgo_PaveFiller::PerformVV`

Pinned native source: `src/ModelingAlgorithms/TKBO/BOPAlgo/BOPAlgo_PaveFiller_1.cxx`, revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe runs the protected stage through a derived test class after native `Init`. It covers a close pair, a disjoint pair, a three-vertex transitive component, and a repeated stage call with an expanded representative box. The expected result records appended same-domain vertices, VV interferences, source-to-representative maps, merged point/tolerance, and preserved box extents. The Rust regression uses the same initialization and stage order.

Build with the repository's pinned OCCT build and link TKBO, TKPrim, TKTopAlgo, TKShHealing, TKGeomAlgo, TKBRep, TKGeomBase, TKG3d, TKG2d, TKMath, and TKernel. This fixture covers one two-vertex component; warning ordering, pre-existing SD chains, cancellation, and multi-component rank diagnostics remain outside this stage's proven behavior.
