# Wire seam ordering

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Seven native observations cover swapped/correct seam curves on a cylinder,
reversed face/edge occurrences, an unloaded analyzer, and an ordinary edge.
Curve outputs and range begin with sentinels to verify preservation before seam
extraction. Rust compares every field exactly and exercises both overloads.

Rows begin with `found DONE1 first_curve_x second_curve_x first last wrapper_found`.
They continue with FixSeam result/DONE1, resulting forward/reverse curve X values,
retained range and edge tolerance, then second FixSeam result/OK. This verifies
repair mutation and idempotence independently of occurrence orientation.

Build with the command in [the curve-gap fixture](tkshhealing_wire_curve_gaps.md),
substituting `tkshhealing_wire_seam.cxx` and output name `wire_seam`.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_seam
```

These observations do not establish full wire/toolkit parity.
