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
| BRep_Tool_Test.cxx | brep_tool | 13 | pass | pass |
| BRepTools_ReShape_Test.cxx | brep_tools_reshape | 16 | pass | pass |
| BRepTools_TrsfModification_Test.cxx | brep_tools_trsf_modification | 4 | pass | pass |
| BRepTools_GTrsfModification_Test.cxx | brep_tools_gtrsf_modification | 13 | pass | pass |
| BRepAdaptor_CompCurve_Test.cxx | brep_adaptor_comp_curve | 4 | pass | pass |
| BRepTools_Test.cxx | brep_tools (not mapped) | 11 | not built (TKBO, TKMesh) | pass (3 translated) |
| BRepGraph_RepId_Test.cxx | brep_graph_rep_id | 10 | pass | pass |
| BRepGraph_ItemId_Test.cxx | brep_graph_item_id | 4 | pass | pass |
| BRepGraph_LayerIterator_Test.cxx | brep_graph_layer_iterator | 4 | pass | pass |
| BRepGraph_LayerHistory_Test.cxx | not mapped (phase 2) | 39 | not run | waits for EditorView (ApplyModification, RemoveNode), ReverseIterator, Tool, BRepBuilderAPI_MakeVertex |
| BRepGraph_LayerTopoSupplement_Test.cxx | not mapped (phase 2) | 22 | not run | waits for EditorView, SupplementEditor, RefsIterator, Copy, Compact |
| BRepGraph_CacheRegistry_Test.cxx | not mapped (phase 2) | 14 | not run | waits for EditorView, MutGuard, Transform |
| BRepGraph_NodeId_Test.cxx | not mapped (phase 2) | 15 | not run | waits for RelatedIterator, MakeBox |
| BRepGraph_RefId_Test.cxx | not mapped (phase 2) | 19 | not run | waits for EditorView, iterators, MakeBox |
| BRepGraph_VersionStamp_Test.cxx | not mapped (phase 2) | 22 | not run | waits for EditorView, Tool, MakeBox |
| BRepGraph_Reconstruct_Test.cxx | not mapped (phase 2) | 24 | not run | waits for EditorView, Tool, Iterator, BRepGProp |
| BRepGraphInc_PopulateRegression_Test.cxx | not mapped (phase 2) | 9 | not run | waits for CacheMesh, MeshView, BRepMesh, Compact |
| BRepGraph_ShapesViewImport_Test.cxx | not mapped (phase 2) | 44 | not run | waits for EditorView, explorers, iterators, Tool, TKPrim solids |
| BRepGraph_Build_Test.cxx | not mapped (phase 2) | 59 | not run | waits for Validate, Tool, iterators, TKPrim solids |
| BRepGraphInc_Test.cxx | not mapped (phase 2) | 80 | not run | waits for Compact, Deduplicate, Validate, BRepGProp |
| BRepGraph_Views_Test.cxx | not mapped (phase 2) | 81 | not run | waits for iterators, Tool, Compact, Deduplicate, Validate |
| BRepGraph_Test.cxx | not mapped (phase 2) | 95 | not run | waits for EditorView, explorers, iterators, Tool, Validate |

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

`BRep_Tool_Test.cxx` is translated in `crates/tkbrep/tests/brep_tool.rs`; all 13
declarations pass natively and in Rust, the two `CurveOnPlane_Projects*` ones through
the real `GeomProjLib::ProjectOnPlane`.

