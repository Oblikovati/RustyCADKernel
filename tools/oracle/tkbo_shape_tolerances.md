# Boolean shape tolerance propagation

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_shape_tolerances.cxx` records 48 CorrectShapeTolerances cases: ordinary
planar wire, an additional internal vertex, and a face containing only an
internal vertex; four avoidance sets; one/two passes; and both native parallel
flags. Replay:
`cargo test -p rustycad-tkbo --test make_pcurve shape_tolerances`.

The Rust port preserves edge-to-vertex propagation before face-to-edge
propagation, avoidance by shape identity, and the last visited edge tolerance
used when checking a direct face vertex. It therefore does not run an extra
vertex update after face edges change. Tests compare every edge/vertex tolerance.

The Rust implementation currently executes both requested scheduling modes
serially. This tests output semantics for these fixtures, not parallel execution
or arbitrary topology. Full API/legacy parity remains unproven; mapping is partial.
