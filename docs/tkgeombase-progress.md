# TKGeomBase port status

**In progress; toolkit acceptance is not complete.** `crates/tkgeombase`
(`rustycad-tkgeombase`) is the port of the ModelingData toolkit TKGeomBase. It
depends on the TKernel, TKMath, TKG2d and TKG3d ports, as the native toolkit
does. Packages are added one module at a time. The `gce`, `GC`, `GCE2d`, `IntAna`,
`IntAna2d`, `LProp`, `GeomLProp`, `GProp`, `Hermit`, `ExtremaPC2d`, `ExtremaPC`,
`GeomTools`, `FEmTool`, `AppParCurves`, `AppCont`, `GeomBndLib`, `BndLib`, `CPnts`,
`GCPnts`, `AdvApp2Var`, `Geom2dConvert`, `Extrema`, `GeomConvert`, `GeomLib`, `Approx`,
`AppDef` and `ProjLib` packages are ported in full, and `tools/audit_api_rows.py` reports
0 missing and 0 stale api rows for each of them.

The Geom and Geom2d objects that TKGeomBase builds follow the class mapping of
[geom-design.md](geom-design.md).

The toolkit test inventory maps these complete suites, with matching native and Rust
passes (828 declarations):

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
| AdvApp2Var_Context_Test.cxx | adv_app2var_context | 1 |
| AdvApp2Var_Framework_Test.cxx | adv_app2var_framework | 3 |
| AdvApp2Var_Iso_Test.cxx | adv_app2var_iso | 1 |
| AdvApp2Var_Network_Test.cxx | adv_app2var_network | 2 |
| AdvApp2Var_Node_Test.cxx | adv_app2var_node | 1 |
| Geom2dConvert_CompCurveToBSplineCurve_Test.cxx | geom2d_convert_comp_curve_to_bspline_curve | 1 |
| Extrema_ExtPC_Test.cxx | extrema_ext_pc | 1 |
| ExtremaPC_SearchMode_Test.cxx | extrema_pc_search_mode | 30 |
| ExtremaPC_Comparison_Test.cxx | extrema_pc_comparison | 17 |
| ExtremaPC_ExtendedGeometry_Test.cxx | extrema_pc_extended_geometry | 40 |
| GeomConvert_Test.cxx | geom_convert | 3 |
| GeomConvert_CompCurveToBSplineCurve_Test.cxx | geom_convert_comp_curve_to_bspline_curve | 8 |
| GeomLib_Test.cxx | geom_lib | 15 |
| GeomLib_CheckCurveOnSurface_Test.cxx | geom_lib_check_curve_on_surface | 7 |
| ProjLib_Cone_Test.cxx | proj_lib_cone | 3 |
| ProjLib_ComputeApproxOnPolarSurface_Test.cxx | proj_lib_compute_approx_on_polar_surface | 1 |

`bnd_lib_behavior.rs` covers the GeomBndLib and BndLib quirks below.
`geom2d_convert_behavior.rs` covers the Geom2dConvert quirks below.
`gcpnts_behavior.rs` covers the CPnts and GCPnts quirks below.
`local_props_behavior.rs` covers the LProp, GeomLProp, GProp and Hermit quirks
below. `gce_behavior.rs` and `gc_behavior.rs` cover the status enumeration, the
StdFail_NotDone results, the conversion operators, the GCE2d aliases and the native
quirks below. GeomTools has no upstream GoogleTest suite; `geom_tools_behavior/`
covers its layouts, reader contract, sets, handler and quirks with named fakes for the
console, the text streams, the progress indicator and a custom handler.
`extrema_behavior.rs` covers the Extrema quirks below. `proj_lib_behavior.rs` covers the
ProjLib quirks below. The three ExtremaPC suites that
compare ExtremaPC with the classic `Extrema_ExtPC` share the random points of
`tests/fixtures/extrema_pc_random_points.txt`, which
`tools/oracle/tkgeombase_extrema_random_points.cxx` draws with the libstdc++
`std::mt19937` and distributions of the native fixtures.

One upstream suite is not mapped: `ExtremaPC2d_Hyperbola_Test.cxx` (9 declarations)
is translated as `extrema_pc2d_hyperbola` and passes in Rust, but
`SingletonInvalidAndValueAccessor` fails natively (see Preserved native behaviour), and
the verifier maps whole suites whose native run passes.

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

`gce_MakeCirc`, `GC_MakeArcOfCircle` and `GC_MakeTrimmedCone` call the Extrema port:
`extrema::ExtElC::lines` on two lines and `extrema::ExtPElC::with_line` on a point and a
line. `gce_MakeCirc2d` and `GC_MakeArcOfCircle2d` call the IntAna2d port
(`int_ana2d::AnaIntersection::lines`).

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

ExtremaPC samples with `Extrema_CurveTool::DeflCurvIntervals` and
`Extrema_Curve2dTool::DeflCurvIntervals`, which it calls through the Extrema port
(`extrema::ExtremaCurveTool::defl_curv_intervals`). It also needs the ProjLib plane
projections of lines and conics; ProjLib is not ported yet, so `src/plane_projection.rs`
is a crate-private port of exactly the code paths it uses, in the native operation
order, until the ProjLib port replaces it.

Its curve-aware sampling calls the public `gcpnts::TangentialDeflection`.
`probe_support` is a hidden public module that exposes the projections and a
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
`ssp.nb_p3d()?`; `LastPoint`, `NbP2d` and `NbP3d` return a `Result`, since
AppDef_MyLineTool reads the line's array and first multipoint, which raise on a null or
empty line); `LeastSquare` and `ResolConstraint` read the line in their
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

AdvApp2Var (`adv_app2var`) has two layers. The f2c kernels of `AdvApp2Var_MathBase`,
`AdvApp2Var_ApproxF2var` and `AdvApp2Var_SysBase` are free functions in `math_base`,
`approx_f2var` and `sys_base` (files split by routine family) that keep the native
routine names, argument order and loop structure, so every floating-point operation
happens in the native order. A scalar the routine only reads is passed by value, a
scalar it writes is `&mut`, and an array is the slice that starts at the native pointer;
the `f2c` views keep the Fortran index shifts. Long argument lists are grouped in
structs in the native order (`PatchApproxRequest`, `SquareTables`, `IsoConstraints`,
`ProfileSystem`...). When a native call passes one array as input and output
(`mmfmca8_`, `mmfmca9_`, `mmarc41_`, `mmfmtb1_`, `mma1noc_`), the port copies the input
first; the probe shows the results are the same. The common blocks are explicit state
except `MMPRCSN`: the public `mmwprcs_` changes the precisions that later `mmeps1_` and
`mmveps3_` calls read, so it stays one process-wide value behind a lock. The Hermite
cache `MMCMHER` is per call (`Mmcmher`), `MBLANK` is `SysBaseUnits`, and the constant
tables of `AdvApp2Var_Data` are generated by `tools/generate_advapp2var_tables.py`. The
allocator of `AdvApp2Var_SysBase` keeps owned zones in `AdvApp2VarSysBase`; the kernels
use local vectors where the native code allocates. The trace routines (`mgenmsg_`,
`mgsomsg_`, `maermsg_`) do nothing, as natively at the debug level 0, and the kernels do
not call them.

The classes keep the native names with the `AdvApp2Var` prefix (`AdvApp2VarContext`,
`AdvApp2VarPatch`, `AdvApp2VarApproxAFunc2Var`...). Node, Iso and Patch are transient
classes shared through `Handle`; their members take `&self` over `SharedState`, and the
getters return copies of the native handles as `Option` values. Overloads take distinct
names (`Iso::with_orders`/`with_domain`, `Iso::change_domain_uv`,
`Framework::node_at`, `Patch::cut_sense_with`, `ApproxAFunc2Var::max_errors` for
`MaxError(Dimension)` and `max_error` for `MaxError(Dimension, Index)`). The evaluator is
the `AdvApp2VarEvaluatorFunc2Var` trait (closures implement it), the criterion the
`AdvApp2VarCriterion` trait (an implementer returns the bound, type and repartition that
the native base class keeps in protected fields), and the cuttings the TKG3d
`AdvApproxCutting` trait. `AdvApp2Var_ApproxAFunc2Var` computes everything in its
constructor; `new` and `with_criterion` return the native exceptions as `Failure`.
`adv_app2var_behavior.rs` covers the full approximation, its getters and exceptions,
the criterion hooks and `Dump`, which the upstream suites do not reach.
## Geom2dConvert

