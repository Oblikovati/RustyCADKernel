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

The toolkit test inventory maps these suites (7 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| TopTrans_SurfaceTransition_Test.cxx | toptrans_surface_transition | 2 | pass | pass |
| Intf_Tool_Test.cxx | intf_tool | 5 | pass | pass |
| Geom2dAPI_ExtremaCurveCurve_Test.cxx | geom2dapi_extrema_curve_curve | 1 | pass | pass |
| Geom2dAPI_InterCurveCurve_Test.cxx | geom2dapi_inter_curve_curve | 6 | pass | pass |
| Geom2dAPI_Interpolate_Test.cxx | geom2dapi_interpolate | 1 | pass | pass |
| Geom2dAPI_PointsToBSpline_Test.cxx | geom2dapi_points_to_bspline | 2 | pass | pass |
| Geom2dConvert_BSplineCurveToBezierCurve_Test.cxx | geom2dconvert_bspline_curve_to_bezier_curve | 1 | pass | pass |

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

The numeric corpus has 3221 requests: TopTrans 981 and Intf 2240.

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
