# Martensite source-bound Bézier pair certificates

This extension addresses the numerical incidence stage of issue #10 and
[ADR 0014](adr/0014-owned-analytic-sketch-regions.md). It does not replace mapped
OCCT intersectors or establish application region/solid acceptance.

`CapturedBezierPair` owns the actual Bernstein controls, optional weights and
closed parameter trims. Distinct mode captures both inputs; canonical self mode
captures one actual source and selects `t < u`. Source and policy are retained
by every report after source mutation, cancellation, failure and query destruction.
Caller occurrence identities and geometric proximity do not certify roots.

The provider derives both original homogeneous coordinate equations. For self
queries it divides both exactly by `t-u`. Generic polynomial pseudo-remainder
row operations retain leading factors and derive a linear ideal row when possible.
Both original equations are substituted into that row. Complete isolation of the
candidate polynomial, exact original-equation signs, domain signs, denominator
isolation and exceptional-fibre exclusion jointly cover the product domain.
A candidate root index refers to the complete retained isolation, including
rejected roots. Selected contacts carry a refinement of that same root and a
conservative enclosure of the paired parameter `u=-beta(t)/alpha(t)`.

Construction uses exact rational interval evaluation and outward binary64 bounds.
Acceptance requires both the exact squared world-error certificate and its
outward reported radius to meet the unchanged request. Refinement re-isolates the
same original polynomial inside the preceding one-root bracket at exact half
width. Root count, containment and multiplicity are checked. A singleton limit
retains its last root, bounds, constructed point and achieved error; it is never
accepted by reducing recorded error.

Primary polynomial degree admission stays in `RootIsolationOptions`. Generated
sign admission has its own versioned `DerivedProjection` or explicit degree cap.
A genuine quartic/quadratic pair with a linear candidate and quadratic domain
signs tests this separation. All stages share one execution-owned completed-split
budget; failed and cancelled stages retain actual spent events.

`BezierPairReport` explicitly distinguishes complete, invalid, cancelled and
incomplete outcomes. Only complete reports expose a borrowed certificate and
selected geometry; complete empty remains distinct from failure. The certificate
exposes retained original-ideal coefficients, root isolation, exact membership
signs, source homogeneous coefficients, denominator and exceptional proofs.
Coefficient views keep the integer dependency behind a Martensite interface.
Report and certificate construction stay private.

The pipeline supports arbitrary admitted polynomial/rational Bézier degrees when
a linear ideal fibre and its required certificates are established. A nonlinear
fibre, common alpha/beta zero, positive-dimensional correspondence, degree/work
exhaustion or inconclusive enclosure returns explicit noncompletion. Overlap
correspondence is not implemented. These limits are not a claim of complete
all-curve incidence or topology certification.

Independent fixtures include dyadic and irrational curved pairs, rational
exceptional exclusions, negative weights, extrapolated domains, cubic crossing,
cusp and the actual initially closed cubic. Native OCCT ordinary observations
are complementary; exact root and world-error assertions remain Rust-only assets.