Geom2dConvert (`geom2d_convert`) ports every class and package member. The static
members of the native `Geom2dConvert` class are free functions
(`split_bspline_curve_knots` and `split_bspline_curve` for the two SplitBSplineCurve
overloads, `curve_to_bspline_curve`, `concat_g1`, `concat_c1` and
`concat_c1_with_angular`, `c0_bspline_to_c1_bspline_curve`,
`c0_bspline_to_array_of_c1_bspline_curve` and its `_with_angular` twin). Default
arguments are explicit. The native members fill their output references
(`ArrayOfConcatenated`, `ArrayOfIndices`, `ClosedFlag`, `tabBS`, `BS`) before they can
raise, so they stay `&mut` parameters (`HArrayOfBSplineCurves` is the shared
`occ::handle<NCollection_HArray1<occ::handle<Geom2d_BSplineCurve>>>`), and the two
output tables of `concat_c1_with_angular` are grouped in `ConcatOutputs`. The array
of curves is the caller's `Array1` indexed from 0, as natively, and it is rotated in
place. The classes are `CompCurveToBSplineCurve` (its private `Add(First, Second,
After)` is `concat`), `BSplineCurveKnotSplitting`, `BSplineCurveToBezierCurve`,
`ApproxCurve` (`from_curve`, `from_adaptor`; `Dump` writes to a `fmt::Write` with `%g`
numbers), `ApproxArcsSegments` (it borrows its adaptor, as the native reference member;
the private `Status` enumeration is `ApproxArcsStatus`) and `PPoint` (a `Copy` value;
`operator==` and `operator!=` are `PartialEq` with both methods written out). The file
statics of Geom2dConvert.cxx are private modules: `continuity` (Continuity,
GeomAbsToInteger), `concat_support` (Pretreatment, NeedToBeTreated, Need2DegRepara,
Indexmin, ReorderArrayOfG1), `reparametrize` (the law and reparameterise evaluators,
MultNumandDenom and the degree-doubling reparametrization, which the native ConcatG1
and ConcatC1 repeat word for word and the port writes once). An exception that an
adaptor raises inside an AdvApprox or BSplCLib callback is carried to the end of the
call (`failure_capture`) and returned there, where the native exception would unwind.
Nothing is blocked: Convert, GC, GCPnts, Hermit, AdvApprox, BSplCLib and PLib are
ported.
## Extrema

`extrema` is the classic Extrema package. Each class drops the `Extrema_` prefix
(`Extrema_ExtPElC` is `ExtPElC`, `Extrema_POnCurv` is `POnCurv`).

- The class templates are generic over the `ExtremaCurveTool` trait, which bundles
  their `TheCurve`, `TheCurveTool`, point, vector, conic, Bezier, B-spline and
  point-conic parameters; `CurveTool` and `Curve2dTool` (Extrema_CurveTool and
  Extrema_Curve2dTool) implement it, and its static members are the trait functions
  (`defl_curv_intervals` included). One Rust type ports each template:
  `GFuncExtPC`, `GGenExtPC`, `GenLocateExtPC`, `GGExtPC`, `GLocateExtPC`,
  `GCurveLocator`, `GFuncExtCC`, `GGenExtCC` and `GenLocateExtCC`. The native
  instantiations are type aliases (`ExtPC = GGExtPC<CurveTool>`, `ExtPC2d`,
  `LocateExtPC`, `EPCOfExtPC`, `PCFOfEPCOfExtPC`, `ELPCOfLocateExtPC`, `ECC`, `ECC2d`,
  `CCLocFOfLocECC`, `LocECC` and the others). `POnCurv` and `POnCurv2d` are
  `POnCurve<Point3>` and `POnCurve<Point2>`; `POnSurfParams` extends `POnSurf` through
  `Deref`.
- Each constructor overload is a named constructor and each Perform or Initialize
  overload a named method: `ExtPElC::with_circle`/`perform_circle`,
  `ExtElC::lines`/`circles`/`line_ellipse`, `ExtElCS::line_plane`/`perform_line_plane`,
  `ExtElSS::planes`, `GenExtPS::with_point`/`with_range`/`initialize_range`,
  `GenExtCS::perform_range`, `ExtCC::initialize_ranges`/`set_curve_range`. Default
  arguments are explicit, and the sampling of GenExtPS is the `Sampling` value.
- Output references are returned values (`Points(N, P1, P2)` returns `(P1, P2)`,
  `TrimmedSquareDistances` returns the distances and the points); the four-argument
  `Values(X, F, G, H)` of the global-optimization functions is `values_hessian`.
  `FuncExtCS::SquareDistances()`, `PointsOnCurve()` and `PointsOnSurf()` return the
  saved sequences as `&mut Vec`.
- Every native exception is a `Failure` (StdFail_NotDone, Standard_OutOfRange,
  Standard_TypeMismatch, Standard_NotImplemented, StdFail_InfiniteSolutions). A native
  null dereference (an algorithm performed before it has a curve or surface, the
  unallocated result arrays of Extrema_ExtElCS, a conic function without its conic) is
  Standard_NullObject. An exception raised inside a math callback is kept by the
  function (`take_failure`) and returned when the solver stops, where the native
  exception would unwind.
- An object that keeps a curve or surface borrows it for its lifetime.
  Extrema_ExtPExtS and Extrema_ExtPRevS keep the handle of their swept adaptor and
  point their grid search to it; the port gives their GenExtPS a shared handle
  (`SurfaceRef`). Extrema_ExtPS creates them at its first Perform and owns them as
  `Option` values (the native handles); they are not Standard_Transient types.
- The GenExtPS grids keep the NCollection_Array2 storage rules (a resize to the same
  number of cells keeps the old cells), and its sphere tree is the NCollection_UBTree
  port over `Bnd_Sphere`.

## GeomConvert and GeomLib

GeomConvert (`geom_convert`) and GeomLib (`geom_lib`) port every class and package
member; `tools/audit_api_rows.py` reports 0 missing and 0 stale api rows for both.

- The static members of the native `GeomConvert` and `GeomLib` classes are free
  functions; overloads take distinct names (`split_bspline_surface_knots`,
  `split_bspline_surface_one_direction`, `concat_c1_with_angular`, `Tool::parameter_2d`,
  `compute_deviation_pso`). Default arguments are explicit.
- Output references that the native member writes unconditionally are returned values
  (`inertia` returns `InertiaAxes`, `axe_of_inertia` returns `AxeOfInertia`,
  `is_closed` a pair, `build_curve3d` the two deviations, `eval_max_*` the distance).
  An output the member may leave untouched stays `&mut` (`same_range`,
  `build_curve3d`, the in-out handles of `adjust_extremity`, `extend_curve_to_point`,
  `extend_surf_by_length`, `cancel_denominator_derivative` and `norm_estim`), or is an
  `Option` when the native `false` leaves it unwritten (`is_iso_line`, `Tool`,
  `CurveToAnaCurve::is_linear`). The `cf`, `cl` and `Deviation` outputs of the
  CurveToAnaCurve recognizers are the in-out `AnaCurveFit`, whose fields keep their
  value where the native member does not write.
- The classes are structs: `CompCurveToBSplineCurve`, the knot splitting and Bezier
  cutting classes, `CompBezierSurfacesToBSplineSurface`, `ApproxCurve`,
  `ApproxSurface`, `CurveToAnaCurve`, `SurfToAnaSurf`, the least-squares functions
  `FuncConeLSDist`, `FuncCylinderLSDist` and `FuncSphereLSDist` (math function traits;
  a native exception inside a callback is kept and returned by `take_failure`), `Units`;
  `CheckBSplineCurve`, `Check2dBSplineCurve`, `CheckCurveOnSurface`,
  `DenominatorMultiplier`, `Interpolate`, `IsPlanarSurface`, `LogSample`,
  `MakeCurvefromApprox` (it borrows the approximation, which the native class copies),
  `PolyFunc` (math function traits, `take_failure` as above) and `Tool`.
