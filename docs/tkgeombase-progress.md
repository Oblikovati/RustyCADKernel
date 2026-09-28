# TKGeomBase port status

**In progress; toolkit acceptance is not complete.** `crates/tkgeombase`
(`rustycad-tkgeombase`) is the port of the ModelingData toolkit TKGeomBase. It
depends on the TKernel, TKMath, TKG2d and TKG3d ports, as the native toolkit
does. Packages are added one module at a time. The `gce`, `GC`, `GCE2d`, `IntAna`,
`IntAna2d`, `LProp`, `GeomLProp`, `GProp`, `Hermit`, `FEmTool` and `AppParCurves` packages are ported in
full, and
`tools/audit_api_rows.py` reports 0 missing and 0 stale api rows for each of them.

The Geom and Geom2d objects that TKGeomBase builds follow the class mapping of
[geom-design.md](geom-design.md).

The toolkit test inventory maps these complete suites, with matching native and Rust
passes (135 declarations):

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

`local_props_behavior.rs` covers the LProp, GeomLProp, GProp and Hermit quirks
below. `gce_behavior.rs` and `gc_behavior.rs` cover the status enumeration, the
StdFail_NotDone results, the conversion operators, the GCE2d aliases and the native
quirks below.

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
their arrays natively). The corpus has 9502 requests (gce, GC, IntAna2d, IntAna, LProp,
GeomLProp, GProp, Hermit, FEmTool and AppParCurves).

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

