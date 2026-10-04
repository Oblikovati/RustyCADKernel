# ChFi3d face tangency oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`tkfillet_tangency.cxx` shares fixture geometry with connection classification.
The 288 rows contain curve kind, second-face normal case, first/second face
reversal, edge reversal, continuity order index (G1/C2/G2), native result.
Rust replays every row in `crates/tkfillet/tests/tangency.rs`.

Includes line and zero-midpoint-tangent Bezier edges, parallel/opposite/orthogonal
planes, missing cylindrical pcurves and same-face inputs. Source translation
preserves C2 null-second-derivative acceptance, repeated parameter increments,
second pcurve range selection, and ignored normal-helper boolean returns.

This is partial evidence: real seams, stored-continuity shortcut, curved/located
and spline boundary domains need additional native comparisons. No OCCT legacy
test is declared ported by this supplemental fixture.
