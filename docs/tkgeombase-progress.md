# TKGeomBase port status

**In progress; toolkit acceptance is not complete.** `crates/tkgeombase`
(`rustycad-tkgeombase`) is the port of the ModelingData toolkit TKGeomBase. It
depends on the TKernel, TKMath, TKG2d and TKG3d ports, as the native toolkit
does. Packages are added one module at a time. The `gce`, `GC`, `GCE2d`, `ExtremaPC2d`
and `ExtremaPC` packages are ported in full, and `tools/audit_api_rows.py` reports 0
missing and 0 stale api rows for each of them.

The Geom and Geom2d objects that TKGeomBase builds follow the class mapping of
[geom-design.md](geom-design.md).

The toolkit test inventory maps these complete suites, with matching native and Rust
passes (408 declarations):

| Source | Rust target | Declarations |
| --- | --- | ---: |
| gce_MakeCirc2d_Test.cxx | gce_make_circ2d | 3 |
| gce_MakeCone_Test.cxx | gce_make_cone | 9 |
| gce_MakeCylinder_Test.cxx | gce_make_cylinder | 3 |
| gce_MakeElips_Test.cxx | gce_make_elips | 3 |
| gce_MakeHypr_Test.cxx | gce_make_hypr | 24 |
| GC_MakeArcOfCircle_Test.cxx | gc_make_arc_of_circle | 3 |
| GC_MakeCircle2d_Test.cxx | gc_make_circle2d | 3 |
| GC_MakeConicalSurface_Test.cxx | gc_make_conical_surface | 2 |
| GC_MakeParabola2d_Test.cxx | gc_make_parabola2d | 3 |
| GC_MakePlane_Test.cxx | gc_make_plane | 3 |
| GC_MakeSegment2d_Test.cxx | gc_make_segment2d | 6 |
| ExtremaPC2d_Test.cxx | extrema_pc2d | 6 |
| ExtremaPC2d_BezierCurve_Test.cxx | extrema_pc2d_bezier_curve | 8 |
| ExtremaPC2d_BSplineCurve_Test.cxx | extrema_pc2d_bspline_curve | 7 |
| ExtremaPC2d_Circle_Test.cxx | extrema_pc2d_circle | 12 |
| ExtremaPC2d_Curve_Test.cxx | extrema_pc2d_curve | 10 |
| ExtremaPC2d_DistanceFunction_Test.cxx | extrema_pc2d_distance_function | 4 |
| ExtremaPC2d_Ellipse_Test.cxx | extrema_pc2d_ellipse | 9 |
| ExtremaPC2d_GridEvaluator_Test.cxx | extrema_pc2d_grid_evaluator | 7 |
| ExtremaPC2d_Line_Test.cxx | extrema_pc2d_line | 8 |
| ExtremaPC2d_OffsetCurve_Test.cxx | extrema_pc2d_offset_curve | 7 |
| ExtremaPC2d_OtherCurve_Test.cxx | extrema_pc2d_other_curve | 5 |
| ExtremaPC2d_Parabola_Test.cxx | extrema_pc2d_parabola | 10 |
| ExtremaPC_BezierCurve_Test.cxx | extrema_pc_bezier_curve | 31 |
| ExtremaPC_BSplineCurve_Test.cxx | extrema_pc_bspline_curve | 27 |
| ExtremaPC_Circle_Test.cxx | extrema_pc_circle | 26 |
| ExtremaPC_Curve_Test.cxx | extrema_pc_curve | 47 |
| ExtremaPC_Ellipse_Test.cxx | extrema_pc_ellipse | 30 |
| ExtremaPC_Hyperbola_Test.cxx | extrema_pc_hyperbola | 24 |
| ExtremaPC_Line_Test.cxx | extrema_pc_line | 31 |
| ExtremaPC_OffsetCurve_Test.cxx | extrema_pc_offset_curve | 13 |
| ExtremaPC_Parabola_Test.cxx | extrema_pc_parabola | 24 |