- BuildCurve3d and MakeCurvefromApprox need only AdvApprox (TKG3d), not the Approx
  package: nothing in either package waits on an unported package.
- Every native `std::min` and `std::max` keeps its argument order (`std_min`,
  `std_max`), so NaN and signed zero operands give the native result.

## Approx

Approx (`approx`) ports every class, generic and enumeration of the package;
`tools/audit_api_rows.py` reports 0 missing and 0 stale api rows for it. OCCT 8.1 has
no `Approx_BSplineApproxInterp`: no header, source or reference names it, so there is
nothing to port.

- The generics are trait-based. `Approx_ComputeLine.gxx` is `ComputeLine`,
  `Approx_BSplComputeLine.gxx` is `BSplComputeLine`; both take any
  `app_par_curves::LineTool` (the MultiLine and its tool are one type), and the
  instantiations of AppDef, GeomInt and BRepApprox are aliases of them. The least
  squares, gradient and BFGS generics come from the AppParCurves port.
  `Approx_ComputeCLine.gxx` is `ComputeCLine` over `app_cont::Function`;
  `Approx_FitAndDivide` and `Approx_FitAndDivide2d` are its aliases.
- `LineTool` has the Approx members `what_status`, `make_ml_between` and
  `make_ml_one_more_point` (`None` for a native null MultiLine); AppDef_MyLineTool
  answers them as natively (NoPointsAdded, the same line, no extra point).
- The native constructor overloads are named constructors (`with_line`,
  `with_line_and_parameters`, `from_parameters`; `CurvlinFunc::from_curve`,
  `from_curve_on_surface`, `from_two_curves_on_surfaces`; `SameParameter::from_geometry`,
  `from_adaptors_and_pcurve`, `from_adaptors`), and the long argument lists with
  defaults are options structs with the native defaults (`ComputeLineOptions`,
  `ComputeCLineOptions`, `CurveOnSurfaceOptions`, `CurvilinearOptions`,
  `SweepOptions`).
- `Approx_CurvlinFunc` and `Approx_SweepFunction` are transients: `CurvlinFunc` keeps
  its state behind a lock (its const members record the previous point, as the native
  const_cast does), and `SweepFunction` is a trait whose native-bodied members (D1, D2,
  Resolution, BarycentreOfSurf, MaximalSection, GetMinimalWeight) raise
  Standard_NotImplemented by default.
- The AdvApprox evaluators of Curve3d, Curve2d, CurveOnSurface, CurvilinearParameter,
  SameParameter and SweepApproximation keep an evaluator exception and return it after
  the approximation, as the native exception propagates. The Curve3d and Curve2d
  evaluators and cut points are shared with GeomConvert_ApproxCurve and
  Geom2dConvert_ApproxCurve (`curve_approx_eval.rs`), which use the same code natively.
- `Dump` members write into a `fmt::Write`; `Status` and `ParametrizationType` keep the
  native values (`code`, `from_code`).
- AppDef maps `AppDef_Compute` and `AppDef_BSplineCompute` onto these generics (type
  aliases over its MultiLine); ProjLib, ported after Approx, does the same with its
  instantiations.

## AppDef

AppDef (`app_def`) ports every class of the package; `tools/audit_api_rows.py` reports 0
missing and 0 stale api rows for it (389 implemented and 5 excluded:
`AppDef_MultiLine::SetParameter` and the four `ResConstraintOf*::Error()` are declared
but never defined natively).

- `AppDef_MultiPointConstraint` is an `AppParCurves_MultiPoint` (reached through
  `Deref`) with four optional shared vector arrays (`Handle<HArray1<...>>`, null until
  a constructor or a setter allocates them). A copy shares every allocated array, as
  the native copy shares its handles; the constructor overloads are named
  (`with_tangents`, `with_tangents_and_curvatures`, `with_tangents_3d`, ...).
- `AppDef_MultiLine` holds its multipoints in one shared array (a copy shares it; the
  default line has none). `AppDef_MyLineTool` is the `LineTool` implementation of
  `MultiLine` (`AppDef_MyLineTool::NbP3d(ML)` is `ml.nb_p3d()?`); `MultiLine::value`
  is `AppDef_MultiLine::Value`, the three-array `AppDef_MyLineTool::Value` is
  `LineTool::value`.
- The instantiation classes are type aliases of the ported generics
  (`instantiations.rs`): `Compute` = `approx::ComputeLine`, `BSplineCompute` =
  `approx::BSplComputeLine`, `TheLeastSquares` and the four `*ParLeastSquareOf*` =
  `LeastSquare`, `TheResol` and the `ResConstraintOf*` = `ResolConstraint`,
  `TheFunction` and the `ParFunctionOf*` = `Function<MultiLine>`,
  `BSpParFunctionOfMyBSplGradientOfBSplineCompute` = `BSpFunction<MultiLine>`,
  `TheGradient` and the `MyGradient*` = `Gradient`, `MyBSplGradientOfBSplineCompute` =
  `BSpGradient`, and the four `*_BFGSOf*` = `GradientBfgs`.
- `AppDef_SmoothCriterion` is a transient trait (`Handle<dyn SmoothCriterion>` is the
  native base handle) whose members take `&self`: `LinearCriteria` keeps its state
  behind a lock, so every handle sees it. `double& EstLength()` is `est_length` /
  `set_est_length`, `SetWeight(Array1)` is `set_point_weights`, and the output
  references are returned values (`QualityValues`, `SmoothErrors`).
- `AppDef_Variational` takes its defaulted constructor arguments in
  `VariationalOptions`; `SetCriteriumWeight(Order, Percent)` is
  `set_criterium_weight_of`. Its translation unit defines `No_Standard_OutOfRange` and
  the other range macros after the headers that already expanded them, so every
  NCollection and math access stays checked natively; the port checks the same
  accesses. The engine (TheMotor, Optimization, Project, ACR, SplitCurve, Adjusting)
  shares the parameter and curve handles as natively: the parameters the criterion
  reads are the arrays the motor sorts and remaps in place.
- The null dereferences of the native code (a vector array never allocated, the
  default line, a criterion before SetCurve for InputVector, a null curve given to the
  first SetCurve, a gradient before any basis cache) are Standard_NullObject errors.

## ProjLib

ProjLib (`proj_lib`) ports every class of the package; `tools/audit_api_rows.py`
reports 0 missing and 0 stale api rows for it (253 implemented, and the destructor of
ProjLib_Projector excluded). OCCT 8.1 has no `ProjLib_ProjectOnSurface` and no
`ProjLib_HSequenceOfHSequenceOfPnt`: CompProjectedCurve spells its sequence type inline
(`SequenceOfSequenceOfPnt` in the port), and `ProjLib_HCompProjectedCurve` is a
typedef (`HCompProjectedCurve`).

- The static `ProjLib::Project` overloads are free functions named after the element
  and the surface (`project_line_on_plane`, `project_circle_on_torus`, ...); they build
  the elementary projector and return its typed result, as natively.
  `MakePCurveOfType` returns the curve, `IsAnaSurf` takes the adaptor handle.
- `ProjLib_Projector` is a struct holding the protected result. The elementary
  projectors (`Plane`, `Cylinder`, `Cone`, `Sphere`, `Torus`) embed it and reach it
  through `Deref`/`DerefMut`; the virtual `Project` overloads are the `Project` trait
  (`project_line` ... `project_hyperbola`), implemented by `Projector` itself with the
  native base behaviour. The native slicing `myResult = P` is `Projector::from(p)`.
- `PrjFunc` and `PrjResolve` borrow the adaptors (the native raw pointers);
  `PrjResolve::perform` takes the `Tol`, `Inf` and `Sup` points in a `PrjResolveBox`.
  PrjFunc keeps an adaptor failure raised inside a math callback and PrjResolve
  returns it after the solver, as the native exception unwinds.
