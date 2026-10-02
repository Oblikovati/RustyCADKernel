# TKBRep port status

**In progress; toolkit acceptance is not complete.** `crates/tkbrep`
(`rustycad-tkbrep`) is the port of the ModelingData toolkit TKBRep. It depends on
the TKernel, TKMath, TKG2d, TKG3d and TKGeomBase ports, as the native toolkit does.
Packages are added one module at a time. The `TopoDS`, `TopExp`, `TopTools`, `BRep`,
`BinTools`, `BRepTools`, `BRepAdaptor`, `BRepLProp` and `BRepGraphInc` packages are
ported in full, and `tools/audit_api_rows.py` reports 0 missing and 0 stale api rows for
each of them. `BRepGraph` is being ported in waves (docs/topology-design.md section 11):
the core, the iterators, explorers, Tool and CopyRemap, and the layers, registries and
caches, the lock, the derived-state and mesh caches and the editor are ported; the
algorithms (Copy, Compact, Deduplicate, Transform, Validate) follow, and its audit lists their members as missing until then.

The Rust mapping of the topology is fixed in [topology-design.md](topology-design.md),
which extends [geom-design.md](geom-design.md). TopLoc is the TKMath port.

The toolkit test inventory maps these suites (1260 declarations; 1271 active native runtime cases and one source-disabled original):

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
| BRepTools_Test.cxx | brep_tools (3/11 mapped) | 11 | pass (11/11) | pass (3/3 mapped) |
| BRepGraph_RepId_Test.cxx | brep_graph_rep_id | 10 | pass | pass |
| BRepGraph_ItemId_Test.cxx | brep_graph_item_id | 4 | pass | pass |
| BRepGraph_NodeId_Test.cxx | brep_graph_node_id | 15 | pass | pass |
| BRepGraph_TypedIdDispatch_Test.cxx | brep_graph_typed_id_dispatch | 2 | pass | pass |
| BRepGraph_Iterator_Test.cxx | brep_graph_iterator | 14 | pass | pass |
| BRepGraph_DefsIterator_Test.cxx | brep_graph_defs_iterator | 11 | pass | pass |
| BRepGraph_RefsIterator_Test.cxx | brep_graph_refs_iterator | 8 | pass | pass |
| BRepGraph_ReverseIterator_Test.cxx | brep_graph_reverse_iterator | 20 | pass | pass |
| BRepGraph_RelatedIterator_Test.cxx | brep_graph_related_iterator | 15 | pass | pass |
| BRepGraph_ChildExplorer_Test.cxx | brep_graph_child_explorer | 37 | pass | pass |
| BRepGraph_ParentExplorer_Test.cxx | brep_graph_parent_explorer | 23 | pass | pass |
| BRepGraph_Tool_Test.cxx | mapped | 16 | passed | original production builders |
| BRepGraph_RefId_Test.cxx | brep_graph_ref_id | 19 | pass | pass |
| BRepGraph_VersionStamp_Test.cxx | brep_graph_version_stamp | 22 | pass | pass |
| BRepGraph_Reconstruct_Test.cxx | mapped | 24 | passed | original builders and BRepGProp integration |
| BRepGraphInc_PopulateRegression_Test.cxx | not mapped (phase 2) | 9 | not run | waits for BRepMesh_IncrementalMesh (TKMesh) |
| BRepGraph_ShapesViewImport_Test.cxx | brep_graph_shapes_view_import_legacy | 44 | passed | original import, copy, mutation and adjacency inputs |
| BRepGraph_Build_Test.cxx | mapped | 59 | passed | original production builders |
| BRepGraphInc_Test.cxx | brep_graph_inc_legacy | 80 | passed | original storage, geometry, relations, locations and mutation inputs |
| BRepGraph_Views_Test.cxx | brep_graph_views_legacy | 81 | passed | original views, editor, UID, cache and supplement inputs |
| BRepGraph_Test.cxx | brep_graph_suite | 95 | pass | pass |
| BRepGraph_LayerIterator_Test.cxx | brep_graph_layer_iterator | 4 | pass | pass |
| BRepGraph_CacheMesh_Test.cxx | brep_graph_cache_mesh_suite | 22 | pass | pass |
| BRepGraph_Polygon_Test.cxx | not mapped (phase 2) | 7 | not run | waits for BRepMesh_IncrementalMesh (TKMesh), BRepPrimAPI_MakeCylinder (TKPrim) |
| BRepGraph_Geometry_Test.cxx | mapped | 44 | passed | original geometry, PCurve, component and representation-reuse inputs |
| BRepGraph_Lock_Test.cxx | brep_graph_lock_suite | 19 | pass | pass |
| BRepGraph_LayerHistory_Test.cxx | brep_graph_layer_history_suite | 39 | pass | pass |
| BRepGraph_LayerTopoSupplement_Test.cxx | brep_graph_layer_topo_supplement | 22 | pass | pass |
| BRepGraph_CacheRegistry_Test.cxx | brep_graph_cache_registry | 14 | pass | pass |
| BRepGraph_MutGuard_Test.cxx | brep_graph_mut_guard | 11 | pass | pass |
| BRepGraph_MutationGen_Test.cxx | brep_graph_mutation_gen | 14 | pass | pass |
| BRepGraph_DeferredInvalidation_Test.cxx | brep_graph_deferred_invalidation | 16 | pass | pass |
| BRepGraph_BatchOps_Test.cxx | brep_graph_batch_ops | 17 | pass | pass |
| BRepGraph_EventBus_Test.cxx | brep_graph_event_bus | 26 | pass | pass |
| BRepGraph_Assembly_Test.cxx | brep_graph_assembly_legacy | 45 | passed | original product, occurrence, placement, removal and reconstruction inputs |
| BRepGraph_EdgeCases_Test.cxx | mapped | 12 | passed | original primitive builders |
| BRepGraph_Convenience_Test.cxx | mapped | 17 | passed | original production builders |
| BRepGraph_PermissionUpdate_Test.cxx | brep_graph_permission_update | 15 | pass | pass |
| BRepGraph_ReplaceVertex_Test.cxx | brep_graph_replace_vertex | 6 | pass | pass |
| BRepGraph_Reverse_Test.cxx | brep_graph_reverse | 16 | pass | pass |
| BRepGraph_Sharing_Test.cxx | brep_graph_sharing | 17 | pass | pass |
| BRepGraph_Compact_Test.cxx | not mapped (phase 2) | 26 | not run | waits for BRepBuilderAPI_Copy, BRepCheck_Analyzer, BRepAlgoAPI_Cut (TKBO), TKPrim cylinder |
| BRepGraph_Copy_Test.cxx | not mapped (phase 2) | 23 | not run | waits for BRepGProp, TKPrim cylinder and sphere, BRepBuilderAPI_Copy, BRepAlgoAPI_Fuse (TKBO) |
| BRepGraph_Deduplicate_Test.cxx | not mapped (phase 2) | 82 | not run | waits for BRepBuilderAPI_Copy and MakeFace, BRepCheck_Analyzer, TKPrim cylinder, sphere and cone |
| BRepGraph_Transform_Test.cxx | mapped | 19 | passed | original geometry, placement and mesh policies |
| BRepGraph_Validate_Test.cxx | mapped | 23 | passed | original mutations, guard scopes and synthetic graphs |
| BRepGraph_SparseModel_Test.cxx | brep_graph_sparse_model | 28 | pass | pass |
| BRepGraph_ScenarioMatrix_Test.cxx | brep_graph_scenario_matrix_legacy | 18 | passed | original mutation, reconstruction, population and seam-split scenarios |
| BRepGraph_SeamRedesign_Test.cxx | mapped | 9 | passed | original builders and seam connectivity |
| BRepGraph_Fuzz_Test.cxx | brep_graph_fuzz_legacy | 2 (14 runtime cases) | passed | original seven seeds and 630 mutation inputs |

