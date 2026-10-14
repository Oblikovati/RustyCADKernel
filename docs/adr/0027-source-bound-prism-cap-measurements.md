# ADR 0027: exact world-plane measures and source-bound prism measurements

Status: staged implementation; full analytic region-to-solid acceptance remains incomplete.

## Decision and units

Issue #10's representative analytic prism needs error-bounded measurements of its
actual copied geometry. The existing signed polynomial boundary integral is in
native plane coordinates squared. Its magnitude is not world cap area for a
general affine embedding, and stored binary64 axes that were normalized in
floating arithmetic need not have exactly unit length.

Behavior version 1 adds a separate conditional plane-measure report. For the
unrounded exact native Green integral I and exact placed support columns a,b,
compute Q = I² ((a·a)(b·b) − (a·b)²), then enclose sqrt(Q) with the existing shared
exact-directed square-root helper. Exact integer/rational arithmetic precedes
that single final rounding. Neither I nor the support norm is rounded separately;
opposing extreme scales can therefore produce a finite result without temporary
floating overflow or underflow. The shared columns were extracted structurally
with the existing Jacobian's arithmetic, input-error precedence and three polls
unchanged (commit 14b0198db).

The new immutable `CertifiedPolynomialPlaneMeasure` reports nonnegative adjacent
binary64 endpoints, their lower endpoint as representative, upward-rounded full
enclosure width as absolute error, and original/effective policy. Its evidence is
`CertifiedAlgebraicBound`, distinct from the existing rational integral report.
The finite `PolynomialIntegralOptions.absolute_error_limit` is in world area
units for this operation. No limit is widened; no policy is clamped. A zero
budget accepts only exact representable square roots. Unsupported polynomial
domains, nonfinite inputs, unrepresentable final measures, cancellation and
unmet error budgets return typed failures with original causes. The error type
identifies the world-plane operation; native failures retain curve indices.

The domain remains supplied polynomial Bézier/affine-line occurrences on their
full native [0,1] ranges, degree at most 16 and at most 64 occurrences, with
explicit reversal. Weights are unsupported even when equal. The scalar exists
for open, multiply traced and degenerate boundaries; interpreting it as region
area additionally requires complete oriented boundary coverage and valid planar
support. A singular support's scalar zero never certifies a region. The fixed
square-root search is polled immediately before/after, with no hidden callback
inside its at-most-63 comparisons. Reports own only scalars and are Send + Sync.

## Composition boundary

The next stage is a private operation-local measurement inspection, separate from
PrismReceipt and the frozen publication projection. It must rebind the current
source-bound Jordan certificate, exact copied coefficients/traversal and support,
actual unique cap and side chart coverage, trims, all history parents and actual
extrusion. A retained success/status row alone cannot satisfy a hypothesis.
Cap roles come from First/Last history, never returned order. The signed sweep
integral continues to use unrounded I times the exact affine determinant; cap
area times extrusion length is not a substitute.

Only the certified cap/signed-volume subset may complete. Side and total surface
area remain explicitly pending until the separately implemented side integral
can be bound at this seam. Scalar measurements do not discharge ordered topology,
geometry, self-intersection, tolerance, ownership or final publication obligations.
The earlier scalar APIs and batch region/prism behavior remain selectable and
unchanged. This is an additive shared-math prerequisite, not a replacement engine;
there is no production path to delete or migration of legacy behavior yet.

## Independent evidence and remaining gates

Closed-form Green integration gives I=12/5 for poles (0,0),(4,0),(0,4),(0,0).
An independently evaluated exact QQ augmented-Gram determinant gives S=338 for
columns (2,3,5),(7,11,13). For stored binary64 axes (.6,0,.8),(0,1,0), it gives
S=81129638414606685298668707040461 / 81129638414606681695789005144064 > 1;
the corresponding rounded floating squared length is 1. Tests compare squared
enclosure endpoints against these exact independent expectations. Further cases
cover placement composition, reflections, traversal, translation, exact and
irrational results, cancellation/retry, source mutation, underflow/overflow,
collinear and near-cancelling supports, and deterministic thread replay.

Final source-bookended suite, process replay, extracted corpus and host sanitizer
status will be recorded in the evidence receipt. No native OCCT oracle or new
native fixture is part of this decision. Until those checks and the composition
stage finish, this decision does not claim the intended operation complete.
