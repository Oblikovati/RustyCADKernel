# Oriented face normals

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.
Eighty cases compare NggeomF, Nt and minDUV on planes, spheres, cones and Bezier
surfaces with all four face orientations. Fields include return flags, parameter
span, geometric vector and oriented direction. Flags -1 identify native
Standard_ConstructionError, whose class is checked in Rust.

The fully collapsed Bezier surface throws from zero-normal construction. A tiny
nonzero Bezier instead returns false and leaves the caller's vector/direction
unchanged. Sphere poles return radius-length geometric vectors; Nt normalizes
them. The cone's apex and shifted-v fallback retain the native branch conditions.

These are supplemental differential fixtures. Located and general curved-spline
coverage remains incomplete, and TKBool's general operation algorithms are not
claimed complete. No legacy-test mapping is added.
