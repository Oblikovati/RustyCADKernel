# TKGeomAlgo port status

**In progress; toolkit acceptance is not complete.** `crates/tkgeomalgo`
(`rustycad-tkgeomalgo`) is the port of the ModelingAlgorithms toolkit TKGeomAlgo. It
depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase and TKBRep ports, as the
native toolkit does (its EXTERNLIB). Each of the 33 OCCT packages is one module of the
crate, declared up front in `src/lib.rs`; a module whose package is not ported yet
holds only its documentation. No package is ported in full yet.

Generic C++ code (the `.gxx` and `.pxx` templates that OCCT instantiates per type) is
ported once as a Rust generic and instantiated where OCCT instantiates it, as
[geomalgo-design.md](geomalgo-design.md) fixes.

A package may start when the packages it includes are ported:

| Level | Packages |
| --- | --- |
| 1 | AppBlend, FairCurve, GccEnt, GccInt, Hatch, IntImp, IntRes2d, IntSurf, Intf, Law, LocalAnalysis, NLPlate, Plate, TopTrans |
| 2 | GccAna, HatchGen, IntCurveSurface, IntImpParGen, IntWalk, TopClass |
| 3 | IntCurve, IntPolyh, IntStart |
| 4 | Geom2dInt |
| 5 | Geom2dAPI, Geom2dGcc, Geom2dHatch |
| 6 | IntPatch, then ApproxInt and GeomInt, then GeomFill and GeomAPI, then GeomPlate |

