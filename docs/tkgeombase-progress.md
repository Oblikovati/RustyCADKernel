# TKGeomBase port status

**In progress; toolkit acceptance is not complete.** `crates/tkgeombase`
(`rustycad-tkgeombase`) is the port of the ModelingData toolkit TKGeomBase. It
depends on the TKernel, TKMath, TKG2d and TKG3d ports, as the native toolkit
does. Packages are added one module at a time. The `gce`, `GC`, `GCE2d`, `IntAna`,
`IntAna2d`, `LProp`, `GeomLProp`, `GProp`, `Hermit`, `ExtremaPC2d` and `ExtremaPC`
packages are ported in full, and `tools/audit_api_rows.py` reports 0 missing and 0
stale api rows for each of them.

The Geom and Geom2d objects that TKGeomBase builds follow the class mapping of
[geom-design.md](geom-design.md).

The toolkit test inventory maps these complete suites, with matching native and Rust
passes (481 declarations):

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
| IntAna_IntQuadQuad_Test.cxx | int_ana_int_quad_quad | 10 |
| LProp_CurAndInf_Test.cxx | lprop_cur_and_inf | 10 |
| GeomLProp_CLProps2d_Test.cxx | geom_lprop_clprops2d | 18 |
| GeomLProp_CurAndInf2d_Test.cxx | geom_lprop_cur_and_inf2d | 13 |
| GProp_PEquation_Test.cxx | gprop_pequation | 4 |
| GProp_PGProps_Test.cxx | gprop_pgprops | 7 |
| Hermit_Test.cxx | hermit | 11 |
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
| GCPnts_AbscissaPoint_Test.cxx | gcpnts_abscissa_point | 5 |
| GCPnts_QuasiUniformDeflection_Test.cxx | gcpnts_quasi_uniform_deflection | 1 |

`local_props_behavior.rs` covers the LProp, GeomLProp, GProp and Hermit quirks
below. `gce_behavior.rs` and `gc_behavior.rs` cover the status enumeration, the
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

`gce_MakeCirc`, `GC_MakeArcOfCircle` and `GC_MakeTrimmedCone` call `Extrema_ExtElC` on
two lines and `Extrema_ExtPElC` on a point and a line; Extrema is not ported yet, and
`src/line_extrema.rs` ports exactly those cases in the native operation order until the
Extrema port replaces it. `gce_MakeCirc2d` and `GC_MakeArcOfCircle2d` call the IntAna2d
port (`int_ana2d::AnaIntersection::lines`).

IntAna and IntAna2d (`int_ana`, `int_ana2d`) are algorithm objects: each native
constructor overload is a named constructor (`QuadQuadGeo::cones`,
`AnaIntersection::line_circle`, `IntConicQuad::circle_plane`...) and each Perform
overload a `perform_*` method on `&mut self`, which keeps the fields that Perform does
not assign, as natively. A Perform or accessor that can raise returns `Result` with the
native exception type (StdFail_NotDone, Standard_DomainError, Standard_OutOfRange,
Standard_ConstructionError from gp). Output parameters become return values
(`Curve::domain`, `IntLinTorus::param_on_torus`, `IntQuadQuad::next_curve` returns the
index and the opposite flag); `Curve::d1u` keeps its two output arguments because the
native member assigns the point even when it returns false. IntAna_Quadric and
IntAna_Curve take the ten quadric coefficients as one `[Qxx, Qyy, Qzz, Qxy, Qxz, Qyz,
Qx, Qy, Qz, Q1]` array, and the curve domain as a pair. The deprecated
IntAna_ListOfCurve is the `#[deprecated]` alias `int_ana::ListOfCurve`. The cone-cone
EstimDist needs `ProjLib::Project(gp_Pln, gp_Lin)`; ProjLib is not ported, so
`quad_quad_geo/estim_dist.rs` holds that one line projection (ProjLib_Plane) until the
ProjLib port replaces it.

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