BRepGraph and BRepGraphInc are ported in two phases (docs/topology-design.md section
11). Phase 1 ports BRepGraphInc whole (the storage, definitions, references,
representations, relations, instances, bit flags, parity orientation, load counts,
population with the private BoundaryBuilder and wire ordering, and reconstruction) and
the BRepGraph core: the graph and its data, the ids (NodeId, RefId, RepId, ItemId, UID,
RefUID, ItemUID, VersionStamp, UsagePath), the TopoView, RefsView, UIDsView and
ShapesView, the parallel policy, and the editor, tool and derived-state members that
`Shapes().Add` and reconstruction call. `BRepGraph_RepId_Test.cxx` and
`BRepGraph_ItemId_Test.cxx` are translated 1:1 (14 declarations, native and Rust pass).
The other BRepGraph suites each call a phase-2 class (the editor, iterators,
explorers, Tool, Validate, Compact, Deduplicate, the caches or layers) or a toolkit
that is not ported (TKPrim solids other than the box, BRepGProp, BRepMesh,
BRepBuilderAPI), so they wait, as the table lists. `BRepGraph_RefTestTools.hxx` is
translated once in `crates/tkbrep/tests/support/ref_test_tools.rs` (its graph
`OuterWireOfFace` waits for `BRepGraph_Tool::Face::OuterWire`).
`crates/tkbrep/tests/brep_graph_behavior.rs` and `brep_graph_shapes_view.rs` cover
population, reconstruction and the shapes view over the support fakes, the supplement
route, and the Rust errors. `tools/audit_api_rows.py`
reports 0 missing and 0 stale api rows for BRepGraphInc (317 rows, 4 excluded: the
storage destructor and allocator and the reconstruction cache TempScope constructor
and destructor). For BRepGraph it reports 0 stale rows (429 rows, 9 excluded: the
destructor, the move constructor and assignment, the allocator and the `std::hash`
operators that repeat a `HashValue` or `HashCode` member); every rust item was checked
to resolve by compiling a reference to it. The members of the phase-1 classes that are
still missing are the phase-2 ones listed under Deliberate Rust deviations.

Phase 2 area B ports the layers and caches (docs/topology-design.md section 11.9):
BRepGraph_Layer, LayerRegistry, LayerHistory, LayerDeferred, LayerParametric,
LayerTopoSupplement, Cache, CacheRegistry, the layer, cache and supplement iterators,
the `LayerRegistry()` and `CacheRegistry()` accessors and the four
`Shapes().AddWithHistory` overloads. A graph now owns both registries and registers the
history layer at construction, as natively, so `Shapes().Add(shape, parent)` on a
supplement route ensures the supplement layer and succeeds, and a supplement layer
registered before population receives the non-core children, which reconstruction
replays. `BRepGraph_LayerIterator_Test.cxx` is translated 1:1 (4 declarations, native
and Rust pass); the LayerHistory, LayerTopoSupplement and CacheRegistry suites also
call the editor, iterators, Tool, Copy, Compact or Transform (area A and later waves),
so they wait, as the table lists. `crates/tkbrep/tests/brep_graph_layers.rs`,
`brep_graph_layer_history.rs` and `brep_graph_caches.rs` cover the registries, the event
dispatch from graph mutations, every history record and query, the supplement route,
the deferred layer and the cache entry freshness. `tools/audit_api_rows.py` gives these
classes 211 rows (13 excluded: the registry move constructors and assignments, the
representation storage move members and destructor, the STL `begin`/`end` of the three
iterators; 6 partial: the LayerDeferred members that lock through BRepGraph_LayerLock);
the BRepGraph missing count falls from 967 to 756. The 9 members of these classes still
missing all take `BRepGraph_CopyRemap` (area A): the five `CopyTo`/`CopyFreshTo`
overrides, `CopyLayersTo` and `CopyFreshCachesTo`.