The shared fixtures in `crates/tkbrep/tests/support` now call the production
BRepBuilderAPI_MakeVertex/MakeEdge/MakeWire and BRepPrimAPI_MakeBox ports. The previous
builder replays served while TKTopAlgo and TKPrim were unavailable; their duplicated
wedge-box implementation has been removed. A deliberately topology-only `edge_between`
case retains repeated vertices that the geometric MakeEdge constructor would reject;
ordinary distinct-vertex edges call MakeEdge. Both toolkits are dev dependencies.

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

Phase 2, wave 2a, area A ports the navigation classes and the first algorithms whole:
BRepGraph_Iterator (with RootProductIterator), DefsIterator, RefsIterator,
ReverseIterator, RelatedIterator, ChildExplorer, ParentExplorer, BRepGraph_Tool (all
77 members, the five phase-1 reads included), BRepGraph_CopyRemap and
`Topo().Edges().WiresOf/FacesOf` (docs/topology-design.md section 11.9). The eight
upstream suites of these classes each also call an editor operation that a later wave
ports (and two call TKPrim cylinders or the mesh view), so they wait, as the table
lists; `BRepGraph_NodeId_Test.cxx` needed only the related iterator and the box, and
`BRepGraph_TypedIdDispatch_Test.cxx` only the phase-1 ids, so both are translated 1:1
(17 declarations, native and Rust pass). `crates/tkbrep/tests/brep_graph_navigation.rs`
pins the box invariants (iteration counts, reverse parents, related nodes, explorer
paths and Tool answers), the module unit tests cover removed entries (which the probe
cannot create before the editor lands), and the `br_bgraph_walk` requests compare
everything else with OCCT byte for byte. `tools/audit_api_rows.py` has rows for every
public member of these classes (377 rows, 28 excluded: the `begin()`/`end()` range
members); the BRepGraph missing list shrank from 967 to 590 members, with 0 stale rows.
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

Phase 2, wave 2b ports the lock and cache classes whole (docs/topology-design.md
section 11.11): BRepGraph_LayerLock (with ScopedOwnerEdit), the
BRepGraph_CacheDerivedState cache service, BRepGraph_CacheMesh (with its driver, slots
and entries), BRepGraph::MeshView and `BRepGraph::Mesh()`. With CopyRemap ported, the
five layer `CopyTo` overrides, `BRepGraph_Cache::CopyFreshTo`, the two
`CopyLayersTo` and `CopyFreshCachesTo`, `RemappedItem` (`layer::remapped_node` and
`remapped_ref`) and the history's private `rebuildCaches` are ported; the six
LayerDeferred members that locked through BRepGraph_LayerLock and the protected
`LayerParametric::lockLayer` are complete. `BRepGraph_Tool` and reconstruction read the
derived flags through the memoising service, as OCCT does. No upstream suite of these
classes builds yet: CacheMesh, Geometry and Lock also call the editor (another wave)
and Polygon calls BRepMesh (TKMesh), as the table lists.
`crates/tkbrep/tests/brep_graph_lock.rs` and `brep_graph_mesh_cache.rs` cover the lock
roots, overlaps, scopes, dispatch and copies, the memoised derived answers and their
invalidation, the mesh freshness stamps, the effective view and a mesh driver, and the
`br_bcache` requests compare everything with OCCT byte for byte.
`tools/audit_api_rows.py` gives these classes and members 180 new rows (2 excluded: the
ScopedOwnerEdit move constructor and assignment); the only member of them without a row
is `MeshView::EditorView::PromoteToPersistent`, which writes the persistent mesh through
the graph editor's `SetPersistent*` setters and waits for them. The BRepGraph missing
count falls from 379 to 199 (the editor, MutGuard, DeferredScope, SupplementEditor, the
algorithms, `Editor() const`, `Shapes().RemoveShape` and `PromoteToPersistent`), with 0
stale rows.

