# `BRepOffset_Offset::Init` with a closed circular path

The path is a full circle; the two section edges are vertical lines at distinct radii. Native
`Init` makes one connector edge and inserts the same edge twice, in opposite orientations, at
the closed path seam. The generated image therefore occurs twice in the face. The returned face
is invalid according to `BRepCheck_Analyzer`; this fixture preserves the native result.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`. Compile with the pinned modeling
build/link libraries described by [`tkoffset_offset_rails_with_connectors.md`](tkoffset_offset_rails_with_connectors.md),
using this source file and executable name `tkoffset_offset_rails_closed_path`.
