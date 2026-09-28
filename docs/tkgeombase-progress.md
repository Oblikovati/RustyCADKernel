# TKGeomBase port status

**In progress; toolkit acceptance is not complete.** `crates/tkgeombase`
(`rustycad-tkgeombase`) is the port of the ModelingData toolkit TKGeomBase. It
depends on the TKernel, TKMath, TKG2d and TKG3d ports, as the native toolkit
does. Packages are added one module at a time. The `gce`, `GC`, `GCE2d`, `IntAna`,
`IntAna2d`, `LProp`, `GeomLProp`, `GProp`, `Hermit`, `ExtremaPC2d`, `ExtremaPC`,
`GeomTools`, `FEmTool`, `AppParCurves`, `GeomBndLib`, `BndLib`, `CPnts` and `GCPnts`
packages are ported in full, and `tools/audit_api_rows.py` reports 0 missing and 0 stale api rows for each of
them.

The Geom and Geom2d objects that TKGeomBase builds follow the class mapping of
[geom-design.md](geom-design.md).

The toolkit test inventory maps these complete suites, with matching native and Rust
passes (685 declarations):

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
| GeomBndLib_Curve_Test.cxx | geom_bnd_lib_curve | 25 |
| GeomBndLib_Curve2d_Test.cxx | geom_bnd_lib_curve2d | 21 |
| GeomBndLib_OffsetCurve_Test.cxx | geom_bnd_lib_offset_curve | 14 |
| GeomBndLib_OffsetCurve2d_Test.cxx | geom_bnd_lib_offset_curve2d | 10 |
| GeomBndLib_Surface_Test.cxx | geom_bnd_lib_surface | 24 |
| GeomBndLib_OffsetSurface_Test.cxx | geom_bnd_lib_offset_surface | 15 |
| GeomBndLib_SurfaceOfExtrusion_Test.cxx | geom_bnd_lib_surface_of_extrusion | 11 |
| GeomBndLib_SurfaceOfRevolution_Test.cxx | geom_bnd_lib_surface_of_revolution | 15 |
| BndLib_Test.cxx | bnd_lib | 63 |
| GCPnts_AbscissaPoint_Test.cxx | gcpnts_abscissa_point | 5 |
| GCPnts_QuasiUniformDeflection_Test.cxx | gcpnts_quasi_uniform_deflection | 1 |
| AppCont_ContMatrices_Test.cxx | app_cont_cont_matrices | 9 |

`bnd_lib_behavior.rs` covers the GeomBndLib and BndLib quirks below.
`gcpnts_behavior.rs` covers the CPnts and GCPnts quirks below.
`local_props_behavior.rs` covers the LProp, GeomLProp, GProp and Hermit quirks
below. `gce_behavior.rs` and `gc_behavior.rs` cover the status enumeration, the
StdFail_NotDone results, the conversion operators, the GCE2d aliases and the native
quirks below. GeomTools has no upstream GoogleTest suite; `geom_tools_behavior/`
covers its layouts, reader contract, sets, handler and quirks with named fakes for the
console, the text streams, the progress indicator and a custom handler.

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

