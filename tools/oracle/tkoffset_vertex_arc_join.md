# Vertex offset sphere primitive

OCCT oracle: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkoffset_vertex_arc_join.cxx` records positive and negative default spherical
vertex joins against the optimized native modeling build. The expected text
records surface kind, native status, topology occurrence counts, radius, frame,
face orientation, validity and area. Rust tests cover the corresponding geometry.

The default `BRepOffset_Offset(Vertex, edges, Offset)` mode is also exposed
through the shared Rust offset facade. It preserves the initial vertex, Good
status, spherical face, and native null `Generated` result for this mode. The
fixture's standalone spherical face is invalid with zero area in both kernels;
no successful complete offset is claimed from this primitive. Polynomial
surface approximation options are not yet exposed.
`tkoffset_standard_offset_occ28131.cxx` is an investigative native probe, not a
Rust legacy test mapping. The remaining StandardOffset regression stays unmapped.
