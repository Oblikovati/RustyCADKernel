# Boolean vertex intersection and construction

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_vertex_tools.cxx` with installed OCCT headers and link TKBO,
TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Default output is `tkbo_vertex_compare.expected`; arguments `list` and `pair`
produce `tkbo_vertex_list.expected` and `tkbo_vertex_pair.expected`.

36 comparison cases sample inside, exact-boundary and outside distances over
three tolerance and three fuzzy scales. ComputeVV uses a strict greater-than
rejection; the point overload always adds Confusion, while the vertex overload
uses the greater of fuzz and Confusion. DTolerance is checked as 1e-12.

Six list cases preserve output identity on empty input and input identity on a
singleton, then check the BoundingVertex result for two to five vertices.
Twelve new-vertex cases cover edge/edge, edge/face, vertex/vertex and point inputs,
checking position, tolerance, fresh identity and output orientation. Floating
columns agree within 1e-12. The midpoint overload intentionally differs from
the minimal bounding sphere used by the list overload.

DTolerance is implemented; the other API rows remain partial for located and
non-finite inputs, broader geometry and exceptional output state. These are
supplemental comparisons, not additional legacy-test mappings.
