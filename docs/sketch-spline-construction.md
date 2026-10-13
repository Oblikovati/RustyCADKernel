# Sketch spline construction — partial issue 9

[ADR 0011](adr/0011-sketch-spline-construction.md) adds an owned constructor for
control and global-fit sketch splines without active derivative handles. It is
an extension for [issue #9](https://github.com/Oblikovati/RustyCADKernel/issues/9),
not a change to the mapped OCCT interpolation or approximation APIs.

Open control curves use clamped uniform knots and degree min(3, count-1).
Closed control curves use cubic uniform periodic knots and three wrapped poles.
Fit curves use the application's centripetal, chord-length, or uniform spacing.
Open fits have averaged knots and one pole per source point; closed fits solve
cubic periodic collocation. One existing BSplCLib evaluator supplies all basis
rows and one existing math_Gauss decomposition solves each fit. No new basis
recurrence, matrix solver, recognizer, retry, type assertion, or fallback is added.

The constructor accepts version-one pivot and metric limits. Defaults use
machine epsilon for pivots and native confusion (1e-7 working units) for each
residual limit; callers can supply separate limits for their working scale. It returns
an owned recipe, source-indexed fit stations, measured maximum fit distance,
closed position/first-/second-derivative seam residuals, and unchanged options.
The normalized parameter is dimensionless; derivative limits are separate from
position limits. Control intent has no fit-station residual. Native curves are
constructed afresh from the recipe and cannot alias it or each other. Failures
remain typed native exceptions, including contextual construction refusals.
No new runtime tolerance literal or geometry perturbation is introduced.

The chord expression preserves the Go recipe's ordinary-scale arithmetic.
Overflow, underflow to a collapsed station, and normalized station collapse are
explicit failures. This is bounded floating-point construction, not support for
all finite-coordinate configurations or a certificate of root completeness.
Uniform fitting can produce a constant curve through coincident points; this is
not a polygon fallback. Operations requiring a regular tangent must check it.
The fit uses a dense matrix and is not a new large-point-count optimization.

[Oracle provenance and replay](../crates/tkgeomalgo/tests/sketch_spline/README.md)
cover six fit and two control recipes from the pinned Go application. Checks
include poles, full knots, stations, 169 Go analytic sample rows and 169
independent OCCT 8.0.0 position/first-/second-derivative rows. Additional checks
cover degree reduction, closure, ownership after failure and source mutation,
independent mutable native curves, all option fields, strict limit refusal,
singular/overflowing inputs, uniform coincident fits, and 240 generated cases
with 3–12 points at scales 1e-9, 1, and 1e9, including nearly collinear inputs.

Active handle semantics, richer conditioning evidence, optimized performance,
owner release packaging, exact application dependency adoption, public spline
creation and evaluation, trim/extend integration, and application acceptance
remain open. The application still pins Martensite v0.2.1; this local module does
not change that pin, publish a release, or close issue #9. There is no application
or renderer change in this slice.

## Local verification

The final TKGeomAlgo all-feature run passes 7 unit tests, 493 integration tests,
and 154 doctests; 49 declared ignored doctests were not executed. The affected
TKG2d suite passes 607 tests. TKMath passes 14 unit tests, 2,838 integration tests,
and 44 doctests. No full-workspace or renderer release gate is claimed here.
The 15 constructor tests also pass under LLVM coverage. New-module line coverage
is 96.34% (342/355); the seven new source files have zero detected duplicate
blocks at the 50-token/5-line threshold (490 scanned lines). These are new-module
metrics, not whole-kernel coverage or duplication claims.

Clippy passes for all TKGeomAlgo targets/features with warnings denied.
The complete workspace formatting checker, toolkit test-discovery check,
warning-denied TKGeomAlgo rustdoc, and Markdown checks for these new documents
pass. New Rust and capture-source files retain SPDX identifiers; fixtures retain
separate provenance. Control Go capture and both OCCT replay families were
executed locally; the fit OCCT replay reproduced its retained CSV byte-for-byte.

A single debug-binary run of the accepted 240-case generated workload takes
0.02 seconds wall time and 17,824 KiB peak resident memory on this Linux host.
That includes construction and test checks; it is a diagnostic measurement,
not an optimized performance comparison or a portable timing gate.

```sh
cargo test --locked -p rustycad-tkgeomalgo -p rustycad-tkmath \
  -p rustycad-tkg2d --all-features
cargo clippy --locked -p rustycad-tkgeomalgo --all-targets --all-features \
  -- -D warnings
cargo test --locked -p rustycad-tkgeomalgo --test toolkit sketch_spline
cargo llvm-cov --locked -p rustycad-tkgeomalgo --test toolkit --json \
  --output-path /tmp/sketch-spline-coverage.json -- sketch_spline
```