ExtremaPC needs parts of two TKGeomBase packages that are not ported yet. They are
crate-private ports of exactly the code paths it uses, in the native operation order,
and the ports of those packages will replace them:

- `Extrema_CurveTool::DeflCurvIntervals` and `Extrema_Curve2dTool::DeflCurvIntervals`,
  in `src/defl_curv_intervals.rs`;
- the ProjLib plane projections of lines and conics, in `src/plane_projection.rs`.

Its curve-aware sampling calls the public `gcpnts::TangentialDeflection`.
`probe_support` is a hidden public module that exposes these functions and a
tangential-deflection sampling helper to the numeric probe.

## CPnts and GCPnts

Every native member of the two packages has a 3D and a 2D overload with the same body;
the port has one member for each pair.

- CPnts takes the curve as `impl Into<PntsCurve>`: `PntsCurve` is the native
  `void* myCurve` with its `my3d` flag, an enum over `&dyn Adaptor3dCurve` and
  `&dyn Adaptor2dCurve2d` (a `&GeomAdaptorCurve` or `&Geom2dAdaptorCurve` converts
  directly). It holds the static helpers `f3d`/`f2d` (`speed`), `order`
  (`gauss_order`) and the `D0`/`D2` lifts of 2D results to Z = 0.
  `CPnts_RealFunction` with its `void*` client data is one boxed closure
  (`RealFunction`), so `MyGaussFunction::init` and `MyRootFunction::init_function`
  take one argument for the native `(F, D)`. The objects that keep the curve
  (`AbscissaPoint<'a>`, `UniformDeflection<'a>`) borrow it for their lifetime.
- The GCPnts templates are generic over `TCurveTypes` (`GCPnts_TCurveTypes`),
  implemented for `dyn Adaptor3dCurve`, `dyn Adaptor2dCurve2d`, `GeomAdaptorCurve` and
  `Geom2dAdaptorCurve`, with the point, chord line and Bezier/B-spline types
  (`PoleCurve`). `GCPnts_DistFunction(2d)` and their `MV` views are one generic
  `TDistFunction`/`TDistFunctionMV` with the native names as aliases. The static
  helpers repeated in GCPnts_UniformDeflection.cxx and GCPnts_QuasiUniformDeflection.cxx
  (`PerformLinear`, `PerformCircular`, `GetDefType`, `PerformComposite`) exist once.
- Overloads take distinct names (`length`, `length_with_tolerance`,
  `length_between`, `length_between_with_tolerance`; `with_*` constructors and
  `initialize*`). Default arguments are explicit (the -1 tolerance of
  GCPnts_UniformAbscissa, `WithControl = true`, `GeomAbs_C1`); the five deflection
  criteria of GCPnts_TangentialDeflection are the `TangentialCriteria` struct
  (`TangentialCriteria::new` has the native defaults).
- Every native exception is a `Failure`, including the ones the adaptors raise inside
  the math callbacks (Gauss integrand, root function, chord distance): the callback
  keeps the failure and answers false, and the caller returns it right after the
  solver, where the native exception leaves it (`failure_capture`).
- `CPnts_UniformDeflection` keeps the native More/Next/Value iteration; `next` and
  `more` return `Result`.

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

The IntAna and IntAna2d requests intersect every pair of each package with random
inputs and with the configurations of each branch: parallel, identical, near-parallel
and axis-aligned 2D lines (each pivot), concentric, tangent (inside and outside),
secant, nested and nearly coincident circles, each gp conic against each implicit
conic, the polynomial solver on multiple, null and degenerate roots; conics in, along
and across quadrics and planes; coaxial, parallel, perpendicular, intersecting and skew
quadric axes, apexes on the other cone, sections beyond the ellipse and hyperbola
limits, torus pairs through and around the tube; and the cylinder and cone
IntAna_IntQuadQuad curves with their links, points, tangents, parameters and
out-of-range accessors.

