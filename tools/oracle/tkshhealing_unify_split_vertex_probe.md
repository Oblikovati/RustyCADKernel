# OCC29544 split-vertex probe

The exact OCCT OCC29544 test depends on `BOPAlgo_Splitter`, which is not yet
available in the Rust kernel. This probe records its native intermediate shape:
the split vertex remains at `(5, 2e-7, 0)` with tolerance `3e-7`, shared by two
line edges on the X axis. `ShapeUpgrade_UnifySameDomain` then merges them into a
single edge with two vertices. The Rust line-chain merger can represent this
post-split topology, but the legacy case remains blocked on the splitter
dependency and is not marked ported.

Source: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`TKShHealing/GTests/ShapeUpgrade_UnifySameDomain_Test.cxx` test
`OCC29544_NearbyLinearSplitIsUnified`.
