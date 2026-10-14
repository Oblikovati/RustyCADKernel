# Conditional polynomial extrusion-side integral v1

## Decision and scope

Martensite adds one complete conditional scalar operation in TKMATH:
`certify_polynomial_side_integral`. For a supplied polynomial curve, affine
embedding, separate affine placement and extrusion vector V it bounds
`integral_0^1 ||C'(t) cross V|| dt`. All supplied binary64 coefficients denote
their exact mathematical values. Translations must be finite even though their
derivatives vanish. Polynomial Bézier curves of stored degree 0 through 16 and
affine lines are admitted; explicit weights and nonfinite inputs are refused.
The only supported range is the full native [0,1], implicit in this API.

The integral includes parameter multiplicity. Zero is a completed scalar for a
constant curve, zero extrusion, or derivative parallel to the extrusion. No
regularity, injectivity, valid face/body, boundary coverage, construction error,
source correspondence, history or application acceptance follows from it.
The caller must bind captured source revisions and actual chart/trim/history
evidence before interpreting the scalar as an output-face measure. This slice
does not alter TKBO or clear any region publication gap.

This is additive behavior version 1. The existing signed polynomial boundary
and sweep integrals and mapped integration APIs remain available unchanged;
there is no superseded production path or migration adapter to delete. The
owning region #10 / C1#12 integration gate must consume source-bound evidence
before its operation can complete. It retains its existing deletion task and
milestone deadline. The next compatible patch target is 0.2.8; version changes
and publication remain the coordinator's work.

## Exact interval proof

Let W(t)=C'(t) cross V. On I=[a,a+h], restrict the polynomial exactly:
W(a+h*s)=sum_i b_i B_i^m(s). Nonnegative Bernstein basis functions sum to one
and each integrates to 1/(m+1). The triangle inequality and norm convexity give

    h * ||sum_i b_i/(m+1)|| <= integral_I ||W(t)|| dt
                            <= h/(m+1) * sum_i ||b_i||.

Exact dyadic de Casteljau restriction supplies child coefficients. It changes
neither the source polynomial nor geometry and constructs no tessellation or
sampled acceptance evidence. Width h is applied once; restricted W coefficients
remain derivatives with respect to the original parameter, not a second local
derivative. Exact affine coefficient construction and placement are reused.
Power-to-Bernstein conversion uses b_i=sum_{k<=i} a_k*C(i,k)/C(m,k).

For uniformly shrinking intervals, restricted polynomial controls converge
uniformly to the local value; their spread tends to zero. Therefore the summed
exact-arithmetic gap tends to zero, including isolated zero derivatives and
norm cusps. No division by speed, nonzero-speed premise or root cut is needed.
This convergence theorem is not a promise that a finite work/rounding budget
will satisfy every requested error cap.

## Rounding, scheduling and completion

Scale exact vectors by h/(m+1) before computing squared norms. The shared exact
square-root search returns enclosing binary64 endpoints, including zero,
subnormal values and maximum finite values. Contributions and frontier updates
are accumulated as exact rationals. Final endpoints and their width are rounded
outward. The deterministic representative is the lower endpoint and the
reported absolute error is an upward bound on the full endpoint width.

Each refinement chooses the largest exact interval-bound gap, with smaller
parameter start breaking ties. Global enclosures are intersected with earlier
enclosures so endpoint rounding cannot widen retained evidence. Cancellation
is polled during coefficient construction, each restriction level and bound
contribution, and before publication. Square-root search has at most 63 fixed
comparison steps and is bracketed by those operation checkpoints.

Defaults are absolute area error 1e-5, degree limit 16 and 4096 subdivisions.
The explicit maximum admitted subdivision budget is 65536, with zero allowed.
There are no clamps. Development workload measurements at this policy were:
PH cubic 300 splits; reversing quadratic 9; active closed cubic with extrusion
(0,0,2), 1482. These are workload counts, not performance or universal convergence
claims. Absolute area error has squared-coordinate units and is separate from
modelling, spatial and parameter tolerances. Scaling a workload may require an
explicit different error request or can produce typed incompletion.

Errors retain requested/applied settings, actual completed subdivisions and
interval bounds, a last finite diagnostic enclosure when available, and typed
causes. A fully bounded first child before later cancellation is counted; a
subdivision is complete only after both children are bounded. Exhaustion,
cancellation and unrepresentable enclosing bounds publish no completed scalar.
An unrepresentable conservative bound does not prove the actual integral infinite.

## Evidence and exclusions

Owned closed forms include lines, zero/parallel cases, a nonconstant PH cubic
with exact integral 4, and a reversing quadratic with integral 5/3 and stationary
parameter 1/3. The PH squared-speed identities are checked independently by
Jacobian exact rational SOS checks, retained in
[the evidence directory](../evidence/polynomial-side-integral-v1-2026-10-14/).
Tests include affine shear and extreme cancelling scales, subnormal rounding,
overflow, degree limits, deterministic work exhaustion/cancellation, source
isolation, process byte replay and ordinary-host thread comparisons.

Results and diagnostics own scalars and expose compile-time Send/Sync checks.
There is no unsafe Send/Sync implementation or shared algorithm state. Ordinary
host thread comparisons do not discharge ThreadSanitizer: its previously
retained ordinary-host startup failure remains incomplete and this suite has
not executed under TSan. No native OCCT, GPU, application or performance evidence
is introduced. Full source-frozen check results are recorded separately.
