# Boolean edge construction oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Source: `BOPTools_AlgoTools_2.cxx` CopyEdge, MakeSplitEdge, MakeSectEdge,
and `BOPTools_AlgoTools.cxx` MakeEdge.

`tkbo_edge_construction.cxx` produces 40 copy/split rows without arguments and
12 section-edge rows with any argument. Compile with installed OCCT headers and
TKBO/TKTopAlgo/TKBRep/TKG3d/TKG2d/TKMath/TKernel, then run both modes.
The Rust replay is `cargo test -p rustycad-tkbo --test make_pcurve edge_construction`.

Copy/split cases cover line and circle geometry, all four source orientations,
ascending/descending parameter bounds, absent endpoints, shared curve and vertex
identity, flags, and tolerances. Section cases cover ascending, descending, and
equal bounds with distinct vertices. Equal bounds raise `StdFail_NotDone`;
MakeEdge retains its earlier endpoint tolerance increases even on failure.
MakeSectEdge restores reversed bounds verbatim after the edge maker succeeds.

These are supplemental differential fixtures. They do not establish complete
legacy-test parity, arbitrary curve/location behavior, or a complete TKBO port.
All four API mappings remain partial.
