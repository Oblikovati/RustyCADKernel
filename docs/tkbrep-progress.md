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

BinTools has no upstream GoogleTest suite. `tests/bintools_behavior` (named fakes: a
recording console, an in-memory OSD file system with a controllable flush, a refusing
file system and a breaking progress indicator) covers the encodings, both formats,
the file variants and the quirks below; the byte-exact comparison is the `br_bin_*`
requests of the numeric oracle. `tools/audit_api_rows.py` reports 0 missing and 0 stale
api rows for BinTools (152 rows, 5 excluded: the four destructors and
`BinTools_Curve2dSet::Dump`, which is declared but defined nowhere).

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
- BinTools (docs/topology-design.md section 10): the static `BinTools` members are the
  free functions of `bintools` (`put_real`, `get_bool`, `write`, `read`, `write_file`...;
  the three write options are `WriteOptions`, and `read_with_console`, `write_file_on`
  and `read_file_on` inject the console and the OSD file system). `BinTools_ShapeSetBase`
  is the trait `BinShapeSetBase` over `ShapeSetBaseData` (also the base class instance);
  `BinShapeSet`, `BinShapeWriter` and `BinShapeReader` implement it. The overloads
  `Write(OS, range)`/`Write(S, OS)` and `Read(IS, range)`/`Read(IS, S)` are `write`/
  `write_shape` and `read`/`read_shape`; the shape set's own `WriteShape`/`ReadShape`
  are `write_shape_geometry`/`read_shape_geometry`. No OCCT class derives from
  `BinTools_ShapeSet`, so its virtual members are inherent methods. The sets are
  `BinCurveSet`, `BinCurve2dSet`, `BinSurfaceSet` and `BinLocationSet`; the streams
  are `BinOStream` and `BinIStream` over the TKernel `IoStream`, whose operators are
  named by type (`write_point3`, `read_bools7`...). `BinTools_FormatVersion` is the
  `format_version` constants and `BinTools_ObjectType` the byte newtype `ObjectType`.
  The file variants put the storage stream on the `std::streambuf` of the stream the
  OSD file system opens, as the native `std::iostream` sits on it.

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

The BinTools requests (`br_bin_scalars`, `br_bin_ostream`, `br_bin_istream`,
`br_bin_geometry`, `br_bin_shape`, `br_bin_read`, `br_bin_truncate`; 281 requests in
`tools/tkbrep_bintools_corpus.py` and its scene and byte-string modules) compare the
written bytes as `t<hex>` tokens (a string over 4096 bytes as its length and FNV-1a
hash). They cover the scalar encodings and their failures, every BinTools_OStream
writer and BinTools_IStream reader with references of every width (8 to 64 bits, zero
and pre-start deltas), every Geom curve, Geom surface and Geom2d curve class (rational
and periodic splines, indirect frames, nested trimmed and offset curves, failing
constructions) through the three sets, read back, evaluated and written again, and
eight BRep scenes (a shelled square face with located and reversed instances,
triangulations with normals and UV nodes, triangulation-only faces, polygons on
triangulations and 3D polygons with parameters, point representations with the tiny
parameter, a location algebra scene, and the four `br_brep` scenarios) written in the
four versions, with and without triangles and normals, by `BinTools::Write` and
`BinTools_ShapeWriter` (twice, for the top-level reference), read back, described
(type, orientation, TShape sharing, flags, location matrix) and written again.
Hand-built byte strings read by every reader cover the version lines (CRLF, junk, a
line over 100 characters), missing and substring-matched tags, unknown shape,
representation, curve, surface and location codes, out-of-range indices, singular
matrices, the shape-reader references (known, unread with a jump back, zero delta,
before the start) and truncated records; `br_bin_truncate` reads every prefix of eight
written shape sets. Failure messages are compared in full for Standard_Failure, whose
nested lines keep only the type of a failure of another class (other packages word
their messages differently), and with bytes from 0x80 on shown as `?`. The probe
never reads what is undefined natively: a shape-reference record never ends where a
byte is read without a check, and a truncation never ends between a table tag and its
count.

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

