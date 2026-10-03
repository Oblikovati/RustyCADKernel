# Wireframe gaps and 3D conversion

OCCT oracle: `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.

Twelve wireframe rows compare return, DONE1/DONE2/FAIL1/FAIL2, shape identity,
context creation, edge count, curve ranges, endpoint and midpoint coordinates,
and spline degree. Inputs include gap-free free/face/empty/null shapes, repeated
compound children, a shortened triangle edge, reversed wire, two shortened
adjacent ends, a single edge with repeated first/last vertex, and both compound
child orders. A thirteenth row checks that a second direct FixGaps3d pass clears
the prior DONE flag and returns false.

Native line conversion retains the original shortened parameter interval,
increases spline degree to two, and moves its endpoint pole. A single-edge
conversion moves both endpoint poles. Compound recursion clears status for each
child: a repaired first child followed by an unchanged child returns false even
though the result shape changes. The port preserves these behaviors.

The conversion path also translates spline segmentation, conic approximation,
trim/offset extraction and nonmanifold-vertex copying, but these domains still
need broader native fixture coverage. Nonzero range-only FixGap3d is explicitly
unported. Full 2D gap repair and broader self-intersection cleanup remain partial.
These fixtures add no legacy-test mapping and do not prove full wireframe parity.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_wireframe_gaps`.
