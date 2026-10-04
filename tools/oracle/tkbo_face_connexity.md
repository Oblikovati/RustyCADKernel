# First face connectivity block

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkbo_face_connexity.cxx` records 20 MakeConnexityBlock cases: unrestricted and
fully/partly avoided box edges, disconnected boxes and empty input, two seeds,
and fresh/prepopulated output lists. Replay:
`cargo test -p rustycad-tkbo --test solid_orientation connexity`.

The source uses ordered frontier maps, appends to existing output, leaves input
and avoidance maps intact, and raises Standard_NoSuchObject for empty input.
The Rust implementation preserves those observations. Native allocator injection
is not exposed by this Rust entry point; it uses project collection defaults.

Supplemental fixtures do not prove arbitrary topology, allocator behavior or
full legacy parity. The mapping remains partial.