Phase 2, wave 2b, editor area ports the mutation classes whole
(docs/topology-design.md section 11.4): BRepGraph::EditorView with all twelve operation
groups (the EditorView_Mut guards and the EditorView_Setters setters, direct and
guarded), BRepGraph_MutGuard, BRepGraph_DeferredScope, BRepGraph_SupplementEditor, the
private `invalidateSubgraphImpl`, `Shapes().RemoveShape`, `BRepGraph::Editor() const`
and `MeshView::EditorView::PromoteToPersistent`, which writes through the editor's
`SetPersistent*` setters. Layer notifications (history records, deferred and supplement
updates) and cache invalidation fire in the native order. Fifteen upstream suites now
build and are translated 1:1 (Iterator, DefsIterator, RefsIterator, ReverseIterator,
RelatedIterator, ChildExplorer, RefId, VersionStamp, CacheMesh, LayerHistory, MutGuard,
MutationGen, DeferredInvalidation, BatchOps, EventBus: 291 declarations, native and Rust
pass); the LayerHistory and CacheMesh suites map to `brep_graph_layer_history_suite` and
`brep_graph_cache_mesh_suite`, since the area-B and wave-2b behaviour files own the plain
names. Every other BRepGraph suite still calls a wave-2c algorithm (Copy, Compact,
Deduplicate, Transform, Validate) or a toolkit that is not ported (TKPrim solids other
than the box, BRepGProp, BRepMesh, BRepBuilderAPI_Copy/MakeFace/Transform, BRepCheck,
BRepAlgoAPI), as the table lists. `crates/tkbrep/tests/brep_graph_editor_setters.rs`,
`brep_graph_editor_structure.rs` and `brep_graph_mesh_promote.rs` cover the editor
members that no suite calls, and the `br_bedit` requests compare every operation group
with OCCT byte for byte. `tools/audit_api_rows.py` gives these classes 176 new rows (the
destructors map to the `Drop` impls, the guard move constructor and assignment to
`move_from` and `move_assign`); the BRepGraph missing count falls from 199 to 23, the
public members of Copy, Compact, Deduplicate, Transform and Validate, with 0 stale rows.

Phase 2, wave 2c, area C1 ports the copy and rebuild algorithms whole
(docs/topology-design.md section 11.12): BRepGraph_Copy (the identity fast path into an
empty graph, the appended copy into a non-empty one, node copies and self copies under
the geometry, mesh and cache policies), BRepGraph_Compact (the dense rebuild of the
active definitions with the old generations and UIDs, root-product reachability, the
validation gate, the in-place installation with the registry callbacks, the layer
migration, the shape bindings and the `Compact:Remap` history batch) and
BRepGraph_Deduplicate (the geometry rewrites and the vertex, wire-order, edge, wire and
face merges with their history records). Compact's validation step calls
BRepGraph_Validate, merged from area C2's commit 1626b60. Two upstream suites now build
and are translated 1:1 (Lock, LayerTopoSupplement: 41 declarations, native and Rust
pass); the Lock suite maps to `brep_graph_lock_suite`, since the wave-2b behaviour file
owns `brep_graph_lock`. The Copy suite waits for TKPrim, BRepGProp, BRepBuilderAPI_Copy
and BRepAlgoAPI; the Compact and Deduplicate suites also call BRepGraph_Validate, and
the other suites that waited for these algorithms now wait for Validate or Transform
(area C2) only, as the table lists. `crates/tkbrep/tests/brep_graph_copy_behavior.rs`,
`brep_graph_compact_behavior.rs` and `brep_graph_dedup_behavior.rs` cover what those
suites would check on editor-built and box graphs, and the `br_balgo` requests compare
every copy, compaction and deduplication with OCCT byte for byte, the history records
and remapped ids included. `tools/audit_api_rows.py` gives the three classes 10 rows;
the BRepGraph missing count falls from 23 to 13, the public members of Transform and
Validate (area C2), with 0 stale rows.

Phase 2, wave 2c, area C2 ports BRepGraph_Validate whole and the in-graph part of
BRepGraph_Transform (docs/topology-design.md section 11.13): every validation check,
in the native order, both `MoveRef` overloads, and the geometry-level, location-only
and mesh steps of `Perform` and `TransformNode`. `Perform` and `TransformNode`
themselves take the `BRepGraph_Copy` policies and copy into the target first; after the
C1 merge they are `transform::perform`, `perform_in_place`, `transform_node` and
`transform_node_in_place` (see below). Six upstream suites now build and are translated 1:1 (ParentExplorer,
BRepGraph_Test, ReplaceVertex, Reverse, Sharing, CacheRegistry: 171 declarations,
native and Rust pass); `BRepGraph_Test.cxx` maps to `brep_graph_suite`, a root file
that includes six fragments in the upstream order. Its two `Allocator()` declarations
keep the rest of their body, since the accessor is excluded (docs/topology-design.md
section 11.8). The Reverse suite reads the box wires in their native connected order,
which the old box fake did not produce, so the fake now replays the box path of
BRepPrim_GWedge (see the fake paragraph above) and the box graph matches the native
one slot for slot. `crates/tkbrep/tests/brep_graph_validate.rs` breaks a box graph
through mutation guards and checks one issue of each audit family, and the transform
unit tests (`brep_graph::transform::tests`) cover the geometry, location, mesh-copy
and copied-range steps. `tools/audit_api_rows.py` gives these classes 11 new rows; the
BRepGraph missing count falls from 23 to 12: the ten public members of Copy, Compact
and Deduplicate (area C1) and `Transform::Perform` and `TransformNode`, with 0 stale
rows.