The LProp, GeomLProp, GProp and Hermit requests (`tkgeombase_corpus_lprop.py`) build
every Geom and Geom2d curve kind (line, conics, rational Bezier and B-spline curves
with C0, C1 and C2 joints, trimmed and offset curves) and every surface kind
(elementary, rational B-spline with a collapsed edge, revolution, extrusion, offset),
and probe them at regular points, range ends, points outside the range, null first
derivatives (coincident poles), inflections, straight parts, sphere poles and cone
apices. The property classes run in four call orders (fresh object, stale-first
order, one object moved by SetParameter, construction without a curve), because they
cache lazily and two native reads are by value. GProp prints every property of point
sets (unit, weighted, 1D and 2D arrays, null and negative masses), point equations at
three tolerances, every elementary framework (including null ranges, where Jacobi
fails) and compositions at other locations; Hermit covers weight ratios up to 1e6 and
the tolerance failures.

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
maximum indices, or the raised exception.

The CPnts and GCPnts requests (`tkgeombase_corpus_gcpnts.py`) build 32 adapted curves:
every 2D and 3D kind (lines, full and partial conics in both senses, two-pole segments
rational or not, Bezier curves with a cusp, a zero-length Bezier curve, C2, C1, C0 and
periodic B-splines, trimmed curves, offsets of an ellipse, of a circle collapsing to a
point and of a cusp, whose undefined derivatives raise inside the math callbacks) and
circles of radius 1e-6 and 1e6. Each class runs every constructor and `Initialize`
overload with full, partial, reversed, extended and empty ranges, zero, negative,
tiny and overlong abscissas, too few points, deflections that raise, tolerances from
0 to 1e3 and -1, and objects reused across runs; each result prints `IsDone`, the
counts, every parameter and point (from the out-of-range index 0), or the raised
exception. Six more requests give circles a null deflection. They are 2210 requests.

The corpus has 8732 requests: 4449 gce and GC, 945 IntAna2d and IntAna, 833 LProp,
GeomLProp, GProp and Hermit, 1253 ExtremaPC2d and 1252 ExtremaPC.

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

IntAna2d:

- `Perform(gp_Parab2d, IntAna2d_Conic)` never returns early: it is always done, even
  when the solver fails (no point) or the elements are identical.
- `Perform(gp_Circ2d, IntAna2d_Conic)` prints "math_TrigonometricFunctionRoots ->
  NotDone" on std::cout when its solver fails; the port does not print (the probe
  discards std::cout around every native intersection).
- `Traitement_Points_Confondus` shifts the remaining points with the implicit
  `SetValue`, so they lose their second parameter (RealLast()).
- `MyDirectPolynomialRoots::IsDone()` returns a double natively; the port returns a
  bool. `Value(i)` reads its 16-slot array unchecked natively; the port raises
  Standard_OutOfRange outside it.

IntAna:

- `IntAna_IntQuadQuad::Perform(gp_Cylinder, ...)` never links its curves
  (InternalSetNextAndPrevious is only called by the cone Perform), so HasNextCurve and
  HasPreviousCurve are always false there; the link loop compares each curve only with
  the curves of lower index (`c2 < NbCurves && c2 != c1` stops at `c1`). No Perform
  finds contact points, and `Parameters()` only prints "not yet implemented" natively
  (the port leaves its outputs unchanged without printing).
- `IntAna_Curve`: the derivative of the Z² coefficient squares its CosSin term where
  the other derivatives double it; FindParameter sorts only the first four of its five
  candidates; V is clamped to ±100000; `SetCylinderQuadValues` keeps the semi-angle of
  a previous cone.
- `IntAna_Quadric::SetQuadric` appends the special points (cone apex, sphere poles)
  without clearing the earlier ones, and the plane and cylinder setters keep them.
- `IntAna_QuadQuadGeo::Perform(gp_Cone, gp_Torus)` starts its circle count from the
  previous NbSolutions() (0 for a new intersection); the lower slots are uninitialized
  natively and keep default values in the port. The Perform overloads keep the fields
  they do not assign (a reused intersection shows the previous values there).