- `ProjectOnPlane` (an `Adaptor3d_Curve`), `ProjectedCurve` and `CompProjectedCurve`
  (`Adaptor2d_Curve2d`) are `Handle`-shared adaptors whose fields sit behind one lock.
  `Load`, `Perform` and `Init` compute on a copy of the fields and store it back, also
  when they fail part way, so the observable state matches the native partial update.
  Their `ShallowCopy` is also available as a concrete value (`shallow_projection`), so
  GeomProjLib and `BRep_Tool::CurveOnPlane` can keep the concrete type.
- `ComputeApprox` and `ComputeApproxOnPolarSurface` are plain structs. Their
  `AppCont_Function` implementations keep the first adaptor failure and return it after
  `Approx_FitAndDivide2d`, as the native exception propagates. The approximations reuse
  the Approx, AppParCurves, Extrema, GCPnts, Geom2dConvert, GeomConvert, GeomLib and
  LProp ports.
- ExtremaPC and the EstimDist of IntAna_QuadQuadGeo call these functions; the private
  copies they carried before (`plane_projection.rs` and a `ProjLib::Project(gp_Pln,
  gp_Lin)` copy) are deleted, and their suites and probes stay bit-exact.

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
instantiations (both probes read an AppDef_MultiLine through AppDef_MyLineTool; the
Rust one uses the AppDef port): least-squares fits of 3D, 2D and mixed lines (smooth,
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

The AppCont requests (`tkgeombase_corpus_appcont.py`, 3831 requests) read every entry
of the five tables directly from the native static arrays, run the four square
providers over classe -1 to 28 and VBernstein over every classe x nbpoints pair (into
matrices with shifted bounds and too small, where the partial writes are compared), and
fit 3D, 2D and mixed analytic functions (polynomial, trigonometric, constant) over
forward, reversed and tiny ranges with every pair of end constraints (CurvaturePoint
included), degrees 1 to 25, 1 to 40 Gauss points, failing derivatives, singular ends
around the 0.1 threshold and periodic coordinates that wrap (the first-sample, sample
and end-pole fixes), and the empty ranges (244 requests): degree -1 (no pole), no
Gauss point and no coordinate, where every native table is empty and its loops never
run. Left out: VBernstein at classe 1 (degree 0), more than 61 Gauss points and
periodic fits with fewer than 3 samples (undefined natively); degree -1 with a fixed
end, which writes pole row 1 or 0 of the empty pole table into its stack buffer, and no
coordinate with a fixed last end, which reads `myPerInfo(1..2)` of an empty array (a
crash); and degree -2, whose negative-size tables native builds over the stack buffer
and overruns (a crash; the TKMath `Matrix` refuses them with Standard_RangeError).

The AdvApp2Var requests (`tkgeombase_corpus_advapp2var.py`,
`tkgeombase_corpus_advapp2var_approx.py` and `tkgeombase_corpus_advapp2var_class.py`,
325 requests) feed every public MathBase and ApproxF2var kernel normal, boundary and
error inputs (orders -1 to 3, point counts off the tables, bad degrees, profile systems
with and without pivots, the precision common changed by `mmwprcs_`), and run the
classes: contexts for every order, degree limit and precision code, cut networks and
frameworks, the iso and patch approximations of a grid (with a failing evaluator), and
AdvApp2Var_ApproxAFunc2Var end to end over four functions, favourite isos, continuities
C0 to C2 (G1 and C3 raise), degrees 3 to 14, patch budgets 1 to 50, both cuttings and
absolute and relative criteria with both repartitions. Each request prints the
results, the resulting surface (knots, multiplicities and poles) and the FNV-1a hash of
`Dump`. Left out: the native reads of memory that the native code leaves uninitialised
(see Preserved native behaviour) and a patch discretisation after an iso without
result, which dereferences a null handle natively.

The Geom2dConvert requests (`tkgeombase_corpus_geom2dconvert.py`, probe
`tkgeombase_probe_geom2dconvert.cxx`) convert every Geom2d curve kind with each
parameterisation: circles and ellipses on direct and indirect frames, whole and
trimmed (arcs from 1e-9 to beyond 2 Pi, across the seam, reversed), trimmed lines,
hyperbolas and parabolas, rational and polynomial Bezier and B-spline curves
(periodic, C0, trimmed beyond their bounds) and offset curves. They split B-splines by
knot indices (equal, reversed, out of range) and parameters, split them at knots for
continuity ranges from -1 to 5, cut them into Bezier arcs (ranges that snap to a knot,
are empty or reversed), join composite curves (circle arcs in and out of order, chains
that touch, touch reversed, close a loop or miss, mixed degrees, rational weights,
both constructors), concatenate G1 and C1 arrays (open, closed, with corners, circle
loops needing the Hermit law or the degree-doubling reparametrization, degree-13
loops that clear the closed flag, separated curves, too short tolerance tables, a
single curve), reduce C0 knots in open and closed curves, approximate offset and plain
curves through a curve or an adaptor at every order, segment count and degree, and
approximate lines, circles, conics, S-curves, straight parts and closed curves by
segments and biarcs at four tolerances. Each request prints every resulting curve
(degree, flags, poles, weights, knots, multiplicities), the output tables and the
rotated input array, or the raised exception. They are 908 requests. Circle arcs of
1e-9 are left out of the biarc requests: GC_MakeArcOfCircle2d then reads the status
that gce_MakeCirc2d leaves unassigned (see GC and GCE2d below).
The Extrema requests come from six corpora, one per family, each with its probe
(`tools/oracle/tkgeombase_probe_extrema_*.cxx`, twins in `examples/gb_probe/extrema_*.rs`).
The closed forms (`tkgeombase_corpus_extrema_el.py`) run every point-conic and
point-elementary-surface pair and every line/circle/conic pair of ExtElC, ExtElC2d,
ExtElCS and ExtElSS in skew, secant, tangent, parallel, antiparallel, nearly parallel
(1e-13 rad), coplanar, axial, concentric and nested configurations, with bounded,
reversed, single-point, periodic-offset and infinite ranges and sequences of performs
on one object. The point-curve classes (`tkgeombase_corpus_extrema_pc_classic.py`)
reuse the curves of the ExtremaPC corpora on whole ranges and sub-ranges from special
and random points, and drive the EPC, PCF and LocEPC template instantiations, the
curve locator and the curve tools directly, cusps included. The curve-curve classes
(`tkgeombase_corpus_extrema_cc.py`) cover the analytic pairs, the trimmed parallel
cases and the global search of conics with Bezier, B-spline and offset curves. The
point-surface classes (`tkgeombase_corpus_extrema_ps.py`) run every surface kind
(quadrics, B-splines with a collapsed boundary, Bezier, revolutions and extrusions of
conics and splines, offset and trimmed surfaces), ranged or not, from general, near,
far and centre points: the functions inside and outside the bounds, the local search
with both criteria, GenExtPS with both algorithms, every flag, 2 to 20 samples and
algorithm switches, ExtPS with periodic ranges beyond one period and infinite bounds,
and the analytical and grid cases of ExtPExtS and ExtPRevS (a point on the axis, a
curve ending on the axis, re-initialization on the same handle). The curve-surface
classes (`tkgeombase_corpus_extrema_cs.py`) cover every closed form of ExtCS and its
trimmed-out fallback to the curve ends, a line facing a surface box down to a single
point, the generic search with sharp points, and the three GenExtCS searches with
their functions. The surface-surface classes (`tkgeombase_corpus_extrema_ss.py`) pair
the surfaces of the point-surface corpus and run parallel, secant and trimmed planes.
Each request prints the done and parallel flags, every extremum (square distance and
points with their parameters), the exception of the out-of-range index 0, or the raised
exception.
They are 1934 requests: 212 closed-form, 994 point-curve, 110 curve-curve, 345
point-surface, 210 curve-surface and 63 surface-surface.

The GeomConvert and GeomLib requests (`tkgeombase_corpus_geomconvert.py`,
`tkgeombase_corpus_geomconvert_ana.py` and `tkgeombase_corpus_geomlib.py`, over the
GeomTools builder specs of `tkgeombase_corpus_geomspecs.py`; probes
`tkgeombase_probe_geomconvert*.cxx` and `tkgeombase_probe_geomlib*.cxx`) cover every Geom
and Geom2d curve and surface kind: conics on direct, tilted and indirect frames (whole,
trimmed, across the seam, reversed), lines, rational and polynomial Bezier and B-spline
curves and surfaces (periodic, C0, the exact rational quarter circle, cylinder and
sphere), trimmed and offset curves and surfaces, surfaces of revolution and extrusion
that are cylinders, cones, spheres and tori, and the elementary surfaces. They convert
curves and surfaces to B-splines with each parameterisation, split them by knots and
parameters (in order, reversed, equal, out of range, one direction), cut them by
continuity and into Bezier arcs and patches, compose curves and Bezier patch grids,
concatenate G1 and C1 chains and loops, raise C0 knots, approximate at several orders,
degrees and segment counts, recognize lines, circles, ellipses, planes, cylinders,
cones, spheres and tori with every conversion type and target (exact, perturbed, short,
closed and non-canonical inputs, ranges inside and beyond the bounds), evaluate the
least-squares functions on exact and noisy clouds with short variable vectors, and
convert pcurve units. The GeomLib requests reach every static member and class: 2D
curves placed in 3D and transformed by affinities, same-range reparametrizations,
3D curves of pcurves, curve and surface extensions at each continuity and end,
inertia of planar, collinear, coincident and spread clouds, parameter arrays, curve
distances, denominator cancellation, normal estimation at regular and singular points,
closedness, isolines, tangent fixes, curve-on-surface checks, interpolation errors,
planarity, logarithmic samples, polynomials, point projections, chord deviations and
the curves of AdvApprox approximations of every space kind. They are 2990 requests
(1844 GeomConvert, 1146 GeomLib). Left out, as the native result is undefined there:
three points within gp::Resolution() for `GetCircle` (the gce status is unassigned),
the empty arrays of `RemovePointsFromArray` and `DensifyArray1OfReal` (read unchecked, a
crash), the MakeCurvefromApprox members of an absent subspace kind (a null table) and
DenominatorMultiplier flat knot vectors other than 8 to 10 values (read out of bounds).

The Approx requests come from two corpora. The drivers (`tkgeombase_corpus_approx.py`,
probes `tkgeombase_probe_approx.cxx` and `tkgeombase_probe_approx_line.cxx`, which
instantiates `Approx_ComputeLine.gxx` over AppDef_MultiLine with a scripted line tool)
run ComputeLine and BSplComputeLine on 3D, 2D and mixed lines (smooth, clustered, noisy,
collinear, looping, with and without end tangents) with every parametrization,
constraint pair, degree range, cutting and squares mode, given parameters and knots, the
scripted WhatStatus/MakeMLBetween/MakeMLOneMorePoint answers, ComputeCLine on AppCont
functions with and without cutting, inverse order and hang checking, and
MCurvesToBSpCurve on sequences of mixed degrees; 1324 requests. The adaptors
(`tkgeombase_corpus_approx_adaptors.py`, probes `tkgeombase_probe_approx_adaptors.cxx`
and `tkgeombase_probe_approx_sweep.cxx`) run Curve3d, Curve2d, CurveOnSurface (every
surface kind, the iso-line paths, only3d and only2d), CurvlinFunc command scripts in its
three cases, CurvilinearParameter, SameParameter from each constructor (already same
parameter, reparametrized, C0 and off-surface pcurves) and SweepApproximation on
polynomial and rational sections with and without 2D curves, with implemented and
missing D1, D2 and Resolution; 673 requests. Left out, as the native result is undefined
there: ComputeLine given fewer parameters than points, BSplComputeLine user
multiplicities with several degrees and cutting after a failed first fit (uninitialized
multiplicities, heap corruption).

The AppDef requests (`tkgeombase_corpus_appdef.py` and
`tkgeombase_corpus_appdef_variational.py`, probes `tkgeombase_probe_appdef.cxx` and
`tkgeombase_probe_appdef_variational.cxx`) run every MultiPointConstraint constructor on
3D, 2D, mixed and empty point sets with the vector setters and getters inside and
outside their ranges, every MultiLine constructor with the AppDef_MyLineTool members
on arrays of every size and lower bound, AppDef_Compute's four constructors,
AppDef_LinearCriteria's members on curves of one to three elements (C0 to C2, matching
and mismatching dimensions, every SetCurve path), and AppDef_Variational on 3D, 2D and
mixed lines (smooth, noisy, collinear, with a repeated point, scaled) without and with
pass, tangency and curvature constraints, every continuity, over-constrained to
generous degrees and segment counts, size and max-error tolerances (Adjusting), every
criterion weight, every query, Distance, Dump and setter, and the degenerate inputs
(one, two and coincident points, FirstPoint above 1, constraints before the last point,
missing or non-normal constraint vectors, zero tangents); 555 requests. Left out, as
the native result reads uninitialized or unowned memory there: vector reads of a point
without them, a default line, SetCurve of a null curve, point ranges whose allocation
size wraps, NoConstraint couples whose uninitialized table entries a search reaches,
InitCutting runs whose back scan compares with the knot it is about to set (the
corpus simulates the scan), a smoothing after a SetConstraints that raised inside Init,
negative tolerances in the max-error mode and C0 smoothings that reduce an element to
degree 1.