BinTools reads a version 1 file with `BRepTools::Update(Face)`, which resets the UV
points of the edges of each face. BRepTools is not ported, so `bintools::face_uv_update`
is a temporary private copy of `BRepTools::Update(const TopoDS_Face&)` and
`BRepTools::UpdateFaceUVPoints`; it is deleted when BRepTools lands.

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
- BinTools writes every value in the platform byte order: `DO_INVERSE` is tested by
  BinTools.cxx, BinTools_OStream.cxx and BinTools_IStream.cxx but defined by no header
  at the pinned revision, so no FSD_BinaryFile swapping happens.
- `BinTools::GetBool` is `IS.get() != 0`: past the end it reads true. The shape readers
  read the flags this way, so a truncated flag list reads set flags.
- A point on a surface is written with its second parameter first and read with the
  first value as the first parameter, in both formats: a round trip swaps U and V.
- Versions 1, 2 and 4 first try the pre-6.2.1 vertex layout (parameter, then type): a
  parameter whose last byte is 1, 2 or 3 (a double below about 1e-290, such as
  2^-1000) is misread, and a following record of type 2 is then read as type 1, whose
  location index is the surface index (Standard_OutOfRange when there is no such
  location). Version 3 reads the type first.
- Version 1 files are read with Checked forced off, and each face then goes through
  `BRepTools::Update`, which marks it Checked: a version 1 round trip sets the
  Checked flag of every face. Versions 2 and 3 store the UV points of each p-curve
  (of the second one for a seam, `UVPoints2`, which the reader restores with the seam
  `UpdateEdge`); versions 1 and 4 store none.
- `BinTools_Curve2dSet` and `BinTools_SurfaceSet` store a null handle like any key and
  have no index 0 (Standard_OutOfRange), while `BinTools_CurveSet` refuses a null curve
  and returns a null curve for 0. The set `Read` members do not clear the set.
- `BinTools_ShapeReader` keeps a point on a p-curve only when its surface is null
  (`!aCurve2d.IsNull() && aSurface.IsNull()`), so every written one is dropped.
- A face without a surface always has its triangulation written, with its normals in
  version 4 whatever `IsWithNormals`; a triangulation first reached from an edge is
  stored without normals. The shape writer writes a triangulation met again as a back
  reference whatever the normal flag of the second use.
- The mesh section headers are matched as substrings (`strstr`), the table tags
  exactly; the messages keep the native wording and typos (`BinTools_LocatioSet`,
  "BinTools_ShapeSet::Triangulation: Not a Triangulation section", and the vertex
  record error naming `BinTools_SurfaceSet::ReadGeometry`). The 2D polygons of edges
  are collected with triangles but never written.
- The shape reader follows a reference to a position it has not read by jumping to
  it, reading the object there (recorded at that position) and jumping back; a
  shape's record is recorded before its own orientation and location are applied. A
  reference before the start of the stream fails the stream; the next type read is
  then 255.
- The sub-shape references of the grouped format are read with `IS >> char`, which
  skips spaces, and count the TShape index from the end of the announced table, so a
  forward reference is out of range.
- `BinTools_ShapeSetBase::SetFormatNb` ignores a number outside 1..4
  (`Standard_ASSERT_RETURN` in a release build); `BinTools::Write` with version 9
  writes the current version.

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
- BinTools: a byte the native reader leaves uninitialised (`ReadByte` and the float
  normals of the shape reader past the end, a table count when the stream ends between
  the tag and the count) reads 0; an orientation or continuity byte outside its
  enumeration raises Standard_OutOfRange where the native code stores the value; a null
  geometry handle given to a writer raises Standard_NullObject (a crash natively). A
  message byte from 0x80 on (`aMsg << (uint8_t)value`) is the Latin-1 character of that
  code, since Rust messages are UTF-8. `BinTools::Write(S, theFile)` checks the opened
  stream before setting its precision (the native code dereferences it first).
