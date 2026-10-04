# Vertex offset sphere primitive

OCCT oracle: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkoffset_vertex_arc_join.cxx` records positive and negative default spherical
vertex joins against the optimized native modeling build. The expected text
records surface kind, native status, topology occurrence counts, radius, frame,
face orientation, validity and area. Rust tests cover the corresponding geometry.

This helper represents the default geometric branch of BRepOffset_Offset::Init;
it does not yet provide the complete object's state/history API or polynomial
options. The fixture's standalone spherical face is invalid with zero area in
both kernels; no successful complete offset is claimed from this primitive.
`tkoffset_standard_offset_occ28131.cxx` is an investigative native probe, not a
Rust legacy test mapping. The remaining StandardOffset regression stays unmapped.
