# Wire analyzer aggregate execution

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Five native observations cover closed square, bow-tie with a nonadjacent crossing,
reversed square, empty wire and open contour. Each runs the aggregate
self-intersection check followed by all eight checks in `Perform`, then clears
statuses. Rust compares every result and status bit exactly.

Rows: `self_found self_DONE1 self_DONE2 self_DONE3 self_FAIL1 self_FAIL2
self_FAIL3 last_DONE last_FAIL perform_found`, followed by all DONE flags then
all FAIL flags in this order: `Order Small Connected EdgeCurves Degenerated
SelfIntersection Lacking Closed LastCheck`. The final flag is cleared self OK.
The single-edge self-intersection fixture additionally verifies local aggregate
success and failure categories on the Bézier loop and missing inputs.

Build with the command in [the curve-gap fixture](tkshhealing_wire_curve_gaps.md),
substituting `tkshhealing_wire_perform.cxx`, output name `wire_perform`, and adding
`-lTKTopAlgo` after `-lTKShHealing`.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_perform \
  --test shape_analysis_wire_self_intersection
```

These observations do not establish full wire/toolkit parity.
