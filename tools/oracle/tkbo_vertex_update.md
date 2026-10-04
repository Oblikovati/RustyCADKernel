# Boolean vertex tolerance updates

Oracle OCCT commit: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`BOPTools_AlgoTools_2.cxx` supplies the three UpdateVertex overloads.

The native probe `tkbo_vertex_update.cxx` produces 90 curve rows without an
argument and nine vertex-pair rows with an argument. It uses the common pcurve
fixture geometry (line, cylindrical generatrix, circle, revolution generatrix,
and Bezier-surface fixture line). Compare with
`cargo test -p rustycad-tkbo --test make_pcurve vertex_update`.

Checks cover three parameters and three starting tolerances for edge and
IntTools_Curve overloads, tolerance growth and no-growth, unchanged points,
and the vertex-pair overload's mutation of its first argument. The tolerance
margin is added only when the strict native growth condition succeeds.

These are supplemental differential fixtures, not legacy declaration mappings.
Arbitrary located edges and spline adaptor cases remain to be exercised;
all three API rows are partial.
