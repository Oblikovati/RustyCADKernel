# Exact Bernstein inputs to the shared root solver

Kernel issue [#10](https://github.com/Oblikovati/RustyCADKernel/issues/10) needs
analytic curve classification. ADR 0004 certifies power coefficients but does
not certify their construction. Add a scalar Bernstein input boundary that
converts supplied finite binary64 control values to integer power coefficients
without rounding. This is an extension; mapped PLib conversion stays unchanged.

For degree n, use B(i,n,t) = choose(n,i) t^i (1-t)^(n-i). The coefficient of t^k
in the supplied Bernstein sum is choose(n,k) times the kth forward difference
of the control values at index zero. Binary64 values share the exact positive
scale 2^-1074. Integer differences and exact binomial recurrence therefore
construct the same polynomial over the rationals, including degree reductions.
Remove positive content only after conversion. No intermediate coefficient is
converted back to binary64, so overflow, cancellation, and subnormal rounding
cannot change roots. This formula follows direct expansion of the stated basis.

Expose `isolate_bernstein_roots` alongside `isolate_polynomial_roots`. Both use
the existing Sturm sequence, multiplicity, interval, work-limit, and failure
implementation. The parameter basis is fixed at [0,1]; the query interval selects
where to find roots, including extrapolation. It does not rescale that basis.
Degree admission uses the true degree after exact conversion, not the nominal
control count. Empty and all-zero inputs remain named zero-polynomial declines.
Cancellation covers conversion as well as isolation. Conversion uses quadratic
integer work and linear temporary storage; the degree limit is not a time bound.

One general basis conversion replaces the need for a future application-side
floating conversion or a second Bernstein root engine. No existing production
engine is deleted at this step. Issue #10 retains the deletion gate for the
sampled diagnostic region classifier when the complete region contract passes.
The new boundary handles scalar control values, not point/weight products,
rational denominator poles, spline knot extraction, periodic trims, or persistent
incidence identity. Those steps must also preserve exact source arithmetic before
this result can certify a geometric intersection.

Independent tests expand the Bernstein sum directly, obtain complete root
identities from Jacobian, and certify returned intervals against those identities.
Degree elevation, reversal, extreme scales, rounded-conversion counterexamples,
ownership, cancellation, and legacy power-basis results are separate gates.