The BRepTools and BRepAdaptor suites are translated 1:1 in
`crates/tkbrep/tests/brep_tools_reshape.rs`, `brep_tools_trsf_modification.rs`,
`brep_tools_gtrsf_modification.rs` (two included suite files) and
`brep_adaptor_comp_curve.rs` (37 declarations, mapped; native and Rust pass). The
CompCurve suite builds its edges and wires with the `support` fakes, whose MakeEdge now
drops trimmed curves to their basis as BRepLib_MakeEdge does. `BRepTools_Test.cxx`
calls BRepAlgoAPI (TKBO), BRepMesh (TKMesh), BRepGProp, BRepCheck, BRepBndLib and
BRepBuilderAPI_NurbsConvert/Transform, so the native closure cannot build it and the
suite is not mapped; `crates/tkbrep/tests/brep_tools.rs` translates its three
declarations that need only TKBRep and the box fake (`DemoBug_25344_4`,
`ModDataBug_20616`, `DemoBug_23130`), which pass. `tools/audit_api_rows.py` reports 0
missing and 0 stale api rows for BRepTools (203 rows, 1 excluded: the ShapeSet
destructor), BRepAdaptor (61) and BRepLProp (8).

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
- BRepTools: the static `BRepTools` members are the free functions of `brep_tools`
  (`update_face`, `uv_bounds_of_wire`, `compare_edges`, `write_with_options` with the
  `BRepWriteOptions` of the long overload, `read_with_console`...). `BRepTools_ShapeSet`
  is `BRepToolsShapeSet`, a `ShapeSet` over the GeomTools sets and the mesh tables.
  `BRepTools_Modification` is the trait `BRepToolsModification`, whose `NewSurface` and
  `NewCurve` outputs are the `NewSurface` and `NewCurve` structs; the Trsf, GTrsf, Copy
  and NurbsConvert modifications implement it (NurbsConvert embeds its Copy parent).
  `BRepTools_ReShape` is the trait `ReShape` (its virtual members provided over
  `ReShapeData`) and the struct `BRepToolsReShape`; `ModeConsiderLocation()` is the
  pair `mode_consider_location`/`set_mode_consider_location`. `BRepTools_History` is
  `History` with the `HistoryAlgorithm` trait for the `TheAlgo` templates.
  `BRepTools_Modifier`, `WireExplorer`, `Substitution`, `Quilt` and `PurgeLocations`
  keep their names; `BRepTools_Debug.cxx` is `debug_write`, `debug_dump` and
  `debug_dump_location`.
- BRepAdaptor: `BRepAdaptorCurve`, `BRepAdaptorCurve2d` and `BRepAdaptorSurface` embed
  the GeomAdaptor/Geom2dAdaptor class they derive from and forward the inherited
  adaptor members; `BRepAdaptorCompCurve` implements `Adaptor3dCurve` itself, and
  `Edge(U, E, UonE)` returns a `CompCurveEdge`.
- BRepLProp: `BRepLProp_CLProps` and `BRepLProp_SLProps` are type aliases of the
  GeomLProp generic structs over borrowed BRepAdaptor curves and surfaces;
  `BRepLProp::Continuity` is `continuity` and `continuity_with_tolerances`.
- BRepGraph and BRepGraphInc (docs/topology-design.md section 11): the kind enums are
  the `i32` newtypes `NodeKind`, `RefKind` and `RepKind`, and `BRepGraph_*Id::Typed<K>`
  is `TypedId<Tag>` with the aliases `FaceId`, `ShellRefId`, `EdgeCurve3DRepId`...;
  `++`, `+` and `-` are `pre_increment`, `post_increment`, `offset` and `retreat`.
  `BRepGraphInc_Storage` is `Storage`, with the native accessor names in snake case.
  The views borrow the graph: `topo()`, `refs()`, `uids()` and `shapes()` for the
  `const` members, `shapes_mut()` and `editor()` for the mutating ones; `Gen()` is
  `generic()` (`gen` is reserved in Rust 2024). `ShapesView::Result` is `AddResult`,
  and the `Add` overloads are `add`, `add_with_options`, `add_under` and
  `add_under_with_options`. `BRepGraphInc_Populate` and `BRepGraphInc_Reconstruct` are
  the modules `populate` (`perform`, `append`, `append_flattened`) and `reconstruct`
  (`node`, `node_with_cache`, `face_with_cache`, `Cache`).

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
`br_brep_dump` compares the JSON dumps of every TShape at depths -1 to 2. A p-curve
projected on a plane by `CurveOnPlane` is printed as `P`, its dynamic type, bounds and
points at the start, middle and end of the returned range (60 projections in the
corpus), and `Parameter(V, E, S, L)` is printed for every vertex, edge and face.

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
before the start) and truncated records; `br_bin_truncate` reads the prefixes (all, or
one in 3 or 5) of eight written shape sets. Failure messages are compared in full for
Standard_Failure, whose nested lines keep only the type of a failure of another class
(other packages word their messages differently), and with bytes from 0x80 on shown as
`?`. The probe never reads what is undefined natively: a shape-reference record never
ends where a byte is read without a check, and a truncation never ends between a table
tag and its count.