After the C1 merge, `Transform::Perform` and `TransformNode` are `transform::perform` and
`transform_node` (the policy checks, `copy::perform` or `copy::copy_node`, then the
copied range) with the self overloads `perform_in_place` and `transform_node_in_place`
(docs/topology-design.md section 11.13). Two more suites now build and are translated
1:1 (PermissionUpdate, which calls `Copy::CopyNode`, `TransformNode` and `MoveRef`, and
SparseModel, which calls Compact, Deduplicate and Validate: 43 declarations, native and
Rust pass). `tools/audit_api_rows.py` reports 0 missing and 0 stale api rows for
BRepGraph. Every BRepGraph suite still unmapped calls a toolkit that is not ported
(TKPrim solids other than the box, BRepGProp, BRepMesh, BRepBuilderAPI, BRepCheck,
BRepAlgoAPI, BRepBndLib), as the table lists.

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
- BRepGraph iterators, explorers and Tool (docs/topology-design.md section 11.9):
  `BRepGraph_Iterator<FaceDef, Full>` is `iterator::DefIterator<FaceTag, FULL>` with
  every native alias (`FaceIterator`, `FullFaceIterator`...); the defs and refs
  iterators share the `parent_traits` structs (`ShellOfSolidTraits`...);
  `BRepGraph_ReverseIterator` is the `reverse_iterator` module with the native aliases
  (`FacesOfEdge`, `RefsEdgesOfVertex`...). Each keeps `more`, `next`, `current`...
  and also implements `Iterator` (the native range). The explorer constructor overloads
  are `new`, `with_config`, `with_mode`, `with_avoid`, `with_target`,
  `with_target_mode`, `with_target_avoid`, `with_accumulation` and `with_start`;
  `GetConfig` is `config`. `BRepGraph_Tool::Edge` and its siblings are the modules
  `tool::vertex`, `tool::edge`, `tool::co_edge`, `tool::face`, `tool::wire` and
  `tool::shell`; overloads by reference or usage carry `_ref` or `_usage`, the oriented
  coedge lookups `_oriented`, the UV-bounded adaptors `_bounded`, and the
  `Bounds(..., double&...)` out-parameters are a returned tuple.

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

