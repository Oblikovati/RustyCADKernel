# `BOPAlgo_PaveFiller::PerformVE`

Pinned native source: `src/ModelingAlgorithms/TKBO/BOPAlgo/BOPAlgo_PaveFiller_2.cxx`, revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The protected-stage probe checks an independent vertex projected to a straight edge midpoint in destructive and non-destructive modes, including VE records, new vertex mapping, generated block ranges, and the new vertex box. A micro-edge fixture verifies that an unsplittable first pave block is ignored after shrunk-range preparation.

This port covers ordinary edge blocks whose split intervals have valid shrunk data. Common-block regeneration, invalid/unsplittable split interval unification, self-interference warnings, cancellation, and parallel scheduling remain gaps. The reusable stage is not a complete PaveFiller or Boolean operation.
