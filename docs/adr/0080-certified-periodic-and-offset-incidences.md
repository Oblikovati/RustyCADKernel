# ADR 0080: Certified sketch incidences of periodic splines and offset curves

Status: implemented on a worktree branch (2026-10-15); integration is the coordinator's.
Extends [ADR 0060](0060-certified-sketch-line-incidences.md) and
[ADR 0067](0067-certified-sketch-curve-incidences.md), which left native periodic
knot vectors, offset curves and spline-source extension open.

Application epic [#3563](https://github.com/Oblikovati/Oblikovati/issues/3563)
trims and extends its offset geometry and closed (periodic) splines with the same
certified selection as other sketch curves. This extends the ADR 0060 operation;
no second root engine is added.

## Decision

### Periodic B-splines

`SketchIncidenceCurve::PeriodicSpline` takes the native OCCT form: degree, the
poles of one period, distinct knots, multiplicities and bounds. It is unwrapped
before certification into the flat-knot spline of ADR 0060:

- flat knots are the ported `BSplCLib::KnotSequence(knots, mults, degree, true)`;
- poles wrap as in `BSplCLib::Unperiodize` (`P_k = poles[k mod n]`);
- the unwrapped curve equals the periodic curve exactly when every period-shifted
  flat knot is exact in binary64. Admission checks `flat[i + n] - flat[i] = period`
  in rational arithmetic; an inexact shift is refused (`PeriodicKnotShift`), as is
  a pole or multiplicity count that `BSplCLib::NbPoles` rejects (`PeriodicShape`).

Reports keep the periodic request. Tests prove the unwrap equals the ported
`Unperiodize` form and that reports equal those of the hand-unwrapped input.

### Offset curves

`SketchIncidenceCurve::Offset { basis, distance }` follows `Geom2d_OffsetCurve`:
`C(u) + d (C'_y, -C'_x) / |C'(u)|`, right of the travel direction; a full conic
travels counterclockwise and an arc in its sector sense. The offset keeps the
basis parameter, sector, bounds and domains. Admission refuses nested offsets
(`OffsetBasis`), a circle offset through its center (`OffsetRadius`), an ellipse
whose major axis has irrational length (`OffsetEllipseAxis`), and a spline basis
that is C0 inside its bounds (`OffsetContinuity`) or has a vanishing derivative on
a span (`OffsetSingular`, an exact Sturm count). A zero distance is the basis.

- A circle offset is the concentric circle of exact radius `r + d` (`r - d` for a
  clockwise arc) and follows the circle paths everywhere.
- Other offsets are exact surd pieces. A basis piece `(x, y) / w` with tangent
  `V = (x' w - x w', y' w - y w')` and squared speed `G = |V|^2 > 0` has the point
  `(x sqrt(G) + d w V_y, y sqrt(G) - d w V_x) / (w sqrt(G))`. An ellipse uses two
  tangent-half-angle charts on `[-1, 1]`; its unit axis is rational when admitted.
- Composing an implicit equation with such a point gives `E = P + R sqrt(G)`. The
  offset equation is squared to its norm `H = P^2 - R^2 G`, whose roots are
  isolated by the ADR 0004 engine; a root of `H` is a contact exactly when `P` and
  `R` vanish together or have opposite signs (exact signs at the root). This is
  the sign filter that rejects the spurious roots of the squared equation.
- Sides are exact surd signs at rational points between consecutive roots of `H`.
  The multiplicity is that of `H` unless `R` vanishes at the root; then it is the
  order of `E` from the orders of `P`, `R` and the leading surd coefficient.
- A radicand that is a polynomial square (a Pythagorean direction such as `(3, 4)`)
  makes `E` rational; it is recovered exactly and isolated directly.
- A line offset has an irrational constant, so as an implicit curve it is squared,
  `(w det)^2 - d^2 |e|^2 w^2 = 0`, and its side is kept by the exact filter
  `sign(w det) = -sign(d)`. Its parameter is the basis line parameter (the offset
  is orthogonal to the line), so its overlaps with lines are rational.
- Position tests, sectors, construction and source keys are generic over
  polynomials and surds. A surd source key `N / D` is compared lazily: exactly
  with a rational by the sign of `N - v D`, by refining enclosures otherwise, and
  only an unresolved comparison builds its image equation
  `Res_v(H, (K d1 - n1)^2 - (K d2 - n2)^2 G)`, whose common factors prove a tie.

Pairs (source against target):

| Source \ target | line, conic, circle offset | line offset | spline | ellipse or spline offset |
| --- | --- | --- | --- | --- |
| line, line offset, conic | certified | certified | certified | certified |
| spline | certified | certified | certified (ADR 0067) | `OffsetPair` |
| ellipse or spline offset | certified | certified | `OffsetPair` | `OffsetPair` |

An ellipse or spline offset has no implicit equation, so it meets only implicit
partners; the refused pairs would need a two-parameter surd elimination. A line
source coincident with a line offset target (and any other offset overlap) remains
`UncertifiedOverlap`; a line offset source certifies coincident line and spline
spans like a line.

### Selection

A line offset source trims by orthogonal projection; a circle offset centrally;
a spline offset by an exact parameter. An ellipse offset source has neither a
point projection rule nor a parameter (its places are the sweep of its basis
points), so it refuses trim picks (`PickKind`). Spline-source extension stays
refused: the application has no rule, so spline and spline offset sources refuse
the support domain (`SourceDomain`), and none is invented here.

## Limits

Version 1 limits are unchanged. The norm of a conic against a cubic spline
offset has degree 20 and its image equation degree 40, beyond the default 32:
such a key comparison fails with the named degree failure only when two keys tie
to more than 256 refinements; distinct keys separate by refinement first.

Cost grows with the basis: a closed Go fit recipe has non-dyadic knots, and its
offset against a conic gives a degree-20 norm with about 2,700-bit coefficients
per span, which the shared isolation engine needs about 15 s per span to isolate
in a debug build (most other offset pairs take milliseconds to a second). This is
a measured local diagnostic, not a timing gate.

## Evidence

Hand-derived closed forms (`periodic_splines`, `offset_circles`, `offset_lines`,
`offset_curves`, `offset_failures`), unit tests of the surd algebra and the
unwrap, 211 generated near-degenerate cases against an independent Fraction
oracle (`oracle_offsets.py`), 108 native OCCT 8.0.0 `Geom2dAPI_InterCurveCurve`
offset and periodic cases plus the mapped Rust intersector, and thread and process
determinism checks. Replay commands are in
[sketch-line-incidences.md](../sketch-line-incidences.md).

This is additive behavior version 1 for the new representations; ADR 0060 and
ADR 0067 results are unchanged.