The toolkit test inventory maps these suites (42 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| TopTrans_SurfaceTransition_Test.cxx | toptrans_surface_transition | 2 | pass | pass |
| Intf_Tool_Test.cxx | intf_tool | 5 | pass | pass |
| GccAna_Circ2d3Tan_Test.cxx | gcc_ana_circ2d_3tan | 18 | pass | pass |
| Geom2dAPI_ExtremaCurveCurve_Test.cxx | geom2dapi_extrema_curve_curve | 1 | pass | pass |
| Geom2dAPI_InterCurveCurve_Test.cxx | geom2dapi_inter_curve_curve | 6 | pass | pass |
| Geom2dAPI_Interpolate_Test.cxx | geom2dapi_interpolate | 1 | pass | pass |
| Geom2dAPI_PointsToBSpline_Test.cxx | geom2dapi_points_to_bspline | 2 | pass | pass |
| Geom2dConvert_BSplineCurveToBezierCurve_Test.cxx | geom2dconvert_bspline_curve_to_bezier_curve | 1 | pass | pass |
| IntSurf_LineOn2S_Test.cxx | int_surf_line_on2s | 3 | pass | pass |
| IntSurf_Quadric_Test.cxx | int_surf_quadric | 1 | pass | pass |
| Plate_Plate_Test.cxx | plate_plate | 2 | pass | pass |

## API mapping

- GccEnt: the static members of the `GccEnt` class are free functions of
  `gcc_ent`; overloads take distinct names (`unqualified_lin`/`unqualified_circ`,
  `enclosed_lin`/`enclosed_circ`, `outside_lin`/`outside_circ`), the `bool` overload of
  `PositionFromString` returns `Option`, and `GccEnt_BadQualifier` is a `Failure` of
  that type (`gcc_ent::bad_qualifier`).
- GccInt: the abstract `GccInt_Bisec` is the trait `Bisec`, reached through
  `Handle<dyn Bisec>` (docs/geom-design.md); the five bisectors are structs with plain
  fields, since none has a mutator.
- Hatch: overloads take distinct names (`add_line_at_distance`, `trim_range`,
  `trim_segment`, `nb_intervals_on`, `set_tolerance`), output references become returned
  tuples, and the friend-only data of `Hatch_Parameter` and `Hatch_Line` stays private
  to the `hatch` module.
- GccAna: each native constructor overload is a named constructor returning `Result`,
  named after its argument kinds and locus (`Circ2d3Tan::circ_lin_pnt`,
  `Circ2d2TanOn::lin_pnt_on_circ`, `Lin2d2Tan::tangent_to_circles`...); each
  `GccAna_<Class>_<n>.cxx` file is one submodule. The fixed-size `NCollection_Array1`
  fields are `solutions::Slots`, 1-based with the native bounds check, so a solver that
  overruns its arrays raises Standard_OutOfRange as natively; `Tangencies` holds the
  `TheSame`/point/parameter arrays of one argument. The consultation members keep the
  checks each native class writes (some skip the `WellDone` test, `Circ2d2TanOn`'s
  `ThisSolution` tests nothing), and output references become returned tuples.

## Verification

`python3 tools/verify_toolkit.py TKGeomAlgo --jobs 16 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, then runs `tools/verify_tkgeomalgo_numeric.py`.
While no suite is mapped it runs the numeric verifier alone and reports 0 mapped
declarations. Evidence is in `target/oracle-tkgeomalgo/`. CI runs it like the TKBRep
verifier.

## Deliberate Rust deviations

- TopTrans: the sector arrays of `TopTrans_SurfaceTransition` (NCollection_Array2 of
  trivial types) are uninitialized natively. The angles are reset before any read; the
  orientations and curvatures are read only after a write, except the curvature of a
  sector that only the plane `Compare` wrote, which the curved `Compare` then reads
  (indeterminate natively). The port starts them at 0 and FORWARD.
- TopTrans: `SurfaceTransition::reset`, `reset_line`, `compare` and `compare_plane`
  return `Result`, carrying the native Standard_ConstructionError.
- Intf: undefined native behaviour gets a defined result: an empty tangent zone's
  `InfoFirst`/`InfoSecond` cast `RealLast` to int as x86-64 does (INT_MIN, checked
  against the native probe); a seventh `HyprBox` segment raises Standard_OutOfRange
  where native code writes past its arrays. Members that index records return
  `Result` with the native Standard_OutOfRange or Standard_NoSuchObject. `Dump`
  writes to a `fmt::Write` where native code writes to `std::cout`.
- Intf: `Intf_Polygon2d` is the trait `Polygon2d`; `Intf_Interference` is held by the
  interferences and reached through `Deref`/`DerefMut`, its protected members through
  `section_points_mut`, `section_lines_mut`, `tangent_zones_mut` and `set_tolerance`.
  The generic `.gxx` is `InterferencePolygonPolyhedron<PT, HT>` over the
  `Polygon3dTool` and `PolyhedronTool` traits ([geomalgo-design.md](geomalgo-design.md)
  section 2); IntCurveSurface and HLRBRep instantiate it when they are ported.

## IntRes2d, IntImpParGen and IntCurve

IntRes2d (`int_res2d`), IntImpParGen (`int_imp_par_gen`) and IntCurve (`int_curve`) are
ported in full:
`tools/audit_api_rows.py` reports 0 missing and 0 stale rows for each. The root class
`IntRes2d_Intersection` is a struct that the algorithms embed and reach through `Deref`;
its protected members (`Insert`, `Append`, `SetValues`, `ResetFields`, the `done` field)
are public so that algorithms of later toolkits can derive from it. Every constructor
overload is a named constructor and every `Perform` overload of IntCurve_IntConicConic a
`perform_<pair>` method; accessors that raise return `Result` with the native exception
type. IntImpParGen_Intersector.gxx is a generic (docs/geomalgo-design.md section 3), and so
are the IntCurve curve templates (sections 4 and 5).

Preserved native behaviour:

- `Clone` of `IntRes2d_Intersection` (and of the algorithms embedding it) is the native
  copy constructor: the copy has the results but is neither done nor reversed;
  `Intersection::assign` is the assignment operator, which copies both flags.
- `IntRes2d_Intersection::Append` merges composite results with references into the
  segment it rewrites, so a later branch of the same iteration reads the rewritten
  segment; a joined segment keeps the positions of the appended part, an appended one
  takes the relabelled positions.
- `IntRes2d_Domain::EquivalentParameters` does not raise on an open domain (0 and 0).
- `IntCurve_PConicTool::D2` of a line does not write the second derivative; the port
  writes through `&mut` outputs so that the intersector reads the stale value as natively.
- `IntCurve_IntConicConic::Perform(gp_Lin2d, gp_Parab2d)` offsets the line along
  `(dy, dx)` of its direction (not its normal); its bounded branch searches with the
  exact tolerance 1e-7 first, then again with the given ones when no point is found.
- `Perform(gp_Parab2d, gp_Hypr2d)` keeps the 50 samples of a hyperbola PConic (the other
  overloads set 20).
- `SET_BOUNDED_DOMAIN` compares the upper bound with the domain's FIRST parameter, which
  raises Standard_DomainError on a domain with only a last point.
- The line-ellipse overload evaluates the ellipse at the first bound of an arc for its
  second point and, when that point is at the end of the ellipse domain, takes the LINE
  domain's last parameter.
- The line-circle and line-line overloads ignore `TolConf` where the native code does,
  and the tube of `LineCircleGeometricIntersection` uses `R` for both radii (`Patch !!!`).
- The circle-circle, line-circle and line-ellipse overloads store their arcs in native
  arrays of 4 that numerical cases could overflow; the port keeps every arc in a vector.
- A default `IntCurve_IConicTool` (no curve) writes "Erreur sur le type de la courbe" to
  std::cout from its members; the port does not print (the probe discards std::cout).
- `IntCurve_IConicTool(gp_Lin2d)` and `IntCurve_PConic(gp_Lin2d)` return `Result` because
  the port's `gp_Ax22d` constructor does; a line direction cannot make it fail.

## Geom2dInt and Geom2dAPI

Geom2dInt (`geom2d_int`) holds the curve tool `Geom2dCurveTool`, the projection
`TheProjPCurOfGInter` and the IntImpParGen instantiation `Geom2dTypes`; its other classes
are aliases of the IntCurve and Extrema generics (docs/geomalgo-design.md sections 4 and
5), among them the curve-curve intersector `GInter`. Geom2dAPI (`geom2d_api`) ports
`ExtremaCurveCurve`, `InterCurveCurve`, `ProjectPointOnCurve`, `Interpolate` and
`PointsToBSpline`; the conversion operators are `as_int`, `as_real`, `as_point` and
`as_curve`. Both packages are ported in full: `tools/audit_api_rows.py` reports 0
missing and 0 stale rows for each.

Preserved native behaviour and deliberate differences:

- `Geom2dInt_Geom2dCurveTool::IsComposite` and `Geom2dAPI_Interpolate::ClearTangents` are
  declared but never defined natively; they have excluded rows.
- Extrema_ExtCC2d and Extrema_ExtPC2d borrow their curves, which the Geom2dAPI objects own:
  the objects keep the results of their construction (with the exception each Extrema
  accessor raised) and `extrema()` rebuilds the deterministic Extrema object on the stored
  adaptors.
- `Geom2dAPI_Interpolate::Load(Tangents, TangentFlags)` shares the caller's flag array,
  which Perform completes (the end tangents it builds are flagged there, visible to the
  caller); BuildParameters reads the previous parameter at the point index, so a point
  array not starting at 1 raises Standard_OutOfRange; `Load(InitialTangent, FinalTangent)`
  writes the flags and tangents at 1 and NbPoints whatever the bounds. PLib::EvalLagrange
  leaves its result unset natively when two parameters are too close; the port reads 0.
- `Geom2dAPI_PointsToBSpline::Init(Points, Params, ...)` reads `Params(i)` for i in
  `2..Nbp-1` whatever the lower bound of `Params`; the variational Init swallows the
  exception of AppDef_Variational::Approximate, as natively.
- IntCurve_IntPolyPolyGen recurses on a tangent zone with `Tol` as the confusion
  tolerance and `TolConf` as the tolerance (the arguments are swapped natively), and
  tests the full polygon representation as `NbSegments() == sample count`, which the
  constructor never gives (a polygon of n samples has n - 1 segments), so the second
  search on full polygons runs whenever the first one misses a root.
- The error flag of IntCurve_ExactIntersectionPoint is never reset: once one solve fails,
  IntPolyPolyGen skips every later section point of the same interference.
- The self intersection keeps only the last of the section points on the same pair of
  chords and gives nothing for a tangent zone whose ends are within EpsX (the pair
  intersection inserts a point there). Its end scan (PosSegment) is computed and
  discarded, as natively.
- HeadOrEndPoint takes the transitions of an end point at the original parameters: the
  second-order test replaces the snapped end point by the curve point there.
- IntCurve_IntCurveCurveGen splits composite curves by the native tail recursion over the
  intervals of the second curve (ported as a loop); a clipped interval of at most 1e-10
  (or NaN) length is skipped. Its self intersection of a curve with both bounds infinite
  returns not done without resetting the previous results.
- `ApproxParamOnCurve` with an index past the last point prints "OutOfRange
  Polygon2d::ApproxParamOnCurve" on std::cout natively; the port does not print (the probe
  discards std::cout). The polygon index slots past the sample count are uninitialized
  natively and 0 in the port, where reading one raises Standard_OutOfRange. Only a chord
  index past the last point reads them (for example a seat given to
  IntCurve_ExactIntersectionPoint by the caller); the intersectors pass only the chords of
  the interference, so the corpus leaves that undefined case out.
- `GetIntersection` leaves its sample indices at -1 when every distance is NaN and then
  reads out of bounds if the current distance exceeds sqrt(RealLast); the port reads the
  first sample.

## Numeric comparison policy

`tools/verify_tkgeomalgo_numeric.py` uses the shared driver `tools/toolkit_numeric.py`.
It builds `tools/oracle/tkgeomalgo_probe.cxx` (a table of operations over
`tkgeomalgo_probe_<package>.cxx`, linked with libTKGeomAlgo and the toolkits below it)
and its twin `crates/tkgeomalgo/examples/numeric_probe.rs` (one module per probe file
under `examples/ga_probe`), runs both on `tools/tkgeomalgo_corpus.py` (one
`tkgeomalgo_corpus_<package>.py` per package) and requires identical tokens: doubles as
IEEE bits (NaN by classification), exception types by name. The corpus runs in 64
round-robin shards (`tools/probe_shards.py`), so every request prints exactly one line
and never relies on an earlier request. `tools/coverage_report.py` replays the saved
corpus under coverage.

The numeric corpus has 11633 requests: TopTrans 981, Intf 2240, GccEnt 55, GccInt 15,
Hatch 38, GccAna 2670, IntRes2d with IntImpParGen and IntCurve 2044, Geom2dInt and
Geom2dAPI 962, the curve-curve classes 1077, IntSurf 334, Law 266, Plate 106,
LocalAnalysis 406, FairCurve 178, IntImp 146 and AppBlend 115.

Intf (2240 requests): `intf_triangle` compares `Intf::PlaneEquation` and `Intf::Contain`
on 54 triangles (both orientations, coincident and collinear vertices, a normal that
underflows below `gp::Resolution`, huge coordinates, points on edges and vertices,
outside and off the plane). `intf_spoint`, `intf_sline`, `intf_tzone` and
`intf_interference` run seeded scripts over every member of the records and of
`Intf_Interference` (the protected members through a probe subclass), with section
points drawn from a few logical identities so that equal points, shared line ends
and shared zone edges are frequent, and with out-of-range indices and empty records
for the exceptions; the dumps are captured text. `intf_tool2d` and `intf_tool3d` run
one to three box computations on one `Intf_Tool` for lines (with exactly zero
components), hyperbolas and parabolas over void, whole, finite and open domains,
tangent faces included. `intf_ipoly2d` runs the performs and constructors of
`Intf_InterferencePolygon2d` on crossing, overlapping, near-parallel, touching,
degenerate and self-crossing polylines. `intf_ipp` instantiates the generic
`Intf_InterferencePolygonPolyhedron.gxx` in the probe over a polygon and a
triangulated height field (with its outer edges as boundary edges, and a degenerate
triangulation), and runs every Perform, Interference and constructor overload, with
and without the grid.

TopTrans (981 requests): `tt_curve` and `tt_surface` run a command sequence (resets
and compares) on one fresh transition and compare the exception type and both states
after every command; `tt_states` compares the static state tables. Directions come
from discrete fans (multiples of 45 and 30 degrees, near-tolerance tilts), so equal
angles, tangent faces, the touch case, the incoherent sectors and the parallel-frame
exceptions all occur; the hand-made sequences cover the upstream scenarios and the
curved ties at a null cosine. A surface request uses only one Compare overload (see
the uninitialized sector curvature in the deviations above).

GccEnt: `gccent_position` feeds the five positions to `GccEnt::PositionToString` and
`GccEnt::Print` with eight names for both `PositionFromString` overloads (case variants,
the empty string, an unknown name, a trailing blank); `gccent_qualified` builds qualified
circles and lines with every position and the seven GccEnt makers (55 requests).

GccInt: `gccint_bisec` builds the five bisector kinds on three frames and calls all six
accessors, the RTTI name and `IsKind` (15 requests).

Hatch: `hatch_hatcher` adds lines of the four kinds, trims them with whole lines, parameter
ranges and oriented segments, and queries every line and interval including the
out-of-range indices around them: a closed square in both modes and orientations, open
and infinite intervals, polygon vertices on hatching lines, two intersections closer than
the tolerance, parallel and identical trimming lines, degenerate segments, segments near
and beyond RealLast, a negative tolerance, and 24 seeded random polygons (38 requests).

GccAna: `gccana_circ3tan`, `gccana_2tanrad`, `gccana_2tanon` and `gccana_tanonrad` run
every constructor overload of Circ2d3Tan, Circ2d2TanRad, Circ2d2TanOn and Circ2dTanOnRad
on hand-built configurations (tangent, nested, concentric, secant, parallel, coincident
and far arguments, each qualifier including the rejected ones, the upstream suite's
cases) and on seeded random arguments; `gccana_tancen`, `gccana_lin2tan`,
`gccana_lintanobl`, `gccana_linbisec` and `gccana_bisec` do the same for Circ2dTanCen,
Lin2d2Tan, Lin2dTanObl, Lin2dBisec and the five bisector classes. Each request prints
the solution count and order and, from index 0 to NbSolutions + 1, every solution and
the consultations its detail mask selects (2670 requests).

IntRes2d, IntImpParGen, IntCurve: `tkgeomalgo_corpus_intcurve.py` adds 2044 requests:
`intcurve_conics` intersects every ordered conic pair of IntCurve_IntConicConic
(crossing, tangent, coincident, disjoint and nearly tangent pairs; direct and indirect
frames; closed, open, wrapped, semi-infinite and unbounded domains; both tolerances and
the reversed flag; 150 seeded random pairs); `intcurve_imppar` runs
IntCurve_IntImpConicParConic on every implicit/parametric kind with its sample count and
EpsX; `intcurve_itool`, `intcurve_ptool` and `intcurve_function` evaluate the tools,
including the centre of a circle or ellipse and the default tool; `intcurve_interval`
covers Interval and PeriodicInterval; `imppar_transition` and `imppar_domain` the
transition, position and normalisation helpers; `intres2d_*` the domain accessors,
Insert and the composite Append.

Geom2dInt curve-curve, Geom2dAPI_InterCurveCurve: `tkgeomalgo_corpus_ginter.py` adds 1077
requests: `g2int_polygon` builds polygons of every parametric curve with 2 to 40 samples,
refines them in overlapping, far and touching boxes and closes them; `g2int_exact` runs
IntCurve_ExactIntersectionPoint from inner, first, last and out-of-range polygon seats and
from explicit bounds; `g2int_polypoly` intersects crossing, tangent, overlapping,
disjoint and self-crossing curves (a Bezier loop, a periodic figure eight) through the
polygon intersector, its self intersection and its protected iterations (up to past the
last one); `g2int_ginter` runs Geom2dInt_GInter on every pair of 22 curves (conics,
Bezier, B-spline with C0 knots, periodic, rational, trimmed and offset curves) through
every constructor and Perform, with ComputeDomain, inner, semi-infinite and infinite
domains; `g2api_intercc` runs Geom2dAPI_InterCurveCurve with its points, trimmed
segments and out-of-range indices, including the upstream OCC23706 and OCC29289 curves.

Geom2dInt, Geom2dAPI: `tkgeomalgo_corpus_geom2d.py` adds 962 requests: `g2int_tool` and
`g2int_proj` run the curve tool and the projection on lines, conics, rational and
polynomial Bezier and B-spline curves (several C1 intervals, periodic), trimmed and offset
curves; `g2int_conic_curve` intersects every conic with every such curve through both
conic-curve classes (whole, inner and unbounded domains, the reversed flag);
`g2int_functions` evaluates the distance and conic functions; `g2api_extrema`,
`g2api_project`, `g2api_interpolate` and `g2api_points_to_bspline` call every member of the
four Geom2dAPI classes, with out-of-range indices, reversed ranges, periodic and
non-periodic interpolations with parameters and tangents, and every PointsToBSpline Init.

IntSurf: 334 requests. `intsurf_quadric_point` and `intsurf_quadric_uv` evaluate every
`IntSurf_Quadric` member and `IntSurf_QuadricTool::Tolerance` for the five quadric
kinds (including a cylinder and a cone of radius 0) and the undefined quadric, on
direct, indirect and oblique frames, at the frame origin, on the axis, on the torus
core circle and at seeded random points; `intsurf_transition` drives
`IntSurf::MakeTransition` through its four branches and `intsurf_same` compares
`IntSurf_PntOn2S::IsSame` with and without the 2d tolerance.

Law: 266 requests. `law_bspline` builds scalar B-spline laws (clamped, rational,
periodic, piecewise Bezier, uniform, C0, quintic and invalid data) and compares every
query and evaluator at the knots, inside, at the ends and outside the range;
`law_edit` applies each editing member (degree elevation, knot insertion and removal,
reversal, segmentation, periodicity, origin, weights, knots, poles,
MovePointAndTangent, Copy) to five laws; `law_function` drives the constant, linear,
S, B-spline, interpolated and composite laws through every `Law_Function` member for
every `GeomAbs_Shape`; `law_mix` covers `MixBnd`, `MixTgt`, `Scale`, `ScaleCub`,
`Reparametrize` (on a circle adaptor) and `Law_BSplineKnotSplitting`;
`law_interpolate` covers `Law_Interpolate` with and without parameters and tangents.

Plate: 106 requests. `plate_solve` loads pinpoint (values and derivatives up to order
2), linear XYZ and linear scalar constraints, solves orders 1 to 10 (the three `SolveTI`
systems and the refused orders, anisotropies, empty and degenerate boxes) and compares
`IsDone`, `Continuity`, `UVBox`, `Evaluate`, four `EvaluateDerivative`s, `CoefPol` and
`UVConstraints` for the plate, its copy and an assigned plate. `plate_constraint`
builds the line, plane, global translation, sampled curve, GtoC (G1/G2/G3, with and
without projection) and FreeGtoC (G1/G2/G3, incremental loads, orientations and the
degenerate normals) constraints, compares their content and the plate that loads them.

LocalAnalysis: 406 requests. `la_curve` compares every query of
`LocalAnalysis_CurveContinuity` and its `Dump` text for the seven `GeomAbs_Shape` orders
and two tolerance sets on joined and broken pairs of lines, circles and Bezier curves
(tangent and reversed joins, curvature jumps, null first and second derivatives,
undefined tangents, straight pieces); `la_surface` does the same for
`LocalAnalysis_SurfaceContinuity` at points, on two 2d lines and through
`ComputeAnalysis`, on planes, cylinders, spheres and tori (tangent and broken joins,
poles, umbilics).

FairCurve: 178 requests. `fc_law` evaluates `FairCurve_BattenLaw` before and after its
setters, with heights that turn negative. `fc_density` compares the tension, sagging
and jerk densities with their gradients and packed Hessians (derivative orders 0 to 2,
with and without the free-sliding value, uniform tension, a vanishing law) on a Bezier
and a two-span B-spline of degree 9. `fc_energy` compares `Variable`, `Value`,
`Values` and the Hessian of `FairCurve_EnergyOfBatten` and `FairCurve_EnergyOfMVC` for
every pair of end-constraint orders, at the initial and at perturbed variables (the
clamped and the infinite sliding), and minimizes them with `FairCurve_Newton`.
`fc_batten` computes batten and minimal-variation curves (end angles that need several
homotopy steps, free and imposed slidings, end curvatures, physical ratios, null
heights, invalid heights and ratios, confused points), moves their end points and
computes again, comparing every getter, the curve and the `Dump` text.

IntImp: 146 requests. `ii_choix` covers `ChoixRef` inside and outside 0..3;
`ii_tangence` drives `IntImp_ComputeTangence` through its early exits (null
derivatives and normals), tangent patches and the sorted isoparametrics, with seeded
random derivatives. The templates run through their native TKGeomAlgo instances and
the Rust generics: `ii_zerpar` (`IntWalk_TheFunctionOfTheInt2S`), `ii_int2s`
(`IntWalk_TheInt2S` from a start point and on each isoparametric, inside and beyond
the patch bounds, on tangent patches), `ii_zerimp` (`IntPatch_TheSurfFunction` on
patches against quadrics) and `ii_intcs` (`IntCurveSurface_TheExactHInter` on lines and
circles against planes, cylinders, spheres and tori, with and without margin).

AppBlend: 115 requests. `ab_appsurf` instantiates `AppBlend_AppSurf.gxx` on a probe
line and section generator (rational quadratic sections with 0 to 2 extra 2d curves,
with or without derivatives) and compares every query after `Perform`,
`Perform(SpApprox)`, `PerformSmoothing` and `Perform(NbMaxP)`, each run twice, with
known parameters, every parametrization and continuity, the no-tangent switch,
invalid criterion weights and a null line; `ab_context` covers the two switches of
`AppBlend_Debug.cxx`.

## Preserved native behaviour

- TopTrans: a `CurveTransition` that was never reset answers the states of its
  stored FORWARD transitions (OUT before, IN after), not UNKNOWN: natively `Init` is
  false after construction. `Reset(Tgt)` of a straight line keeps the previous
  normal. The default `gp_Dir` members are +X.
- TopTrans: `SurfaceTransition::Reset` stores the tangent and the normal before the
  cross product that raises Standard_ConstructionError for parallel ones; the
  transition then stays undefined. A tangent that follows no principal direction
  leaves it undefined (not implemented natively, "NYIxpu"). A non-positive `Tole`
  means `Precision::Angular`. The curved `FUN_refnearest` compares curvatures with a
  fixed `1.e-4`, not with the tolerance. The second pass of an INTERNAL curve takes
  the opposite sector of the indices that the first pass left behind, as natively.
- TopTrans: the header declares the private `UpdateReference` and `ComputeCos`, which
  have no definition in OCCT; they have no Rust counterpart.
- `Intf::PlaneEquation` of a degenerate triangle (normal shorter than
  `gp::Resolution`) returns the undivided normal and a polar distance of 0.
- Intf records: `Intf_SectionPoint::IsOnSameEdge` compares the second argument's first
  addresses in the vertex-vertex case of the first argument. `Intf_SectionLine::IsEnd`
  returns the number of points (not 2) for the last point; `IsClosed` compares the
  first and last points (the `closed` flag is never read, and the copy constructor
  drops it); `Append`/`Prepend` of a line empty the argument. The `Dump` of section
  points and tangent zones prints nothing (their bodies are compiled out), and
  `Intf_TangentZone::Insert` returns false (its body is commented out);
  `PolygonInsert` only appends, or prepends a point inside the first range.
- `Intf_Interference::Insert(TangentZone)` inserts the shared point a second time when
  its walk comes back to it, then re-inserts the grown zone recursively.
- `Intf_InterferencePolygon2d`: the self Perform keeps the segment count of the last
  two-polygon computation; `Clean` never resets its one-segment flag between zones;
  the suppression of close candidate points shifts the points but not their
  parameter arrays.
- `Intf_Tool`: the segment arrays keep their values between calls; `Hypr2dBox`,
  `Parab2dBox` and `ParabBox` sort the parameters and border codes but not the
  coordinates; `Inters3d` merges equal parameters inside its exchange sort, so one
  pair can be removed several times; `HyprBox` bounds its segments to [-10, 10] and a
  whole domain to [-100, 100].
- The generic polygon-polyhedron interference classifies the plane point on the
  first vertex, then the first edge, before the second vertex, so a line through the
  second corner of a triangle is an EDGE point of the first edge; `TriConnex` is
  called on an edge hit and its result is unused.
- `GccEnt_QualifiedCirc::Qualifier` and `GccEnt_QualifiedLin::Qualifier` return the stored
  `GccEnt_unqualified` for an unqualified argument, although the header documents
  `GccEnt_noqualifier`.
- `GccInt_Bisec` accessors raise `Standard_NotImplemented`, not the `Standard_DomainError`
  that the header documents.
- `Hatch_Line::AddIntersection` removes an existing intersection closer than the tolerance
  and drops the new one (akm OCC109), so two close crossings cancel each other.
- `Hatch_Hatcher::StartIndex` and `EndIndex` accept J = 1 on a line without intersections
  in unoriented mode (they return 0, 0), where `Start` and `End` raise
  `Standard_OutOfRange`. Every line and intersection index outside its sequence raises
  `Standard_OutOfRange`, as the native `NCollection_Sequence` bounds check does.
- `Hatch_Hatcher::Trim(P1, P2)` scales a segment vector whose X (else Y) exceeds
  0.9 RealLast by the inverse of that component; a vector that overflowed to infinity
  becomes NaN and the segment is ignored.
- The GccAna solvers keep their native slips: Circ2d3Tan(circle, circle, line) tests
  Qualified2 instead of Qualified3 for its outside and unqualified lines; the
  three-circle solver measures the qualifier distances of circles 2 and 3 from the
  centre of circle 1; Circ2d3Tan(line, line, line) compacts its solutions without
  moving their qualifiers; Lin2d2Tan(circle, circle) stores some qualifiers in slot 1
  again and copies Qualified1 into qualifier2; Lin2dTanObl(circle) intersects the first
  solution for the second; Circ2d2TanOn(circle, circle, line) qualifies its general
  solutions with the radius at the bisector index (a third bisector raises
  Standard_OutOfRange) and computes the first argument parameter at the second tangency
  point; Circ2d2TanOn(line, line, circle) builds its circles at intersection point `i`;
  Circ2dTanOnRad(line, circle) truncates its side test to an integer; the `CenterOn3`
  members return the tangency point, not the centre; Circ2dTanOnRad(circle, circle)
  raises Standard_OutOfRange in its single-centre case (it writes qualifier slot 0).
- Some native paths leave array slots uninitialized and later read them (the Rbid(2)
  radius of an unqualified first circle in Circ2d2TanOn, TheSame1 of the nbsol 3 case of
  Circ2d2TanRad(circle, line), TheSame2 of an enclosing and an unqualified circle in
  Circ2d2TanRad, the side cote3(2) of two unqualified-line triples in
  Circ2d3Tan(line, line, line), the slot-2 data of Circ2dTanOnRad(point, circle), the
  argument parameters of Circ2d2TanOn(circle, point, line), the qualifier-2 slots of
  Lin2d2Tan(circle, circle) and the qualifier of Circ2d2TanRad(line, point) beyond the
  diameter). The port starts every slot at zero (default positions and circles), which
  is deterministic; the numeric corpus avoids reading those slots, since their native
  value is whatever the allocator returned.

### IntSurf

- `IntSurf_Transition::SetValue(Tangent, Type)` and `SetValue()` change only the type
  (and tangent flag): the situation and opposite flag of an earlier TOUCH transition
  stay. `IntSurf::MakeTransition` writes its results with them, so the Rust
  `make_transition` updates two `&mut IntSurfTransition` in place instead of returning
  fresh values.
- `IntSurf_Quadric::Normale(U, V)` of a cone is the unnormalized `D1u ^ D1v` (null when
  `|D1u| < 1e-7`) and ignores the frame handedness, unlike every other normal of the
  class. The plane constructor leaves the axis line at its default (the Z axis).
- `IntSurf_Quadric::D1`, `ValAndGrad` and `Parameters` of the undefined quadric leave
  their output arguments unchanged; the Rust members return `None` there.
  `Normale(P)` raises `Standard_ConstructionError` on a cylinder axis and at a sphere
  centre (the `gp_Dir` of a null vector), and so does `Normale(U, V)` on a cylinder of
  radius 0.
- `IntSurf_LineOn2S::Split` gives the new line the default allocator, not the
  allocator of the split line, and keeps the (possibly stale) boxes of the split line.
- A default-constructed `IntSurf_PathPoint` holds a null parameter sequence that
  `Value2d`, `AddUV`, `Multiplicity` and `Parameters` dereference natively; the port
  returns `Standard_NullObject` there instead of crashing. A copy of a path point
  shares the parameter sequence, as the native copy of the handle does.

### Law

- `Law_BSpline` is an older copy of `Geom_BSplineCurve` with real poles, and keeps its
  differences: `DN` evaluates at the given parameter while `D0` to `D3` normalize a
  periodic parameter first; `LocateU` tests the given, not the normalized, parameter
  against the first and last knots; `IncreaseMultiplicity` inserts with `Add = false`.
  `InsertKnots` and `RemoveKnot` natively pass the stored knots and multiplicities as
  their own outputs when the knot count does not change; the port writes fresh arrays,
  which gives the same result because BSplCLib reads each entry before rewriting it
  and `NbPoles` forces equal first and last multiplicities on a periodic law.
- `Law_BSpline::MovePointAndTangent` on a periodic law makes BSplCLib write past the
  new pole array (undefined behaviour natively); the port raises
  `Standard_OutOfRange`. The `Local*` evaluators pass knot indices to the flat-knot
  `LocateParameter`; when the located flat index exceeds the knot count,
  `BSplCLib::FlatIndex` reads past the multiplicities natively and the port raises
  `Standard_OutOfRange`. The corpus avoids both undefined cases.
- `Law_BSpFunc::NbIntervals` tests the first split knot against the range, but
  `Intervals` first overwrites it with the start index, so the two can disagree;
  `Intervals` writes `T(1..n+1)` at absolute indices, then `T(Lower)` and
  `T(Lower + n)`. `Law_Composite::Intervals` writes `T(1)`, `T(2)` and the following
  entries at absolute indices too, and `Law_Composite::Continuity` raises
  `Standard_NotImplemented`.
- `Law_Composite::ChangeElementaryLaw` and `ChangeLaws` return references natively;
  the port lends the current law and the list to a closure.
- `Law_Interpolate` builds chord-length parameters reading `Parameters(ii)` with the
  point index `ii` (correct for one-based points only), stores the tangent flags that
  `Load` receives before it checks their length, shares that flag array (Perform marks
  the end tangents it computes in it) and never reads its tolerance.
  `Law_Interpolate::ClearTangents` is declared but never defined at this commit, so its
  row is excluded.
- `rustycad_tkgeombase::geom2d_convert::split_knots` (the split table of the knot
  splittings) is public so that `Law_BSplineKnotSplitting` shares it.

### Plate

- `SolEm`'s derivative table (the closed forms of the fundamental solution for every
  `(IU, IV)` with `IU + IV <= 7`) is generated from the native switch into
  `plate/sol_em_terms.rs`: integer factors become the equal `f64` literals and the
  products keep their left-to-right order, so every value is bit-identical (the probe
  compares it through all the solves). `SolEm` keeps the native point cache
  (`Uold`, `Vold`, `U2`, `R`, `L`) behind a lock; a fresh cache holds `(1e20, 1e20)` with
  `R = 0`, so a first call at exactly that scaled point returns 0, as natively.
- `SolveTI2` allocates its right-hand side once, so the linear XYZ rows accumulate the X,
  then Y, then Z terms with `+=`; the port keeps the accumulation. `SolveTI` sets the
  order before it refuses an invalid request, so `CoefPol` of a refused plate reads a
  missing solution: the port raises `Standard_NullObject` (a null dereference natively).
  `IsDone` is true after `Init`, as the native test documents.
- `Plate_Plate::UVBox` (and so `SolveTI`) reads the first pinpoint constraint: without
  one the port raises `Standard_OutOfRange`, as the native checked sequence access.
  Derivative orders past the ten scale factors `ddu`, `ddv` (undefined natively) raise
  `Standard_OutOfRange`.
- The linear constraints share their arrays through handles, so `SetPPC` and `SetCoeff`
  on a copy (or on the constraint after `Load`) are seen by the loaded plate; a
  default-constructed linear constraint has no arrays and `Load` raises
  `Standard_NullObject`. `Plate_SampledCurveConstraint` builds its arrays before it
  raises `Standard_DimensionMismatch` for `n > m`.
- `Plate_GtoCConstraint` with `D3` and `nP` ignores `nP`, as natively. The GtoC and
  FreeGtoC constructors stop with the constraints built so far on a degenerate or
  orthogonal normal; a FreeGtoC incremental load between parallel normals raises
  `Standard_ConstructionError` (the null rotation axis). `GetPPC` and `LSC` beyond their
  fixed arrays raise `Standard_OutOfRange` (undefined natively). The G1/G2/G3 blocks,
  which the native constructors repeat verbatim, are written once in `gtoc_math.rs`.

### LocalAnalysis

- `StatusError` is uninitialized natively until an analysis fails, and
  `ContinuityStatus` of a surface analysis built from tolerances only is uninitialized
  until `ComputeAnalysis`: the port returns `None` and `Standard_NoSuchObject` there.
  The surface analysis on two 2d curves fails without a reason when the parameter is
  outside either curve, so its `Dump` prints only its first line.
- The two `Dump` reports keep their native texts, including the two spaces of
  "one of the  second derivatives is null", the blank line after the surface G1 values
  and the reasons each report omits (TangentNotDefined for surfaces,
  CurvatureNotDefined for curves). Reals are written as the default `std::ostream`
  (`%g`).
- Every angle is folded into `[0, PI/2]` when it is measured, so the curve tests'
  second clause `|angle - PI| <= eps` never holds; it is kept, as natively. `IsG2` of a
  curve compares the fractional part of `(G2Angle + EpsG2) / PI` with `EpsG2`.

### FairCurve

- The densities keep copies of the batten law: the energies' own `SetSliding` reaches
  no density, and only the tension receives the sliding length (`SetLengthSliding`), so
  the sagging and jerk heights use the sliding of the step's law. The uniform tension
  of the minimal-variation energy freezes the middle height at construction, whatever
  its sign.
- `FairCurve_EnergyOfBatten::Compute` records a failed sagging integral as
  `Ok = false` and goes on; a later interval can set it back. The minimal-variation
  energy returns at the first failed integral.
- The sliding guard of both energies reports `InfiniteSliding` beyond ten times the
  original length and clamps it below one hundredth; the clamped length is what
  `Variable` writes back.
- `FairCurve_Energy::Hessian1` reads the intermediate values `Vect(5)` and `Vect(6)`
  for the start G2 term and `Vect(jj)` for the end one; the jerk Hessian omits the
  `ProduitV` and `HNumRho` terms in its XiXj and YiYj entries and mixes the (ii, ii-1)
  and (ii-1, ii) orders in its diagonal XiYi entry. All are kept.
- `SlidingOfReference` tests the requested (`New`) angle of the opposite end, not its
  argument, before halving it for an unconstrained end.
- After a successful minimization the batten keeps the poles of the last energy
  evaluation (not of the returned location), and reloads the variables from the energy
  before the convergence and infinite-sliding checks. A failed minimization returns
  false with the energy status as the code, or true with `NullHeight` when the law
  vanishes at an end.
- `FairCurve_MinimalVariation` does not check its physical ratio at construction; the
  first `Compute` raises the `Standard_DomainError` of `FairCurve_EnergyOfMVC`. NaN
  passes both checks.
- `Dump` pads only the old value to 7 characters (`o.width(7)`), writes flags as 0/1
  and reals as the default `std::ostream`.
- The FairCurve sources define `No_Standard_RangeError` and `No_Standard_OutOfRange`:
  an index out of range is undefined natively; the port panics there. The knot
  refinement computes its native `int` products in `i64` (their overflow is undefined
  natively and unreachable with finite angles).
- The batten constructor passes a 2-entry weight array to a non-rational
  `BSplCLib::IncreaseDegree`, which never writes it; the port passes none.

### NLPlate

- NLPlate has no sources at this commit: `PACKAGES.cmake` lists it, but its
  `FILES.cmake` is empty since OCCT commit caad597813 ("Clean up dead headers #1442")
  deleted every NLPlate file. Only two deprecated aliases under
  `src/Deprecated/NCollectionAliases` refer to it: `NLPlate_StackOfPlate.hxx` (a
  typedef of `NCollection_List<Plate_Plate>`) and
  `NLPlate_SequenceOfHGPPConstraint.hxx`, which includes the deleted
  `NLPlate_HGPPConstraint.hxx`. The audit finds no public member;
  `crates/tkgeomalgo/src/nl_plate` stays an empty module.

### IntImp

- The five `.gxx` templates are generics (docs/geomalgo-design.md, "IntImp
  templates"); their tool calls can raise, and the failure reaches the caller of
  `IntImpInt2S`/`IntImpIntCS` as the native exception would.
- `IntImp_IntCS::Perform` is declared with the bounds `u0, v0, u1, v1` but defined
  (and called) with `u0, u1, v0, v1`; the port follows the definition.
- The bound check of `IntImp_Int2S::Perform` re-reads the moved start parameters, not
  the new solution, when it checks the reciprocal patch, and compares the second
  parameter there without the resolution. It then restarts on `ChoixRef` of the
  1-based index of the moved parameter (4 wraps to 0): the isoparametric of the next
  parameter, where the first check fixes the moved one.
- `IntImp_ZerImpFunc::IsTangent` uses the gradient of the last `Derivatives`/`Values`
  (zero before any): after `Value` alone it reuses a stale one. `Direction2d` of a
  vanishing 2d tangent raises `Standard_ConstructionError` from `IsTangent`.
- `IntImp_ZerParFunc` reads its isoparametric before `ComputeParameters` sets it
  (uninitialized natively; U on the first patch here), and the second
  `IntImp_Int2S` constructor leaves `tangent` uninitialized (false here).

### AppBlend

- Two sections narrow the stored degrees (`dmin = 1`, and `dmax = 1` without
  tangents): later computations of the same object keep them.
- `Perform` without `SpApprox` computes the reached tolerances but does not store
  them (`TolReached` keeps its earlier values), and with known parameters it raises
  `StdFail_NotDone` from `AppDef_Compute::SplineValue`.
- The continuity chain of the B-spline approximation turns C0 into continuity 3
  (`if C0 ...; if C1 ... else if C2 ... else 3`).
- `Perform(NbMaxP)` measures the pole distances of each Bezier piece on the curve of
  the piece's own index, and `NbMaxP = 0` divides by zero (a crash natively; a
  `Standard_DivideByZero` error here, not in the corpus). The native `scal` switch of
  its box scaling is a constant true.
- `Curves2dShape` returns the U pole count of the surface as the 2d pole count.
- The `AppBlend_Debug.cxx` switches are process-wide, as the native file statics.
