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

## Conic and spline sources

`oracle_curves.py` generates 244 near-degenerate cases with seed 6100 for
sources other than a line (ADR 0067). It uses different eliminations from the
kernel: conic sources against lines and splines substitute the target piece into
the conic; conic pairs eliminate `y` (or `x`) between the Cartesian equations
instead of using the sweep chart; spline sources substitute their de Boor
interpolated spans into the target's implicit equation; spline pairs implicitize
the target span with a symbolic Sylvester determinant. Roots are isolated with
exact Fraction Sturm sequences, multiple roots are refined on their square-free
part, and every sign, position, transition and multiplicity is exact. Only the
sweep order of distinct contacts on a conic source and the coordinate brackets
use 2^-300 refinements; the generator refuses any order decided within 1e-200.

Families: circles tangent to lines within ±2 ulps at scales 1e-6 to 1e6, an
exact dyadic tangency, arc end rays perturbed by one ulp, Pythagorean segment ends
within ±2 ulps of a circle source, externally and internally tangent circle pairs
within ±2 ulps (exact at zero steps), near-tangent rotated circle pairs, ellipse
sources near tangency to circles, circle sources touching a cubic spline's top,
cubic spline sources near tangency to lines, circles through spline knot points,
a polyline source through a circle arc's end ray, and parabola pairs within a
few ulps of tangency at scales 2^-20 to 2^20; angle-bounded arcs whose bound lies a few ulps from a
contact angle, as targets of a polyline and as circle and ellipse sources.

```sh
python3 crates/tkmath/tests/sketch_line_incidences/oracle_curves.py > /tmp/curves.csv
cmp /tmp/curves.csv crates/tkmath/tests/sketch_line_incidences/oracle-curves.csv
cargo test --locked -p rustycad-tkmath --test toolkit sketch_line_incidences::generated_curves
```

## Periodic splines and offsets

`oracle_offsets.py` generates 211 near-degenerate cases with seed 8080 (ADR 0080).
An offset point is `C + d N` with `N = (V_y, -V_x) / sqrt(G)`; the oracle composes
curve equations as `a + b sqrt(G)` with its own Fraction polynomial algebra
(`oracle_offsets_exact.py`), isolates the norm `a^2 - b^2 G` on its square-free
part, keeps a norm root only when `a` and `b` vanish together or have opposite
exact signs, splits multiplicities through `gcd(a, b)` (the kernel uses derivative
orders), and takes sides from exact surd signs at rational points. Keys and
coordinates are 320-digit Decimal values at a 2^-300 refinement; the generator
refuses orders within 1e-200. Ellipses use their own tangent-half-angle charts;
periodic splines are unwrapped by periodic indexing, `base[j mod N] + period *
floor(j / N)`, independently of `BSplCLib::KnotSequence`.

Families: lines within +-2 ulps of the top of an axis-aligned and a (3, 4) ellipse
offset at scales 2^-10 to 2^10; lines near the sampled top of cubic spline offsets
(including the crossing of a self-intersecting offset); circles about the origin
within +-2 ulps of tangency to line offsets with directions (1, 2), (2, -1) and
the Pythagorean (3, 4); a line offset source near tangency to circles and to an
ellipse offset; spline and ellipse offset sources near tangency to lines and
lines through the ellipse axes; full-circle and clockwise-arc offsets; periodic
cubic sources and targets near their tops.

```sh
python3 crates/tkmath/tests/sketch_line_incidences/oracle_offsets.py > /tmp/offsets.csv
cmp /tmp/offsets.csv crates/tkmath/tests/sketch_line_incidences/oracle-offsets.csv
cargo test --locked -p rustycad-tkmath --test toolkit sketch_line_incidences::generated_offsets
```