The ProjLib requests (`tkgeombase_corpus_projlib.py` and
`tkgeombase_corpus_projlib_curves.py`, probes `tkgeombase_probe_projlib.cxx` and
`tkgeombase_probe_projlib_curves.cxx`) run the elementary projectors on every line and
conic kind over planes, cylinders, cones, spheres and tori in direct, tilted and
indirect frames (generatrices, lines through the cone apex, parallels, meridians,
circles through the poles, torus iso circles, rejected and off-surface curves,
default-constructed projectors, SetInBounds, UFrame and VFrame), the static Project
overloads on points at axes, apexes and poles, PrjFunc and PrjResolve with every fixed
variable, invalid ones and bounded searches, ProjectOnPlane on every curve kind along
normal, oblique and in-plane directions with and without the parametrization kept, and
ComputeApprox, ComputeApproxOnPolarSurface (every constructor, BuildInitialCurve2d,
initial and seam curves, C0 B-splines split into C1 spans), ProjectedCurve (every
constructor and option) and CompProjectedCurve (every constructor, the parts, their
evaluation and intervals, Perform with 2D and 3D results) on elementary, swept,
trimmed, offset, Bezier and B-spline surfaces, across seams and periodic ranges,
through poles, leaving bounded surfaces, far from them and degenerate (point and iso
projections); 2225 requests. The given initial curves of the polar projection rotate
over the curves and the ComputeApprox constructor runs once per surface: the far initial
curves and the 1e-7 fits dominate the debug Rust twin, and the numeric verifier has a
900 s budget. Left out: the projection of a line normal to the plane by
the default ProjectOnPlane (a direction in the plane, all values NaN), whose native
reparametrization never returns.

The corpus has 44476 requests: 4449 gce and GC, 945 IntAna2d and IntAna, 833 LProp,
GeomLProp, GProp and Hermit, 1253 ExtremaPC2d, 1252 ExtremaPC, 296 GeomTools, 1230
FEmTool, 2045 AppParCurves, 3831 AppCont, 15198 GeomBndLib and BndLib, 2210 CPnts and
GCPnts, 325 AdvApp2Var, 908 Geom2dConvert, 1934 Extrema, 1844 GeomConvert, 1146 GeomLib,
1997 Approx, 555 AppDef and 2225 ProjLib.

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
  rejected by VBernstein). At degree -1 a fixed end (pass or tangency) writes pole row
  1 or 0 of the empty pole matrix; AppCont_LeastSquare.cxx compiles its range checks
  out, so natively the write lands in the matrix's unused stack buffer. The port raises
  Standard_OutOfRange at that write. The empty ranges themselves (degree -1, no Gauss
  point, no coordinate) fit over empty tables as natively. The `int` offsets of the providers overflow natively past
  classe 1861; the port wraps.
