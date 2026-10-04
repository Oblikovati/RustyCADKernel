# Pcurve projection oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The 100 native rows exercise both MakePCurveOnFace overloads with cached and
fresh adaptors, five starting tolerances, and plane, cylinder, torus,
revolution and flat Bezier supports. Full-curve cases use trimmed spatial curves.
Rust checks exact parameter ranges, absolute reached-tolerance error below
1e-14, and UV sample distance below 1e-12. These are numerical comparisons,
not a claim of bitwise equality. Native fixtures reproduce their checked-in rows
exactly after compilation against the pinned kernel.

The translation preserves projection settings, revolution handling, retry,
periodic adjustment, same-range correction and final distance tolerance updates.
The U/V extension ordering intentionally reloads each trim from the original
adaptor, matching the source. The current fixtures do not force all retries or
extensions, arbitrary spline geometry, locations, singularities or exceptions.
These supplemental cases do not complete a legacy regression mapping.
