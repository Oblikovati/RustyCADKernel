# Wire edge self-intersection

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Six observations cover a cubic Bézier loop with an interior crossing, large
endpoint tolerances that suppress it, missing vertices, missing pcurve, zero
range, and unloaded wire. Output sequences start with sentinel values; the
native check clears them before readiness and geometry checks. Rust compares
all fields exactly, including both curve parameters and the spatial crossing.

Rows: `found DONE1 FAIL1 FAIL2 count2d count3d [parameter1 parameter2 x y z]*
wrapper_found aggregate_found aggregate_DONE1 aggregate_DONE2 aggregate_DONE3
aggregate_FAIL1 aggregate_FAIL2 aggregate_FAIL3`.

The final `remove_loop` row uses a same-range-disabled planar edge whose 3D curve
is projected from the same self-intersecting Bézier p-curve. It exercises the
default geometry-mode `FixSelfIntersection` path and records the DONE4 repair.
The `remove_loop_mode0_spatial` row uses a spatial 3D Bézier curve with the same
projected p-curve and checks the trimmed 3D curve at three interior parameters.

Build with the command in [the curve-gap fixture](tkshhealing_wire_curve_gaps.md),
substituting `tkshhealing_wire_self_intersection.cxx`, output name
`wire_self_intersection`, and adding `-lTKGeomAlgo` after `-lTKShHealing`.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_self_intersection
```

These observations do not establish full wire/toolkit parity.
