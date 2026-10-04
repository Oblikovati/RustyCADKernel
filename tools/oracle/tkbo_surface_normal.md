# Surface and approximate normal oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`tkbo_surface_normal` has eight surface cases and 49 direction-pair cases. It
checks zero and tiny derivatives, parallel derivative construction failure,
unchanged output on failure, and the native direction-coincidence threshold.
The source tests its U direction against itself in GetNormalToSurface; this
behavior is retained. Rust checks sense values exactly and normal columns with
absolute error below 1e-12; native Standard_ConstructionError is checked explicitly.

`tkbo_approx_normal` adds 1,440 cases across the three overloads, five regular
supports plus a missing-pcurve case, edge/face orientations and tolerance pairs.
Rust compares success, sampled point and output normal within 1e-12. The native
algorithm ignores the Boolean from GetNormalToSurface and returns success based
on the nearby-point status; that ordering is preserved.

Broader curved/singular normals, exception state changes, locations and arbitrary
face domains remain to cover. These supplemental fixtures do not complete an
OCCT legacy regression mapping.