- `AppCont_Function::PeriodInformation` defaults to not periodic with period 0.
  `VBParam` is computed natively and never read; the port does not keep it.

AdvApp2Var:

- Several native tables are only partly written, and the port zeroes the rest where the
  native memory is uninitialised: the zero-root row of an even Gauss table
  (`mmapptt_`), the last two root slots of a Context with an order above -1, the scratch
  tail of `Patch::Coefficients` beyond `NCFLMU*NCFLMV*NDIMEN`, the `mmfmtb1_` scratch
  that it copies when the table is smaller than its capacity, and the iso tolerances of
  a position outside 0..4. `mdsptpt_` and `mmresol_` dereference a null work pointer for
  100 values or fewer natively; the port uses owned vectors.
- `mma2jmx_` reads past its table of maxima for a Jacobi degree beyond it (a Context
  with a large limit and precision code); the port leaves those entries unwritten.
  Only a Context whose degree `mmapptt_` then refuses (code 31) reaches it, so the
  result is the same Standard_ConstructionError.
- `mma2cd2_` writes the diditb correction at `(jj, 0)`, not `(0, jj)`, and the odd-root
  middle correction of `mma1cdi_` reads only the last order.
- `mma2ds2_` adds 100 to the evaluator code and `mma2ds1_` adds 100 again; for the
  favourite iso 1, `mmfmtb1_` then clears the code, so the evaluator failure is lost.
  `Patch::Discretise` also loses an `mma2cdi_` failure, because `mma2ds1_` clears the
  code first.
- `mmgaus1_` with an invalid Gauss rule index `k` sets the code to -1 before the first
  callback, and stops only after a callback that does not clear it.
- `Framework::IsoU`/`IsoV` search a strip up to the strip count of the other direction,
  so a strip's last iso is returned when no constant matches. The isos that
  `UpdateInU`/`UpdateInV` insert get the position 0.
- `Patch::AddErrors` uses one subspace (`NBSESP = 1`), and Iso and Patch assume a 3D
  function (`NDIMSE = 3`), whatever the context.
- `Patch::AddConstraints` and `Discretise` copy the iso tables by `operator=`, which
  takes the length of the source table; the scaling loops then index the copy.
- The `lesparam` point counts skip 15, 25 and 61 (the "Bug Uneven number" comments), and
  the `mdnombr` value of pi/2 is the literal of the common, not `FRAC_PI_2`.
- `AdvApp2Var_ApproxAFunc2Var::ComputePatches` with a criterion keeps a maximum `m1`
  of the criterion values that is never read; the port does not keep it.
Geom2dConvert:

- `SplitBSplineCurve(C, FromU1, ToU2, ParametricTolerance, ...)` ignores the tolerance
  and checks no bound (the B-spline `Segment` raises); the knot overload raises
  Standard_DomainError and Standard_OutOfRange without a message.
- `CurveToBSplineCurve` clamps the bounds of a trimmed curve to a non-periodic basis,
  but a trimmed line keeps the trimmed curve's own bounds as knots; offset curves are
  approximated from the trimmed curve, not its basis; an untrimmed line, hyperbola or
  parabola raises Standard_DomainError. With RationalC1, arcs of 6 radians or more are
  joined from two halves by CompCurveToBSplineCurve at Precision::PConfusion().
- `ConcatG1` tests G1 junctions with Precision::Confusion() and Precision::Angular(),
  not with `ArrayOfToler`, while `ConcatC1` uses them; both call the junction test
  twice (the first `< GeomAbs_C0` test cannot hold; the port calls it once, as it has
  no side effect), and raise Standard_Failure ("Courbes non jointives") for separated
  ends. The open-array loop merges with the input tolerances even after a closed array
  was rotated (the rotated copy is used only by the closed single-group branch).
- Pretreatment divides the weights of the caller's own curves in place when their two
  first and two last weights are equal; ConcatC1 multiplies other rational curves by
  their Hermit law. A closed single group whose closing curve has a degree above
  `MaxDegree() / 2` clears `ClosedFlag` instead of doubling the degree.
- `C0BSplineToArrayOfC1BSplineCurve` counts the first knot among the C0 knots and
  compares the square distance of the ends with the tolerance itself, while
  `C0BSplineToC1BSplineCurve` starts after the first knot, squares the tolerance and
  uses a fixed angular tolerance of 1e-7.
- `CompCurveToBSplineCurve::Add` raises the degree of the stored curve in place, so a
  handle returned by an earlier `BSplineCurve()` sees it.
- The AdvApprox evaluator of ApproxCurve sets error code 1 (dimension) or 2
  (parameter outside the interval) and still evaluates; an order beyond 2 sets 3.
- `ApproxArcsSegments`: when a chord shorter than Precision::Confusion() cannot be a
  checked segment, `makeApproximation` still appends the null segment and fails, so
  `GetResult()` can end with a null curve; the line test of `getLinearParts` compares
  the chord's square length with Precision::Confusion(); `makeLine` takes
  `std::max(AngleTol, 0.01)`; `calculateLines` is declared and defined but never
  called (a unit test reaches it).
- `PPoint::operator!=` is its own test (`> PConfusion()`), so `==` and `!=` are both
  false for a NaN parameter.
- A null handle, dereferenced natively, raises Standard_NullObject; an empty curve
  array allocates `NCollection_Array1(0, -2)` natively (an allocation failure) and
  raises Standard_RangeError in the port; the corpus leaves both out.
Extrema:

- `NbExt()`-guarded accessors test `N < 1` first, so an index below 1 raises
  Standard_OutOfRange before the StdFail_NotDone of an undone algorithm.
- Extrema_ExtPElC2d differs from the 3D class: both extrema of a circle are minima,
  no extremum of an ellipse, hyperbola or parabola is a minimum, and a line whose
  projection is outside the bounds is done with no extremum (the 3D line is not done).
  A degenerate polynomial of the hyperbola and parabola cases raises
  StdFail_InfiniteSolutions.
- Extrema_ExtElSS sets done, then raises Standard_NotImplemented for every sphere pair;
  the Extrema_ExtElCS line and circle overloads with a cone or a torus raise it and
  leave the state unchanged. The Extrema_ExtElCS result arrays are handles: a Perform
  keeps the arrays of an earlier one, and reading them before any allocation is a null
  dereference (Standard_NullObject in the port).
- Extrema_GenLocateExtPC is done only when `|F|` at the root is below 1e-7;
  Extrema_GenLocateExtCC when both components of F are below 1e-10 (signed, so a
  negative value counts). Extrema_GGenExtCC appends the solutions of each Perform to
  those of the earlier ones.
- Extrema_GenExtPS: SetAlgo with a new algorithm rebuilds the grid but keeps the
  parameters and the tree of the earlier algorithm; a grid resized to the same number
  of cells keeps its cells; Initialize raises Standard_OutOfRange for fewer than two
  samples. The tree algorithm raises the samples of a B-spline to degree times knots
  (at most 300), CorrectNbSamples multiplies the V samples in both branches, and the
  farthest-node selector starts its bound at 0, so it rejects every subtree whose
  nearest distance is positive and usually starts from the first grid node.
- Extrema_ExtPExtS and Extrema_ExtPRevS read `NbExt()` of their grid search in
  Perform, which raises StdFail_NotDone when it found nothing. Extrema_ExtPRevS resets
  its analytical flag before testing whether the handle is new, so an Initialize with
  the same handle always takes the grid search.
- Extrema_ExtPS replaces infinite bounds by ±1e10 and samples 300 times across a
  degenerated boundary iso; `TrimmedSquareDistances` returns members nothing computes
  (null in the default object, uninitialized after the performing constructors; the
  port returns 0 and the origin).
- Extrema_GenLocateExtPS widens the tolerances of a B-spline for large parameters and
  then always runs the Newton pass, and raises Standard_OutOfRange when no root was
  saved.
