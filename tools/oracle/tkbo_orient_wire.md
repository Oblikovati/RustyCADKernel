# Boolean wire orientation

OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_orient_wire.cxx` records 192 native OrientEdgesOnWire cases: chains,
cycles, branch vertices, closed edges, vertex-free edges, and empty wires,
combined with eight edge-direction masks and four wire orientations.
Rust replay: `cargo test -p rustycad-tkbo --test orient_wire`.

Checks include output identity and orientation, edge identity, insertion order,
and child orientation without cumulative wire orientation. The source traversal
only propagates through vertices with exactly two ancestor edge occurrences.
It preserves closed edge seeds and leaves vertex-free inputs unchanged.

This source translation and supplemental oracle do not prove arbitrary located
or malformed topology behavior, or full legacy-test parity. The mapping is partial.
