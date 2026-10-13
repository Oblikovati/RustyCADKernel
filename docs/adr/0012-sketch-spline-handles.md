# Analytic sketch spline handles

This decision extends ADR 0011 for kernel issue
[#9](https://github.com/Oblikovati/RustyCADKernel/issues/9). The natural global
fit remains the source of normalized stations and derivative magnitudes.
An active handle supplies its source point index and authored endpoint.
The endpoint direction and length relative to one third of the neighbor chord
scale the natural derivative, as in Go commit `aa8297d1`.

Any active handle selects Hermite construction for every station interval,
even when its endpoint coincides with its anchor. A coincident handle retains
the natural derivative and produces an explicit diagnostic. Nonrepresentable
lengths and derivatives are refused, without an axis fallback or perturbation.
Duplicate indices, non-finite endpoints, and out-of-range indices are refused
before fitting. Handle input order must not change the result.

Each cubic span has its exact Hermite-to-Bezier controls. Shared endpoints
and triple interior knots form one owned nonperiodic B-spline. Native analytic
span evaluation checks both sides of every station against the requested
position and derivative. Closed curves share their first station derivative
and have measured position and first-derivative closure bounds. Second
derivatives can jump; they are not subject to the natural fit's C2 seam gate.
The knot representation has formal C0 continuity. No tolerant knot removal
or claim of exact C1 topology is part of this construction.

The handled report retains the independently checked natural fit, the final
recipe, effective derivatives, diagnostics, and explicit achieved errors.
Natural-fit options apply to that fit only; handled acceptance has separate
position and derivative limits. Empty handle requests use the existing
unhandled constructor, preserving its recipe without reconstruction.

This replaces the legacy sampled Hermite model path when the released kernel
is consumed by the Rust application. Issue #9 remains the deletion, release,
and application acceptance task within R3. Go samples and independently built
OCCT Bezier spans are complementary oracles. Neither samples nor residuals
prove root completeness, topology, or general downstream operation support.
