# ShapeFix_Wire shifted-pcurve oracle

`tkshhealing_wire_shifted.cxx` probes periodic, bimeridian, cyclic-stop, configured-precision, and revolution-basis behavior. Its output is checked against `tkshhealing_wire_shifted.expected` and the Rust regressions in `shape_fix_wire_shifted.rs`.

Build against the pinned OCCT modeling tree with the command in `tkshhealing_wire_degenerated.md`, replacing both occurrences of `tkshhealing_wire_degenerated` with `tkshhealing_wire_shifted`.
