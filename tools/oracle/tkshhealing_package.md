# ShapeFix package utilities

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`LeastEdgeSize` samples each stored 3D curve at its two bounds and midpoint,
then takes the least bounding-box diagonal. It is not an arc-length query.
The native probe covers an empty compound, an edge without a curve, a line,
a circle, and a box. Empty/no-curve results retain native sqrt(RealLast).
`EncodeRegularity` delegates to BRepLib; the cylinder oracle counts six edge
occurrences with recorded continuity after encoding. These are additional
oracle cases, not new legacy declaration mappings or whole-package proof.
