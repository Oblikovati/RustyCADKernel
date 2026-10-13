# Exact directed Bézier wire classification

Kernel issue [#10](https://github.com/Oblikovati/RustyCADKernel/issues/10) needs
closed-wire seed classification before analytic-region acceptance. Compose ADR
0009 contributions through one ordered wire query. Preserve all existing scalar,
line, ray, and segment APIs and their failure order. This is an extension, not
a replacement for a mapped OCCT operation or a complete B-rep validator.

Inputs are borrowed rational or polynomial Bézier segments, unique caller-owned
segment occurrence IDs, closed parameter intervals, and explicit direction flags.
All segments use one seed/ray frame and one versioned root policy. Reversing an
occurrence swaps its endpoints and negates its non-boundary contribution without
rounding the parameter transform or modifying control points.

Compute endpoint triples from the exact normal, longitudinal, and denominator
Bernstein controls already used by the segment query. Homogeneous de Casteljau
at an exact dyadic parameter multiplies each level by its parameter denominator;
the common factor cancels within the triple. The two projected coordinate ratios
form an invertible frame for a nondegenerate ray. Cross multiplication therefore
certifies endpoint equality without normalizing coordinates or moving geometry.
Keep unreduced projected controls for this check: separate primitive reductions
would lose the relative scales needed by the ratios. Certify denominator
regularity before using the endpoint triples for incidence.

Validate every consecutive join and the closing join. Zero-width intervals and
single closed segments remain mathematically admissible; this query does not
claim they form a valid face. Reject empty input and duplicate occurrence IDs.
Every segment must finish, even after a boundary contact. Return all incident
segment IDs in input order, or the complete signed winding number and its nonzero
inside/outside classification. Use checked accumulation and one subdivision budget
across the whole request. Failures retain affected segment IDs and typed causes;
no partial sum or partial boundary list succeeds.

This supplies closure and seed location for supplied curves. It does not certify
self-intersection freedom, face topology, measured modelling tolerance, curve
extraction, or persistent split identity. Full region construction and the
sampled-classifier deletion gate remain in issue #10. No second intersection or
winding engine, approximate endpoint weld, or native production fallback is added.

## Native curved-vertex discrepancy

The OCCT 8.0.0 face classifier reports IN at both quadratic-lens endpoints and
at the single cubic's closing point. The retained valid-face receipt also proves
exact native vertex-coordinate equality. The cubic seed has zero vertex distance
and zero surface-projection distance; both 3D and projected 2D classifiers return
IN for tolerances from 1e-12 through 1e-5. An explicit XY face plane leaves all
three mismatches unchanged. Other 45 corpus classifications agree.

The new extension deliberately returns boundary at these exact incidences.
Implicit-equation receipts and exact occurrence endpoints establish that result.
Tests retain the raw native IN values and require the three named disagreements
and their vertex evidence; they do not claim complete native classifier parity.
The mapped OCCT classifier is unchanged, and its discrepancy remains unresolved.