The BRepTools, BRepAdaptor and BRepLProp requests (195 requests in
`tools/tkbrep_breptools_corpus.py` and `tools/tkbrep_breptools_edit_corpus.py`) reuse
the BRep scene script, which gains Bezier surfaces, over closed scenes (two squares with
p-curves in a shell, a solid and a compound, with located copies, a mesh, polygons and
INTERNAL vertices; a cylinder with its seam; triangulation-only faces; a bilinear Bezier
patch with Bezier edges; planar faces whose edges have no p-curves, so every p-curve
query projects on the plane) and the BRep scenes. `br_tools_text` writes every shape in each
format option, reads it back (console and text) and dumps it, and runs one shape set
over a whole scene in the three versions; `br_tools_read` reads a written face and
twelve malformed variants. `br_tools_query` prints the UV bounds and boxes, outer wire,
closedness, `IsReallyClosed`, `OriEdgeInFace`, `Map3DEdges`, `CheckLocations`,
`Triangulation` and `Compare`; `br_tools_mutate` runs each in-place tool on every shape
and writes the scene. `br_tools_modif` runs `BRepTools_Modifier` with the Trsf (three
locations, with and without mesh), GTrsf, Copy and NurbsConvert modifications, mutable
input included, and writes each result; `br_tools_wire` explores every wire alone, on
each face and with UV bounds; `br_tools_reshape`, `br_tools_history`,
`br_tools_subst`, `br_tools_quilt` and `br_tools_purge` drive those classes with
command sequences (lists whose native order follows TShape addresses are sorted).
`br_adaptor` evaluates every curve, 2D curve, surface and composite curve adaptor at
five parameters with their trims and copies, the curve and surface local properties
and the junction continuity. Edge and face pairs are taken from the faces' own edges.

The BRepGraph requests (`br_bgraph`; 32 requests in `tools/tkbrep_brepgraph_corpus.py`)
ingest BRep scenes into one graph with a list of actions (`Populate::Perform`, `Append`
and `AppendFlattened`, sequential and parallel; `Shapes().Add` with and without the
auto product, flattened and tracking the added nodes; `Add` under a product, compound,
compsolid, solid, shell and face parent; `ClearCached`), then print, through the public
views only, every definition with its base counters, relations and derived queries
(edge faces and curves, coedge links and p-curves, face surfaces and triangulations,
product roots and components, occurrence products and locations), every generic node
and reference query, every reference entry, the grouped and reverse reference lists
and `RefAtStep`, the UIDs, items, stamps (with `ToGUID` of a fixed GUID), hashes and a
usage path of every node, reference and representation, and every original, cached and
fresh reconstruction (`Shapes().Shape` twice, `Reconstruct`, the shared-cache
`BRepGraphInc_Reconstruct::Node` pass and `FaceWithCache`) as a tree whose TShapes are
numbered in first-seen order, each vertex, edge and face with the full `br_brep`
report. The scenes are the solid, cylinder, mesh and Bezier scenes of the BRepTools
corpus, the `br_brep` edge, face and plane scenes (every shape appended and added one
at a time), natural-restriction faces (a trimmed plane, a full and a half cylinder
band, an infinite plane), a face with a degenerated edge in a compsolid of two solids,
and a compound of a free wire listed out of chain order, located and repeated free
edges, vertices and faces, an EXTERNAL edge in a wire and a face with an INTERNAL wire
(both raise Standard_ProgramError midway, and the partial storage is compared). No
request routes a shape to the topology supplement layer; the layer requests below do.

The BRepGraph layer and cache requests (`br_blayer`; 22 requests in
`tools/tkbrep_brepgraph_layers_corpus.py`, probes
`tools/oracle/tkbrep_probe_brepgraph_layers*.cxx` and
`crates/tkbrep/examples/br_probe/brep_graph_layers*.rs`) run the `br_bgraph` ingestion
actions on the solid and free scenes and then: every history record kind (with self
images, duplicates, empty replacements, a Deleted kind with images, a short batch that
raises Standard_OutOfRange, disabled recording and Clear), UID and item-UID records
(invalid UIDs, reference item UIDs), registry dispatch to logging layers that subscribe
to all, none or some kinds (with the modifications `Add` under a parent triggers,
unregistration, `InvalidateAll`, `ClearAll` and graph `Clear`), supplement attachments
through population, the `Add` route and the layer API (incompatible kinds, uid
collisions, removal, replacement and removal callbacks), `AddWithHistory` and the
cross-graph `Absorb` over BRepTools histories, a probe cache service (registration,
replacement, `Ensure`, entries bound to node, reference and item generations before and
after a mutation) and a standalone deferred layer (merging, the eight-representation
limit, bulk mode, moves). The reports print the registry slots, names, GUIDs, revisions
and masks, the logged events, every history event with its maps in map order and every
node, UID and item-UID query, the deleted sets in set order, every owner's
attachments, the cache entries and generations, the deferred entries in map order, every
node's shapes, and, after the graph is dropped, the detachment events and cache clears.