- Extrema_ExtCS: the parallel case stores one distance and no point, so `Points(1)`
  raises Standard_OutOfRange; the single-point line case adds its projections with a
  default curve point (the origin); a line and a plane that are not parallel, and a
  line with any surface but a sphere or a cylinder, take the facing-range search; of
  the sharp-point projections only the nearest and the farthest remain when more than
  two were added. Extrema_ExtSS runs its generic search on the whole surfaces and
  filters afterwards, and keeps IsParallel from an earlier Perform in that case.
- Extrema_GenExtCS divides a closed curve that covers more than two thirds of its
  period and then keeps only the extrema within SquareConfusion of the minimum;
  Extrema_GlobOptFuncCQuadric::QuadricParameters on another surface returns
  `(Uf, Ul)`; Extrema_GlobOptFuncConicS and Extrema_GlobOptFuncCQuadric built without
  their conic or quadric dereference an unset pointer (Standard_NullObject in the
  port). Extrema_GlobOptFuncCS returns the half gradient and Hessian.
- Extrema_FuncExtCS merges saved states by the curve parameter only,
  Extrema_FuncPSNorm by `(u, v)` within PConfusion, and Extrema_FuncExtSS saves every
  state. Extrema_GenExtSS stores its BFGS minimum through `Value(UV, UV)` (the input
  is read before the output overwrites it) and GetStateNumber.
- Every native comparison keeps its form where a NaN operand changes the branch, and
  `std::min`/`std::max` keep their argument order.

Approx:

- `Approx_ComputeLine` never initializes `tolreached` and the current errors in its
  constructors, and a first fit on fewer points than parameters leaves `TheParam`
  uninitialized; the port starts from false and 0.
- ComputeLine's CheckMultiCurve divides the first leg of a 2D curve by
  `Epsilon(1.)` instead of its length, and records the bad indices in a three-slot
  array that overflows past three curves (a vector in the port).
- ComputeLine's ComputeCurve stores the interpolated piece and then halves the range,
  so consecutive pieces overlap; the MakeMLBetween test `nbpdsotherligne` is
  `FirstPoint - LastPoint`, which is 0 only for one point.
- `Approx_BSplComputeLine::FindRealConstraints` queries the tangents a second time for
  a curvature end, so a line with tangents and no curvatures keeps curvature
  constraints; `ComputeCurve` is declared and never defined; `Parameters()` before a
  Perform dereferences a null table (Standard_NullObject in the port); user
  multiplicities are read past their bounds for several degrees (left out).
- `Approx_ComputeCLine` never clears its pieces: a second Perform appends to the first.
- `Approx_MCurvesToBSpCurve` keeps its pole counter across spans, and an empty sequence
  raises Standard_OutOfRange.
- `Approx_Curve2d` discards the result of its trim.
- `Approx_CurvilinearParameter` in the one-curve-on-surface case never assigns
  `myMaxError2d2` (the port reads 0).
- `Approx_CurvlinFunc`: in the two-curve case Trim builds the second curve's tables
  with the first curve's, `GetUParameter` uses the tables of the curve number whatever
  curve it is given, and one Adaptor3d_CurveOnSurface is reloaded for both curves, so
  its interval cache survives the Load (the port reuses one adaptor the same way).
- `Approx_SameParameter` leaves `TolReached` unassigned when the distribution fails,
  returns a null Curve2d when the pcurve is already same parameter and a null
  CurveOnSurface on the early returns; the fallback CurveOnSurface uses the approximated
  pcurve even when it is not kept; its Interpolate raises instead of returning false.
- `Approx_SweepApproximation::Dump` writes its parts to `std::cout` (the port to the
  given stream), and the 2D poles are appended across Performs, so `Curve2dPoles(1)`
  keeps the first run's poles.

AppDef:

- `AppDef_MultiPointConstraint::SetTang` (and the three other setters) allocates its
  array before the index check, so a failing call still makes the point a tangency
  (curvature) point; `Tang` and the other getters dereference the array unchecked
  (Standard_NullObject in the port when it was never allocated). The six-array
  constructor copies the curvatures over the tangent count (equal after its check).
- `AppDef_MultiLine::NbPoints`, `AppDef_MyLineTool::NbP2d` and `NbP3d` read the first
  multipoint: Standard_OutOfRange on an empty line. `Dump` prints only the count.
- `AppDef_LinearCriteria::Gradient` and its dimension check raise with the message of
  ErrorValues; `SetWeight(Array1)` takes the bounds of its argument; the element cache
  survives SetCurve (only SetParameters invalidates it), and an element without points
  leaves the previous cache and the point range `IL..=IL - 1`.
- `AppDef_Variational` estimates the last tangent from the points at
  `(LastPoint - 3) * Dimension + 1` (not relative to FirstPoint), so any FirstPoint
  above 1 reads past the points (Standard_OutOfRange); two points read past them too.
  The constraint searches of EstTangent, EstSecnd and ACR evaluate the table entry
  before their count test, so constraints ending before the last point raise
  Standard_OutOfRange; with NoConstraint couples the table is longer and the entry read
  is uninitialized natively (0 in the port).
- `Init` counts a constraint before reading its vectors: a SetConstraints that raises
  leaves the counters and freshly allocated (uninitialized natively, zero in the port)
  tables behind, and `IsOverConstrained` is never cleared by a later valid Init
  (SetConstraints then answers false although the criterion is rebuilt).
- `SetMaxDegree` neither checks nor clamps the degree (the constructor clamps to 30),
  `SetWithCutting(false)` tests the over-constraint with the knot count as segment
  count, and no setter clears the done flag; `Approximate` never clears it either, so a
  failed conversion keeps the previous result. Approximate stores its knots in a new
  array (earlier `Knots()` handles keep the old ones) and rebases the parameters on 1.
- `Adjusting` restores the errors and criteria of a rejected pass but keeps its
  parameters, distances and worst point index. `Project` shifts the worst point index
  by FirstPoint - 1 even when no distance exceeds 0 (the index is then the caller's
  uninitialized value natively, 0 in the port).
- `InitCutting` can compare a constrained parameter with the knot it is about to set,
  which FEmTool_Curve leaves uninitialized natively (0 in the port).
