# Source-bound Bézier boundary regularity, behavior version 1

## Decision and precise scope

Add `certify_bezier_regularity`, explicitly selected by
`BezierRegularityOptions::v1`, as the TKMath numerical prerequisite for regular
analytic region boundaries. The new operation classifies the supplied placed
three-coordinate rational Bézier on a closed native interval in [0,1]. It owns
all original poles, optional signed weights, affine placement coefficients,
occurrence key, interval and requested root policy. The key is caller supplied;
mathematical evidence is bound to the coefficients, not inferred geometric identity.

The result distinguishes `Regular`, `Stationary` with every isolated zero-speed
parameter, and `EverywhereStationary`. All three are complete classifications;
only `Regular` establishes nonvanishing derivative. A failure is owned and typed
as invalid, declined, cancelled or incomplete, and never exposes partial success.

## Exact proof and bounded work

Reuse the existing shared exact binary64-to-homogeneous coefficient construction,
raw polynomial derivative and convolution, and complete root isolation. No rounded
world points, numerical differentiation, sampling, nearest root choice or new
solver is introduced. For the placed homogeneous coordinates N_i and D,

    H_i = N_i' D - N_i D'
    Q = H_0² + H_1² + H_2²
    ||C'||² = Q / D⁴.

First certify D has no zero on the entire closed interval. Any denominator zero,
including a removable factor or endpoint zero, is a typed refusal. Identically
zero D is separately refused. Signed finite and zero control weights are admitted
only by this new operation, because denominator freedom is proved explicitly.
Legacy metric operations retain their strictly positive weight admission.

Construct Q before any independent content reduction; reducing each H_i before
squaring would change the curve's metric polynomial. If Q is identically zero,
return the explicit everywhere-stationary outcome. Otherwise complete isolation
of Q supplies the decision and all zero-speed witnesses. Multiplicity is the
algebraic multiplicity in Q, not geometric contact order. Endpoints and singleton
intervals are included. Root brackets retain exact dyadic bounds and achieved
width; a regular result makes no numerical lower-speed-bound claim.

The original root degree limit applies separately to D and Q after exact degree
reduction. Neither degree nor width nor total subdivision budget is widened.
Completed subdivisions are counted even if later work cancels or exhausts the
budget. Exact construction, isolation and final publication poll caller
cancellation. Degree refusal, exhaustion and root proof failure retain their
original causes and requested policy. No partial list means regularity.

## Independent expectations and preserved baseline

The retained closed cubic with poles (0,0,0),(4,0,0),(0,4,0),(0,0,0) has
C=(12t(1-t)²,12t²(1-t),0), hence

    ||C'||² = 144[(1-4t+3t²)² + (2t-3t²)²]
             = 16 + 32(9t²-9t+2)² >= 16.

The owned `tests/exact_roots/bezier_regularity/cubic-jacobian.json` receipt
retains the exact Jacobian normalization request and zero-polynomial response
for the difference of these expressions. This verifies that supplied identity;
the Bernstein expansion and derivative above are explicit independent reasoning,
not a claim that the external computation verified Rust. Unit tests pin the raw
ascending Q coefficients [144,-1152,3744,-5184,2592] and homogeneous scaling.

Additional closed forms pin t² endpoint zeros, (2t-1)³ interior zero speed,
constant curves, and derivative roots (3±sqrt(3))/6 for 3t-9t²+6t³. Tests cover
signed denominator freedom, exact denominator failures, reversal, placement
collapse, finite extremes, policy refusal, exhaustion, cancellation, owned input
isolation, and serial/thread/process byte replay. The earlier positive-weight
and raw-coefficient fixtures were committed before the admission extraction
(`e8826146b`) and pass unchanged after structural commit `4fcff8fab`.
No baseline value was replaced.

## Versioning, ownership and exclusions

This additive operation has behavior version 1; other versions are invalid.
Existing metric APIs, weight policy and all earlier fixtures remain selectable.
No workspace version or existing caller is migrated. Any future incompatible
change requires a new behavior version, semantic version change and migration
notes, retaining version 1 for replay until a recorded migration. There is no
replacement engine or production path to delete in this slice.

The certificate owns all reachable inputs and exposes immutable borrowed reads.
No global state, locks, caches or unsafe code is added. Compile-time Send/Sync
and bounded concurrent/process replay are verification evidence, not full
thread-safety acceptance. Host ThreadSanitizer acceptance remains incomplete:
startup controls already fail as retained in
`docs/evidence/native-display-lifecycle-v1-2026-10-14/TSAN.md`; no bypass is used.

Regularity does not establish whole-curve injectivity, C1 closure, correct trims,
solid validity, self-intersection freedom, volume or publication acceptance. This
cubic has a corner at its authored closure: C'(0)=(12,0,0), C'(1)=(0,-12,0).
The side rectangle identifies its two boundary parameters at the authored seam;
regularity does not prove rectangle injectivity or smoothness of that quotient.
TKBO ordered checks and body success are unchanged. No GPU, exchange, new native
oracle, broad inventory credit or release performance claim is included.