## Dependency gap

None left in TKBRep's own packages. `BRep_Tool::CurveOnPlane` (and the planar fallback
of `CurveOnSurface`, and `Parameter` for an internal vertex through it) projects the 3D
curve with `GeomProjLib::ProjectOnPlane` and `ProjLib_ProjectedCurve` from the
TKGeomBase port. BinTools reads a version 1 file with `BRepTools::Update(Face)` through
`brep_tools::update_face`; its former private copy is deleted.

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

- `BRepTools_ShapeSet` writes the UV points of a curve on surface only in version 2
  (those of the second p-curve for a seam), the regularity of a seam right after its
  second p-curve without a space, and the triangulation of a face as `2  index` with
  no newline; the triangulation reader keeps its has-normals flag from one record to
  the next before version 3.
- `BRepTools::Triangulation` looks the polygon of a free edge up with the lookup
  overload of `BRep_Tool::PolygonOnTriangulation` on its still-null `anEdgeTri`, so a
  free edge without a 3D polygon is never meshed.
- `BRepTools_CopyModification()` copies the geometry and the mesh (both default
  arguments are true), so `BRepTools_NurbsConvertModification` keeps triangulations
  and polygons on triangulations.
- `BRepTools_GTrsfModification` scales tolerances and deflections by the largest
  `gp_GTrsf::Value(i, j)`, which applies the scale factor of a similarity form.
- `BRepTools_WireExplorer::Init` starts an open wire without a FORWARD open end from
  the last first vertex its scan met (the native `V1` keeps its value), and `Next`
  overwrites its edge with every candidate examined.
- `BRepTools_NurbsConvertModification::NewCurve2d` converts both p-curves of a seam at
  its first visit and takes the second one from its list at the second visit by the
  edge's rank; the two lists lose alignment when a first visit returns early.
- `BRepTools_Modifier` rebuilds through a reference into its map: the port stores the
  result back where the native code, or a recursive call, reads the map.
- `BRepTools_Quilt::Shells` appends the other bound shapes from a hash map, and
  `BRepTools_History` lists merged from maps, in TShape address order, natively as in
  Rust (the probes sort them).
- A release build returns a null handle from `Adaptor3d_Curve::Bezier()` and
  `BSpline()` for another curve type (`Standard_NoSuchObject_Raise_if` is compiled
  out); the probe calls them only for their type.
- BRepGraphInc population stores only FORWARD and REVERSED children in the core model;
  an INTERNAL or EXTERNAL wire in a face or edge in a wire raises Standard_ProgramError
  from `ParityOrientation` in the middle of the build, and what was stored before stays
  in the graph. Non-core container children go to the topology supplement layer only
  when one is registered; a new graph has none.
- `BRepGraphInc_Reconstruct::Node` sets the edge range from the 3D curve record (0..0
  without one) also on a degenerated edge; the edge builder of `FaceWithCache` sets it
  only with an active curve record. A seam installs its p-curves, 2D polygons and
  polygons on triangulation once per edge per wire, from the FORWARD and REVERSED
  halves whichever half comes first.
- `Shapes().Shape` returns the construction-time original of a node whose SubtreeGen is
  0; a product or occurrence has none, so its shape is a reconstruction with new
  TShapes. The debug-only `ValidateMutationBoundary` assertion after `Add` is not run,
  as in a release build.

- `BRepGraph_LayerHistory::Absorb` walks its input map, whose TopTools_ShapeMapHasher
  order follows the TShape addresses, so the order of the records of several inputs
  differs between runs, natively as in Rust; the corpus gives images to one input per
  history. The removed inputs are recorded together after the loop.
