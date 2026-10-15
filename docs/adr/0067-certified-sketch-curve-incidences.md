# Certified sketch curve incidences: curve sources and angle bounds

Status: accepted. Extends [ADR 0060](0060-certified-sketch-line-incidences.md),
which admitted only a line as the trimmed or extended source.

Application epic [#3563](https://github.com/Oblikovati/Oblikovati/issues/3563)
trims circles and arcs (Go `TrimCircle`, `TrimArc`) and needs the same certified
selection for every admitted sketch curve. This extends the ADR 0060 operation; it
reuses its exact root engine, report and selection types. No second engine is added.

## Decision

`SketchIncidenceSource` now carries a `curve`: a bounded line, a circle or
ellipse with an optional sector, or an owned spline. `SketchIncidenceSource::line`
keeps the ADR 0060 line source. Each source has exact places:

- Line and spline sources are placed by their parameter.
- A conic source is placed by the chart of the contact direction around a
  reference ray (the sector start, or the frame axis of a full conic or an
  angle-bounded arc) and, in each open half turn, by the cotangent key
  `-dot(r, q)/det(r, q)`; a linear map with positive determinant keeps the cyclic
  order of directions, so this is the sweep order of ellipses too.

Every pair is reduced to one exact univariate equation:

- implicit source (line, conic) with parametric target (line, spline span): the
  source equation composed with the span; the source key is the image of a rational
  map, recovered as an isolated root of `Res_u(N, P - T Q)`;
- parametric source (line, spline span) with implicit target (line, conic): the
  target equation composed with the source span; the parameter is the key;
- spline with spline: the coordinate equations eliminated in `u` (Sylvester
  resultant by exact interpolation) and the single fibre point `u = -beta/alpha`
  of the first subresultant;
- conic with conic: the source point `C + lambda d(K)` eliminated in `lambda`,
  `R(K) = E^2 - 4 c_S Q_S B^2`, with fibre `lambda = -E/(2 Q_S B)`. The line
  `B = 0` through the center (both half turns, `K` rational) and the reference
  line are certified with `sqrt(c_S/Q_S)` as an exact quadratic root; concentric
  pairs use the even system `E(K) = c_S Q_T - c_T Q_S`.

Exact signs at isolated roots (common factor for zero, exact enclosure otherwise)
decide sectors, charts, fibres and target positions; ties between different
equations are proved by common factors, as in ADR 0060.

## Support, closure and selection

The support of an arc is its full conic: contacts outside the sweep are
`AfterEnd`, ordered from the start through the gap. Extension of an arc searches
that gap from its end (`select_extension(End)`) or backwards from its start.
A full conic, and a spline whose ends are the same exact point, is closed: trim
opens it between the cyclic neighbours of the pick, as Go `TrimCircle` does, and
needs two distinct cut points (`ClosedSourceCuts`). Conic picks are projected
centrally (Go `arcParamOf` uses the angle about the center); spline picks are an
exact parameter (`SketchTrimPick::Parameter`), since no projection rule is
defined. No spline extension rule is defined by the application (Go
`ExtendLine` only extends lines), so a spline source refuses the support domain
by name (`SourceDomain`) instead of inventing one. Coincident conics report the
target sweep as a cyclic overlap and selection compares it cyclically.

## Angle bounds

`SketchConicAngles` admits the application's radian spans: a frame axis, sense and
binary64 bounds, interpreted as exact real angles of the parameterization
`C + ra cos t a + rb sin t n`. For a nonzero rational `t`, `tan t` is
transcendental (Lindemann-Weierstrass) while every contact direction is algebraic,
so a contact is never on a nonzero bound: membership and order are decided by
refining rigorous enclosures (Taylor series with Lagrange remainders, Machin's pi)
together with the contact root, and only a zero bound is an exact ray
(`AtStart`/`AtEnd`). A rounded right angle therefore excludes the exact quarter
point. Spans must be below one full turn (certified against pi), magnitudes at most
64, and an ellipse's frame axis parallel to its major axis (`AngleBounds`,
`AngleAxis`). Rays remain the preferred representation of authored arcs.

## Open representations

Native periodic knot vectors and offset curves are admitted by
[ADR 0080](0080-certified-periodic-and-offset-incidences.md). Spline-source
extension (no application rule), overlaps other than line-source and conic-conic
ones, and the exceptional cases named below remain open. A future extension needs its own exact equation, evidence and behavior
version.

## Named incomplete results

`UncertifiedOverlap` (a piece on the other curve outside the line-source and
conic-conic overlaps), `ExceptionalFiber` (a spline-pair fibre that is not one
target point, e.g. an unevenly parameterized straight span of degree two or more),
`KnotTangency` (a tangential piece at a target interior knot), and
`ExceptionalTangency` (tangential conics at a center-line pair). None returns
partial contacts.

## Evidence

Hand-derived closed forms for every pair kind, angle bounds and selection; 244
generated near-degenerate cases (ulp-level tangencies, arc end rays and angle
bounds, Pythagorean segment ends, tangent circle pairs, ellipse sources, spline
knots, parabola pairs) against an independent Python `Fraction` oracle that uses
different eliminations (Cartesian conic resultants, implicitized target spans,
320-digit angles with a refused 1e-200 gap); 76 native OCCT 8.0.0
`Geom2dAPI_InterCurveCurve` cases and the mapped Rust intersector; reports equal
on eight threads and across processes. Replay commands are in
[sketch-line-incidences.md](../sketch-line-incidences.md).

This is additive behavior version 1 for the new sources; ADR 0060 line-source
results are unchanged.
