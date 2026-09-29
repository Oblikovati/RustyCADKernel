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

The toolkit test inventory maps these suites (25 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| TopTrans_SurfaceTransition_Test.cxx | toptrans_surface_transition | 2 | pass | pass |
| Intf_Tool_Test.cxx | intf_tool | 5 | pass | pass |
| GccAna_Circ2d3Tan_Test.cxx | gcc_ana_circ2d_3tan | 18 | pass | pass |

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

The numeric corpus has 5999 requests: TopTrans 981, Intf 2240, GccEnt 55, GccInt 15,
Hatch 38 and GccAna 2670.

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