The BRepGraph navigation requests (`br_bgraph_walk`; 13 requests in
`tools/tkbrep_brepgraph_walk_corpus.py`) build a graph with the `br_bgraph` actions and
then edit products (a part placed twice in an assembly, a nested sub-assembly, a
compound root, and refused appends of a product under itself and of an invalid root).
They print every dense definition and reference walk (with a start id), the defs and
refs walks of every parent (the first index past each kind's end included), every
reverse walk (vertex edges, edge coedges, wires and faces from three start indices,
wire, face, shell, solid and occurrence parents, compound and occurrence parents of
every node, the ref-based lookups), the related nodes of every node, the child
explorer from every node under 19 configurations (all, direct, nine targets, three
direct targets, avoid kinds with and without emission, no accumulation, a start
transform) with the parent, link, reference, usage path, depth and the
`LocationOf`/`NodeOf`/`LocationAt`/`NodeAt` queries of every match, the parent
explorer under 14 configurations with the child, link, reference, leaf transform and
branch-root flag of every match, and every `BRepGraph_Tool` member over every vertex,
edge, coedge, face, wire and shell and their references, with usages placed at the
scene's last location (a scale in the solid scene). The scenes are the solid scene
under products, the cylinder (seam), mesh, Bezier, natural-restriction, degenerated
and free scenes, and the `br_brep` edge and face scenes with every shape appended.

The BRepGraph lock and cache requests (`br_bcache`; 14 requests in
`tools/tkbrep_brepgraph_cache_corpus.py`, probes
`tools/oracle/tkbrep_probe_brepgraph_cache*.cxx` and
`crates/tkbrep/examples/br_probe/brep_graph_cache*.rs`) ingest the solid, cylinder, mesh
and a free-edge scene and then: set owners on nodes and references (roots, ancestor
overlaps with the same and another owner, same-owner descendants absorbed, conflicting
descendants rejected, the asserting overload), unset them (by item and by owner), scope
an owner edit, dispatch removals and replacements, invalidate and clear, and use a
detached lock layer; register, bulk-register, reserve and unregister items of an
attached deferred layer (which locks them); query every derived-state answer, seed a
wrong wire closure and read it back through the cache and `BRepGraph_Tool`, invalidate it
by `Clear` and by `Add` under the wire or shell; write, clear and bump the mesh cache
through the mesh editor and the slot members of three slots, register, replace and
unregister logging drivers with `Ensure` over the graph, a node and a node list, and
`Needs`; and copy every layer and cache into a second graph with the identity and an
explicit item map (in copy and compaction mode) or migrate the layers in place. Both
graphs print their layer and cache registries, the owner of every node and reference,
the deferred entries, every derived answer, the slot states and every face, edge and
coedge entry of three slots with `Needs` of every face and compound, and every
persistent, cache and effective mesh-view query; then the drivers' Fill logs.

The BRepGraph editor requests (`br_bedit`; 18 requests in
`tools/tkbrep_brepgraph_edit_corpus.py`, probes `tools/oracle/tkbrep_probe_brepgraph_edit*.cxx`
and `crates/tkbrep/examples/br_probe/brep_graph_edit*.rs`) build a triangle face, its
shell, solid, compound, compsolid, products and occurrences from scratch, and edit the
ingested solid scene: every setter of every operation group, splits, vertex replacement
and removal, edge and wire reversal, edge replacement, coedge removal and reordering,
node, subgraph and reference removal with cleanup, node replacement, the container
removals, `RemoveShape`, the node and reference guards (dirty, clean, marked, refused),
`ApplyModification` with a nested action, deferred invalidation and a nested
`DeferredScope`, lock owners refusing each group's writes, the supplement editor for
every owner kind, and `PromoteToPersistent` over fresh, stale and locked mesh-cache
entries. The requests mix in the `br_blayer` logging layers, history and caches and the
`br_bcache` lock owners and mesh-cache writes. The probes print the graph (every
definition, reference and relation), the editor state and mutation boundary, the layer
registry with every logging layer's events, the history records and queries, the
supplement, the caches, the mesh cache and view and every node's shapes. A setter whose
target id is out of range, and a child rebound out of range, are not probed: the release
build writes or reads out of bounds there.

The BRepGraph algorithm requests (`br_balgo`; 32 requests in
`tools/tkbrep_brepgraph_algo_corpus.py`, probes
`tools/oracle/tkbrep_probe_brepgraph_algo.cxx` and
`crates/tkbrep/examples/br_probe/brep_graph_algo.rs`) run the `br_bedit` actions on one
of two graphs and copy between them: the identity copy of the meshed solid scene under
every geometry and mesh policy (with a logging layer, history, a lock owner, cache
meshes and a removed edge), the self and empty-source copies, node copies of every kind
into an empty and then a non-empty target followed by the full appended copy, self copies
of a face, solid, compound, product, edge and vertex, compactions after removals (history
on and off, both cache policies, a vertex added after the compaction, root products, a
removed subgraph, nothing removed, an empty graph) and deduplications of two triangles
built from equal but distinct geometry (analyzed and applied, with and without merges,
other tolerances, the removed-edge raise, a compaction afterwards) and of the ingested
scene. The probes print both graphs whole: every definition with its generations and
UID, every reference, the node and reference UIDs and stamps, the layer registry with the
logging layers' events, the history records with their mappings in map order, the
supplement, the caches, the mesh cache and view and every node's shapes.

The BRepGraph validation and transform requests (33 requests in
`tools/tkbrep_brepgraph_validate_corpus.py`, probes
`tools/oracle/tkbrep_probe_brepgraph_validate.cxx` and
`crates/tkbrep/examples/br_probe/brep_graph_validate_transform.rs`; action codes 260-273,
beside `br_balgo`'s 240-246) extend `br_bedit` with raw
field writes through a mutation guard's `Internal()` (edge vertex references and
representations, coedge edge, face, representations and wire, face representations,
occurrence and child-reference children, vertex-reference vertices), every
`BRepGraph_Validate::Perform` overload printing each issue (severity, node, text) in
report order, and both `MoveRef` overloads with translations, rotations, scales, the
three mirrors, general and singular transforms, removed and out-of-range references and
a lock owner. They validate an empty graph, a triangle face built from scratch (valid,
then each broken definition field restored after its audit, moved and removed vertex
references, seam pairs, a face without wires), assemblies (document roots, broken
occurrence children, cycles, a removed product with a surviving occurrence), compound
cycles and cross-container solids, and the ingested solid scene before and after
removals. A field written out of range is restored before the final reports, which
read the graph unchecked natively. Thirteen `br_balgo` requests run `Transform::Perform`
and `TransformNode` on the meshed solid scene (a cache mesh on the first face) for a
translation, a rotation, a scale, a point and a plane mirror and a general transform: in
place under each geometry and mesh policy pair, the refused ones included, and into the
other graph (empty, then not) for the whole graph and a face, a refused geometry-level
product, and an edge and a face in place, plus an empty source; both graphs are then
printed whole, their geometry, persistent meshes and mesh cache included. Six more
`br_balgo` requests (one per transform kind) add the meshed solid scene with an auto
product and run the location-only `TransformNode` of its product and of its occurrence
into the other graph (geometry shared, mesh dropped): the rigid transforms copy the node
with `CopyNode` and compose the placement, and the scale and the mirrors are refused
before the copy. These two actions were once left out because the native call crashed;
that was a probe setup error, not an upstream bug. The transform scenes are ingested
with `Perform`, which creates no product and no occurrence, so `PRODUCT 0` and
`OCCURRENCE 0` named ids past their stores, and `ensureProduct`/`ensureOccurrence` of
BRepGraph_Copy.cxx read `ProductRelations(theSrcId)` and `Occurrence(theSrcId)` without
a range check (a segmentation fault, undefined behaviour at the pinned commit). With
the product present the native and Rust answers agree; the Rust port refuses the
out-of-range ids with Standard_OutOfRange (docs/topology-design.md, section 11.14).

The corpus has 732 requests: TopoDS, TopExp and TopTools 84, BRep 8, BinTools 281,
BRepTools, BRepAdaptor and BRepLProp 195, the BRepGraph core 32, the BRepGraph layers and
caches 22, the BRepGraph navigation 13, the BRepGraph lock and caches 14, the BRepGraph
editor 18, the BRepGraph algorithms 32 and the BRepGraph validation and transform 33.

## Dependency gap

None left in TKBRep's own packages. `BRep_Tool::CurveOnPlane` (and the planar fallback
of `CurveOnSurface`, and `Parameter` for an internal vertex through it) projects the 3D
curve with `GeomProjLib::ProjectOnPlane` and `ProjLib_ProjectedCurve` from the
TKGeomBase port. BinTools reads a version 1 file with `BRepTools::Update(Face)` through
`brep_tools::update_face`; its former private copy is deleted.

`BRepTools_Test.cxx` has three mapped declarations: ASCII shape save/restore, empty-shape
save/restore for all eight topology kinds, and box face/edge/vertex tolerances. Its pinned
OCCT CTest filter passes 11/11; Rust `brep_tools` passes the three mapped tests. The other
eight declarations remain unmapped: parallel reads need the Rust OSD parallel API; format
version and binary round trips need the corresponding NURBS conversion, BinTools, mesh,
validity, bounds and global-property behavior; triangle-option and Clean regressions need
TKMesh meshing; exact versus triangulated properties needs TKBO Fuse plus mesh and global
properties; the UV-bounds regression needs TKBO Cut. Passing native CTest results are not
counted as Rust mappings.

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
- The BRepGraph iterators keep their release-build answers past the end:
  `DefsOfParent::CurrentId` returns the last child found, and `RefsVertexOfEdge` and
  `DefsVertexOfEdge` read the end vertex reference. `EdgeParentsOf` (`WiresOf`,
  `FacesOf`) resumed at a start index reports a parent only at its first coedge, so a
  parent whose first coedge precedes the start index is not reported at all.
