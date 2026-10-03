# TKG2d port status

**In progress; toolkit acceptance is not complete.** TKG2d is the first ModelingData
toolkit. The `Geom2d`, `Adaptor2d`, `Geom2dAdaptor`, `Geom2dEval`, `Geom2dGridEval` and
`Geom2dHash` packages are ported in full in `crates/tkg2d` (`rustycad-tkg2d`), and
`tools/audit_api_rows.py` reports 0 missing and 0 stale api rows for each of them.
The source inventory has 33 GoogleTest files and 523 declaration candidates. All 33
complete source suites account for 523 mapped declarations, with matching native and
Rust passes.

The class mapping (traits behind `Handle<dyn Trait>`, one lock per object for
shared mutation, `Failure` results) is described in [geom-design.md](geom-design.md).

| Source | Rust target | Declarations |
| --- | --- | ---: |
| Adaptor2d_Line2d_Test.cxx | adaptor2d_line2d | 23 |
| Adaptor2d_OffsetCurve_Test.cxx | adaptor2d_offset_curve | 20 |
| Geom2d_AxisPlacement_Test.cxx | geom2d_axis_placement | 14 |
| Geom2d_BSplineCurve_Test.cxx | geom2d_bspline_curve | 32 |
| Geom2d_BezierCurve_Test.cxx | geom2d_bezier_curve | 35 |
| Geom2d_CartesianPoint_Test.cxx | geom2d_cartesian_point | 13 |
| Geom2d_Circle_Test.cxx | geom2d_circle | 18 |
| Geom2d_Direction_Test.cxx | geom2d_direction | 15 |
| Geom2d_Ellipse_Test.cxx | geom2d_ellipse | 17 |
| Geom2d_Hyperbola_Test.cxx | geom2d_hyperbola | 14 |
| Geom2d_Line_Test.cxx | geom2d_line | 20 |
| Geom2d_Parabola_Test.cxx | geom2d_parabola | 13 |
| Geom2d_Transformation_Test.cxx | geom2d_transformation | 19 |
| Geom2d_TrimmedCurve_Test.cxx | geom2d_trimmed_curve | 17 |
| Geom2d_VectorWithMagnitude_Test.cxx | geom2d_vector_with_magnitude | 24 |
| Geom2dAdaptor_Curve_Test.cxx | geom2d_adaptor_curve | 14 |
| Geom2dEval_AHTBezierCurve_Test.cxx | geom2d_eval_aht_bezier_curve | 29 |
| Geom2dEval_ArchimedeanSpiralCurve_Test.cxx | geom2d_eval_archimedean_spiral_curve | 13 |
| Geom2dEval_CircleInvoluteCurve_Test.cxx | geom2d_eval_circle_involute_curve | 14 |
| Geom2dEval_LogarithmicSpiralCurve_Test.cxx | geom2d_eval_logarithmic_spiral_curve | 13 |
| Geom2dEval_SineWaveCurve_Test.cxx | geom2d_eval_sine_wave_curve | 12 |
| Geom2dEval_TBezierCurve_Test.cxx | geom2d_eval_tbezier_curve | 27 |
| Geom2dGridEval_BezierCurve_Test.cxx | geom2d_grid_eval_bezier_curve | 6 |
| Geom2dGridEval_Curve_Test.cxx | geom2d_grid_eval_curve | 29 |
| Geom2dGridEval_Ellipse_Test.cxx | geom2d_grid_eval_ellipse | 5 |
| Geom2dGridEval_Hyperbola_Test.cxx | geom2d_grid_eval_hyperbola | 5 |
| Geom2dGridEval_Parabola_Test.cxx | geom2d_grid_eval_parabola | 6 |
| Geom2dHash_CurveHasher_Test.cxx | geom2d_hash_curve_hasher | 39 |

