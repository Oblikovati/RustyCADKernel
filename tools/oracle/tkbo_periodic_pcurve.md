# Periodic pcurve adjustment oracle

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The 140 rows cover plane, cylinder and torus support faces with short and requested
multi-period parameter boxes. Each row records face case, U/V input indices,
returned handle identity, and the translated midpoint. The Rust fixture checks
fresh and cached face adaptors and verifies that the original curve is unchanged.

The source's adjusted-U plus DU classifier ordering is preserved, including the
cylinder tolerance fallback and nearest-midpoint V shift selection. Requested
multi-period boxes pass through MakeFace's native normalization; this fixture does
not independently prove every classifier branch on arbitrary face boundaries.
Periodic spline surfaces, located faces and exceptional domains remain to cover.
These supplemental fixtures do not map an OCCT legacy regression.
