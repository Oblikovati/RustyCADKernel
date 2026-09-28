# TKBRep port status

**In progress; toolkit acceptance is not complete.** `crates/tkbrep`
(`rustycad-tkbrep`) is the port of the ModelingData toolkit TKBRep. It depends on
the TKernel, TKMath, TKG2d, TKG3d and TKGeomBase ports, as the native toolkit does.
Packages are added one module at a time. The `TopoDS`, `TopExp` and `TopTools`
packages are ported in full, and `tools/audit_api_rows.py` reports 0 missing and 0
stale api rows for each of them (223 rows). BRep, BRepTools, BRepAdaptor, BRepLProp,
BinTools, BRepGraph and BRepGraphInc are not ported.

The Rust mapping of the topology is fixed in [topology-design.md](topology-design.md),
which extends [geom-design.md](geom-design.md). TopLoc is the TKMath port.

The toolkit test inventory maps these suites (28 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| TopoDS_Builder_Test.cxx | topods_builder | 19 | pass | pass |
| TopExp_Test.cxx | topexp | 9 | pass | pass |
| TopoDS_Edge_Test.cxx | topods_edge | 1 | pass | pass |
| TopoDS_Iterator_Test.cxx | topods_iterator | 17 | pass | pass |
| TopoDS_TShape_Test.cxx | topods_tshape | 8 | pass | pass |
| BRep_Tool_Test.cxx | brep_tool (not mapped) | 13 | pass (13) | pass (11 translated) |

The suites build their inputs with BRepBuilderAPI_MakeVertex, BRepBuilderAPI_MakeEdge
(TKTopAlgo) and BRepPrimAPI_MakeBox (TKPrim). Those toolkits are not ported, so the Rust
translations use the named fakes of `crates/tkbrep/tests/support`, which replay the
native builders with the real BRep types: `BRep_Builder` vertices with the BRepLib
precision, line and curve edges with the vertices, range and closed flag of
BRepLib_MakeEdge, and a box of six planar faces with a p-curve for each edge, sharing
12 line edges and 8 vertices as BRepPrim does. They are replaced by the real builders
when those toolkits are ported.

`TopExp_Test.cxx` calls BRepPrimAPI_MakeBox, so the shared native oracle closure builds
TKPrim beside TKTopAlgo (`tools/native_build.py`), and the unchanged file runs natively
like every other suite. The TopExp behaviour is compared with the native kernel by the
numeric oracle as well (`br_explore`, `br_maps`, `br_vertices`).

`BRep_Tool_Test.cxx` is translated in `crates/tkbrep/tests/brep_tool.rs` for the 11
declarations that do not project an edge on a plane; all 11 pass in Rust. Its
`CurveOnPlane_ProjectsShiftedPeriodicRange` and
`CurveOnPlane_ProjectsPeriodicEdgeBuiltFromEqualParameters` need the projection that
ProjLib and GeomProjLib provide (see "Dependency gap"), so they are not translated, and
the suite is not mapped in the ledger: the verifier maps whole files only. The
unchanged file passes natively (13 of 13, checked with temporary ledger rows). The
other TKBRep suites (BRepTools, BRepAdaptor, BRepGraph) wait for their packages.

`topods_behavior.rs`, `topexp_behavior.rs` and `toptools_behavior.rs` cover the quirks
below, the builder errors, the identity levels and hash, the iterator and explorer
protocols, the maps and the location and shape set texts, with the same fakes and a
recording console for the readers.

## Class mapping

- `TopoDS_Shape` is the value `topods::Shape` (TShape handle, location, orientation);
  the typed shapes are newtypes that deref to it. Getter/setter pairs are
  `name`/`set_name`; `Move` is `move_by`. The members that dereference the TShape of
  a null shape (`ShapeType`, the flags, `EmptyCopy`) return `Result` and raise
  Standard_NullObject; natively they crash.
- The `TopoDS` casts are `topods::vertex` ... `topods::compound`; they return a typed
  copy, also for the non-const overload that natively returns an alias.
  `Vertex::unchecked` is the unchecked reinterpretation that TopExp uses.
- `TopoDS_TShape` is the trait `TShape` over `TShapeBase` (locked child list and state
  word); `TopoDS_TVertex` is the embeddable parent struct `TVertex`, which has no
  `EmptyCopy`. The flag setters of `Shape` and `TShape` take `&self`, since the flags
  belong to the shared TShape.
- `TopoDS_Iterator` is `ShapeIterator` (the name `Iterator` is the Rust trait);
  `TopoDS_Builder` is `Builder`, whose protected `MakeShape` is public for the
  builders of other crates.
- `TopTools_ShapeSet` is the trait `ShapeSet` over `ShapeSetData`, with the base class
  instance `TopToolsShapeSet`; the virtual hooks return `Result` so that a derived set
  can raise. `TopTools_ShapeSet.cxx` includes `BRep_TFace.hxx` but uses nothing from
  it: the whole class is ported without BRep.
- The TopExp functions return their output vertices (`vertices` and `wire_vertices`
  return pairs, `common_vertex` an `Option`).

- BRep (docs/topology-design.md section 9): `BRep_TVertex`, `BRep_TEdge` and
  `BRep_TFace` are `BRepTVertex`, `BRepTEdge` and `BRepTFace`, which embed their TopoDS
  parent. The abstract `BRep_CurveRepresentation`, `BRep_GCurve`,
  `BRep_PointRepresentation` and `BRep_PointsOnSurface` are traits over embedded base
  data; the members their base classes define by raising Standard_DomainError return
  `Result`. `BRep_Builder` is `BRepBuilder` (it derefs to `topods::Builder`) and
  `BRep_Tool` is `BRepTool`; their overloads get names that say what they take, and
  output arguments become returned values (`EdgeCurve3D`, `EdgePCurve`,
  `EdgeCurveOnSurface`, tuples). `BRepGCurve::set_range`, `Copy` and
  `TopoDS_TShape::EmptyCopy` return `Result`, because updating a p-curve range
  evaluates the p-curves.
- The vertex-only `brep` module that the TKernel TopLoc legacy test and the TKTopAlgo
  `MakeVertex` used before this port is replaced: both use `BRepBuilder` and `BRepTool`.

## Verification

`python3 tools/verify_toolkit.py TKBRep --jobs 8 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, lists the native-blocked rows, then runs
`tools/verify_tkbrep_numeric.py`. Evidence is in `target/oracle-tkbrep/`.

## Numeric comparison policy

`tools/verify_tkbrep_numeric.py` uses the shared driver `tools/toolkit_numeric.py`.
It builds `tools/oracle/tkbrep_probe.cxx` (a table of operations over
`tkbrep_probe_*.cxx`, linked with libTKBRep and the toolkits below it) and its twin
`crates/tkbrep/examples/numeric_probe.rs`, runs both on `tools/tkbrep_corpus.py`, and
requires identical output tokens: doubles as bits (NaN by classification),
exception types, and texts as bytes.

A request starts with a graph script built only with TopoDS on both sides (the probe
vertex and edge TShapes derive from the abstract TopoDS classes): datums
(translations, rotations, a scaling), products, powers and inverses of locations,
then the builder commands (make, add, remove, place, moved and empty copies, flags,
raising location setters, reversed, complemented and composed copies, nullify). Shapes
are printed as address-free descriptors: the index of the first graph shape with the
same TShape, the orientation, and the location as its chain of (datum index, power).

The 84 requests run six scenarios (one shape of every type with copies and flags; a
reversed, located face with a closed and an open wire and internal and external
vertices; a box-like solid, plain and located; nested compounds and composite solids
under reversed, located parents with a shared sub-shape and removals; the builder
errors for every parent and child type pair, frozen parents, self insertion and
scaled locations) through every report: flags and dynamic types, the four iterator
cumulations, the identity matrix, the hash (checked on real addresses on each side,
compared on synthetic ones), the casts, the JSON dumps with pointer texts masked,
HShape and the alerts, the explorer for every searched and avoided type with the end
state, the restart, ReInit and Clear, the indexed, plain and ancestor maps for every
type pair, the vertex queries, and the shape set (five format numbers) and location
set with their dump, write and read texts. Given texts exercise the readers: missing
and malformed tags and versions, out-of-range references, singular matrices,
overflowing numbers, truncated tables and incompatible sub-shapes.

The BRep requests (`br_brep`, `br_brep_dump`; 8 requests in
`tools/tkbrep_brep_corpus.py`) script a scene: locations, geometry registries (lines,
circles, Bezier and trimmed curves; planes, cylinders, rectangular trimmed and offset
surfaces; 2D lines and circles; 3D and 2D polygons, triangulations with mesh purposes
and node arrays) and `BRep_Builder` commands, then print every `BRep_Tool` query on
every shape, every edge and face pair, every edge and registry surface or
triangulation at three locations, and every vertex and edge pair: points, raw and
floored tolerances, flags, ranges, curves (and transformed copies evaluated), p-curves
by surface and by index, UV points, polygons and polygons on triangulations, closed
tests, regularities and their continuity, vertex parameters, and every representation
with its accessors (or their exceptions), range, point and UV points. The four scenes
cover located and scaled edges, a closed circle edge, a Bezier edge, a degenerated
edge, internal vertices with points on curves, p-curves and surfaces, seams on a
cylinder with both UV point pairs, regularities, the quirky seam polygon location,
triangulation replacement and lists, transfers, empty copies, and the builder errors
(locked shapes, missing p-curves, infinite parameters, bad triangulation lists).
`br_brep_dump` compares the JSON dumps of every TShape at depths -1 to 2. A query that
reaches the unported planar projection prints `P` on both sides: natively when it
returns a projected curve, in Rust for its Standard_NotImplemented, so the gap is pinned
to exactly the native projection cases (92 of them in the corpus).

## Dependency gap

`BRep_Tool::CurveOnPlane` projects the 3D curve of an edge on a plane with
`GeomProjLib::ProjectOnPlane` and `ProjLib_ProjectedCurve`, and `CurveOnSurface`
falls back to it for a plane without a stored p-curve. ProjLib and GeomProjLib are not
ported, so the port keeps every step before the projection (the planar test, the 3D
curve lookup, the relative location, the transformed curve and parameters, the range
validation, the trimmed curve) and raises Standard_NotImplemented, naming the missing
packages, at the projection itself. The api rows of `CurveOnPlane`, both
`CurveOnSurface(E, S, L, ...)` and `CurveOnSurface(E, F, ...)`, and
`Parameter(V, E, F)` and `Parameter(V, E, S, L)` (which call `CurveOnSurface` for an
internal vertex) are `partial`. Two `BRep_Tool_Test.cxx` declarations wait for it.

## Preserved native behaviour

- `TopoDS_Builder::Add` freezes the component before it checks the parent, so a
  failed Add leaves the component frozen and self insertion raises
  TopoDS_FrozenShape. `Remove` sets Modified only when it removes a child.
- `TopoDS_Shape::Location(L, true)` and `Move(L, true)` refuse a scaling or negative
  transformation with Standard_DomainError; without the flag they accept it.
- `TopoDS_TShape::Modified(true)` clears Checked. TSolid and TCompound are created not
  orientable, TVertex closed and convex.
- `TopoDS_TEdge::EmptyCopy` returns a plain TopoDS_TEdge, also for a derived edge
  class that does not override it.
- `TopoDS_Iterator::Next` past the end does nothing; `Value` then raises
  Standard_NoSuchObject.
- `TopExp_Explorer::Next` after the end restarts the exploration from the root, and
  `Current` at the end is the explored shape; `Depth` counts the stacked iterators
  (the shape found at the top level is at depth 1, the explored shape at depth 0).
- TopExp casts are unchecked (`No_Standard_TypeMismatch`): `FirstVertex` of a shape
  whose FORWARD child is not a vertex returns that child.
- `TopExp::Vertices(E)` reads the whole child list, so with two FORWARD vertices it
  returns the last one, while `FirstVertex` returns the first.
- `TopExp::Vertices(W)` picks the open ends from a hash map: with two ends of the same
  orientation the choice follows the TShape addresses, natively as in Rust.
- `MapShapes(S, M, cumOri, cumLoc)` applies the flags to the first level only; the
  recursion uses the defaults.
- `TopTools_LocationSet::Read` keeps `l1`, `p` and the record type `typLoc` across
  records. `typLoc` is uninitialised natively; the pinned build keeps the previous
  record's value, so a table truncated after an elementary location reads a null
  matrix and raises Standard_ConstructionError. A failed read inside a datum list
  loops forever natively; the port stops at the failure.
- `TopTools_ShapeSet::Read` reads the flags of each shape from the same 255-byte
  buffer that held the `TShapes` tag: a flag word shorter than seven characters leaves
  the older bytes after its terminator in place, and those bytes are read as flags.
  The checked flag is read only from version 2 on, and `Check` is called for version
  1 files.
- `TopTools_ShapeSet::Write` does not restore the stream precision when the progress
  indicator breaks before the TShapes table.
- `TopTools_ShapeSet::SetFormatNb` ignores a number outside 1..3
  (`Standard_ASSERT_RETURN` in a release build).
- The base `TopTools_ShapeSet` hooks make no shape, so reading a non-empty TShapes
  table with the base class dereferences a null shape (a crash natively); the port
  raises Standard_NullObject.

- BRep_Builder `UpdateCurves` for a 3D curve reassigns its cast at every list item: a
  new 3D curve takes the range of the last parametric curve read only when the list
  ends with a parametric curve.
- `BRep_Builder::UpdateEdge(E, C1, C2, S, L, Tol, Pf, Pl)` sets the UV points of the
  second p-curve (`SetUVPoints2`); the single p-curve overload removes every curve on
  the surface, the seam overload only the first one, and the seam overload reads the 3D
  curve range only from the 3D curves before that first match.
- `UpdateEdge(E, P1, P2, S, L)` stores the seam polygon at the identity location, not
  at the edge-relative one, so a later search at that location misses it;
  `UpdateEdge(E, P, L)` appends a new 3D polygon representation even for a null
  polygon when the edge has none.
- `BRep_Builder::Degenerated(E, true)` stores the null 3D curve at the edge location
  itself, not relative to it; `UpdateVertex(V, P, E, S, L, Tol)` searches the p-curve at
  `L` itself (xpu020198, BUC60407) but stores the point at the vertex-relative location.
- `BRep_Builder::Transfert(Ein, Eout)` transfers only the first p-curve of a seam (a
  seam also answers `IsCurveOnSurface`), with the raw edge tolerance.
- `BRep_Tool::Range(E, S, L)` (and the Face overload) sets the Modified flag of the
  edge. `BRep_Tool::Triangulation(F, L)` returns the face location without the TFace
  location. `BRep_Tool::IsClosed(E, S, L)` is false on planes and on offset and
  rectangular trimmed planes (NIZNHY-PKV).
- `BRep_TFace::Triangulations` clears the Active bit of the triangulations before a null
  member, then raises Standard_ProgramError (a `Standard_ASSERT_RAISE`, raised in release
  builds too); an active triangulation outside the list raises it after clearing all.
- `BRep_PointOnCurve` and `BRep_PointOnCurveOnSurface` dump their curve as a pointer
  number; `BRep_PointOnCurveOnSurface` dumps the `BRep_PointRepresentation` base, not
  `BRep_PointsOnSurface`, and `BRep_PointOnSurface` dumps as `BRep_PointsOnSurface`.
- `BRep_CurveOn2Surfaces::D0` always raises Standard_NullObject.

## Deliberate Rust deviations

- A null TShape dereference (null shape flags, `ShapeType`, a null explored shape in
  `TopExp_Explorer::Next`, a null parent or component in `TopoDS_Builder`) raises
  Standard_NullObject instead of crashing.
- The iterator walks the child list by index. While the list does not change the
  result is the same; removing the current child during an iteration is undefined
  natively (the node is freed) and continues at the next index in Rust.
- `TopExp_Explorer::Next` returns `Result`, and `Explorer` and `ShapeIterator` also
  implement `std::iter::Iterator`.
- BRep_Builder and BRep_Tool reinterpret the TShape of their shapes without a check
  natively (a null shape or another TShape class is undefined behaviour); the port
  raises Standard_NullObject and Standard_TypeMismatch. `BRep_Curve3D::D0` on a null
  curve, the constructors of p-curve representations on a null p-curve and
  `BRep_Tool::Parameter` on a matching point with a null p-curve raise
  Standard_NullObject instead of crashing.
- `BRep_Tool::CurveOnSurface(E, C, S, L, First, Last, Index)` with an index below 1
  returns the null result; natively it leaves its output arguments unchanged.
- `BRep_TEdge::Curves()`, `BRep_TVertex::Points()` and `BRep_TFace::Triangulations()`
  return copies of the lists, and `Transfert` walks a copy of the input list, so an
  edge transferred onto itself does not iterate over the representations it adds.
- The planar projection of `BRep_Tool::CurveOnPlane` raises Standard_NotImplemented
  ("Dependency gap").
