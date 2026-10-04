# `BRepOffset_Offset::Init` with a partially matching `FirstEdge`

This fixture supplies an edge whose first vertex matches one start rail but whose second vertex
is unrelated to the other rail. OCCT only checks the supplied edge's first vertex. It keeps and
updates that edge, resulting in a face that has the native four-edge topology and area but fails
`BRepCheck_Analyzer`. The mapping intentionally preserves this invalid output rather than repairing
the malformed connector.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`. The pinned build command and
link libraries match [`tkoffset_offset_rails_with_connectors.md`](tkoffset_offset_rails_with_connectors.md);
compile this fixture source and use executable name `tkoffset_offset_rails_partial_connector`.
