# Exact Bézier-to-line parameter classification

Kernel issue [#10](https://github.com/Oblikovati/RustyCADKernel/issues/10) needs
analytic boundary predicates. ADRs 0003–0005 supply exact orientation and root
isolation. Add an immutable two-dimensional polynomial/rational Bézier query
against an infinite line, returning parameter contacts rather than built geometry.
Mapped OCCT intersection APIs and their results remain unchanged.

For endpoints A and B, control points P_i, and weights w_i, construct Bernstein
controls N_i = w_i det(B-A,P_i-A) and D_i = w_i. Every supplied finite binary64
value is converted to an exact integer before subtraction or multiplication.
The homogeneous curve lies on the line exactly where N(t) is zero and D(t) is
nonzero. Common positive scales do not change either root set. Unweighted curves
use equal unit weights through the same path. Reuse the exact determinant,
Bernstein conversion, and Sturm isolation; do not add a second intersection engine.

First certify D(t) has no zeros on the closed query interval. Any denominator
root, including an endpoint or a simultaneous numerator zero, is a named decline
with its certified parameter bracket. An identically zero denominator also
declines. Cancelling common factors would change the supplied representation;
removable singularities require a separate explicit normalization operation.
Signed and zero individual weights remain admissible when D(t) stays nonzero.

Then isolate every numerator root with its exact multiplicity. An identically
zero numerator is a distinct completed coincident result, not an empty list.
A nonzero numerator with no roots is completed empty. All nonfinite inputs,
empty control arrays, mismatched weight counts, and degenerate lines have typed
errors naming the offending source. The existing versioned root policy controls
both stages; subdivision work is shared across them. Failure or cancellation
returns no successful partial contact list. Inputs and results own no shared
mutable geometry. The basis stays [0,1], including extrapolated queries.

This is one parameter-classification prerequisite. It creates no edge or face
and claims no B-rep validity, model tolerance, persistent naming, finite-line
clipping, ray-side classification, or curve/curve intersection. Issue #10 keeps
the gate for deleting the sampled diagnostic region classifier after the complete
region contract passes. No production engine is removed by this addition.
Independent exact rational fixtures, counterexamples to floating projection,
orientation regressions, ownership, and success/failure/cancellation transitions
must pass before local integration.
