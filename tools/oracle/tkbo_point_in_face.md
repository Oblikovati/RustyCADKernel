# Hatching and context nearby-point oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`tkbo_point_in_face` records 80 cases across five surface families and all four
face orientations, using automatically chosen hatch lines, explicit lines,
positive steps and lines outside the face. `tkbo_point_near_edge_context` adds
1,440 cases for the edge-ray PointInFace overload and both parameterized context
PointNearEdge overloads: six support/presence cases, sixteen orientation pairs,
and five tolerance pairs. All Rust comparisons use absolute error below 1e-12.

Native status codes and caller-output preservation are retained. Hatcher state is
cleared before adding the current curve; it is removed after normal completion,
including status-code failures. Exceptions propagate at their source operation.

These fixtures do not cover arbitrary holes, periodic splines, singularities,
unbounded hatch domains, every failed-trim status, or exception state changes.
The no-parameter PointNearEdge overload and OrientEdgeOnFace are still pending.
No legacy regression is marked complete by these supplemental cases.
