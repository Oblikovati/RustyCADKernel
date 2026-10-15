# Certified sketch line incidences for trim and extension

Application epic [#3563](https://github.com/Oblikovati/Oblikovati/issues/3563)
selects trim and extension targets from the incidences of a sketch line with the
other sketch curves. Its released adapter replays mapped `Geom2dInt_GInter` and
returns native completion, points, overlaps, parameters and measured point
residuals. Those values do not establish that every root was found or bound the
geometric error of a root. ADR 0013 orders supplied coordinates only. Kernel
[issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10) shares the
exact classification dependencies; this is not a second intersection engine.

## Decision

Add `predicates::roots::sketch_incidence::certify_sketch_line_incidences`, an
extension in TKMath. One source line (two distinct points, segment or support
domain) is certified against owned targets: segments and infinite lines,
circles and ellipses with optional angular sectors, and non-periodic B-splines
with flat knots and optional positive weights. Every contact is an isolated root
of an exact integer equation built from the supplied binary64 values:

- Conics substitute `A + t (B - A)` into their implicit equation. Circles give
  `|q + t d|^2 - r^2`; ellipses `rb^2 (q.a)^2 + ra^2 det(a,q)^2 - ra^2 rb^2 |a|^2`,
  with any nonzero major-axis vector. All terms are homogeneous in the exact
  integer lift, so the scale cancels. The leading coefficient is positive: a conic
  never overlaps a line.
- Lines and B-spline spans use exact Cox–de Boor power pieces `(X, Y)/W` on each
  nonempty knot span clipped to the bounds. `N = det(d, C - A)` gives contacts in
  the target parameter; `R(T) = Res_u(N, L - m T W)` carries each contact's line
  parameter. `R` is recovered exactly by interpolating Sylvester determinants.
  The unique `R` root inside the exact enclosure of `L/(mW)` is the line
  parameter; the target bracket is refined until that root is unique.

Isolation, Sturm chains and exact dyadic arithmetic reuse the ADR 0004 engine
and the shared certificate arithmetic. Full supports are isolated inside an exact
Cauchy bound. The new code adds equations, comparisons and selection, not a
solver. Mapped OCCT intersectors and the existing Bézier/line operations are
unchanged.

## Exact facts and construction error

Line positions compare the line root with 0 and 1 by exact bracket splits.
Target positions compare exactly with span ends and bounds. Sector membership uses
signs of linear functions of `t` (cross and dot with each boundary ray), so an
arc end on the line is exactly `AtStart` or `AtEnd`. Transitions come from exact
equation signs on both sides: crossing, touching (tangency) or one-sided target
end. Multiplicities are reported per side and differ only at spline knots.

Ordering compares isolated roots of different equations exactly. Disjoint
brackets order; otherwise a root of `gcd(P, Q)` inside both brackets proves
equality, and its absence permits bisection until the brackets separate. Equal
contacts form one group in request order; member order is not identity.

World points are the nearest binary64 points to the exact enclosure midpoint,
with a certified Euclidean bound no larger than the requested limit. When the
enclosure lies inside one binary64 gap and that gap cannot meet the limit,
construction reports `ConstructionResolution` with the measured bound. Target
parameter brackets are refined until the target point enclosure also meets the
limit.

## Overlaps, seams and selection

An identically zero span equation is an overlap with exact target range, exact
rational line range, end positions and orientation. A span that is not strictly
monotone along the line is a named incomplete result. Shared knot contacts and
overlap boundaries are merged, not duplicated. A closed wrapped spline whose seam
lies on the line reports both ends in one exact group.

`select_extension` returns the nearest group strictly beyond an end of a support
report, or completed empty. `select_trim` returns the groups enclosing an exactly
projected pick on a segment report. Both refuse a coincident part in the edited
range, a pick on an incidence, a pick outside the segment and a domain mismatch.
Incomplete comparisons are failures, never partial selections.

## Failures, limits and ownership

Invalid input names the field, index and value. Cancellation is distinct.
Exhausted isolation, refinement or construction, non-monotone overlaps and
degenerate resultants are incomplete. No failure carries a contact list. Version
1 limits are explicit: equation degree, subdivisions per isolation, total
refinements and world error; none is widened. Reports own their inputs, are
`Send` and `Sync`, use no global state and are deterministic.

## Excluded representations (open)

[ADR 0067](0067-certified-sketch-curve-incidences.md) admits conic and spline
sources and angle-bounded arcs; [ADR 0080](0080-certified-periodic-and-offset-incidences.md)
admits offset curves and native periodic knot vectors. The remaining exclusions
below stay open.

Rational splines with nonpositive weights and Bézier inputs with zero
denominators are not admitted. Callers must
not route them through this operation. A future extension needs its own exact
equation, evidence and behavior version.

## Evidence and migration

Acceptance uses hand-derived closed forms, 195 generated ulp-level cases against
an independent Python `Fraction` oracle, 38 native OCCT 8.0.0
`Geom2dAPI_InterCurveCurve` cases, the mapped Rust intersector and 40 owned
sketch spline cases. Details and replay commands are in
[sketch-line-incidences.md](../sketch-line-incidences.md).

This operation is additive behavior version 1; nothing is deleted or migrated in
the kernel. The application epic owns replacing its residual-only trim/extension
selection after an owner release, with atomic endpoint ownership, identity,
undo, save/reopen and native edit acceptance. A seed point that moves during
recompute must be re-certified against the final solved geometry.
