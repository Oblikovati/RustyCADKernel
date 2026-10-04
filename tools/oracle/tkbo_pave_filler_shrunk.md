# `BOPAlgo_PaveFiller::FillShrunkData`

Pinned native sources: `BOPAlgo_PaveFiller_9.cxx` and `BOPAlgo_PaveFiller_3.cxx`, revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe calls native `Init` followed by protected `FillShrunkData(Vertex, Edge)` for a point at the midpoint of a straight unit edge. It compares source DS shape count, generated pave-block count, shrunk interval, splittability, and tolerance-expanded box. Native `FillShrunkData` lazily creates the edge's endpoint pave blocks through `ChangePaveBlocks`; the Rust stage does the same before computing the shrunk range. The initialized `IntTools_Context` is shared with each shrink-range computation.

Reproduce with the OCCT build commands documented in the adjacent oracle fixtures, linking TKBO, TKPrim, TKTopAlgo, TKShHealing, TKGeomAlgo, TKBRep, TKGeomBase, TKG3d, TKG2d, TKMath, and TKernel. This fixture establishes one straight, splittable edge interval. Alert order and unsplittable/failure branches have not yet been compared. The general `PerformVE` interference stage and full Boolean pipeline remain unimplemented.