`gce_behavior.rs` and `gc_behavior.rs` cover the status enumeration, the
StdFail_NotDone results, the conversion operators, the GCE2d aliases and the native
quirks below.

Four upstream suites are not mapped:

- `ExtremaPC2d_Hyperbola_Test.cxx` (9 declarations) is translated as
  `extrema_pc2d_hyperbola` and passes in Rust, but `SingletonInvalidAndValueAccessor`
  fails natively (see Preserved native behaviour), and the verifier maps whole suites
  whose native run passes.
- `ExtremaPC_SearchMode_Test.cxx` (30), `ExtremaPC_Comparison_Test.cxx` (17) and
  `ExtremaPC_ExtendedGeometry_Test.cxx` (40) compare ExtremaPC with the classic
  `Extrema_ExtPC`, which is not ported. They wait for the Extrema package.

## Class mapping

Each `gce_Make*` class is a struct with one named constructor per native overload
(`MakeCirc::through_points`, `MakeCone::from_points_radii`...). A constructor returns
`Result` when a gp constructor it calls can raise, so a native exception stays a
`Failure` instead of a status. `gce_Root` is the `Root` trait (`status`, `is_done`,
`is_error`); `Value()` and `Operator()` return `Result` (StdFail_NotDone, with the
status in the message, when the construction failed), and the conversion operator
`operator gp_Circ()` is `TryFrom<&MakeCirc> for Circle3`. The transformation makers
(`MakeMirror`, `MakeRotation`, `MakeScale`, `MakeTranslation` and their 2D twins) are
not gce_Root algorithms: `value()` returns the transformation and the conversion is
`From`.

The `GC_Make*` classes follow the same rules and build Geom objects (the `*2d`
classes Geom2d objects) held by `Handle`: `value()` returns the handle, and
`operator const occ::handle<T>&()` is `TryFrom<&MakeCircle> for Handle<GeomCircle>`
(`From` for the transformation makers). GC_Root has the members of gce_Root, so the
port shares the `Root` trait (`gc::Root` re-exports it). Since OCCT 8.0.0 every GCE2d
header is a deprecated alias of a GC class (`using GCE2d_MakeCircle = GC_MakeCircle2d`);
`gce2d` declares them as `#[deprecated]` type aliases with the native message, and
re-exports `Root` for GCE2d_Root.

`gce_MakeCirc`, `gce_MakeCirc2d`, `GC_MakeArcOfCircle(2d)` and `GC_MakeTrimmedCone` call
`Extrema_ExtElC` on two lines, `IntAna2d_AnaIntersection` on two 2D lines and
`Extrema_ExtPElC` on a point and a line, which are TKGeomBase packages not ported yet.
`src/line_extrema.rs` ports exactly those cases in the native operation order; the
Extrema and IntAna2d ports will replace it.

## ExtremaPC2d and ExtremaPC

The two packages are one generic implementation in `src/extrema_pc_core`, over the
`PcSpace` trait (`Space2d`, `Space3d`): the result (`PcResult`, `PcExtremum`), the
`Status` and `SearchMode` enumerations, the validity functions and constants, the
distance function, the grid evaluator, the endpoint extrema and the analytic
endpoint handling. `extrema_pc2d` and `extrema_pc` give the native names:
`ExtremaResult` and `ExtremumResult` for `Result` and `ExtremumResult`,
`DistanceFunction` and `GridEvaluator` for the class templates, the evaluators without
their prefix (`extrema_pc::Circle` for ExtremaPC_Circle). The native code of the two
packages differs in a few places, and the core keeps these differences as parameters:
the neighbour step ratio (1e-6 in 2D, 0.01 in 3D) and the catch policy of the grid
evaluator (the 2D evaluator catches every failure; the 3D evaluator catches only a
failed sign evaluation, which reads 0, and lets a failed curve value propagate).

