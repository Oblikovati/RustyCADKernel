# Edge pcurve and normal oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The 56 native rows cover five projection supports, a zero-range plane edge,
and a stored cylinder pcurve, across all four face orientations and cached/fresh
contexts. Outputs cover both HasCurveOnSurface overloads, ranged CurveOnSurface,
Make2D, edge IntermediatePoint, and both GetNormalToFaceOnEdge overloads.
Rust compares numerical columns with absolute error below 1e-12.

The zero-range presence query leaves the caller's existing curve unchanged.
Make2D preserves incoming tolerance when a pcurve already exists. Only the
intermediate-parameter normal overload reverses the normal for a reversed face.
Absent nonplanar pcurves initially report zero parameter bounds, matching BRepTool.

Null spatial curves produce an explicit Rust Standard_NullObject boundary error;
this is not claimed equivalent to native null dereferencing. Located/seam edges,
singular normals, error output mutation and broader spline supports need evidence.
These supplemental cases do not complete a legacy regression mapping.
