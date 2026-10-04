# Container construction and edge evaluation

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_construction.cxx` with the installed OCCT headers and link TKBO,
TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Default output is `tkbo_construction.expected`; argument `point` emits
`tkbo_point_on_edge.expected`.

Eighteen MakeContainer cases cover every ShapeEnum value with null and existing
reversed-vertex outputs. The five supported containers replace the output with
a fresh Forward shape; unsupported types preserve its identity and orientation.
This exhausts the helper's switch; its API row is marked implemented.

Fifteen PointOnEdge cases sample both endpoints and midpoint over the five
shared geometric fixtures. Numeric values agree within 1e-12. The row remains
partial for located curves, broader spline domains and exceptional inputs.
These are supplemental fixtures, not legacy-test declaration mappings.