- `BRepGraph_RelatedIterator` does not deduplicate (a face adjacent through two edges
  is reported twice), resumes each relation from saved indices, and ends a relation at
  its first inactive node even when later ones are active (the native `setCurrent`
  returns false and the stage moves on).
- `BRepGraph_ChildExplorer` pops a frame when a child reference names an invalid node,
  so later siblings of that frame are skipped. An occurrence explored without an
  incoming reference takes the location of the first live reference to it in id order.
  A push past the node count (the cycle guard) is refused, and the emitted frame is
  then the parent's. The CoEdge to Edge, Wire to CoEdge and Occurrence to child steps
  are structural (no reference, step -1).
- `BRepGraph_ParentExplorer` emits a branch bottom-up once it reaches its root, with
  the transforms prepared from the root down. A reference step records its position
  among the parent's active references, while its transform reads `RefAtStep`, which
  indexes the raw relation list; the two differ after a removed reference. The CoEdge
  to Wire step is found by position but recorded as structural. The compound parents
  are counted before their step is checked, so a compound that does not list the child
  still consumes an ordinal.
- `BRepGraph_Tool`: the derived flags (`Degenerated`, `IsClosed`, `SameRange`,
  `SameParameter`, `Wire::IsClosed`, `Shell::IsClosed`) answer false for an inactive
  entity, as the cache does, even where the static computation says true (an inactive
  coedge is same-range there). `Face::OuterWire` keeps the last wire whose UV box
  contains the candidate's within PConfusion, starting from the first wire with a box.
  `Wire::FaceOf` returns the parent face of the first reference that is not removed
  without checking the face. `CoEdge::PCurveAdaptor` projects on a plane or a
  rectangular trimmed plane only (one trim level). `Edge::IsSeamOnFace` compares the
  orientations of the edge's coedges on the face; `CoEdge::SeamPair` also requires the
  opposite orientation. `CurveOnSurface` takes the face argument, not the coedge's.

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

- The editor keeps the release behaviour of its assertions: `Standard_ASSERT_RETURN`
  checks return their value (`RemoveVertex`, `RemoveWire` and `RemoveRef` with ids out of
  range return false; `SetRefChildOccurrenceId` with an occurrence another reference
  owns leaves the reference unchanged; a new vertex reference that is owned or removed
  is ignored by `SetStartVertexRefId`/`SetEndVertexRefId`), and `Standard_ASSERT_VOID`
  is compiled out, so `CommitMutation` has no effect.
- `Edges().Reverse` and `Wires().Reverse` do not check the lock layer: they rewrite a
  locked edge or wire, while every setter and `Mut` refuses it with
  Standard_ProgramError. `Edges().Split` and `ReplaceVertex` with an inactive input
  return the invalid ids without raising.
- `ApplyModification` runs the modifier before it records anything; a modifier that
  raises leaves no history record. `DeferredScope` commits only when it opened the
  deferred mode, so a nested scope leaves the outer one deferred.
- `PromoteToPersistent` re-reads each cache entry after the earlier writes: a face's
  promoted triangulation notifies the face, so the coedge polygons on that face's
  triangulation turn stale and are not promoted; a refused write stops the promotion.
- `BRepGraph_Validate` runs the mutation boundary check in both modes, and its
  lightweight mode adds only removed-node isolation and the document roots. The audit
  reports a missing 3D curve only when some face of the edge has a live surface (a
  free edge always needs one), and reports the owner checks of the representation
  stores after all per-use checks. A product that contains itself directly stops its seed
  loop but still searches the children seeded before it, so one product can report the
  direct and the transitive cycle; the compound search does the same. The failure of
  the supplement layer's `Validate` becomes one error whose text ends with the
  failure's `what()`.
- `BRepGraph_Transform::MoveRef` refuses only a scaling transformation (`|scale|` off
  1 by more than `TopLoc_Location::ScalePrec()`): a mirror has scale -1 and is composed
  into the placement. The copied-range mesh step transforms the persistent
  triangulations and polygons of the copied faces and edges in place, as the native
  code does for the fresh copies of BRepGraph_Copy: a handle still shared with nodes
  before the range is transformed for them too.

- `BRepGraph_Copy` copies a removed node as a node that is then marked removed, so the
  editor refuses a parent over it: a face over a removed wire, or a compsolid over a
  removed solid, is not created (its copy is the invalid id). A wire whose stored coedge
  order is not connected is refused the same way, so its face is copied without it.
  `CopyNode` of a coedge copies nothing (coedges are copied with their wire). The
  occurrences and products it appends get no UID, as natively.
- `BRepGraph_Compact` rebuilds the vertex references' orientations and the wire,
  solid and compound references through guards, so the rebuilt references carry the
  generations of those writes; the definitions take the old generations and UIDs, the
  references and coedges keep the fresh UIDs, and the UID counters restart after the
  largest surviving UID. A free coedge is matched to the first new coedge with its edge
  and face whose id is not bound as an *old* key of the coedge map (the native check). A
  rebuild that fails the mutation boundary or the validation keeps the graph as it was.
- `BRepGraph_Deduplicate` takes each edge's guard before it checks the edge is removed
  when it redirects merged vertices, so a graph with a removed edge and a vertex merge
  raises Standard_ProgramError (the deferred scope still ends). Its edge and face groups
  hash the curve and surface handle addresses (the process-dependent `std::hash` of a
  handle), and it sorts the vertices with libstdc++'s `std::sort`, whose tie order it
  follows. `NbNullifiedSurfaces`, `NbNullifiedCurves`, `NbRemovedSurfaces` and
  `NbRemovedCurves` are never written.
