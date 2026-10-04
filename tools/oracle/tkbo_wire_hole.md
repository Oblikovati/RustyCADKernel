# Boolean wire-hole predicate

Oracle revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_wire_hole.cxx` exercises BOPTools_AlgoTools::IsHole against six wire
fixtures, three missing-pcurve placements, and all wire/face orientations:
288 native observations. Rust replay:
`cargo test -p rustycad-tkbo --test make_pcurve wire_hole`.

The fixtures include rectangular planar and Bezier supports, cylindrical and
toroidal seams, a surface of revolution, and a planar circular edge. Missing
pcurves stop accumulation rather than discarding the accumulated area. Internal
and external edges are skipped. Face orientation is forced forward, while
wire orientation is composed into its children. Circle sampling uses the native
Geom2dInt curve-tool sample count and multiplier.

This is supplemental differential evidence. Arbitrary spline pcurves and full
legacy-test coverage remain unproven; the API mapping is partial.
