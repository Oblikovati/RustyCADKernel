# Wire small-area screening

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Nine native observations exercise the precision threshold, reversed wire,
a different argument wire from the analyzer's loaded contour, and empty/missing
face/geometry cases. The loaded contour supplies the 23-point sampled screen;
the argument supplies the final mass-property check. Rust compares all flags
exactly, including the strict threshold at precision 0.5 for a unit square.

Rows: `found DONE1 FAIL1 FAIL2`.

Build with the command in [the curve-gap fixture](tkshhealing_wire_curve_gaps.md),
substituting `tkshhealing_wire_small_area.cxx`, output name `wire_small_area`,
and adding `-lTKTopAlgo` after `-lTKShHealing`.

```sh
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_small_area
```

These observations do not establish full wire/toolkit parity.
