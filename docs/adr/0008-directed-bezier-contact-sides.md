# Exact directed sides at isolated Bézier line contacts

Kernel issue [#10](https://github.com/Oblikovati/RustyCADKernel/issues/10) needs
ray-side classification without substituting an approximate intersection
parameter. Connect the exact projected inputs from ADR 0006 to the algebraic
sign operation from ADR 0007. Preserve the existing infinite-line query.

For ray origin A, direction B-A, controls P_i and weights w_i, retain the exact
normal numerator N_i = w_i det(B-A,P_i-A), denominator D_i = w_i, and longitudinal
numerator L_i = w_i dot(B-A,P_i-A), all in Bernstein form. Lift each coordinate
and weight once; a shared projection traversal captures the requested functional.
The ordinary line query does not compute an unused longitudinal projection.

First certify D is nonzero throughout the closed query interval, using the
existing denominator stage. At each N root, the directed coordinate has sign
L/D. Since D² is strictly positive, this is exactly the sign of L*D, which the
shared Sturm–Tarski operation certifies. No denominator-sign sample, approximate
parameter, direction normalization, or division is required. The nondegenerate
line check proves dot(B-A,B-A) is positive, so omitting that normalization cannot
change the side. Signed and zero individual weights retain the same admission
rules as the line query.

Return every supporting-line root with its unchanged bracket and multiplicity,
and its ordered side: behind the ray origin, at the origin, or ahead. Retaining
behind contacts avoids silently dropping evidence before endpoint ownership or
winding rules run. A zero longitudinal polynomial means all isolated contacts
are at the origin. An identically zero normal numerator is a named supporting-
line coincidence decline: finite isolated contacts cannot represent a continuum.
This query does not pretend that such a decline is completed empty.

The normal, longitudinal, and denominator degrees obey the input degree policy.
Internal products may have higher degrees; they do not change the input limit.
Shared subdivision limits, cancellation, typed causes, denominator failures,
input ownership, and isolation results remain intact. Extract the common
prepared-polynomial and zero-query boundaries; add no second root/sign engine.

This supplies isolated directed-contact classification, not overlap intervals,
wire endpoint ownership, winding, or complete point-in-region classification.
Those consumers and full region acceptance remain under issue #10, including
the sampled-classifier deletion gate. No mapped OCCT operation is replaced.
