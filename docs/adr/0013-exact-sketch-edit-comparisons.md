# Exact comparisons for sketch edit selection

This decision extends the exact predicate boundary in ADR 0003. Application
[port epic #3563](https://github.com/Oblikovati/Oblikovati/issues/3563) needs
line-end selection and ordered incidences for trim and extension. Kernel
[issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10) remains the
broader sketch-region task, including exact classification and release.

Add squared-distance and directed-projection comparisons to `predicates`.
Reuse its exact binary64-to-integer conversion and typed non-finite-coordinate
errors. Compare mathematical values without first rounding a distance,
difference, dot product, normalization, or midpoint. There is one integer
path for every finite input, including subnormals and maximum magnitudes.
Do not change mapped OCCT distance and projection operations.

Equal distances remain an explicit equality result. The application can apply
the Go endpoint-pick rule (an exact tie selects the first end) without a false
tie from overflow or rounding. Projection equality means equal projections;
a zero direction yields equality and does not certify a valid line. Geometry
construction still validates nondegenerate lines separately.

These predicates certify ordering of supplied binary64 coordinates. They do
not certify coordinates produced by an intersection algorithm, establish root
completeness, bound geometric root error, or select an approximate contact
without its separate geometry checks. Keep those obligations explicit in the
edit operation contract.

Completion replaces Go `pickNearerEnd`'s rounded distance comparison and
`nearestExtension`'s rounded projection ordering during the Rust R3 port.
The deletion gate is exact predicate corpus acceptance plus actual extension
selection, identity, transaction, save/reopen, and native edit tests in the
application. Owner release and exact app intake remain required; no local
source override or application arithmetic engine is a production fallback.
The migration remains within R3, with the port epic owning legacy deletion.
