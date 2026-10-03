# Shape repair dispatch

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The adjacent C++ fixture has sixteen rows comparing return values, all DONE
status bits, result topology counts and the shared face tool's small-area mode.
Cases cover solids, shells, faces, edges, vertices, disabled modes, compound
recursion, cancellation, repeated calls, Init retention and vertex-position mode.

The port replaces the earlier all-descendant-face pass and generic shell
orientation helper with the native dispatch through shared solid/shell/face
repair tools. The old bowtie-prism regression still passes through this path.
Native solids set DONE4 despite the header describing DONE5. Repeated calls can
return true with no DONE bits; Init keeps the visited map. A repeated face takes
an early return that leaves its small-area mode enabled. These behaviors are
preserved instead of normalizing them.

Free-wire dispatch currently raises Standard_NotImplemented while aggregate
ShapeFix_Wire::Perform is being ported. ShapeFixFace::Perform also remains a
partial dependency. The pre-existing Rust null-shape no-op differs from native
ShapeType failure and is explicitly retained as a known parity gap, not an
approved exception. General geometry, assembly placements and failure domains
need additional evidence. These tests do not add legacy declaration mappings.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_shape_dispatch`.
