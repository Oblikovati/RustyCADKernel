# PaveFiller vertex–face differential fixture

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`,
BOPAlgo_PaveFiller_4.cxx, PerformVF and TreatVerticesEE.

Nine scenarios, each repeated, produce 161 compared rows:

- Interior planar vertex, destructive tolerance growth, and non-destructive copy.
- Disjoint vertex, boundary rejection, and full-glue face initialization.
- Two crossing edges on a face: four source vertices and the generated EE vertex.
- VV same-domain grouping retains both original VF records and one representative.
- A preceding VE hit excludes VF while still initializing the face record.

Rows compare shape counts, every VF index/UV/new-index record, every vertex's
tolerance and same-domain representative, and FaceInfo presence/internal map order.
TreatVerticesEE deliberately repeats its VF record on subsequent calls; ordinary
VF interferences are deduplicated. This preserves native behavior.

Projection is staged before vertex mutation. Generated-vertex selection uses the
native linear BVH pair-selection primitives and normalized/sorted subset pairs;
it does not claim a complete standalone BOPDS_SubIterator API implementation.

Remaining proof gaps include curved surfaces, warning paths, cancellation and
parallel scheduling, allocation behavior, pathological shapes, wider subset
traversal configurations, and complete legacy declaration parity. This fixture
is supplemental and does not count as a ported legacy declaration.

Build the C++ probe against the pinned native modeling include and libi paths,
link TKBO, TKTopAlgo, TKBRep, TKGeomAlgo, TKG3d, TKMath and TKernel; compare stdout
with the expected fixture. Replay with
`cargo test -p rustycad-tkbo --test bop_algo_pave_filler_vf`.
