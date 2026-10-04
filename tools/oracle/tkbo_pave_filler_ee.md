# `BOPAlgo_PaveFiller::PerformEE`

Pinned native source: `src/ModelingAlgorithms/TKBO/BOPAlgo/BOPAlgo_PaveFiller_3.cxx`, revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The checked fixture intersects two interior straight-edge paves at one vertex. It records the `InterfEE` indices/common-part parameters/new vertex, then compares both split pave-block lists. Candidate pairs come from the initialized edge-edge iterator; block pairs are rejected by invalid parameter width or disjoint shrunk boxes before invoking the existing `IntTools_EdgeEdge` port.

This slice also binds an edge-type common part when the native pave blocks have identical endpoint indices. It does not yet reproduce endpoint-adjacent `ForceInterfVE`, fusion of multiple coincident new vertices, the analytical line/circle tolerance expansion, far-origin transform, full `PerformCommonBlocks` tolerance updates, warning ordering for every solver failure, cancellation, or parallel execution. Those paths remain explicit gaps; this is not full `PerformEE` or a complete Boolean operation.
