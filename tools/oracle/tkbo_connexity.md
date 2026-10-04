# BOPTools connectivity oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_connexity.cxx` covers a box with edge/face and vertex/edge connections,
a single closed face wire, an empty compound, and two disconnected face wires.
The Rust fixture compares all emitted rows byte-for-byte after serialization.

All three MakeConnexityBlocks overloads are translated. Existing output contents
are retained; two invocations with the same ancestor map accumulate native list
entries. Fixtures verify block and connection-map traversal order, shape identity,
orientation, duplicate inputs and regularity. A repeated input is emitted Forward
and Reversed and marks its block irregular. The ConnexityBlock lists use the
supplied allocator and retain regularity/list mutations.

These are supplemental oracle cases, not completed OCCT legacy regressions.
Prepopulated connection maps containing elements outside the input shape and
broader shape-kind combinations still need native differential evidence.
