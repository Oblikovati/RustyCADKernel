# Fillet face connections and side propagation

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.

The connection fixture records 192 combinations of line/Bezier edges, opposing
or perpendicular planar normals, face/edge orientations, correction mode,
missing nonplanar pcurves, and the same face supplied twice. The Bezier fixture
has zero tangent at the midpoint and exercises the native alternate parameter.
Spline boundary correction has additional source-formula unit checks; curved,
located, real-seam, and corrected spline surface behavior needs native coverage.
DefineConnectType is therefore still marked partial.

The orientation fixture records 5,440 cases: every declared orientation in both
NextSide overloads and SameSide, with pair choices -8 through 8 covering signs and
parities. Rust avoids abs(INT_MIN) overflow, whose native behavior is undefined;
that input is not included in the equivalence claim. No legacy mappings are added.