API mapping:

- A constructor with a domain is `with_domain`. The ExtremaPC(2d)_Curve constructors
  are `from_adaptor`, `from_adaptor_range`, `from_geom`, `from_geom_range` and (3D)
  `from_transformed`, `from_transformed_range`; the dispatcher keeps the chosen
  evaluator in `EvaluatorVariant`.
- A native exception propagates out of `Perform` as a `Failure`. The 2D analytic
  evaluators cannot raise, so their `perform` returns `&ExtremaResult`; the 2D Bezier,
  BSpline, offset, other-curve and dispatcher `perform` return
  `Result<&ExtremaResult, Failure>`. In 3D every `perform` returns `Result`, because
  the plane projections build gp and Geom2d objects whose constructors can raise, and
  the Bezier, BSpline, offset and other-curve constructors return `Result` too.
- The native evaluators keep a mutable result in a `const` object; the Rust `perform`
  takes `&mut self` and lends the result.
- Output references of the distance function become returned values; the
  `CurveEvaluator` template parameter of `AddEndpointExtrema` is the `CurveEvaluator`
  trait. The planar helpers keep their `gp_Pln&` and 2D-curve out-parameters as `&mut`
  parameters; `PerformPlanar` takes the `PlanarEvaluator2d` trait for `Evaluator2d` and
  groups its scalar arguments in `PlanarQuery`.
- The x87 `long double` arithmetic of the rational Bezier certification and of the
  exact pole-plane test uses `rustycad_tkmath::math_poly::LongDouble`.

ExtremaPC needs parts of three TKGeomBase packages that are not ported yet. They are
crate-private ports of exactly the code paths it uses, in the native operation order,
and the ports of those packages will replace them:

- `GCPnts_TangentialDeflection` and `GCPnts_DistFunction(2d)` (the curve-aware
  sampling of the grid), in `src/tangential_deflection`;
- `Extrema_CurveTool::DeflCurvIntervals` and `Extrema_Curve2dTool::DeflCurvIntervals`,
  in `src/defl_curv_intervals.rs`;
- the ProjLib plane projections of lines and conics, in `src/plane_projection.rs`.

`probe_support` is a hidden public module that exposes them to the numeric probe.

## Verification

