# Owned sketch spline construction

Kernel issue [#9](https://github.com/Oblikovati/RustyCADKernel/issues/9) owns the
Go application's sketch spline policy. Add an extension in `sketch_spline`;
do not alter mapped `Geom2dAPI_Interpolate` or approximation behavior.

Fit intent uses normalized uniform, chord-length, or centripetal stations.
Open curves use degree min(3, count-1), averaged knots, and one unknown pole per
station. Closed fits use cubic periodic collocation with wrapped pole indices.
Control intent uses the authored polygon, with open clamped uniform knots or a
closed cubic periodic knot vector. Closure is represented by wrapped poles and
extended knots, matching the application recipe, rather than changing a native
periodic flag. Both fit forms share the existing BSplCLib basis evaluator and
math_Gauss solver. There is no second basis recurrence, elimination engine,
ordered retry, sampled replacement, or geometry perturbation.

Return an owned recipe and source-indexed stations. Construct fresh native
curves from that recipe; do not retain or share mutable kernel handles. Measure
fit-point residuals and closed position/first-/second-derivative seam errors.
Requested acceptance limits and pivot policy remain explicit and unchanged.
Version-one defaults use machine epsilon for the dimensionless pivot and the
mapped native confusion (1e-7 working units) for each separate residual limit.
Derivatives are with respect to a dimensionless normalized parameter.
Non-finite, singular, or out-of-limit results are failures. These measurements
are curve-construction checks, not exact topology or root-completeness proofs.

The source policy is defined by application commit
`aa8297d1b0ac33c024f19e54821536d76e8a447a`, `fitted_bspline.go`,
`fitted_bspline_params.go`, `bspline_closed.go`, and `curvesample_spline.go`.
Retain the Go construction corpus and use native analytic evaluation as
complementary evidence. The new solver can differ in floating-point roundoff;
compare the recipe and analytic curve against explicit bounds, not a claim of
bitwise identity to a different elimination algorithm.

Completion removes the application's missing-native-construction decline and
its legacy Go fitting/control construction and sampled fallback paths. Issue #9
is the deletion and release task; its corpus gate includes every fit method,
control intent, small counts, closed C2 seams, handles, scales, degenerates,
ownership, and application/API acceptance. Do not close it after the initial
unhandled-curve constructor. Active handle semantics, release packaging, and
application adoption remain required. Complete consumption and legacy deletion
within the same application R3 milestone; a second production engine is not a
permanent fallback. No replacement of mapped OCCT behavior
or production Go fallback is authorized by this ADR.
