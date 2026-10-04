# Explicit-step PointNearEdge oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

480 native rows cover plane, cylinder, torus, revolution and Bezier supports,
a missing cylinder pcurve, all edge/face orientation combinations, and five edge
and face tolerance pairs. Rust checks status and UV/XYZ outputs with absolute
error below 1e-12. Missing pcurves return 1 and preserve caller outputs.
Cylinder cases exercise both acos conversion and its out-of-domain fallback.

The source's NPAL19220 spline face-tolerance replacement is translated; spline
and spherical branch fixtures, locations and singular/error cases remain pending.
Context-based overloads with hatching fallback are separate work. These are
supplemental fixtures, not completed legacy regression mappings.