`crates/tkg2d/tests/geom2d_supplemental.rs`, `adaptor_behavior.rs` and
`grid_eval_behavior.rs` add regression
tests for the quirks listed below, and `crates/tkernel/tests/handle_interfaces.rs` covers the handle
extension that the design needs.

## Verification

`python3 tools/verify_toolkit.py TKG2d --jobs 4 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, which already
builds `libTKG2d`, runs them and the Rust targets, checks exact runtime names, and
then runs `tools/verify_tkg2d_numeric.py`. Evidence is in `target/oracle-tkg2d/`.

## Numeric comparison policy

`tools/verify_tkg2d_numeric.py` builds `tools/oracle/tkg2d_probe.cxx` (a table of
operations) and its twin `crates/tkg2d/examples/numeric_probe.rs`, runs both on the
1787 requests of `tools/tkg2d_corpus.py`, `tools/tkg2d_adaptor_corpus.py`,
`tools/tkg2d_eval_corpus.py` and `tools/tkg2d_grid_corpus.py`, and uses the TKMath comparator: finite
values, infinities and signed zeros need identical bits, NaNs matching
classification. Exception types and the `DumpJson` text are compared too.

The corpus builds every curve kind (including invalid data, trimmed and offset
curves nested in each other) and checks, for each one, the parameter range,
closure, periodicity, period, continuity, `IsCN`, `EvalD0` to `EvalD3`, `EvalDN`
for orders 0 to 5, `Value` and `ReversedParameter`, then every Geom2d_Geometry move
and its copying twin. Other requests cover the point, direction, vector, axis and
transformation classes; conic getters and setters; trimming with and without
periodic adjustment; offset curves at cusps and at tangents so small that the
fallback formulas run; the `CalculateD0` to `CalculateD3` utilities directly;
Bezier edition (degree elevation, pole insertion and removal, segment, weights);
B-spline edition (degree elevation, knot insertion and removal, multiplicity
changes, periodicity, origin, segment, poles, weights, `MovePoint`,
`MovePointAndTangent`), `LocateU`, `LocalD0` to `LocalDN`, `IsG1`, `Resolution`,
the deprecated copying getters, and evaluation representations of all three kinds.

The adaptor requests (`a2_*`, `tools/oracle/tkg2d_probe_adaptor.cxx`) build every
adaptor kind (`Adaptor2d_Line2d` in its three constructions, `Geom2dAdaptor_Curve` on
every Geom2d curve kind with whole and partial ranges, `Adaptor2d_OffsetCurve` over
lines, circles, B-splines, cusps and other offsets, and the plain `Adaptor2d_Curve2d`),
and compare the bounds, `Continuity`, `NbIntervals` and `Intervals` for C0 to CN,
closure, periodicity, `Resolution`, the kind queries, `NbSamples`, `EvalD0` to `EvalDN`
and the non-virtual `Value`, `D0` to `DN` at the range bounds (the B-spline boundary
spans), at knots and inside the spans (the `BSplCLib_Cache` path, in call order), then
`Trim`, `ShallowCopy` and `Geom2dAdaptor::MakeCurve`. Other requests cover the empty
adaptors, the `Load` members (same curve, invalid ranges, null curves, `Reset`) and
evaluation representations recorded at load time.
The Geom2dEval requests build every eval curve (spirals, involute, sine wave, rational
and non-rational T-Bezier and AHT-Bezier curves of every basis shape, with shifted
array bounds, closed curves and each constructor refusal), bare and inside trimmed and
offset curves. Each goes through the curve summary, the refused moves, the class
accessors of the curve and of its copy, `EvalDN` up to order 9 and depth-limited
dumps. The Geom2dHash requests hash and compare, under seven tolerance pairs (a zero
and a 1e-300 `HashTolerance` included), copies, near copies, every Geom2d curve class,
eval curves, null handles and aliases of one handle, and the point, direction and
`gp_Ax22d` hashers on huge, signed-zero and rounding-boundary values. Hash values are
printed with an `h` prefix so that the comparator never reads them as doubles.
The Geom2dGridEval requests (`g2_grid*`, `tools/oracle/tkg2d_probe_grid.cxx`, 502 of
them) run `EvaluateGrid`, `EvaluateGridD1` to `D3` and `EvaluateGridDN` for orders -1 to
7 of `Geom2dGridEval_Curve` built on the curve handle and on a `Geom2dAdaptor_Curve`,
of the specialized evaluator of the curve's own class and of the adaptor fallback.
The curves cover every class: lines and conics on direct and indirect frames, the
zero-focal parabola, Bezier curves up to degree 25, B-splines with simple, multiple and
C0 knots, periodic and rational ones, trimmed curves (nested too), offsets at cusps and
tiny tangents and of every basis kind, and eval curves bare and wrapped. Each curve
gets dense sorted grids (the span-cache runs), sparse and shuffled grids (the direct
calls), one parameter, parameters beyond the range and the empty grid, with lower
bounds from -3 to 7. Other requests evaluate every adaptor kind through the
dispatcher (the base class raises from `GetType`) and the fallback, curves that carry
an evaluation representation, and every evaluator on null curves.

## Preserved native behaviour

- `Geom2d_BSplineCurve::EvalD0` to `EvalD3` normalize a periodic parameter and then
  pass the raw `U` to `LocateParameter`, which overwrites the normalized value.
- `updateKnots` maps `Degree - MaxKnotMult` to the continuity with `default: C3`, so
  an interior multiplicity above the degree gives C3.
- `Geom2d_Direction::Copy` passes the stored vector to the `gp_Dir2d` constructor,
  which normalizes it again: the copy can differ in the last bit.
- `Geom2d_TrimmedCurve::SetTrim` stores the new bounds before it raises the out of
  range error, and `Copy` re-trims with the default `Sense` and `theAdjustPeriodic`,
  so it adjusts periodic bounds again and fails on such stored bounds.
- `Geom2d_OffsetCurve` shares its basis with the caller unless it is trimmed, so
  `Reverse` and `Transform` also change the caller's curve. `SetBasisCurve`
  unwraps nested offsets (the curve itself included) and keeps the accumulated
  offset and continuity when it rejects a C0 B-spline that is not G1. Only
  B-spline bases get this check; other C0 bases are accepted. `EvalDN` above
  order 3 returns the basis derivative without the offset term.
- `Geom2d_OffsetCurveUtils::CalculateD3` subtracts the normal term of the second
  derivative in its low-magnitude branch, where `CalculateD2` adds it.
- `Geom2d_BezierCurve::InsertPoleAfter` refuses a curve of `MaxDegree()` poles,
  one fewer than the constructor accepts. The Bezier weight checks read
  `Weights(1..n)` whatever the lower bound, so other bounds raise
  Standard_OutOfRange; the deprecated `Weights(W)` of a non-rational Bezier curve
  writes `W(1..n)` the same way.
- Every change of an offset curve clears the evaluation representation, and so do
  the Bezier `Transform` and the Bezier edits that rebuild the poles (`Increase`,
  `InsertPoleAfter`/`Before`, `RemovePole`). Bezier `Reverse`, `Segment`,
  `SetPole` and `SetWeight`, and every B-spline change, `Transform` included,
  keep it.
- The copy constructor of `Geom2d_BSplineCurve` keeps the cached `Resolution`
  value (marked stale); the Bezier one resets it to 0.
- `Geom2dAdaptor_Curve` records a curve's evaluation representation when it loads the
  curve. Clearing it later still sends evaluation to the curve, and loading the same
  curve again only drops the span caches.
- A trimmed curve is loaded as its basis, so `Curve()` returns the basis. `Period()`
  returns the curve range without checking periodicity.
- `Geom2d_OffsetCurveUtils::EvaluateDN` passes one dummy vector as both D1 and D2 of
  `EvaluateD3`. For N = 3 the basis D2 replaces D1 in the singularity test, in
  `AdjustDerivative` and in the offset normal, so the offset `EvalDN(U, 3)` of
  `Geom2dAdaptor_Curve` differs from `EvalD3(U).D3` (`evaluate_d3_aliased`).
- `Adaptor2d_OffsetCurve` evaluates with `CalculateD0` to `CalculateD3` directly and
  ignores their result: at a singular point it returns the basis values, without the
  `AdjustDerivative` step of `Geom2dAdaptor_Curve`. Its `EvalD3` evaluates the basis
  `EvalDN(U, 4)` first, so an offset of an `Adaptor2d_OffsetCurve` raises
  Standard_NotImplemented there. `Resolution(R)` is `Precision::PConfusion(R)`,
  `Confusion() / R`.
- `Adaptor2d_OffsetCurve::NbIntervals` and `Intervals` scan the basis bounds with
  range-checked indices: bounds outside the basis range, or the `[0, 0]` bounds of the
  one-argument constructor, raise Standard_OutOfRange. `Load(Offset)` stores the
  offset before it reads the basis bounds, so a failing basis keeps the new offset.
- `Geom2dAdaptor_Curve::NbIntervals` of an offset curve raises Standard_DomainError for
  G1 and G2 with the message `GeomAdaptor_Curve::NbIntervals`.

- The Geom2dEval curves raise Standard_NotImplemented from `Reverse`,
  `ReversedParameter` and `Transform`, so every copying move (`Reversed`,
  `Transformed`, `Mirrored`...) fails too, a trimmed curve on such a basis cannot be
  reversed, and an offset curve on it can be neither reversed nor transformed.
  `IsCN(N)` is true for every N, negative ones included, and `Period` raises
  Standard_NoSuchObject. Their constructors test `x < 0` or `x <= 0`, so NaN
  parameters are accepted.
- `Geom2dEval_AHTBezierCurve` evaluates a purely algebraic basis with Horner's scheme in
  `EvalD0` to `EvalD3`, but with explicit powers in `EvalDN` above order 3 (with a
  separate `U == 0` branch). Its rational `EvalD1` and `EvalD2` compute the first
  derivative as `(N1 W0 - N0 W1) / W0^2`, while `EvalD3` uses `(N1 - C W1) / W0`, so
  the first derivatives of the two can differ in the last bits.
- `Geom2dEval_TBezierCurve::EvalDN` of a non-rational curve sums the poles one by one
  for every order, while `EvalD1` to `EvalD3` add each sin/cos pair at once, so
  `EvalDN(U, 1)` can differ from `EvalD1(U).D1` in the last bits. Weights equal to
  each other still make a rational T-Bezier or AHT-Bezier curve (unlike
  Geom2d_BezierCurve), and both classes keep the bounds of the arrays they are given.
- `Geom2dEval_AHTBezierCurve::DumpJson` writes every pole under the same key `"Pole"`;
  `Geom2dEval_TBezierCurve::DumpJson` writes `"Poles.Length"`.
- The Geom2dHash hash of a value is `hash(int64_t(round(value / HashTolerance)))`. The
  cast is `cvttsd2si` on the pinned profile, which gives INT64_MIN for NaN and
  out-of-range values (a zero or tiny `HashTolerance`); the port reproduces it
  instead of Rust's saturating cast. The Bezier and B-spline hashes cover only degree,
  counts and rationality. `Geom2dHash_CurveHasher` hashes classes outside Geom2d (the
  Geom2dEval curves) by type name through Standard_CStringHasher and compares them by
  object identity.

- `Geom2dGridEval_Curve` strips trimmed wrappers without applying their bounds, reads
  the curve of a `Geom2dAdaptor_Curve` without its range or its recorded evaluation
  representation, and leaves any class other than the eight specialized ones (the
  Geom2dEval curves, also inside a `Geom2dAdaptor_Curve` or an offset curve)
  uninitialized: every grid is empty and `GetType` is `OtherCurve`. Only other
  adaptors take the `Geom2dGridEval_OtherCurve` fallback, with the adaptor's `GetType`.
- No grid evaluator uses the evaluation representation of a Bezier, B-spline or
  offset curve, but the offset evaluator's `EvalDN(U, 4)` and `AdjustDerivative` calls
  go through the basis curve's own members, which do.
- `Geom2dGridEval_Parabola` rebuilds Y as `(-X.Y, X.X)`, so the grid of a parabola on
  an indirect frame is its mirror image; it multiplies by `1 / (4F)` and `1 / (2F)`,
  so a zero focal gives infinite and NaN coordinates where `ElCLib` returns the line.
- The Bezier and B-spline `EvaluateGridDN` return zero vectors above the degree, also
  for rational curves. The B-spline one locates the span in the flat knots and passes
  that index to `BSplCLib::DN` with the knots and multiplicities; `PrepareEval` keeps an
  index inside the knot index range, so the derivative comes from another span.
- The offset evaluator returns an empty grid when the normal is degenerate at any
  parameter; its D2 and D3 grids first apply `AdjustDerivative` at singular points.

## Deliberate Rust deviations

- Getters that return `const&` natively return copies, because a reference cannot
  outlive the object's lock. `WeightsArray()` lends the stored array to a closure,
  so its identity and its non-owning unit-weight view stay observable.
- Where native code dereferences a null handle (a null offset basis, or an
  evaluation representation cleared after validation), the port returns
  Standard_ConstructionError or Standard_ProgramError.
- Where BSplCLib reads outside its buffers (for example after `Segment` with a
  parametric tolerance wider than a knot span), the port returns
  Standard_OutOfRange, as documented for BSplCLib. The corpus avoids those inputs.
- A periodic B-spline evaluated at an infinite parameter loops in
  `PeriodicNormalization`, natively and in Rust; the corpus avoids it.
- Every `Adaptor2d_Curve2d` member returns `Result`, because the base class raises
  Standard_NotImplemented for all of them but `NbSamples`. `Adaptor2dCurve2dBase` is an
  instance of that non-abstract base class. The adaptors take `&self` like the Geom2d
  objects; the `Geom2dAdaptor_Curve` span caches are rebuilt under its write lock.
- Where native adaptors dereference a null curve (an empty `Geom2dAdaptor_Curve` or
  `Adaptor2d_OffsetCurve`), the port returns Standard_NullObject. `Geom2dAdaptor_Curve.cxx`
  disables the array range checks, so an `Intervals` array smaller than
  `NbIntervals + 1` is undefined natively and Standard_OutOfRange in Rust. The corpus
  avoids both.
- The eval curves have no successful mutator, so they keep plain fields instead of
  the per-object lock of the Geom2d classes.
- The non-rational `Geom2dEval_AHTBezierCurve` stores `myWeights(1, 1)`, one
  uninitialized double; the port stores 0.0 there. The corpus prints only its bounds.
- The private helpers `evalBasis`/`evalBasisDeriv` of `Geom2dEval_AHTBezierCurve` and
  `evalNonRationalPoint`/`evalNonRationalDeriv` of `Geom2dEval_TBezierCurve` are never
  called natively and are not ported; the T-Bezier `evalBasis`/`evalBasisDeriv`, which
  the rational `EvalDN` uses, are.
- The typed Geom2dHash hashers take the curve object, where the native ones take a
  handle and dereference it without a null check. A B-spline pair with equal knots and
  multiplicities but different pole counts (periodic against non-periodic) raises
  Standard_OutOfRange inside the native `noexcept` comparison, which terminates the
  process; the port reports the pair as different. The corpus avoids that pair.
- When the flat span index of `Geom2dGridEval_BSplineCurve::EvaluateGridDN` equals the
  last knot index of a non-periodic B-spline, the native `BSplCLib::BuildKnots` reads
  past the knot and multiplicity arrays (undefined); the port raises
  Standard_OutOfRange. The corpus leaves those parameters out (`dn_safe`).
- `Geom2dGridEval::CurveD1` to `CurveD3` are aliases of the `Geom2d_Curve::ResD1` to
  `ResD3` structs, which have the same fields. The evaluators return `Result` where a
  native call can raise (the gp getters behind `Circ2d()`..., BSplCLib, the offset
  utilities, an adaptor); the line evaluator returns plain arrays.
- `Geom2dGridEval_OtherCurve` borrows the adaptor with a lifetime, as the native
  `std::reference_wrapper`, so `Geom2dGridEvalCurve::from_adaptor` ties the evaluator
  to the adaptor; `from_curve` has no borrow.
- `Geom2d_TrimmedCurve::SetTrim` on a non-periodic basis with a NaN bound fails `U1 < U2`,
  takes the reversed sense and calls `Reverse`, whose NaN bounds call `SetTrim` again,
  without end (natively the program never returns; the port overflowed its stack). The
  port raises Standard_ConstructionError naming the NaN bounds. A periodic basis keeps
  the native NaN trim.

## Recovered cross-toolkit legacy suites

The final five source suites (17 declarations) now use the ported dependencies below
and are mapped in `geom2d_modeling_legacy.rs`:

- `Geom2d_CurveEval_Test.cxx` (7): `Geom2dAPI_PointsToBSpline` (TKGeomAlgo).
- `Geom2d_OffsetCurve_Test.cxx` (5): `GCPnts_AbscissaPoint`, `Geom2dConvert` and
  `Geom2dConvert_ApproxCurve` (TKGeomBase).
- `Geom2dAPI_InterCurveCurve_Test.cxx` (3): `Geom2dAPI_InterCurveCurve`,
  `Geom2dAPI_ExtremaCurveCurve` and `IntRes2d` (TKGeomAlgo, with Extrema from TKGeomBase).
- `Geom2dGcc_Circ2d2TanOn_Test.cxx` (1): `Geom2dGcc_Circ2d2TanOn`,
  `Geom2dGcc_QualifiedCurve`, `GccEnt` and `Geom2dAPI_ProjectPointOnCurve` (TKGeomAlgo).
- `Geom2dGcc_Circ2d2TanRad_Test.cxx` (1): `Geom2dGcc_Circ2d2TanRad`,
  `Geom2dGcc_QualifiedCurve` and `GccEnt_Position` (TKGeomAlgo).
- `DumpJson` is compared on the corpus; the `InitFromJson` counterparts do not
  exist in Geom2d.

All 523 GoogleTest declaration candidates now have complete source-suite mappings and
matching native/Rust passes. The newly recovered fixtures retain original parameter
bounds, sampling, exception assertions and approximation tolerances. TKGeomBase and
TKGeomAlgo are test-only dependencies. This completes the inventoried GoogleTest
mapping for TKG2d; full API and other legacy-test acceptance still follows the project
coverage inventory, rather than this count alone.

## Compiler inventory constructor audit (2026-10-03)

Corrected 36 existing Geom2d constructor ledger IDs to the compiler-qualified
`Class::Class(parameters)` spelling. An unqualified class name identifies the
record declaration and cannot prove a constructor overload. The identity
constructor of `Geom2d_Transformation` now names the externally reachable
`<core::default::Default>::default` implementation explicitly. No geometry
implementation or legacy declaration count changed in this audit.

Validation against pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`:

- `cargo test --locked -p rustycad-tkg2d`: 602 tests passed across 41 targets.
- `python3 tools/verify_tkg2d_numeric.py --source OCCT --native-build BUILD`:
  1,787 numeric comparisons passed (exact finite bits, NaN classification).
- The compiler crosswalk resolves each corrected constructor overload separately.

These checks support the audited slice. They do not prove full API equivalence or
join every GoogleTest runtime instance to a Rust execution; those obligations
remain visible in the global inventories.
