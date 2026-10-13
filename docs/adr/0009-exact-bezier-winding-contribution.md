# Exact oriented Bézier winding contributions

Kernel issue [#10](https://github.com/Oblikovati/RustyCADKernel/issues/10) requires
an exact seed classifier. Extend ADR 0008 with a directed curve contribution,
using the same projection, denominator gate, isolation, and algebraic signs.
This replaces no mapped OCCT API and introduces no second intersection engine.

For increasing curve parameter, count upward crossings of the positive ray as
+1 and downward crossings as -1. Interior even-multiplicity tangencies contribute
zero. At the lower parameter endpoint count only departure into the positive
normal half-plane; at the upper endpoint count only arrival from that half-plane,
with contribution -1. Adjacent segments therefore count a shared crossing once
and cancel a shared touch. Closed-wire assembly must still prove exact endpoint
closure and orientation before interpreting the sum as a winding number.

The sign immediately after an isolated normal root is certified at its nonroot
upper bracket endpoint. For a point bracket use the first nonzero derivative,
whose order is the certified root multiplicity. Multiply by the denominator
sign to obtain the rational curve's normal side. Odd multiplicity reverses the
side before the root; even multiplicity preserves it. This is exact local sign
classification, not evaluation at a guessed contact parameter.

Any isolated contact at the ray origin is boundary. For an identically zero
normal numerator, solve the longitudinal numerator times the already regular
denominator: an interval root means boundary; otherwise the contribution is
zero. Identically zero longitudinal projection is boundary throughout. A
zero-length parameter interval has zero contribution unless it is boundary.
The existing isolated-contact query keeps its named coincidence decline.

Return a complete boundary-or-signed-contribution result with typed causes and
cancellation. No partial sum succeeds. Reuse exact input admission and the shared
subdivision limit. Source and query ownership remain unchanged. Overlap intervals
are not constructed, and this primitive does not validate a closed wire or B-rep.
The full region contract and sampled-classifier deletion gate remain in issue #10.
