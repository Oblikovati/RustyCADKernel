# `BRepOffset_Offset::Init` with an unmatched located connector

The supplied start connector is translated away from the rail endpoints. Native `Init` compares
its located first vertex, finds no match, and creates a replacement connector; the supplied edge
remains without a face p-curve. The matching last connector is reused. The result is valid and
has the same face measurements as the unmatched identity-located case.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`. Compile with the pinned modeling
build/link libraries described by [`tkoffset_offset_rails_with_connectors.md`](tkoffset_offset_rails_with_connectors.md),
using this source file and executable name `tkoffset_offset_rails_nonidentity_connector`.
