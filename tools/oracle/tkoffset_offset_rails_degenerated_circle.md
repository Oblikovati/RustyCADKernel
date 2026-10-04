# `BRepOffset_Offset::Init` with a sub-confusion circular rail

The first rail is a full circle with radius `0.5 * Precision::Confusion()`. Native `Init` removes
its 3D curve and marks the edge degenerated before assigning the face p-curve. The output has four
edge occurrences, one 3D curve, and one degenerated edge. The Rust regression checks those counts.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`. Compile with the pinned modeling
build/link libraries described by [`tkoffset_offset_rails_with_connectors.md`](tkoffset_offset_rails_with_connectors.md),
using this source file and executable name `tkoffset_offset_rails_degenerated_circle`.
