# Independent annular scalar expectations

The owned fixture `crates/tkmath/tests/fixtures/prism_complete_measures/closed_cubic.json`
retains the independent rational enclosure for the extrusion side of the closed cubic
with poles `(0,0),(4,0),(0,4),(0,0)` and extrusion `(0,0,2)`. Its Green integral
is exactly `12/5`. The inner cubic is translated by `(1/2,1/2)` and scaled by
`1/2`, so its area is `3/5` and its side area is half the outer side area `S`.
Thus each material cap has area `9/5`, the positive signed material sweep is
`18/5`, and the complete surface area is `18/5 + (3/2) S`.

`crates/tkbo/tests/fixtures/annular_measures/derive.py` applies only these exact
rational transformations and outward binary64 endpoint rounding. The original
fixture is not changed or regenerated. `material.json` records its SHA-256 and
all exact transformed fractions; `material.tsv` supplies endpoint bits to tests.
`jacobian-total.json` independently checks both rational total endpoints using
Jacobian's exact `polynomial.map.evaluate` operation over QQ.

The tests exercise stored-pole reversal independently from traversal reversal.
Neither changes the normalized material quantities. Reflection of the source
plane or extrusion reversal changes the signed sweep sign; applying both
preserves it. Cap, side and total areas remain nonnegative and unchanged for
these isometries. These are scalar expectations conditional on the sealed
annular domain, not substitutes for per-face orientation or body validation.