- `RecordBatch` with fewer replacements than originals raises Standard_OutOfRange from
  the array read of its first loop, after the item-UID inputs of the earlier pairs were
  registered; the release build skips its size, kind and duplicate assertions, so a
  duplicated original keeps its last pair and a Deleted kind with images feeds the
  Modified map (as `Record` does).
- A supplement attachment clears the owner's cached reconstruction, but `Shapes().Shape`
  of a node never mutated still answers its construction-time original; only
  `Reconstruct` replays the attachments.
- `BRepGraph_Cache::RefOwnGen` tests the removal plane of the reference before its range,
  an unchecked read for a reference past the end (the probes query references in range
  only).
- The layer registry and the cache registry read one slot at a time under their lock
  and call the layer or cache without it, so a callback may use the registry; the
  registration callbacks (`OnAttached`, `OnDetached`) run under the lock, as natively.

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
- BinTools: a byte the native reader leaves uninitialised (`ReadByte` and the float
  normals of the shape reader past the end, a table count when the stream ends between
  the tag and the count) reads 0; an orientation or continuity byte outside its
  enumeration raises Standard_OutOfRange where the native code stores the value; a null
  geometry handle given to a writer raises Standard_NullObject (a crash natively). A
  message byte from 0x80 on (`aMsg << (uint8_t)value`) is the Latin-1 character of that
  code, since Rust messages are UTF-8. `BinTools::Write(S, theFile)` checks the opened
  stream before setting its precision (the native code dereferences it first).
- BRepTools: a shape record or edge representation list read from a failed stream
  stops at the failure (natively the loop never ends); the periodicity test of
  `UVBounds` stops after its first sample when the across range is empty (natively it
  never advances). A null curve, p-curve or surface given to `EvalAndUpdateTol`,
  `AddUVBounds` on a face without surface, a NurbsConvert seam without 3D curve and
  `Update` of a null shape raise Standard_NullObject instead of crashing, and the
  shape set's unchecked TShape casts raise Standard_TypeMismatch. The load members of
  the triangulation tools take the file system only for their signature: the TKMath
  deferred triangulations own their input.
- `br_tools_read` leaves out a text that ends right after the version line: the native
  location reader then compares a `char` buffer the failed `>>` left uninitialized.
- Reconstruction reads SameRange and SameParameter from the derived-state computations
  directly; the native cache service (CacheDerivedState, a later wave) returns the same
  values on an unchanged graph.
- A BRepGraph layer or cache cannot point back into the graph that owns it: the members
  that read `AttachedGraph()` or `Graph()` natively (history recording and `Absorb`,
  supplement attachment and removal, deferred registration, the cache generation reads
  and entry binds) take the graph as a parameter and use it only when it is the graph
  they are attached to (docs/topology-design.md section 11.9); the callbacks receive the
  attached graph from the registry. `Record(i)` lends the event to a closure, and the
  getters that return a `const&` into a map (`FindModified`, `DeletedNodes`,
  `AttachedTo`, `FindByUid`, `FindDeferred`) return copies. `ForEachDeferred` copies the
  entries before visiting them, and `SupplementIterator` copies the owner's uid list at
  construction (natively both walk the live containers); its `Value()` is `None` past the
  end, where the native reference is undefined.
- `BRepGraph_LayerDeferred` locks its items through `BRepGraph_LayerLock`, which needs the
  explorers (a later wave): on an attached layer, registering an item and
  `ReserveDeferredItems` return Standard_NotImplemented where they would `Ensure` the lock
  layer; the unlock and `TouchOwners` steps find no lock layer, which is exact while no
  such layer can exist. A detached deferred layer is exact. `OnNodeReplaced` stops
  appending at eight representations where the native noexcept callback terminates the
  process.
- `BRepGraphInc_Reconstruct::Node` with a node index out of its kind's range raises
  Standard_OutOfRange (natively an unchecked vector read). Parallel face extraction
  returns the error of the failing face with the lowest queue index, so a parallel and
  a sequential population fail the same way.
- Phase 2 still has to port these members of the phase-1 classes: `BRepGraph::Editor()
  const` and `Mesh()`; `Shapes().RemoveShape` (the editor's `RemoveNode`);
  `Topo().Edges().WiresOf` and `FacesOf` (they return iterators). The
  editor, Tool and CacheDerivedState classes are partly ported: only the members that
  phase 1 calls have api rows.
