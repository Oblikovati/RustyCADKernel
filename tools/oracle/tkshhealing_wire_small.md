# Small-edge classification

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Eleven native observations exercise single-edge suppression, missing vertices,
shared versus distinct endpoint identity, long endpoint span, 3D midpoint
rejection, pcurve fallback, and degenerate edges with/without pcurves. All
values are compared exactly in Rust. Repeating the aggregate check at negative
tolerance verifies that the previously accumulated DONE flags remain set.

Rows: `single_found DONE1 DONE2 FAIL1 FAIL2 aggregate_found aggregate_DONE
aggregate_FAIL negative_tolerance_found retained_DONE cleared_OK`.

Build with the command in [the curve-gap fixture](tkshhealing_wire_curve_gaps.md),
substituting `tkshhealing_wire_small.cxx` and the output binary name `wire_small`.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_small
```

These observations do not establish full wire/toolkit parity.
