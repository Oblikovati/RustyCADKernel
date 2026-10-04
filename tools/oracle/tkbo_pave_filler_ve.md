# `BOPAlgo_PaveFiller::PerformVE`

Pinned native source: `src/ModelingAlgorithms/TKBO/BOPAlgo/BOPAlgo_PaveFiller_2.cxx`, revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The protected-stage probe checks an independent vertex projected to a straight edge midpoint in destructive and non-destructive modes, including VE records, new vertex mapping, generated block ranges, and the new vertex box. A micro-edge fixture verifies that an unsplittable first pave block is ignored after shrunk-range preparation. Additional cases compare invalid split-interval vertex unification, open common-block rebuilding, and closed common-block rebuilding. Closed groups are first partitioned by their endpoint-index pair; geometric projection only groups blocks inside each pair, matching the native indexed-map order.

The pinned native build segfaults when `PerformVE` is called a second time on the same filler after a successful call. That observed failure is documented as an unsupported repeated-call domain; Rust does not fabricate an exception or parity result for it.

Self-interference warnings, cancellation, and parallel scheduling remain gaps. This reusable stage is not a complete PaveFiller or Boolean operation.
