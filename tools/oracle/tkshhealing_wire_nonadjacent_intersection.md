# `ShapeFix_Wire::FixIntersectingEdges(num1, num2)` oracle

This fixture calls OCCT's private explicit-pair overload directly with indices
`(1, 3)` on a three-edge wire. The first and third p-curves cross at the origin;
the middle edge is spatially unrelated. It verifies endpoint tolerance growth
and the `DONE1` status, then repeats with `MaxTolerance=0.5` to verify `FAIL2`
without mutation. A disjoint `(1, 2)` pair returns false without setting `DONE1`
or `FAIL1`. The overloaded method is private and has no call sites in the pinned
OCCT source; the Rust implementation therefore remains internal too. A fourth
case gives both edges divergent 3D line representations and checks sampled
local deviation, edge tolerance growth, and all four endpoint tolerances.

The native fixture was compiled against OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