`python3 tools/verify_toolkit.py TKGeomBase --jobs 8 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and
the Rust targets, checks exact runtime names, then runs
`tools/verify_tkgeombase_numeric.py`. Evidence is in `target/oracle-tkgeombase/`.

## Numeric comparison policy

`tools/verify_tkgeombase_numeric.py` uses the shared driver `tools/toolkit_numeric.py`
(also used by TKG2d and TKG3d). It builds `tools/oracle/tkgeombase_probe.cxx` (a
table of operations over `tkgeombase_probe_*.cxx`, linked with libTKGeomBase and
the toolkits below it) and its twin `crates/tkgeombase/examples/numeric_probe.rs`,
runs both on `tools/tkgeombase_corpus.py` (one `tkgeombase_corpus_*.py` module per
package) and requires identical bits for finite values, infinities and signed
zeros, NaN by classification and identical exception types.

The gce requests build every constructor overload of every class with random valid
inputs and with the degenerate inputs that reach each reachable gce_ErrorType and each
raising gp constructor: coincident points (exact, within gp::Resolution() and
denormal), collinear points, null and negative radii, null, right and negative
angles, frames along every coordinate axis (each branch of the X-direction choices),
and magnitudes near 1e-160 and 1e150 where squares underflow or overflow. Each
request prints the status, `IsDone`, `IsError`, `Value()` and `Operator()` (or the
raised exception). The native algorithm is created with `new`, which zeroes it, so
the paths that never assign the status compare as the port reads them.

The GC requests build every overload of every class the same way, and print the Geom or
Geom2d result by its class name and defining data (the gp element of each conic, line,
plane, cone and cylinder, the bounds and basis of each trimmed curve and surface, the
gp_Trsf of each transformation), through `Value()` and the conversion operator. They
add equal trim parameters, points on and off the conics, tangents null, along and
1e-13 rad from the chord, projections outside the `Extrema_ExtPElC` bounds, null
scale factors and bad cone angles at both resolution bounds. They leave out the inputs
for which a GC constructor reads the status of a stack gce algorithm on one of its
unassigned paths (three points within gp::Resolution() for `GC_MakeCircle` and
`GC_MakeArcOfCircle`, collinear points for their 2D versions): the native status is
indeterminate there.

The ExtremaPC2d requests (`tkgeombase_corpus_extrema_pc2d.py`) run every 2D curve
kind through the four ExtremaPC2d_Curve constructors with bounded, unbounded,
degenerate, reversed, whole-period and multi-period domains, and query each evaluator
in the three search modes, with and without endpoints: random points, the center,
the foci, points on the axes, the vertex, points on the curve, invalid tolerances and
points at Precision::Infinite(). The ExtremaPC requests (`tkgeombase_corpus_extrema_pc.py`)
do the same through the six ExtremaPC_Curve constructors, with identity, translation,
rotation and positive and negative scaling transformations, conics in random frames
queried above and below their plane, and planar (exact pole planes, collinear and
coincident poles) and non-planar Bezier, BSpline and offset curves, rational and not.
Both add direct requests for the grid evaluator, the uniform and curve-aware
partitions, the distance function, AddEndpointExtrema, the plane projections and the
tangential deflection sampling. Each request prints the status, the extrema in their
order (parameter, point, square distance, minimum and maximum flags), the minimum and
maximum indices, or the raised exception. The corpus has 6954 requests (4449 gce and
GC, 1253 ExtremaPC2d, 1252 ExtremaPC).

## Preserved native behaviour

gce:

- gce_Root has no constructor, and three paths never assign the status:
  `gce_MakeCirc(P1, P2, P3)` with three points within gp::Resolution() or a computed
  center on P1 (both return a null circle), and `gce_MakeCirc2d(P1, P2, P3)` when the
  bisectors are parallel (collinear points; the circle keeps its default value). A
  stack algorithm reads an indeterminate status there; the port reads gce_Done, the
  value of a heap algorithm (the OCCT allocator zeroes memory).
- gce_IntersectionError is unreachable in `gce_MakeCirc` and `gce_MakeCirc2d`: the line
  extremum and the line intersection are always done. Identical bisectors in 2D would
  raise Standard_OutOfRange from `IntAna2d_AnaIntersection::Point(1)`.
- `gce_MakeCirc(Center, Plane, Radius)` reads `Value()` of the center-normal
  construction, so a negative radius raises StdFail_NotDone.
- `gce_MakeCirc2d(Circ, Dist)` takes `|Radius + Dist|` and never fails, while the 3D
  and cylinder versions report gce_NegativeRadius.
- `gce_MakeElips(A2, ...)` checks gce_InvertRadius before gce_NegativeRadius, the 2D
  version the other way round; `gce_MakeHypr` accepts a major radius smaller than the
  minor one.
- `gce_MakeElips2d(S1, S2, Center)` normalizes both directions before any check, so
  S1 or S2 at the center raises Standard_ConstructionError; it reports gce_InvertAxis
  before gce_NullAxis.
- `gce_MakeCone(P1, P2, P3, P4)` normalizes P3 - P1 and P4 - P1 without a check (P3 or
  P4 at P1 raises Standard_ConstructionError); it reports gce_NullAngle when P3 and P4
  project to the same axis point and gce_NullRadius for a null or right semi-angle,
  while `gce_MakeCone(P1, P2, R1, R2)` reports gce_NullAngle for the latter; its
  gce_NegativeRadius test on two distances is unreachable.
- `gce_MakeDir(X, Y, Z)`, `gce_MakeLin2d(A, B, C)` and `gce_MakePln(A, B, C, D)`
  compare the square length with gp::Resolution().
- `gce_MakeScale` raises Standard_ConstructionError for a null factor (gp_Trsf), while
  `gce_MakeScale2d` accepts it (gp_Trsf2d).

GC and GCE2d:

- The GC constructors that wrap a gce algorithm on a stack object read its status as
  it is; on the unassigned gce paths the native value is indeterminate, while the port
  reads gce_Done (and the 3D arc through three coincident points then raises
  Standard_ConstructionError from the trimmed curve, whose bounds are equal).
- `GC_MakeArcOfCircle(P1, V, P2)` does not test `IsParallel` on the extremum of the
  bisector and the normal at P1, unlike gce_MakeCirc: for a tangent a few 1e-13 rad
  from the chord, the lines are parallel and the center is the default
  Extrema_POnCurv point, the origin. A tangent along the chord raises
  Standard_ConstructionError from `gp_Dir::Crossed`.
- `GC_MakeArcOfCircle2d(P1, V, P2)` converts gce_MakeLin2d(P1, P2) directly, so confused
  points raise StdFail_NotDone; a tangent along the chord (parallel bisector and
  normal) leaves the initial gce_ConfusedPoints; the arc is direct when
  `V ^ chord > 0`.
- `GC_MakeArcOfCircle2d(P1, P2, P3)` trims from the parameter of P1 to that of P3; the
  3D version trims from 0 (the X axis of the gce circle passes through P1).
- `GC_MakeSegment(P1, P2)` converts GC_MakeLine(P1, P2), so confused points raise
  StdFail_NotDone, while `GC_MakeSegment2d(P1, P2)` reports gce_ConfusedPoints first.
- `GC_MakeEllipse(A2, ...)` reports gce_NegativeRadius first, then gce_InvertAxis (not
  gce_InvertRadius) for a major radius smaller than the minor one.
- `GC_MakeConicalSurface(A2, Ang, R)` refuses `Ang >= PI/2 - Resolution` where
  gce_MakeCone tests `PI/2 - Ang <= Resolution`; `GC_MakePlane(A, B, C, D)` compares
  `sqrt(A² + B² + C²)` with gp::Resolution() where gce_MakePln compares the square.
- `GC_MakeCylindricalSurface(Cyl, Dist)` uses the radius `|R - Dist|` (always done)
  where gce_MakeCylinder uses `R + Dist`.
- `GC_MakeTrimmedCone(P1, P2, P3, P4)` projects P3 and P4 with `Extrema_ExtPElC`
  bounded to `[-2e100, 2e100]` (tolerance 1e-7); a projection outside raises
  StdFail_NotDone from `NbExt()`.
- The GC transformation makers set a new Geom transformation through its own setter:
  `GC_MakeTranslation(P1, P2)` uses `gp_Trsf::SetTranslation(P1, P2)` where
  gce_MakeTranslation builds `gp_Vec(P1, P2)` first.

ExtremaPC2d and ExtremaPC:

- `ExtremaPC2d_Hyperbola_Test.SingletonInvalidAndValueAccessor` fails natively with
  GCC 13. The test compares `Value(-0.7)` with `gp_Hypr2d` evaluated through
  `ElCLib`; GCC folds `sinh(-0.7)` in the test at compile time to
  `-0x1.8465153d5bdbdp-1`, while glibc's `sinh` returns `-0x1.8465153d5bdbcp-1` at run
  time, one ulp apart. The port calls the run-time `sinh`, as the native library does,
  and the Rust test passes.
- The 2D and 3D packages use different neighbour steps for the endpoint extrema
  (`THE_NEIGHBOR_STEP_RATIO` is 1e-6 in 2D and 0.01 in 3D), and their grid evaluators
  catch failures differently (see above).
- `std::max` and `std::min` keep their argument order semantics with NaN
  (`geom2d::native::std_max`, `std_min`).
