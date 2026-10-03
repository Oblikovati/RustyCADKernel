# ShapeFix_Wire shifted-pcurve oracle

`tkshhealing_wire_shifted.cxx` probes periodic, bimeridian, cyclic-stop, configured-precision, and revolution-basis behavior. Its output is checked against `tkshhealing_wire_shifted.expected` and the Rust regressions in `shape_fix_wire_shifted.rs`.

Build against the pinned OCCT modeling tree with the command in `tkshhealing_wire_degenerated.md`, replacing both occurrences of `tkshhealing_wire_degenerated` with `tkshhealing_wire_shifted`.

The `singular_traversal` case uses fixer precision 0.4 so geometric singularity
recognition actually enters the singular stop branch. Native traversal revisits
the prefix and leaves Surface().Gap() at 0.26996968193783172; the prior fixed-stop
translation left 0.07078154273460796. The regression fails before the correction
and compares the corrected native gap exactly. Near-degenerate revolution
classification now precedes the same stop/bi-meridian decisions; a missing
previous p-curve in that early path is skipped without adding FAIL1, as upstream.
Period shifts reuse the existing ShapeAnalysis routine, including its tiny-period
guard, and the 80% threshold preserves native `period - 0.2 * period` arithmetic.