- `IntAna_IntConicQuad`: the constructors other than the empty one leave the flags
  that their Perform skips uninitialized (for example IsParallel after two identical
  planes in the circle-plane case); the port starts from the empty state.
- The internal array overflows that are undefined natively (more than 12 IntQuadQuad
  curves, more than 4 cone-torus circles) raise Standard_OutOfRange in the port.
LProp and GeomLProp:

- `GeomLProp_CLPropsBase` and `GeomLProp_SLPropsBase` are class templates over the
  curve or surface type and an access policy. The port makes them generic structs:
  `CLPropsBase<V, C, A>` (V the derivative type, `gp_Vec` or `gp_Vec2d`) and
  `SLPropsBase<S, A>`, with the `DirectAccess`/`ToolAccess` policies as traits;
  `GeomLProp_CLProps`, `GeomLProp_CLProps2d`, `LProp_CLProps3d` and
  `GeomLProp_SLProps` are type aliases. The `LProp_CurveUtils` and
  `LProp_SurfaceUtils` helpers take the property object and read the fields it
  upgraded after the call, as the native references do; a null curve or surface
  handle gives Standard_NullObject where the native code dereferences it.
- `LProp_SurfaceUtils::RequireCurvature` and `IsUmbilic` take the curvature fields by
  value, read before the curvature is computed: the first `MaxCurvature()`,
  `MinCurvature()`, `MeanCurvature()`, `GaussianCurvature()` or `IsUmbilic()` of a
  point answers with the values of the previous point (0 on a fresh object).
- `LProp_SurfaceUtils::TangentU`/`TangentV` take the significant derivative order by
  value, read before `IsTangentUDefined()` sets it: on a fresh object the first
  tangent orients the second derivative along the chord (Standard_ConstructionError
  where it is null, a plane); the next calls use the first derivative.
- `FindSurfTangentOrder(D1U(), D2U(), ...)` evaluates its arguments right to left
  (GCC): D2U() first; the port calls them in that order.
- `IsNormalDefined()` uses the current D1U and D1V without evaluating them (null with
  N = 0).
- `CLProps::Curvature()` returns RealLast() when the first derivative is null without
  storing it; `CentreOfCurvature()` then divides by the stored curvature of an earlier
  call (0 initially).
- `LProp_CurAndInf` drops a parameter equal to the last one stored (and NaN).
- `GeomLProp_CurAndInf2d::Perform` reports the status of the inflection search only.
  The private numeric search keeps the curve's evaluation exception in its root
  function and returns it right after the root finder, where the native exception
  leaves it.

GProp:

- `math_VectorBase::Multiply(Passage, V)` with V aliasing the result writes each
  component before the next row reads it; the elementary frameworks keep this
  sequential product.
- `GProp_CelGProps::Perform(gp_Lin)` computes Izz with the linear and constant terms of
  Ixx; the cone surface and solid use the ICn2 formula for ISn2; the solid torus reads
  `Teta2` as `Alpha2` in its sines and cosines; `GProp_CelGProps::Perform(gp_Circ)`
  keeps the Z of the circle location as the centre's Z.
- `GProp_GProps::MatrixOfInertia()` applies the Huygens operator to the centre of mass
  *relative* to the system location.
- `GProp_PGProps::Barycentre` on an empty array raises Standard_OutOfRange (the native
  checked element read); `GProp_PEquation::PrincipalAxis`/`Extent` check the index.

Hermit:

- `PolyTest` locates its parameters on the knots of the original curve (not the ones
  mapped onto [0, 1]) with the flat-knot `LocateParameter`, reusing the LocateU
  indices of Ux and Uy as hints; the 3D and 2D copies of the native file share one
  implementation over `HermitSpline`.
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

CPnts and GCPnts:

- `CPnts_AbscissaPoint(C, Abscissa, U0, Resolution)` initializes a 3D curve with the
  tolerance `Resolution` ("rbv's modification") and a 2D curve without; `order` reads
  `Degree()` and `NbPoles()` of a 3D adaptor but of `Bezier()` and `BSpline()` in 2D.
  The root search runs on `[U1, U2]` widened by its width on each side and the
  commented-out validity test stays out, so a parameter outside the curve bounds is
  done; a curve shorter than Precision::Confusion() gives `U0`, done.
