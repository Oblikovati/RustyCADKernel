# TKG2d port status

**In progress; toolkit acceptance is not complete.** TKG2d is the first ModelingData
toolkit. The `Geom2d` package is ported in full in `crates/tkg2d`
(`rustycad-tkg2d`), together with the two `Geom2dEval` headers that `Geom2d`
includes. The other TKG2d packages (`Adaptor2d`, `Geom2dAdaptor`, `Geom2dEval`
curves, `Geom2dGridEval`, `Geom2dHash`) are not ported yet. The source inventory
has 33 GoogleTest files and 523 declaration candidates. 13 complete source suites
account for 251 mapped declarations, with matching native and Rust passes.

The class mapping (traits behind `Handle<dyn Trait>`, one lock per object for
shared mutation, `Failure` results) is described in [geom-design.md](geom-design.md).

| Source | Rust target | Declarations |
| --- | --- | ---: |
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

`crates/tkg2d/tests/geom2d_supplemental.rs` adds regression tests for the quirks
listed below, and `crates/tkernel/tests/handle_interfaces.rs` covers the handle
extension that the design needs.

## Verification

`python3 tools/verify_toolkit.py TKG2d --jobs 4 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, which already
builds `libTKG2d`, runs them and the Rust targets, checks exact runtime names, and
then runs `tools/verify_tkg2d_numeric.py`. Evidence is in `target/oracle-tkg2d/`.

## Numeric comparison policy

`tools/verify_tkg2d_numeric.py` builds `tools/oracle/tkg2d_probe.cxx` (a table of
operations) and its twin `crates/tkg2d/examples/numeric_probe.rs`, runs both on the
773 requests of `tools/tkg2d_corpus.py`, and uses the TKMath comparator: finite
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

## Remaining coverage

- `Geom2d_CurveEval_Test.cxx` (7) needs Geom2dAPI and `Geom2d_OffsetCurve_Test.cxx`
  (5) needs GCPnts and Geom2dConvert; both wait for those toolkits.
- The Adaptor2d, Geom2dAdaptor, Geom2dEval, Geom2dGridEval, Geom2dHash,
  Geom2dAPI and Geom2dGcc suites wait for their packages.
- `DumpJson` is compared on the corpus; the `InitFromJson` counterparts do not
  exist in Geom2d.
