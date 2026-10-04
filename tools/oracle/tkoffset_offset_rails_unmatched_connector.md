# `BRepOffset_Offset::Init` with an unmatched `FirstEdge`

This fixture passes a valid line edge whose vertices do not match either start rail vertex.
OCCT ignores that edge, creates a new start connector, and leaves the supplied edge without a
face p-curve. The matching `LastEdge` is reused. Native status, edge/curve counts, area, generated
history identity, and validity are recorded. The behavior follows the native branch that replaces
an edge when its first endpoint shares neither expected rail vertex.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile with the same pinned modeling build described by
[`tkoffset_offset_rails_with_connectors.md`](tkoffset_offset_rails_with_connectors.md), replacing
the fixture source and executable name with `tkoffset_offset_rails_unmatched_connector`.
