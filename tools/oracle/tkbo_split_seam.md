# Seam pcurve reconstruction oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

208 original-seam cases and 176 periodic-boundary cases cover cylinder and torus
support, both seam sides, displaced or reversed pcurves, all edge/face orientations,
and already-closed edges. The original-seam overload also checks absent pcurves
and nonseam originals. Rust compares status, closedness, both pcurve ranges and
sample points, using absolute numerical error below 1e-12.

The original-seam overload projects onto both original curves, preserves the
second lookup's range, and selects pair order using the native distance/tangent
comparison. The periodic overload preserves closed-versus-periodic handling,
U-then-V side selection and strict resolution thresholds.

The translated trimmed-surface fallback and general spline seams still need
native fixtures, as do failed projection exceptions and located faces. Null
pcurves in the periodic overload are outside these fixtures (native dereferences
them). These supplemental cases do not complete a legacy regression mapping.
