# Complete conditional prism measurements v1

## Decision and scope

Issue #10's analytic region-to-solid prerequisite composes the ADR0027 cap and
signed-sweep measures with the existing ADR0024 side integral. The additive private
complete-measure entry preserves the original subset entry, its options,
pending rows and portable receipt for replay. Completion covers current source
and actual chart-bound scalar measurements; ordered body validity, tolerances,
embedding and immutable publication remain separate obligations.

The reusable ADR0029 current binding is the only source/chart validator. The
actual side basis and actual affine placement supply C(t). The existing exact
proof retains positive trim height h, stored direction D and placement P, and
proves requested extrusion E = -h linear(P) D with all four trim roles over
u in [0,1], v in [-h,0]. The side API receives actual C and E directly. No rounded
height, vector norm, length or intermediate area is multiplied into a result.
Final callback-free rebinding covers every used coefficient, trim and identity.

## Conservative total arithmetic

`sum_nonnegative_enclosures` is an additive general arithmetic helper, behavior
version 1. It certifies exact Minkowski endpoint addition conditional on supplied
component intervals; it cannot certify their originating geometric claims.
Finite ordered nonnegative components, count at most 64, and a finite nonnegative
full-width error budget are required. Empty input is a completed zero sum.

Each binary64 endpoint denotes its exact dyadic value. The helper adds all lower
endpoints exactly and all upper endpoints exactly, then rounds the resulting
lower down and upper up. The representative is the lower endpoint. Achieved
error is the upward-rounded exact difference between published endpoints. It
includes component uncertainty and both endpoint roundings. Supplied component
bits/order and requested/effective options remain unchanged in the owned report.

The complete prism entry retains cap, side and signed-volume reports unchanged
and applies a separate total-area policy. It does not allocate tolerances, shrink
child budgets, retry integration, clamp options or widen a limit to achieve a
completed total. Overflow, exhausted side work, excessive width, unsupported
geometry and cancellation remain typed incomplete outcomes. Scalar and sum
reports are immutable and Send/Sync; source keys remain operation-local.

## Independent representative oracle

For cubic poles (0,0),(4,0),(0,4),(0,0), extrusion (0,0,2), the side integrand is
`f(t)=24 sqrt(p(t))`, where
`p=18t^4-36t^3+26t^2-8t+1=1/9+18(t-1/3)^2(t-2/3)^2`.
Jacobian exact QQ factorization independently reconstructs p-1/9. Bernstein
coefficients of p' are (-8,28/3,-28/3,8), and of p'' are (52,-56,52).
Consequently p>=1/9, |p'|<=28/3, |p''|<=56, and
`|f''| <= 36*56 + 162*(28/3)^2 = 16128`.
The composite midpoint error is at most `16128/(24 N^2)=672/N^2`.

The retained offline fixture generator uses N=2^17 and integer-square-root
brackets on a 2^-80 grid. Every node bracket is checked by exact integer squares;
exact rational sums include the analytic remainder before outward conversion.
This code imports no production rounding or integration code. The two-cap exact
area 24/5 is added rationally for the independent total. Tests require the entire
narrow oracle interval to lie within the returned production enclosure. The
131,072-node generation is offline; ordinary tests read the fixed fixture bytes.

## Migration and remaining gates

This is an additive composition of existing general arithmetic, geometry and
history engines. No replacement or deletion gate is passed, no native OCCT
capture is introduced, and no faceted/sampled acceptance or valid-body result is
provided. The original subset remains selectable without migrating its behavior.
The entry is implemented through the shared current binding; the final frozen
verification receipt records its source-isolation, degenerate, cancellation and
independent-oracle checks. This scalar composition never closes the body gates.

## Replay and callback ordering

The original subset golden receipt remains byte-pinned. The additive entry owns
one CurrentPrismBinding, the original subset report, the side report, the total
report and all three independent requested policies. It calls the unchanged side
engine once. Its last external callback precedes shared callback-free rebind.
Tests edit original/copy/side geometry at the side engine's final callback and at
the outer final callback, and require refusal from fresh binding before return.

The general sum report exposes the explicit evidence tag
CertifiedConditionalEndpointArithmetic. Its inputs remain conditional, and the
caller retains the actual component reports and source/role identities. Failed
private wrapper diagnostics retain legacy typed causes and may contain mutable
shape aliases; they are not immutable published operation results.