- `Dump` prints the flags `o.setf(std::ios::scientific)` returns before the tolerance
  (4098 for a stream with the default flags, the port's assumption), then leaves the
  stream in scientific mode with precision 3.
- A C0 smoothing can reduce an element to degree 1, whose second derivative
  FEmTool_Curve::D2 evaluates as a polynomial of degree -1, reading outside the element
  natively; the port raises Standard_OutOfRange.
- A point range with `LastPoint < FirstPoint - 1` wraps the allocation size of the
  point weights; the native allocation fails and the first write crashes, the port
  raises Standard_OutOfMemory.

GeomConvert:

- The 3D `SplitBSplineCurve` raises Standard_DomainError for both of its index
  checks, where Geom2dConvert raises Standard_DomainError and Standard_OutOfRange.
- `SplitBSplineSurface(S, FromParam1, ToParam2, USplit, ...)` still orients the result
  in U for a V split (it tests IsUPeriodic and calls UReverse).
- `GeomConvert_BSplineSurfaceKnotSplitting`: when the surface is already C
  `VContinuityRange` in V, the V bounds overwrite the U table and the V table stays
  null, so every V accessor dereferences a null handle (Standard_NullObject in the
  port).
- `CompBezierSurfacesToBSplineSurface` raises the degrees of the caller's patches in
  place, and its V knot reduction tests the last U multiplicity (`multU`); natively
  the loop can run on when V removals keep succeeding.
- The ApproxSurface evaluator writes nothing for the orders it does not handle (order
  3 with UOrder 0 or 3, orders above 4), so the AdvApp2Var result keeps its values.
- The concatenation members merge knots within Precision::PConfusion() or a fifth of
  the range when that is smaller; the closed single-group test compares the degree
  with `Geom2d_BSplineCurve::MaxDegree()` in the 3D code (both are 25).
- `CurveToAnaCurve::ComputeCurve`: GeomConvert_Target with a target other than a line,
  circle or ellipse falls through to GeomConvert_MinGap; Target copies the range of a
  recognizer that returned early (0, 0 when the line or circle test stops before
  writing it). In MinGap, a curve
  whose ends and middle are within 2 x tolerance takes the circle unless it is null or
  the line deviates less, so the null line can be returned with a gap of RealLast. The
  converter's gap is `Precision::Infinite()` until a conversion, and `Init` resets it.
- `GetCircle` writes the circle as soon as gce_MakeCirc is done, also when it then
  returns false for a radius below gp::Resolution(); gce_MakeCirc leaves its status
  unassigned for three points within gp::Resolution() (see gce).
- `SurfToAnaSurf::ConvertToAnalytical` limits the tolerance to a thousandth of the
  distance from (U1, V1) to (U1 + U2, (V1 + V2) / 2) (U not halved); the sphere and
  torus search reads the isolines of the input surface, not of the segmented copy; a
  non-circular end isoline of the cylinder and cone search contributes radius 0 and its
  point at (Last - First) / 2; the Gauss-field cylinder sizes its point array NbU x NbU
  and keeps `aSign = -1` once a sample swapped its curvatures; a Target of another type
  returns null with gap RealLast, and Target plane on a non-planar surface returns null
  with gap RealLast. `ConvertToAnalytical(tol)` on a converter without a surface
  dereferences it natively (Standard_NullObject in the port).
- `FuncCylinderLSDist::Gradient` scales G(6) of its 4-variable gradient, which raises
  Standard_OutOfRange once G is doubled; `Values` scales G(4). The functions read the
  variables with bound checks, so a short variable vector raises before F is reset.
- `GeomConvert_Units`: in the plane case a circle or ellipse pcurve is scaled; any
  other pcurve continues with both factors set to the length factor.

GeomLib:

- `SameRange` treats a null range whose `RequestedLast + RequestedFirst` sum (not
  difference) is null as nothing to do, and leaves `NewCurve2dPtr` unchanged.
- `GTransform` returns a null handle for parabolas, hyperbolas and offset curves (whole
  or trimmed) under a non-similarity.
- `GeomLib_CheckBSplineCurve::FixTangent` and its 2D twin move the poles of the
  caller's curve (the class keeps its handle); `ExtendSurfByLength` modifies a B-spline
  input in place (degree raised, periodicity removed) before replacing the handle.
- `GeomLib_Interpolate` does not assign `myError` on success (the port reads NoError).
- `GeomLib_DenominatorMultiplier::Value` fills its basis table only for 8, 9 or 10
  flat knots, and in the 10-knot case only for the span indices 1 to 3; the table is
  uninitialized natively in the other cases, where the port reads zeros. A vector
  shorter than the cubic basis needs is read out of bounds natively
  (Standard_OutOfRange in the port).
- `GeomLib_CheckCurveOnSurface` stores the parallel flag, but its sub-interval results
  do not depend on the order, so the port computes them in sequence in both modes.
- `GeomLib_LogSample::GetParameter` returns the bounds at 1 and N and raises
  Standard_OutOfRange outside `1..=N`; the port's math_FunctionSample hook, which
  math_FunctionAllRoots calls only inside, gives NaN there instead.
- `GeomLib_PolyFunc` of a constant polynomial (one coefficient) is built with an empty
  derivative table, and its callbacks raise Standard_OutOfRange (kept by
  `take_failure` in the port).
- `RemovePointsFromArray` and `DensifyArray1OfReal` read the first value of an empty
  input unchecked (Standard_OutOfRange in the port); the MakeCurvefromApprox members of
  a subspace kind the approximation does not have pass their index check at 0 and
  dereference a null pole table.

ProjLib:

- `ProjLib_Projector()` sets the BSplineCurve type, not the OtherCurve its comment
  announces, with null curves. `ProjLib_ProjectedCurve` keeps that result when a
  Bezier or B-spline surface gives no polar projection or another surface gives no
  CompProjectedCurve part; `Degree`, `IsRational`, `NbPoles` and `NbKnots` then
  dereference the null B-spline natively (Standard_NullObject in the port). These two
  branches never reset the result, so a second `Perform` keeps the previous one.
- `ProjLib_Cone::Project(gp_Circ)` sets the Line type before its axis test, so a
  circle not normal to the axis leaves a Line type over the previous line, and
  `ProjLib::Project(gp_Cone, gp_Circ)` returns that line instead of raising.
- `ProjLib_Cylinder::Project(gp_Elips)` does nothing (the ellipse is left to the
  approximation, for periodicity reasons). `ProjLib_Sphere::Project(gp_Circ)` of a
  circle that is neither a meridian nor a parallel keeps its type and stores the
  default `gp_Lin2d`. `ProjLib_Torus` does not check that the circle lies on the torus;
  a U iso whose centre is on the axis raises gp_VectorWithNullMagnitude from the
  angle of a null reference vector.
- `ProjLib_Sphere::SetInBounds` mirrors across a pole with a 1e-7 tolerance (OCC13116)
  and raises StdFail_NotDone before a successful projection.
- `ProjLib_ProjectedCurve` does not override the point evaluators, so `Value`, `D0` to
  `DN` raise the Standard_NotImplemented of `Adaptor2d_Curve2d`; `Continuity`,
  `NbIntervals`, `Intervals`, `Trim`, `IsClosed`, `Period` and `Resolution` raise it
  too. Its `GetCurve` is the curve after the pole trims.
- `ProjLib_ComputeApprox`: a B-spline or Bezier curve on a plane projects the poles of
  the adaptor's whole curve (its trimmed range is ignored) and keeps the tolerance.
  `Function_SetUVBounds` evaluates the cylinder at `(U1, U2)` for a closed ellipse, and
  tests `S > U1 || S < U2` (almost always true) before the sphere U period; the
  "return curve home" step moves U by whole multiples of PI, not 2 PI. The smoothing
  loops of ComputeApprox and ComputeApproxOnPolarSurface run over the knot count taken
  before the removals, the one of ProjectedCurve reads it at each step; all stop at the
  first refused removal.
- `ProjLib_ComputeApproxOnPolarSurface`: `BuildInitialCurve2d` stores the initial curve
  as the B-spline result and raises Standard_NoSuchObject for a plane or a swept
  surface, so `Perform` without an initial curve raises there. The bilinear-patch pole
  case compares the static `MaxDegree()` of the surface class (25) with 1, so it never
  applies; its Bezier loop would clear the done flag after each pole and its distance
  test truncates the squared distance to an int. The iso-line test of the projection
  evaluates the surface at `(U, t)` and `(t, V)` with the curve parameter `t`, and the
  sphere V period test compares with `Vsup + (Vsup - Vinf)`. The C1-span split raises
  the degree of the span projections in place and joins them without their weights.
  The seam constructor keeps the B-spline of `BuildInitialCurve2d` when the projection
  is not done.
- `ProjLib_CompProjectedCurve`: `Init` appends to the existing parts (a second `Init`
  adds them again). The extrema curve-surface start the parts only when MaxDist is
  positive and the surface is not an extrusion. The default constructor has a null
  sequence and leaves the approximation options uninitialized (the port gives them the
  values of the other constructors); `ShallowCopy` copies neither those options nor
  the results. `Perform` clears the 2D approximation for every part after an iso part
  (those parts then store the previous part's curve) and widens the non-periodic
  surface bounds by ten tolerances again at each non-iso part. `BuildIntervals` passes
  `(Tol, TolV)` for both surface directions and raises Standard_OutOfRange for G1 and
  G2; `EvalDN` raises Standard_NotImplemented above order 2.
- `ProjLib_ProjectOnPlane`: a parabola that projects on a line (its X direction along
  the projection) trims the null parabola handle, which it dereferences
  (Standard_NullObject in the port), and a failed B-spline approximation dereferences the null
  curve the same way. A projected end `GeomLib_Tool::Parameter` does not find keeps the
  previous bound. A direction parallel to the plane is not refused (the check is
  commented out) and gives NaN values; the default constructor projects along X onto
  the XOY plane. `Period` returns `false` (0.0) for an approximation, which
  `IsPeriodic` makes unreachable.
