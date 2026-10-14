# Sketch line incidence oracles

`oracle.py` generates 195 near-degenerate cases with seed 3563. Expectations are
computed with Python `Fraction` arithmetic on the exact supplied binary64 values,
independently of the Rust implementation:

- Circles and ellipses use the closed-form quadratic in the line parameter. The
  exact discriminant decides the contact count and tangency. Positions and
  sector membership compare `(-b ± sqrt(D)) / 2a` with rationals by exact sign
  reasoning (`oracle_exact.py`).
- B-spline spans are rebuilt by exact interpolation of de Boor points, not by the
  Cox–de Boor basis used in the kernel. An exact Sturm sequence counts roots per
  span; `gcd(G, G')` chains give multiplicities; shared knot contacts are merged
  (`oracle_spline.py`).
- Each coordinate is recorded as the binary64 pair enclosing the exact value.
  Irrational coordinates come from 320-digit decimal square roots or a 2^-300
  parameter bracket; only a value that close to a binary64 boundary could be
  misplaced.

Families: lines within ±2 ulps of tangency to circles and ellipses at scales
1e-9 to 1e9; exact dyadic tangency; segment endpoints within ±2 ulps of a circle;
lines through arc centers with sector rays perturbed by one ulp; concentric
circles whose radii differ by 1, 2 and 5 ulps; target segment ends within ±2 ulps
of the line; nearly parallel segments; horizontal lines within ±2 ulps of a cubic
B-spline extremum; and a line through exactly representable knot points.

Rows hold `id;kind;domain;source;targets;contacts`. The Rust test certifies each
case on the supporting line with a relative world error of 1e-12 and checks the
contact count, exact order, target, both side multiplicities, line and target
positions, and that the certified coordinate bounds meet the exact coordinate.

```sh
python3 crates/tkmath/tests/sketch_line_incidences/oracle.py > /tmp/incidences.csv
cmp /tmp/incidences.csv crates/tkmath/tests/sketch_line_incidences/oracle.csv
cargo test --locked -p rustycad-tkmath --test toolkit sketch_line_incidences
```

The analytic tests in the sibling modules use hand-derived closed forms. Native
OCCT comparisons live with the TKGeomAlgo integration tests. See ADR 0060.
