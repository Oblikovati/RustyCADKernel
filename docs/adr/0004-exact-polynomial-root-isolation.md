# Exact polynomial roots for extension classification

Kernel issue [#10](https://github.com/Oblikovati/RustyCADKernel/issues/10) needs
certified curve-intersection parameters before bounded-region classification.
Extend the exact predicate boundary with isolation of all distinct roots of a
supplied binary64 polynomial on a finite closed interval. Use a signed primitive
integer remainder sequence and exact dyadic subdivision; do not alter mapped
OCCT root solvers or introduce an approximate fallback.

Coefficients have ascending degree. Their common positive binary64 scale is
removed exactly, using the same coordinate conversion as ADR 0003. Polynomial
pseudo-division multiplies by the absolute divisor leading coefficient before
each subtraction. This is a positive scaling of ordinary division; reduction
by positive integer content keeps every Sturm sign unchanged. Negate each
remainder. The last nonzero member is the polynomial GCD up to a nonzero scalar.

At a zero endpoint, use each sequence member's right-hand sign, determined by
its first nonzero Taylor derivative. Variation differences then count roots in
(a,b], including repeated roots once. Dividing the whole sequence by its common
GCD preserves variations away from zeros; right limits extend that identity at
zeros. The reduced first polynomial has a simple zero and the reduced second
has the same derivative sign there, multiplied by the positive multiplicity.
This justifies the same one-root jump at every distinct real root. GCD towers
count multiplicity independently of interval width.

Emit a root at the initial lower bound separately. Cache endpoint signs and
variations, including shared midpoints. Return either an exact dyadic root or a
closed dyadic bracket whose endpoints are not roots and which contains exactly
one root. Brackets are ordered and disjoint. Their exact width must meet the
caller's positive approximation bound; it is not a modelling tolerance. A
single general path handles linear, repeated, and higher-degree roots.

Version 1 defaults to degree 32 and 4096 subdivisions; the approximation width
is always explicit. Unknown policy versions are refused. Degree and subdivision
budgets are explicit caller limits. Cancellation is
checked during sequence construction and subdivision. Invalid input, zero
polynomials, cancellation, and exhausted work return distinct typed failures;
no partial root list is a successful result. Big-integer operations themselves
are synchronous. No latency or optimal complexity claim is made.

This establishes a numerical prerequisite, not a complete region operation.
It certifies the supplied polynomial, not a rounded conversion from a geometric
curve. Analytic curve conversion, periodic trims, incidence ownership, complete
validation, and naming remain under issue #10. That issue owns deletion of the
application's diagnostic sampled classifier after the complete region corpus
passes. No existing production engine is replaced by this primitive.

References: the [CGAL algebraic kernel](https://doc.cgal.org/Manual/latest/doc_html/cgal_manual/Algebraic_kernel_d/Chapter_main.html)
uses isolating intervals to identify algebraic roots; the independent test oracle
uses rational arithmetic and square-free polynomial isolation, not this integer
pseudo-remainder implementation.