- `CPnts_MyGaussFunction()` keeps a null function pointer that `Value` would call; the
  port fails with Standard_NullObject. `CPnts_UniformDeflection::Value()`/`Point()` past
  the three stored points read outside the native arrays; the port raises
  Standard_OutOfRange.
- `GCPnts_DistFunction(const GCPnts_DistFunction&)` and its 2D twin are declared but
  never defined in TKGeomBase (a call does not link); the port's `Clone` is the
  member-wise copy they declare.
- GCPnts_AbscissaPoint classifies on the CN intervals, GCPnts_UniformAbscissa on the C1
  intervals; `GetParameterLengthRatio` evaluates `DN(0.0, 1)` of a Bezier or B-spline
  segment at the parameter 0, not at FirstParameter. `Compute` pushes past the last
  interval forward by 0.1 on `[U0, U0 + 0.2]` whatever the sign of the abscissa, while
  `AdvCompute` pushes in the walk direction and clamps to the bounds of a non-periodic
  curve; `AdvCompute` has no zero-abscissa shortcut and compares the span before the
  first interval with its tolerance, the other spans with PConfusion (Confusion in
  `Compute`). `GCPnts_AbscissaPoint(C, Abscissa, U0)` raises Standard_ConstructionError
  for a curve shorter than Confusion, `(Tol, C, Abscissa, U0)` starts at `U0` instead.
- `GCPnts_UniformAbscissa` reuses its parameter array when it holds enough values, so
  `Parameter(i)` past NbPoints returns the values of an earlier run; one failed
  abscissa search leaves the whole result not done even when later searches succeed; a
  negative abscissa on a length-parametrized curve ends at once; the NbPoints variant
  sets `Abscissa()` only for a curve longer than Confusion. `(int)aSizeR` of a NaN size
  is INT_MIN on x86-64 (undefined behaviour natively), as in the port.
- `GCPnts_QuasiUniformAbscissa` delegates curves that are neither Bezier nor B-spline to
  GCPnts_UniformAbscissa and reads its `NbPoints()` unconditionally, so a failed
  distribution raises StdFail_NotDone; on the polygon path a raise keeps the previous
  `IsDone`.
- The circular deflection steps accumulate the parameter (the last one is not set to
  U2), and a null deflection makes the native `(int)` of an infinite count INT_MIN, so
  no point is stored and the result is done. The composite deflection paths walk the C2
  intervals in both classes, though GCPnts_QuasiUniformDeflection classifies on C1.
  `GCPnts_UniformDeflection::Value()` and `GCPnts_QuasiUniformDeflection::Value()` raise
  StdFail_NotDone with the messages of GCPnts_UniformAbscissa and
  GCPnts_QuasiUniformAbscissa.
- `QuasiFleche` returns early at 2000 calls, or after 100 calls with fewer than two
  points, without decrementing its counter, so every later call of the subdivision
  returns at once; the C1 path takes the end tangent at `U2 - 10 Epsilon(U2)` but keeps
  U2 as end parameter; a Bezier or B-spline curve whose parametric resolution is below
  `Epsilon(max(|First|, |Last|))` is not done.
- GCPnts_TangentialDeflection: the line test clamps the CN interval bounds in place
  and the march reads the clamped bounds; a span grown once and still below the
  coefficient 0.55 becomes the previous point without being stored; `EvaluateDu` never
  sets its flag back, so after one success the deflection-driven steps are no longer
  extrapolated; the linear distribution steps by `(Last - First) / MinimumOfPoints`, so
  its last step is twice as long.
- Every native comparison keeps its form where a NaN operand changes the branch
  (`!(a > b)` for the native `if (a > b) ... else`), and `std::min`/`std::max` keep
  their argument order.