GeomTools (`geom_tools`) keeps the class layout: the statics of `GeomTools` are free
functions (`GeomTools::Dump(C, OS)` for a `Geom_Curve` is `geom_tools::dump_curve`,
`GeomTools::Read(C, IS)` returns the curve), and `GeomTools_CurveSet`,
`GeomTools_Curve2dSet` and `GeomTools_SurfaceSet` are `CurveSet`, `Curve2dSet` and
`SurfaceSet`, which share one indexed table (`geometry_table.rs`); the three native files
differ only in labels, element type and null handling. `Standard_OStream` and
`Standard_IStream` are the TKernel model of the libstdc++ iostream
(`rustycad_tkernel::storage::IoStream`), so `operator<<(double)` is `printf("%.*g")` at
the stream precision, `std::setw` pads (the model gained width handling for this), and
`operator>>` follows the libstdc++ sentry, clamping and failbit rules. The printers
return `Result`: a null handle, which the native code dereferences, is
Standard_NullObject. The text native code writes to `std::cout` (the "UNKNOWN ... TYPE"
line of a compact print, "Not a ... table", the debug dump) goes to an injected
`ConsoleOutput`: `CurveSet::with_console`, `UndefinedTypeHandlerBase::with_console`, and
the console argument of `geom_tools_dump`; the defaults write to standard output.
`GeomTools_UndefinedTypeHandler` is the trait `UndefinedTypeHandler` (its virtual members
are provided methods with the native behaviour) and the struct
`UndefinedTypeHandlerBase` is `new GeomTools_UndefinedTypeHandler`; the read members
return the handle instead of filling an output argument and returning the stream. The
active handler is process wide, as the native static is. `GeomTools_Dump` of
GeomTools_Debug.cxx is `geom_tools_dump` for a `Standard_Transient` handle; its `void*`
overload exists only for debuggers.
FEmTool (`fem_tool`) keeps the native class split. The transient classes
(`FEmTool_Curve`, the three linear criteria and `FEmTool_ProfileMatrix`) hold their
mutable state in a `SharedState`, as the Geom classes do, so every `Handle` to one
object sees the same state and the non-const members take `&self`. The abstract
`FEmTool_ElementaryCriterion` and `FEmTool_SparseMatrix` are the traits
`ElementaryCriterion` (its protected fields are `CriterionBase`) and `SparseMatrix`,
registered with `impl_transient_interface!`; `ChangeValue` returns an `EntryRef` that
dereferences to the stored `f64` and keeps the matrix locked while it lives, and
`Knots()` returns the write guard of the knot array. `FEmTool_LinearFlexion`,
`FEmTool_LinearJerk` and `FEmTool_LinearTension` are one generic `LinearCriterion<K>`:
the kinds `Flexion`, `Jerk` and `Tension` name the derivative order, the scale power
and the three places where the native files differ (the negative-J clamp, the
DependenceTable indexes and the Hessian origin). `FEmTool_Assembly` is a plain struct;
the rows of G are vectors of coefficient segments (the native lists of handles, never
shared outside the object). Output references become return values
(`Curve::length`, `Curve::reduce_degree`), and the unchecked native accesses
(FEmTool_Curve.cxx and FEmTool_ProfileMatrix.cxx compile the range checks out) become
Standard_OutOfRange where they would leave the storage (undefined natively).

