# Split-edge reversal oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

112 rows combine seven edge relationships and sixteen orientation pairs:
shared spatial curve, distinct parallel curves, opposite parameterization,
perpendicular curves, either edge degenerated, and two empty spatial curves.
Rust compares return/error/tangent columns exactly. Degenerated split edges leave
the tangent output unchanged; the same-curve fast path compares all orientation
values directly. Two null curve handles take that identity fast path; the fixture
does not call EdgeTangent on a null curve.

The translation samples ten interior points after FindValidRange and retains
native error codes for tangent/projection failures. These line cases do not cover
spline tangencies, zero derivatives, every retry/error branch, locations or the
face/shape overloads. The fixture does not complete a legacy regression mapping.
