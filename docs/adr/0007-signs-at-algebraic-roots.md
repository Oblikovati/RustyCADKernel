# Exact polynomial signs at isolated algebraic roots

Kernel issue [#10](https://github.com/Oblikovati/RustyCADKernel/issues/10) needs
ray-side and incidence classification at curve/line parameters. ADR 0006 returns
certified root brackets; a bracket midpoint does not determine the sign of a
second polynomial at the enclosed root. Add exact polynomial sign evaluation
at every isolated source root, reusing the integer remainder implementation.

For a nonzero source P and query Q, the Sturm–Tarski theorem gives the sum of
signs Q(r) over distinct P roots in (a,b) as the variation difference of the
signed remainder sequence starting with P and P'Q. Source isolation supplies
brackets containing exactly one distinct root, with nonroot endpoints. The sum
is therefore exactly -1, 0, or 1. Zero proves a common root, not small distance.
Exact dyadic point roots use exact substitution instead. Multiplicity remains
source metadata and does not multiply the returned sign.

Reduce P'Q modulo P before building the signed sequence. This removes only a
polynomial part of the rational quotient, preserving its Cauchy index. Every
pseudo-remainder/content scale stays positive, as in ADR 0004. Extract the shared
sequence builder and integer negation; do not introduce a second remainder
engine, approximate root sampling, or refinement until a sign appears stable.
The source-root brackets and subdivision counts remain unchanged. Query degree
and source degree are admitted separately under the existing root policy;
internal derivative/product degrees do not redefine that public limit.

Return the complete isolation and a position-aligned immutable sign list, or a
typed error naming the failing input/stage. A zero query is valid and returns
zero at every source root; a zero source remains a named decline. Cancellation
covers convolution, remainder construction, and each sign classification. A
variation difference outside {-1,0,1} is an invariant failure, never a sign.
No partial sign list is a successful result. No new subdivision or time bound
is implied beyond the existing root policy.

This supplies the numerical operation needed by a future directed Bézier/ray
query; it does not itself select a ray, close a wire, or validate a region.
Issue #10 retains the deletion gate for the sampled diagnostic classifier after
complete region acceptance. No existing production engine is replaced here.

The theorem and algebraic-sign construction are described in
[Li, Passmore and Paulson, 2017](https://doi.org/10.1007/s10817-017-9424-6).
Independent root identities/signs, repeated and common roots, query roots inside
the source bracket, endpoint cases, extreme scales, and cancellation are required
regressions. Existing isolation and orientation results must remain unchanged.
