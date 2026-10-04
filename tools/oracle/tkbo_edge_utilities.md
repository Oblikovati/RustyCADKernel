# TKBO edge pcurve utility oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_edge_utilities.cxx` uses the shared pcurve fixture header. Link against
TKBO, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
The default invocation emits `tkbo_edge_utilities.expected`; passing `isoline`
emits `tkbo_isoline.expected`.

Forty cases cover plane, cylinder, torus, revolution and flat Bezier supports,
all four face orientations, and fresh versus cached contexts. They compare
IntermediatePoint, the short CurveOnSurface overload, PointOnSurface, the
full-curve AdjustPCurveOnFace overload, repeated BuildPCurveForEdgeOnFace,
stored ranges/tolerance/UV samples and IsEdgeIsoline. Eight further cases cover
both signs of axis directions and all diagonal quadrants for isoline flags.
The source's CrossMagnitude is absolute, so orientation signs do not change
those flags. Rust compares numeric columns with an absolute 1e-12 tolerance.

These are supplemental oracle fixtures, not newly mapped legacy declarations.
Located geometry, general splines, degenerate derivative and exceptional paths
remain unproven for these helpers. Existing broader projection evidence remains
applicable to their delegated operations; these API rows remain partial.
