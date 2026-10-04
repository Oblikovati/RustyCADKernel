# Empty-shape geometry presence oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Fourteen cases compare null and empty topology, a vertex without geometric
initialization, a spatial edge, surface/triangulated faces, compound descendants,
null/non-null 3D polygons and regularity representations. A PolygonOnSurface
representation raises native Standard_DomainError because IsEmptyShape calls
Polygon3D before testing the remaining polygon predicates; Rust preserves this
ordering and exception class. Presence answers and exception results match exactly.

More complex shared topology, all polygon representation subtypes and locations
remain to cover. These supplemental tests do not complete a legacy regression.
