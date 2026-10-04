# Inverted-solid predicate

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_solid_orientation.cxx` with OCCT headers and link TKBO, TKPrim,
TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Output: `tkbo_solid_orientation.expected`.

Sixteen cases compare box, cylinder, sphere and empty solid under every
orientation. The wrapper classifies the infinite point at exactly 1e-7 and
returns true only for TopAbs_IN. Empty solids return true in every orientation,
as the pinned classifier does. The row remains partial for multi-shell,
located, self-intersecting and general spline solids, and exceptional inputs.
No legacy declaration is mapped by these supplemental fixtures.