- The three algorithms restore the history's enabled state with plain statements on
  their return paths, so after a raise the history keeps the run's `HistoryMode`.

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
  `UVBounds` stops at the sample whose increment does not move the across parameter (an
  empty or few-ulp range, or an infinite bound; natively it never advances). The
  sampled estimate of `EvalAndUpdateTol` (steps of at least Confusion, which round back
  at a parameter beyond about 1e9) and the step search of `BRepTools_WireExplorer`
  (`GetNextParamOnPC`, a thousandth of a range of a few ulps) raise
  Standard_DomainError where the native scan stalls, through the TKGeomBase
  `scan_step` guard (docs/tkgeombase-progress.md); every scan that ends natively takes
  the same samples. A null curve, p-curve or surface given to `EvalAndUpdateTol`,
  `AddUVBounds` on a face without surface, a NurbsConvert seam without 3D curve and
  `Update` of a null shape raise Standard_NullObject instead of crashing, and the
  shape set's unchecked TShape casts raise Standard_TypeMismatch. The load members of
  the triangulation tools take the file system only for their signature: the TKMath
  deferred triangulations own their input.
- `br_tools_read` leaves out a text that ends right after the version line: the native
  location reader then compares a `char` buffer the failed `>>` left uninitialized.
- Reconstruction and `BRepGraph_Tool` read SameRange, SameParameter, the edge flags and
  the wire and shell closures through `BRepGraph_CacheDerivedState`, which the first read
  registers in the graph's cache registry. An entry is kept while the entity's OwnGen is
  unchanged, so a stored `SetWireIsClosed` value is served until the wire changes, and a
  coedge's SameParameter reuses the SameRange flag of its entry. `Clear()` resets only
  the node binding of each entry (the packed flags stay, unread).
- `BRepGraph_LayerLock::SetOwner` refuses a root under an ancestor root or above a root
  of another owner and absorbs same-owner descendant roots; a re-registration by the
  same owner only re-sets the item's bit (and bumps the revision). `UnsetOwner` clears the
  root's subtree bits and then sets the bits of every remaining root again.
  `OnNodeReplaced` moves only the two nodes' own bits. The native `removeRootEntry` is
  called nowhere; it is ported for completeness.
- `BRepGraph_CacheMesh::Clear` bumps every slot's generation and keeps the drivers;
  `RegisterDriver` with the same ID and recipe keeps the slot. `CopyFreshTo` clears each
  target slot, bumps the mesh generation of every copied face entry and copies polygons
  on triangulation only when the target coedge's face has a cached triangulation. A
  coedge entry bound for a coedge without face keeps its previous face stamp. A face
  entry is bound to the SubtreeGen (not the OwnGen) of its face.
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
- `BRepGraph_LayerDeferred::OnNodeReplaced` stops appending at eight representations
  where the native noexcept callback terminates the process. `UnregisterDeferred` and
  `EndBulkRegistration` take the graph, since they unlock or touch the attached graph's
  lock layer.
- `BRepGraph_LayerLock` writes the owned bits from `const` members and noexcept
  callbacks through the graph it is attached to, so the storage owned plane is an atomic
  bit plane whose `SetOwned` takes `&self` (docs/topology-design.md 11.11). The members
  that walk the parents or children (`FindOwnerId`, `SetOwner`, `UnsetOwner`, the
  scope) return `Result`, as the explorers raise; the noexcept callbacks (`OnNodeRemoved`,
  `Clear`...) stop walking on an error where the native process terminates.
- `BRepGraph_LayerRegistry::CopyLayersTo` and `BRepGraph_CacheRegistry::CopyFreshCachesTo`
  are associated functions taking the source graph (whose registry they use) and the
  target; `copy_layers_in_place` and `copy_fresh_caches_in_place` are the calls with the
  graph itself as the target (the compaction). A layer's `CopyTo` and a cache's
  `CopyFreshTo` return `Result` (the native assertions and nested registrations raise).
  The caches copy their source entries before they lock the target, so a copy into the
  same service cannot deadlock (natively both mutexes are taken).
- The mesh-cache getters return copies of the entries (natively pointers into the slot);
  the `Change*Mesh` members lend the entry to a closure under the cache lock, so the
  native "change then `BindFresh`" sequence binds a copy and stores it back (the mesh
  editor does both under one lock). `Change*Mesh` and `BumpFaceMeshGeneration` refuse
  the invalid id with Standard_OutOfRange (natively a 2^32-entry allocation), and the
  derived-state wire and shell stores refuse an entity they cannot bind before growing
  (the answer is the same). A coedge out of range binds no face stamp (natively an
  unchecked read). When another class holds the derived-state or mesh-cache GUID
  (natively a null dereference), the Tool reads compute without the cache and the mesh
  views find no cache.
- `BRepGraphInc_Reconstruct::Node` with a node index out of its kind's range raises
  Standard_OutOfRange (natively an unchecked vector read). Parallel face extraction
  returns the error of the failing face with the lowest queue index, so a parallel and
  a sequential population fail the same way.
- The editor holds the graph's one mutable borrow (docs/topology-design.md section
  11.4): a guarded setter is an associated `*_guarded` function taking the guard, a
  second guard is taken through the first one's `graph_mut()`, the guard move members
  are `move_from`/`move_assign`, `Editor() const` is `editor_state()`, and
  `PromoteToPersistent` is an associated function taking the graph mutably. The
  `ApplyModification` modifier returns `Result`. A setter or guard on an id out of its
  kind's range panics in the storage (natively an unchecked write).
- The BRepGraph iterators and explorers keep the native `More`/`Next`/`Current`
  protocol and implement `Iterator` for the range form. The child and parent explorers
  and the related iterator compose locations or build p-curve adaptors while they
  advance, which raises natively in degenerate cases, so their constructors and `next`
  return `Result` and their `Iterator` yields `Result` items (an error ends the
  iteration). `BRepGraph_Tool` members that construct adaptors, transform a curve,
  evaluate a p-curve or compute the outer wire return `Result` for the same reason.
- Reads the native code leaves unchecked give a defined answer or panic in Rust:
  `RefsOfParent::CurrentId` past the end returns the invalid id (natively a read past
  the list); `ChildExplorer::LocationAt/NodeAt` below level -1 return identity and the
  invalid id (natively a read before the stack); a parent exploration from a node out
  of its kind's range yields nothing (natively the Wire, Face, Shell, Solid and
  Occurrence cases read its relations unchecked); a `BRepGraph_Tool` member given an id
  out of range panics, as `Definition()` does.
