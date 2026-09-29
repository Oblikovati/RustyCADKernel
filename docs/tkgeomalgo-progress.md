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