AppParCurves (`app_par_curves`) ports the package functions (`bernstein_matrix`,
`bernstein`, `second_derivative_bernstein`, `spline_function`), the containers and the
six `.gxx` generics. `AppParCurves_MultiPoint` and `AppParCurves_MultiCurve` hold their
points and poles in shared arrays (`Handle<HArray1<...>>`), so a copy shares them, as
the native copies share their handles (the least-squares fit, its Bezier view and the
function's curve are one pole array, which the Uzawa correction writes through). The
virtual members of `AppParCurves_MultiCurve` are the `MultiCurveEval` trait, implemented
by `MultiCurve` (Bezier evaluation) and `MultiBSpCurve`, which reaches the non-virtual
members of its base through `Deref`; the slicing copy `(AppParCurves_MultiCurve)` is
`MultiBSpCurve::as_multi_curve`. The `MultiLine`/`ToolLine` template parameters of the
generics are the `LineTool` trait (implemented by the line: `ToolLine::NbP3d(SSP)` is
`ssp.nb_p3d()`); `LeastSquare` and `ResolConstraint` read the line in their
constructors only and are plain structs with generic constructors, `Function<L>` and
`BSpFunction<L>` keep a copy of the line. The `Squares`, `ResolCons` and
`*_BFGS` parameters are the ported classes themselves; the BFGS subclass the AppDef,
GeomInt and BRepApprox instantiations generate is `GradientBfgs` (math_BFGS with the
fit-tolerance stopping test, over the `FitFunction` trait). The optimizer callbacks
cannot return an exception: a function keeps the first one and the gradients return it
after BFGS, where the native exception would unwind. `AppParCurves_ResolConstraint::
Error()` is declared natively but never defined (a link error if called) and has no
counterpart. The protected members that the generics call from outside
(`NbConstraints`, `NbColumns`, `Perform`) are public; `BSpGradient` takes its inputs in
a `BSpGradientInput`.
GeomBndLib (`geom_bnd_lib`) has one struct per native evaluator class (`Line`,
`Circle`, `Ellipse`, `Hyperbola`, `Parabola`, `BezierCurve`, `BSplineCurve`,
`OffsetCurve`, `OtherCurve` and their `*2d` twins; `Plane`, `Cylinder`, `Cone`,
`Sphere`, `Torus`, `BezierSurface`, `BSplineSurface`, `OffsetSurface`,
`SurfaceOfExtrusion`, `SurfaceOfRevolution`, `OtherSurface`). The `std::variant`
dispatchers `GeomBndLib_Curve`, `GeomBndLib_Curve2d` and `GeomBndLib_Surface` are
structs over a private evaluator enum (holding a borrowed or an owned adaptor), with
`from_adaptor` and `from_curve`/`from_surface` constructors; they match the
exact dynamic type of the handle, as the native `typeid` tests do. The static helpers
of `GeomBndLib_Line`, `_Circle`, ... (`LinBox`, `CircBox`, ...) are associated
functions, and the internal headers are private modules (`infinite`, `sampling`,
`optimization`, `spline`, `ranges`, `patch`, `swept`...). A null geometry handle gives
Standard_NullObject where the native code dereferences it. The surface patch bounds
are one `[UMin, UMax, VMin, VMax]` array. BndLib (`bnd_lib`) keeps the free functions of
the native `BndLib` class (`add_circ_arc`, `add_torus_patch`...) and the classes
`BndLib_Add3dCurve`, `BndLib_Add2dCurve` and `BndLib_AddSurface` as unit structs with
associated functions; it calls GeomBndLib as natively (and GeomBndLib_Torus calls
BndLib). The native `GeomBndLib_SplineHelpers` templates `BezierCurveBox`,
`BSplineCurveBox` and the pole-range `ReduceSplineBox` overload are declared but never
instantiated in TKGeomBase; they are not ported, and the audit rows say so.

AppCont (`app_cont`) ports `AppCont_Function`, `AppCont_LeastSquare` and the
`AppCont_ContMatrices` namespace. The abstract `AppCont_Function` is the `Function`
trait; its protected `myNbPnt`/`myNbPnt2d` live in a `FunctionBase` (both -1 until the
implementation sets them) that the trait reaches through `base()`, which gives the
non-virtual count getters. `LeastSquare::new` takes `&dyn Function`, `Error` returns
`(F, MaxE3d, MaxE2d)` and `Value` lends the stored MultiCurve. The five
`AppCont_ContMatrices_*.pxx` tables (126,095 doubles) are generated, not hand-copied:
`tools/generate_appcont_tables.py` copies every literal verbatim into include!-able
fragments under `app_cont/cont_tables/` (rustc and g++ round the same decimal strings,
and the three IBP entries written `a - b` stay constant expressions), and its
`--check` mode runs before the numeric comparison. `app_cont_behavior.rs` covers the
function base, every resolution path, the singular and periodic ends and the
exceptions with named fake functions.

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

The GeomTools requests (`tkgeombase_corpus_geomtools.py`, probes
`tkgeombase_probe_geomtools*.cxx`) print every text as one hexadecimal token, so the
comparison is byte exact. They build every Geom curve, Geom2d curve and Geom surface class
(conics on tilted frames, direct and indirect frames, rational, periodic and clamped
splines, trimmed and offset curves on every basis, nested trimmed and offset surfaces)
with numbers that exercise the `%g` layouts (signed zeros, 1e-5, 1e21, 123456.5, 1e300,
1e-300, repeating fractions), and compare `Dump` and `Write` at precisions 6 and 17, then
the geometry read back from the precision-17 text by its class name, its compact text at
17 digits (which identifies every stored double) and a point on it. The read requests feed
native and malformed text to the three readers: unknown, negative, overflowing and
non-numeric type codes, null and non-unit directions, parallel axes, invalid radii,
weights, degrees and knots, booleans other than 0 and 1, `nan`, `inf`, hexadecimal,
subnormal and overflowing numbers, words longer than the GetReal buffer, and bases that
fail inside trimmed and offset records; each prints the result, the stream state and the
unread rest. The set requests cover Add/Index (duplicates, null handles), Dump, Write
into a precision-3 stream, two appending Reads, Clear, wrong and overlong tags, tables
with failing records, a progress indicator that breaks after 0 to 10 steps, the debug
dump and a custom handler that stores the eval helix, sine wave and ellipsoid.

The FEmTool requests (`tkgeombase_corpus_femtool.py`) build each object and run a
list of commands on it, printing every result or exception, so the cached state is
compared along with the values: the Hermite-Jacobi reference integrals of every
derivative order at interior points, the ends and outside [-1, 1] (short output
vectors, bad orders and shapes); the three criteria for every constraint order and
WorkDegree 0 to 15 in a fixed order (the process-wide integral cache: a cached order
past WorkDegree 14 and a new one), coefficient arrays with other lower bounds, fewer
or more rows than the basis, a null or reversed element, Hessians and gradients in
larger, smaller and shifted matrices; profile matrices from diagonal to full and
block-diagonal profiles, positive definite, indefinite and huge pivots, solves before
and after the factorization, entries outside the profile, the OutM/OutS text and the
unimplemented iterative members; curves of every dimension, constraint order and up
to four elements with coefficient arrays on other bounds, evaluation on the knots,
between and outside them, after a knot change under the span cache, lengths across
and outside the elements, degree changes and reductions at three tolerances; and
assemblies of chained elements in one to three dimensions with constraint segments
added in every order (each merge case of AddConstraint), nullified, reset and
refused systems, a table indexed from 0 and a full dependence table.

The AppParCurves requests (`tkgeombase_corpus_appparcurves.py`) run the Bernstein and
B-spline basis functions inside, at and outside their ranges, down to 2 poles and with
interior multiplicities up to C0; every MultiPoint, MultiCurve and MultiBSpCurve
constructor and member with valid and invalid indexes, the transformations, the Dump
text and the sharing of copies; and the generics through the native AppDef
instantiations (the Rust probe implements `LineTool` with the AppDef_MultiLine and
AppDef_MyLineTool semantics): least-squares fits of 3D, 2D and mixed lines (smooth,
noisy, collinear, with a repeated point) for every pair of extremity constraints each
Perform overload accepts, too few poles, tangents against the chord and missing
tangents or curvatures, Bezier and B-spline bases; the Uzawa correction with inner
pass, tangency and curvature points and its derivative; the Bezier and B-spline
parameter functions (value, gradient, curves, errors); and both optimizers at loose and
tight tolerances with 0 to 200 BFGS iterations. The paths that are undefined natively
are left out: curvature constraints in `Perform(Parameters)` (unassigned pole rows), a
fit read before any Perform, the constrained B-spline function, more Householder
unknowns than points and more Uzawa constraints than pole coordinates (both read past
their arrays natively).

The GeomBndLib and BndLib requests (`tkgeombase_corpus_bndlib.py`, seed 20260928) box
every 3D and 2D curve kind and every surface kind through each evaluator member
(`Box`, `Box` on a range, `BoxOptimal`, the BndLib `Add*` functions and the adaptor
classes), and print the six (or four) limits, the gap, the open flags and the finite
part. Each evaluator is reached through the dispatchers from a Geom handle, a
GeomAdaptor over the natural range and one over a sub-range, and through the
`BndLib_Add*` classes on a box that already holds [-1, 1]^n. The ranges cover the
natural bounds, sub-ranges, reversed and degenerate ranges, ranges beyond the bounds,
infinite ends (±1e101, and exactly ±1e100 on the Precision::Infinite() boundary) and
the doubly infinite ones that raise Standard_Failure. The curves add axis-aligned
lines, periodic and trimmed B-splines and offsets of lines, circles (in plane and
collapsing), ellipses and splines; the surfaces add trimmed spheres, degenerate and
offset tori, offset planes and B-splines, extrusions and revolutions. The static gp
overloads get axis-aligned and tilted frames, arcs across the period seam, spans
beyond 2 Pi, cylinder and cone V ends in every infinite combination, sphere and torus
patches with negative, large and reversed V, and self-intersecting tori (R < r). The
internal helpers the upstream tests call are probed directly. The legacy `math_PSO` path of
`GeomBndLib_OtherCurve2d::BoxOptimal` is left out for reversed ranges: native reads an
uninitialized value there.

The AppCont requests (`tkgeombase_corpus_appcont.py`, 3587 requests) read every entry
of the five tables directly from the native static arrays, run the four square
providers over classe -1 to 28 and VBernstein over every classe x nbpoints pair (into
matrices with shifted bounds and too small, where the partial writes are compared), and
fit 3D, 2D and mixed analytic functions (polynomial, trigonometric, constant) over
forward, reversed and tiny ranges with every pair of end constraints (CurvaturePoint
included), degrees 1 to 25, 1 to 40 Gauss points, failing derivatives, singular ends
around the 0.1 threshold and periodic coordinates that wrap (the first-sample, sample
and end-pole fixes). Left out: VBernstein at classe 1 (degree 0), more than 61 Gauss
points and periodic fits with fewer than 3 samples (undefined natively), and empty
ranges (degree < 0, no Gauss point, no coordinate), where the native `math_Matrix`
builds empty or negative-size tables on its stack buffer (degree -2 crashes) while the
TKMath `Matrix` refuses them with Standard_RangeError.

The corpus has 29711 requests: 4449 gce and GC, 945 IntAna2d and IntAna, 833 LProp,
GeomLProp, GProp and Hermit, 1253 ExtremaPC2d, 1252 ExtremaPC, 296 GeomTools, 1230
FEmTool, 2045 AppParCurves, 15198 GeomBndLib and BndLib, and 2210 CPnts and GCPnts.

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

GeomTools:

- `GeomTools::Write` and `GeomTools::Read` for one geometry use the stream precision (6
  by default, so the text does not round-trip); only the set `Write` sets precision 17,
  and it restores the caller's precision afterwards.
- `GeomTools_CurveSet::Add` and `Index` refuse a null handle (0), but
  `GeomTools_Curve2dSet` and `GeomTools_SurfaceSet` store it like any element and
  `Index(null)` then returns its index. `Dump` and `Write` dereference a stored null
  handle; the port returns Standard_NullObject there.
- `Read` appends to the set (the header says it clears it first). A record that fails to
  read is stored as a null handle, once, since every null handle is the same key.
- A table with another tag writes "Not a Curve table", "Not a Curve2d table" or "Not a
  surface table" to `std::cout` and reads nothing more; the tag is extracted into a
  `char[255]`, so at most 254 characters are compared.
- The dumps keep their layout slips: the 2D B-spline count line runs into the pole list
  (` KnotsPoles :`), the 3D B-spline dump puts two spaces before each knot and the 2D one
  one, the offset curve has no newline after its tag (`OffsetCurveOffset : `), the torus
  radii have no comma, the surface table dump says `surfaces` in lower case, and only the
  surface dumps report `uclosed`/`vclosed`.
- For a class GeomTools does not know, the base handler writes the "UNKNOWN ... TYPE"
  line into a dump, but to `std::cout` for the compact form, whose record is then missing
  from the stream.
- Every failure inside a record read (a native Standard_Failure) gives a null result with
  the stream where the failure happened: a null direction raises right after its three
  words; `gp_Ax2` of a curve conic raises only after the radii are read, while `gp_Ax3`
  of an elementary surface raises before its radius; the cone record writes the radius
  before the semi-angle. The offset surface is read with `isNotCheckC0` true, the offset
  curve with false.
- A trimmed or offset curve read on a basis that failed dereferences the null basis
  natively; `OCC_CATCH_SIGNALS` turns that into a caught failure when the OSD signal
  handlers are installed (DRAW and the oracle install them with `OSD::SetSignal`), which
  gives a null result. The port refuses the null basis with Standard_ConstructionError,
  with the same null result.
- `GetReal` returns 0 without reading at end of file; it extracts at most 30 characters
  (the stream width is set to 31 and then restored), leaving the tail of a longer word
  for the next read, and converts the word with `Strtod`, so `1.5abc`, `inf`, `nan` and
  hexadecimal floats are accepted and garbage reads as 0.
- A boolean other than 0 or 1 reads as true and fails the stream; every later
  extraction of the record then fails and reads 0.
- The type code of a record is extracted into an uninitialised `int`. When the stream
  has already failed or reached its end (a table that announces more records than it
  holds), the native value is indeterminate; the port reads 0, an unknown code, which
  gives a null record. A multiplicity read from a failed stream is 0 in the port. The
  corpus leaves these inputs out.

FEmTool:

- The linear criteria compute their reference integrals once per process and per
  class (function-local statics keyed by the constraint order, for WorkDegree 14):
  the `WorkDegree > 14` check (Standard_ConstructionError) only runs when the order
  changes, so a later criterion of a cached order and WorkDegree 15 reads past the
  120 integrals (Standard_OutOfRange). The port keeps one static cache per class.
- `FEmTool_LinearFlexion` clamps a negative J to 0 after the whole sum,
  `FEmTool_LinearJerk` after every row term, `FEmTool_LinearTension` never.
  `FEmTool_LinearTension::DependenceTable` sets `(i, i)` for `i = 1..=RowLength`, so
  coefficient columns not starting at 1 raise Standard_OutOfRange.
  `FEmTool_LinearFlexion::Hessian` writes from index 0 whatever the bounds of H (a
  matrix from 1 raises Standard_OutOfRange); the other two write from the lower
  bounds of H, mirroring through `H(j0 + j, i0 + i)`, so with different row and column
  origins the write order decides the result.
- `FEmTool_ElementaryCriterion::Value()` and `Gradient()` dereference a null
  coefficient handle natively; the port raises Standard_NullObject.
  `FEmTool_ElementsOfRefMatrix` as a `math_FunctionSet` reports a short F (its
  Standard_OutOfRange) as a failed evaluation.
- `FEmTool_ProfileMatrix::ChangeValue` has no range check (compiled out): an entry
  outside the profile addresses a coefficient of another row. The port keeps that
  aliasing and raises Standard_OutOfRange only for an address outside the storage.
  The coefficients start uninitialized natively (0 in the port). `OutM`/`OutS` pad a
  row with `0 ` up to its first stored column.
- `FEmTool_Curve::GetElement` rescales the columns `LowerCol..=Dimension` instead of
  `LowerCol..LowerCol + Dimension`; `D1` and `D2` scale every entry of the output
  array, not only the first Dimension ones. The knots and coefficients start
  uninitialized natively (0 in the port), and `Knots()` hands out a writable array
  from a const member; changing a knot invalidates the span cache on the next
  evaluation, as natively.
- `FEmTool_Assembly::AddConstraint` scans the segments of a row keeping the *last*
  segment starting before the new range and the last one ending after it, so a range
  can be prepended before an overlapping segment; the merge cases replace the first,
  the last, s1 or s2 segment and the three-way merge removes the segment after s1,
  whichever it is. Overlapping segments then write the same variable of `gi` in
  `Solve`, the later one winning. A row with no segment (a skipped constraint index)
  leaves its Schur FirstIndexes uninitialized natively (0 in the port), and a scan
  that finds no segment dereferences a null handle (Standard_NullObject in the
  port). `Solve` stores IsSolved before building the Schur matrix, so after a
  Standard_OutOfRange there (a table not indexed from 1) `Solution` dereferences a
  null matrix natively.

AppParCurves:

- `AppParCurves_MultiPoint::Dimension(0)` is 3 (the check is `Index < 0`);
  `AppParCurves_MultiCurve::Pole(CuIndex, Nieme)` raises only when both indexes are
  `<= 0` (then `Point(0)` raises). `MultiCurve::Degree()` dereferences the pole array
  (Standard_NullObject in the port for a curve without one), `NbPoles()` answers 0.
  `AppParCurves_MultiBSpCurve::SetNbPoles` keeps the degree, which only
  `SetMultiplicities` recomputes; `Knots()` before `SetKnots` is a null dereference
  natively (Standard_NullObject). The package functions compile their range checks out;
  the port raises Standard_OutOfRange where they would leave their arrays (fewer than
  2 poles, DDA not starting at 1).
- `AppParCurves_LeastSquare::Perform(Parameters)` does not return after the pass-point
  solve: it falls through to the full system with the tangency unknowns, which
  recomputes the poles (the first solve only decides `done`). `Init` fills the last
  fixed pole when the *first* constraint is a curvature point (`LastConstraint == Pass
  || LastConstraint == Tangency || FirstConstraint == Curvature`), and `Distance()`
  takes the square roots in place once per Perform: after `ErrorGradient`, it returns
  the squared distances. The matrices and lambdas start uninitialized natively (0 in
  the port).
- `FirstConstraint(TheConstraints, Point)` returns the constraint of the last couple
  when none is on `Point` (the loop variable keeps it).
- `AppParCurves_ResolConstraint` reads the 2D pass points at `tabP2d(i - nb3d)` (the
  pass-point index, not the curve), so mixed lines raise Standard_OutOfRange; in the
  tangency-ratio rows the 3D case `T1 ~ T1y` leaves `T1` at its previous value; `tabV` is shared by
  the loops, so a point without tangent keeps the previous tangent.
  `ConstraintDerivative` reads the 2D tangents at the curve index, does not advance the
  column offset for 2D curves, and writes its tangency-ratio rows into the constraint
  matrix (Cont), which `ConstraintMatrix()` and the function's gradient then read.
- `AppParCurves_Function` reads its 2D line points at the curve index (mixed lines
  with inner constraints raise Standard_OutOfRange); in the gradient DKPTC keeps the
  last curve's sum only, DPTCO uses `DPTCO1(Pi, j + Inc)` for every k, the correction
  term `Scal` keeps the last pole only (`=` instead of `+=`) and uses the X error for
  Z, and the constraint of a point without couple is the previous point's. `Error()`
  reads the squared distances of the constrained branch, so the Gradient's errors are
  0 for a line without inner constraints.
- `AppParCurves_BSpFunction` has an empty constrained branch: F and the errors are left
  unassigned there. `AppParCurves_Gradient` selects the 3D-and-2D line overload
  whenever there are 2D points (it tests `nbP != 0` with the total count).


GeomBndLib and BndLib:

- Two equal infinite range ends (both -inf or both +inf) raise Standard_Failure
  ("bad parameter") in the line, parabola, hyperbola, cylinder and cone boxes. A doubly
  infinite line adds its origin point and opens both sides of each axis it is not
  perpendicular to.
- The BndLib 3D arcs (`Add(gp_Circ, U1, U2, ...)` and `Add(gp_Elips, U1, U2, ...)`)
  read the accumulated box back with the gap removed before they add the extrema, so
  the content of the box given in changes the result.
- `BndLib::Add(gp_Hypr2d, ...)` adds only the two ends and the tolerance (the parabola
  rule), without the coordinate extrema that GeomBndLib_Hyperbola2d adds: an arc whose
  extremum lies inside is under-bounded.
- `GeomBndLib_Circle2d` and `GeomBndLib_Ellipse2d` box the arc `[U1, U2]` as given,
  without the `ElCLib::AdjustPeriodic` of their 3D twins and of BndLib.
- `GeomBndLib_BSplineCurve` (and 2D) on a sub-range clips the segmented box to the hull
  of the poles of the original curve, not of the segment.
- `BndLib::Add(gp_Torus, U1, U2, V1, V2, ...)` computes its section indices with
  `static_cast<int>` of `floor(V / (Pi/4))`; a V beyond the int range gives INT_MIN
  (x86-64 cvttsd2si), an empty section range, and a return before the final Enlarge.
- An infinite plane that is not aligned with a coordinate plane is `SetWhole` without
  Enlarge (the gap stays 0).
- The generic `GeomBndLib_OffsetSurface` path enlarges the basis box by
  `max(|offset|, tol)`, not their sum.
- `GeomBndLib_SurfaceOfExtrusion` and `GeomBndLib_SurfaceOfRevolution` catch every
  Standard_Failure of the sweep and fall back to GeomBndLib_OtherSurface.
- The handle constructors of the dispatchers match the exact dynamic type
  (`Geom_TrimmedCurve` is unwrapped to its basis evaluator on the trimmed range, so a
  trimmed offset curve uses GeomBndLib_OffsetCurve). The adaptor constructor of
  `GeomBndLib_Surface` builds the evaluator from the adaptor's type and down-casts the
  surface handle: when the kinds disagree the native evaluator holds a null handle and
  crashes on use; the port raises Standard_NullObject there, as for every null handle.
- The sample counts wrap like the native int arithmetic (`RealToInt(du * N) + 1`, at
  least 5 samples).
- `GeomBndLib_OtherCurve2d::BoxOptimal` keeps the legacy `math_BrentMinimum` and
  `math_PSO` search, while the other optimal boxes use MathOpt (`brent`, `pso`,
  `powell`). With a reversed range the legacy PSO reads an uninitialized value natively
  (undefined behaviour); the port starts from 0 and the corpus leaves that case out.
- The optimizer objectives keep the first evaluation failure (which the native code
  raises through the optimizer) and return it right after the optimizer.
- GeomBndLib_Torus calls BndLib, and BndLib calls GeomBndLib, as natively.

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

AppCont:

- `AppCont_LeastSquare::IsDone()` is true only when free poles were solved for after
  a constraint: the unconstrained fit (poles computed directly with InvMMatrix) and the
  fits without free pole (degree 1 with two pass points, degree 3 with two tangency
  points) report not done although their poles are set.
- Only the first two coordinates of the last pole are moved back into the period of
  the last sample (the loop runs `aDimIdx <= 2`, where the first pole's runs over every
  coordinate), whatever the point counts.
- A tangency constraint whose D1 fails becomes a pass constraint; a CurvaturePoint end
  is neither fixed nor eliminated, so the fit solves the full `M^T M` system (inverted,
  not read from the tables). The IBP and IBT tables cover classe up to 26, but the
  mass matrices stop at 24, so a degree above 23 always raises Standard_DimensionError.
- `FixSingleBorderPoint` ignores the result of `Value`, and the end pole takes the
  limit from inside only when it is more than 0.1 from the end value (per point).
- `VBernstein` writes at `M(1, 1)` whatever the bounds of the matrix, unlike the other
  providers, which start at its lower corner. At classe 1 it reads before the table
  (undefined natively); the port raises Standard_OutOfRange. The port also raises
  Standard_OutOfRange where the fit would read past its samples (a periodic function
  with fewer than 3 Gauss points) and Standard_DimensionError, before sampling, for more
  than the 61 tabulated Gauss orders (natively read past the Gauss tables, then
  rejected by VBernstein). The `int` offsets of the providers overflow natively past
  classe 1861; the port wraps.
- `AppCont_Function::PeriodInformation` defaults to not periodic with period 0.
  `VBParam` is computed natively and never read; the port does not keep it.