- `BRepGraph_CopyRemap` holds the one mutable borrow of its target: the native context
  gives the mutable target from a `const` reference and lets source and target alias
  during compaction, so the Rust context has `target_graph(&mut self)`, an `in_place`
  constructor for the aliasing case, and `items()` returns `None` for the identity
  mapping (a null map natively); an explicit mapping without a map maps nothing
  (natively a null dereference).
- `BRepGraph_Copy`, `BRepGraph_Compact` and `BRepGraph_Deduplicate` are modules of free
  functions returning `Result` (docs/topology-design.md section 11.12). A self copy (the
  same graph as source and target) cannot be passed as a `&BRepGraph` and a
  `&mut BRepGraph`, so it is `copy::perform_self` (the identity no-op) and
  `copy::copy_node_in_place`; the copy reads each source record before it writes the
  result, which natively sees the same values through references that stay valid. A
  face or compsolid the editor refused (the invalid id) pairs no references (natively
  its relations are read out of range).
- The compaction keeps the graph's registries in place and swaps its storage with the
  rebuilt graph's instead of moving the registries out and back; the callbacks of the
  native moves are replayed (the fresh caches adopted under `CopyFresh`, the saved layers
  re-attached, the kept caches cleared under `Drop`), and the rebuilt graph's own layers
  and caches are detached when it is dropped. The part-occurrence lookup skips an
  occurrence reference whose occurrence id is out of range (natively an unchecked read).
- `BRepGraph_Validate` is the module `brep_graph::validate` with `perform`,
  `perform_with_mode` and `perform_with_options`; `Result` is `ValidationResult`. The
  native sets whose order never reaches a report (expected wires, visited products and
  compounds, compound-owned solids) are standard hash sets. The two `MoveRef` overloads
  are `transform::move_child_ref` and `move_occurrence_ref`, returning `Result` because
  a locked reference raises. Every native caller of `applyMeshCopy` passes the
  transformed graph as source and destination and no poly source, so the Rust step
  takes the one graph; its per-handle maps are keyed by object address and keep the
  source handle alive, as the native handle keys do.

## Recovered BRepGraph edge-case suite

All 12 declarations from `BRepGraph_EdgeCases_Test.cxx` are now mapped using the original
box, sphere and compound builders. They cover empty input, invalid IDs, rebuild counts,
UID generations and the sequential/parallel import options. All previously mapped
TKBRep tests also run with the shared production-builder fixtures. The broader graph
source suites still require translation; toolkit acceptance is not complete.

### Recovered BRepGraph build declarations

All 59 original `BRepGraph_Build_Test.cxx` declarations run through the production
TKPrim and TKTopAlgo builders. The tests retain the original primitive dimensions,
TopExp comparisons, flattening options, UID checks, and natural-boundary inputs.
Unbounded faces retain OCCT's warning behavior; synthesized plane PCurves are
checked through the adaptor without requiring a stored PCurve.

### Recovered fixed-seed graph mutations

Both original `BRepGraph_FuzzSeedTest` declarations retain all seven seeds and their
50 box / 40 cylinder iterations. The fixtures store input indices and random deltas,
not expected kernel outputs. Each Rust mutation still runs through the guarded editor,
followed by the original audit assertion. The native cases run unchanged in the gate.

`tools/generate_tkbrep_fuzz_inputs.py --source ../OCCT --native-build BUILD` reproduces
`tests/graph_fuzz/inputs` with the compiler recorded in the oracle CMake cache (this
capture: Ubuntu GCC/libstdc++ 13.3.0, `-std=c++17 -O2`). Its C++ translation unit includes
the unchanged upstream test and checks the RNG state, bitwise coordinate/tolerance
results, and audit after every call to the original `applyOne`. This pins libstdc++'s
uniform distributions and GCC's coordinate argument evaluation order. Every one of the
630 captured operations applies; no retry or skip is omitted from these fixtures.
The source excludes split/removal from this stream under its Phase 5.10/5.11 comment.

### BRepGraph polygon suite in progress

A Rust draft covers all seven assertions from `BRepGraph_Polygon_Test.cxx`, and
the pinned native cases and Rust target pass. Its seam-edge fixture currently
attaches a constructed triangulation instead of exercising Rust incremental
meshing, so these declarations remain unmapped until the end-to-end path matches.

Fixture SHA-256:

- `box.txt`: `34689d0d04d03e8322fe5b394970327c69876ac9e55b0d35028dbe4efe97b323`
- `cylinder.txt`: `20fad48d5b540c6d23618e1f970e7b0fb48a4786704a7f6a648cf64d81a23abf`

The toolkit verifier now recognizes `TEST_P` and `TYPED_TEST_P`, with parser regressions;
its existing runtime expansion checks require a distinct Rust test for every native case.

### Deduplicate source-disabled regression

All 82 Deduplicate declarations are mapped. The 81 active originals pass natively
and in Rust. The original disabled assertion remains disabled upstream and fails
when forced (eight vertices remain, so `8 < 8` is false). Its assertion is retained
as a Rust reference, alongside a separate passing current-behavior regression.
The gate records that original as `upstream_disabled`, never as a passing legacy
test, and requires the expected native failure when explicitly enabled.

### BRepGraphInc population regressions

All nine declarations in `BRepGraphInc_PopulateRegression_Test.cxx` are mapped and
pass in the filtered native and Rust gate. The two mesh-sensitive box cases use
deterministic face-local fixtures captured from pinned OCCT: at deflections 0.5 and
0.2, each planar box face has four corner nodes and two triangles. The Rust fixtures
preserve the native node ordering, triangle diagonals, and edge-to-face polygon
associations, while avoiding a dependency on the not-yet-ported `BRepMesh_IncrementalMesh`.
