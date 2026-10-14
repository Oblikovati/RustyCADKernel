# ADR 0015: exact constant-equation exclusion in paired Bézier certificates

Status: implemented; exact validation scope is recorded in the runtime receipt.
No new mapped OCCT API credit.

The paired extension in ADR 0014 currently requires a linear parameter fibre.
A self domain whose exact diagonal quotient is a nonzero constant therefore
reports `NonLinearFiber`, although that equation has no root anywhere. A constant
is an algebraic property of the actual owned equation, independent of curve
shape, polynomial degree, parameter orientation, or construction tolerance.

After the unchanged source, denominator, primary admission, and exact diagonal
preparation, preserve the existing projection cancellation check. If no linear
fibre is available, a canonical nonzero total-degree-zero original equation may
certify that there are no common roots on the entire captured product domain.
Retain its exact coefficient, original coordinate axis, both checked equations,
and whether the self diagonal was divided. Do not invent a graph, candidate
isolation, contact, sign evaluation, or world construction for this proof.

The completed result retains the actual captured sources, both denominator
certificates, requested policy, and completed subdivision work. Generated graph
signs and world construction are explicitly absent. Existing graph-backed
accepted computations retain their numerical pipeline. All-zero, nonconstant,
nonlinear, overlap, and unresolved exceptional-fibre cases remain incomplete
unless an existing proof already certifies them. This changes only the private
extension's previous unsupported result; old Bézier-line and ray operations
are unchanged.

Independent exact regression cases must include linear self domains, high-degree
curves separated by a constant coordinate equation, both coordinate axes,
reversal, extrapolation, negative constant weights, source mutation, cancellation,
invalid policy, singular denominator precedence, and unsupported all-zero cases.
The certificate is the exact nonzero constant, not sampled distance or a residual.

The checked extension certificate now exposes `graph() -> Option<BezierPairGraphCertificate>`
and `constant_exclusion() -> Option<BezierPairConstantExclusion>`. Graph-only
`candidates`, `membership`, `accepted_candidates`, `projection`, `leading_factors`,
`exceptional_fibers`, and `effective_signs` move to the graph view. Their original
values and calculations remain unchanged for all existing accepted graph cases.
Common contacts, captured source, denominator, and work accessors remain on the
complete certificate. `sign_stage()` explicitly records either evaluated graph
settings or skipped constant-exclusion predicates. This is an intentional
checked-extension API adjustment to prevent meaningless fabricated evidence.

The fallback is considered only after the unchanged linear-fibre derivation
returns no row. Its original cancellation poll therefore remains before the new
constant admission; previously supported graph cases keep their proof variant
and call sequence. The witness is selected from an original reduced coordinate
equation, not an exceptional-factor-dependent pseudo-remainder. The generated
sign-policy version is still validated before any newly successful exclusion;
no generated sign degree or evaluation is invented for a skipped stage.
