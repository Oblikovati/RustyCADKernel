# TopoDS, TopExp and TopTools in Rust

This note fixes how the topology packages of TKBRep (`TopoDS`, `TopExp`,
`TopTools`) map to Rust in `crates/tkbrep` (`rustycad-tkbrep`). It extends
[geom-design.md](geom-design.md): handles, RTTI, shared mutable state and
exceptions follow that note, and this one only covers what Geom does not have.
The TopLoc package that the shapes use is the TKMath port
(`rustycad_tkmath::toploc`), reused as it is.

## 1. TopoDS_Shape is a value

`TopoDS_Shape` is not a transient. It is a small value with three fields:

| OCCT field | Rust field | Meaning |
| --- | --- | --- |
| `occ::handle<TopoDS_TShape> myTShape` | `Handle<dyn TShape>` | the shared topology (null for a null shape) |
| `TopLoc_Location myLocation` | `toploc::Location` | the placement of this use of the TShape |
| `TopAbs_Orientation myOrient` | `top_abs::Orientation` | the orientation of this use |

`topods::Shape` derives `Clone` (a handle copy and a location copy, as the C++
copy constructor). A default shape is null, with the identity location and the
`EXTERNAL` orientation, as `TopoDS_Shape()`. Setters (`set_location`,
`set_orientation`, `reverse`, `move_by`...) take `&mut self` and change only this
value; `located`, `oriented`, `reversed`, `moved`... return a changed copy, as the
native `Located`, `Oriented`... members do. The OCCT member names that are both a
getter and a setter (`Location()` / `Location(L)`, `Free()` / `Free(bool)`) become
`location` / `set_location`, `free` / `set_free`, and so on.

The typed shapes `TopoDS_Vertex`, `TopoDS_Edge`, `TopoDS_Wire`, `TopoDS_Face`,
`TopoDS_Shell`, `TopoDS_Solid`, `TopoDS_CompSolid` and `TopoDS_Compound` add no
field natively. In Rust each one is a newtype over `Shape` (`topods::Vertex`...)
with `Deref`/`DerefMut` to `Shape` and `From<Vertex> for Shape`, so every `Shape`
member is available on it and a typed shape goes wherever a `&Shape` is expected.
The builder takes the typed shape it makes (`make_wire(&mut Wire)`), as natively.

The casts of the `TopoDS` namespace (`TopoDS::Vertex(S)`...) check the type
(`Standard_TypeMismatch` when the shape is not null and has another type) and
return a typed copy: a Rust reference cannot be reinterpreted as another type
without `unsafe`. The non-const overloads, which return a `TopoDS_Vertex&` alias
of the argument, therefore also return a copy: write the modified copy back when
the original must change. The TopExp functions use the unchecked cast, because
`TopExp.cxx` defines `No_Standard_TypeMismatch` (see section 7).

## 2. The TShape hierarchy

`TopoDS_TShape` is abstract (its `EmptyCopy` is pure virtual), so it is the trait
`topods::TShape` with a `Transient` supertrait, registered as `TopoDS_TShape` by
`impl_transient_interface!`. The shared state of the base class (the child list
and the 16-bit state word) is the struct `TShapeBase`; each concrete class embeds
it and the trait reads it through the required accessor `tshape_base()`. Every
flag getter and setter, `ShapeType` and `NbChildren` are provided trait methods
written once over that base, as the non-virtual native members are.

| OCCT | Rust | Native constructor defaults |
| --- | --- | --- |
| `TopoDS_TVertex` (abstract) | `TVertex` struct, not a `TShape` | Closed, Convex |
| `TopoDS_TEdge` (protected constructor) | `TEdge` | none |
| `TopoDS_TWire` | `TWire` | none |
| `TopoDS_TFace` | `TFace` | none |
| `TopoDS_TShell` | `TShell` | none |
| `TopoDS_TSolid` | `TSolid` | not Orientable |
| `TopoDS_TCompSolid` | `TCompSolid` | none |
| `TopoDS_TCompound` | `TCompound` | not Orientable |

Every TShape starts Free, Modified and Orientable, with its type in bits 0-3 of
the state word (the `BitLayout` enumeration is the `bit_layout` constants).

`TopoDS_TVertex` has no `EmptyCopy`: it cannot be instantiated alone. The Rust
`TVertex` is a transient struct with the `TopoDS_TVertex` descriptor that holds
the vertex defaults; a derived class (BRep_TVertex when BRep is ported, or a test
fake) embeds it as its parent field, implements `TShape` and supplies
`empty_copy`. `TopoDS_TEdge` has a protected constructor but a public
`EmptyCopy` that returns a plain `TopoDS_TEdge`, so `TEdge` implements `TShape`
and `TEdge::new` is public (Rust has no protected visibility across crates); its
`empty_copy` returns a `TEdge`, which is the native result for a derived edge
class that does not override it.

The derived classes register their parents with `impl_transient!(..., interfaces:
[dyn TShape])`, so `DynamicType`, `IsKind` and the descriptor chain follow
`TopoDS_TSolid -> TopoDS_TShape -> Standard_Transient`.

## 3. Shared mutable state of a TShape

A TShape is shared by every shape that references it, and `Builder::Add`,
`Builder::Remove` and the flag setters change it for all of them. As in
geom-design.md section 3, `TShapeBase` keeps the state behind the TKernel
`SharedState` lock: the child list and the state word are one locked struct, and
every member takes `&self`. A member never calls another shape while it holds the
lock: `Builder::Add` reads the component, then takes the parent's write lock only
to append. The child list is a `Vec<Shape>` (the private
`NCollection_List<TopoDS_Shape> myShapes`): only `TopoDS_Iterator` and
`TopoDS_Builder` read it natively (they are friends), and in Rust only the
iterator (by index) and the builder (append and remove) use it.

The native iterator walks the list nodes. The Rust iterator keeps the TShape
handle and an index. The results are the same while the list does not change;
native behaviour when the current node is removed during an iteration is
undefined (the node is freed), and the Rust iterator then continues at the next
index.

## 4. Shape flags

`Free`, `Modified`, `Checked`, `Orientable`, `Closed`, `Infinite`, `Convex` and
`Locked` are bits 4 to 11 of the TShape state word; they are shared by every
shape of one TShape. `set_modified(true)` also clears `Checked`, as natively.
The native `TopoDS_Shape` flag members dereference the TShape handle without a
check: a null shape crashes. The Rust members return
`Result<_, Failure>` and raise `Standard_NullObject` for a null shape, which is
the exception `TopoDS_Shape::ShapeType` documents. `NbChildren` of a null shape
is 0 natively and in Rust.

## 5. TopoDS_Builder rules

`TopoDS_Builder` is a stateless struct. `make_wire` ... `make_compound` create a
new TShape and set the shape to it with the identity location and the `FORWARD`
orientation (`MakeShape`). `add` first clears the component's `Free` flag
(freezing it even when the call then fails, which prevents self insertion), then
checks, in this order:

1. the parent is free, else `TopoDS_FrozenShape`;
2. the pair is allowed by the native table, else `TopoDS_UnCompatibleShapes`:

| Component | Allowed parents |
| --- | --- |
| COMPOUND, COMPSOLID | COMPOUND |
| SOLID | COMPOUND, COMPSOLID |
| SHELL | COMPOUND, SOLID |
| FACE | COMPOUND, SHELL |
| WIRE | COMPOUND, FACE |
| EDGE | COMPOUND, SOLID, WIRE |
| VERTEX | COMPOUND, SOLID, FACE, EDGE |

The child is stored reversed when the parent is REVERSED, and moved by the
inverse of the parent location, so that iterating with the cumulated orientation
and location gives the component back. `add` sets `Modified`. `remove` raises
`TopoDS_FrozenShape` for a frozen parent, then removes the first child equal
(same TShape, location and orientation) to the component in the parent's
frame, and sets `Modified` only when one was removed. The three exception
classes (`TopoDS_FrozenShape`, `TopoDS_LockedShape`,
`TopoDS_UnCompatibleShapes`, all Standard_DomainError natively) are `Failure`
constructors in `topods::errors`. A null parent or component crashes natively
(null handle dereference); Rust raises `Standard_NullObject`.

TopoDS itself raises no `Standard_ImmutableObject`: that exception is not used by
any TKBRep package at the pinned revision. TopoDS only stores the `Locked` flag;
`BRep_Builder` reads it and raises `TopoDS_LockedShape` (section 9).

## 6. TopoDS_Iterator and TopExp_Explorer

`TopoDS_Iterator` composes each child orientation with the cumulated orientation
of the shape (`cumOri`) and moves each child by the shape location (`cumLoc`).
`next` past the end is a no-op, as the native list iterator's `++` is. `value`
raises `Standard_NoSuchObject` when there is no current shape.

`TopExp_Explorer` keeps the native depth-first stack of iterators. Its quirks are
kept: `next` after the end restarts the exploration from the root (the stack is
empty again), `current` at the end returns the explored shape, and `next` on an
explorer whose shape is null raises `Standard_NullObject` (a crash natively). Both
types also implement `std::iter::Iterator` (the native `begin`/`end` range).

## 7. Hashing and equality

`IsPartner` compares the TShape handles, `IsSame` also the locations, and
`IsEqual` (and `==`) also the orientations. The native `std::hash<TopoDS_Shape>`
is

```text
hl = hash(Location)            // TopLoc: Murmur chain over datum addresses and powers
hash(shape) = hl == 0 ? MurmurHash64A(&tshape_ptr, 8, 0xA329F1D3A586)
                      : MurmurHash64A(&tshape_ptr, 8, hl)
```

`Shape::hash_code` computes it from the TShape object address (the address the
handle owns, as `TShape().get()`) and `Location::hash_code`. Addresses are
process-specific natively as in Rust, so the formula is also exposed on explicit
parts (`Shape::hash_from_parts(address, location_hash)`); the oracle compares it
bit for bit on synthetic addresses and checks, on each side, that the real hash
equals the formula on the real address. `TopTools_ShapeMapHasher` is the
`HashPolicy` `ShapeMapHasher` (hash above, equality `IsSame`), and the maps of
TopExp are `IndexedMap<Shape, ShapeMapHasher>`,
`IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>` and `Map<Shape, ShapeMapHasher>`.
An indexed map keeps insertion order, so its order is the exploration order in
both implementations. The iteration order of a plain `Map` follows the hash and
therefore the object addresses, natively as in Rust: it is deterministic for a
given set of addresses and is compared through the synthetic hash only.

## 8. TopTools_ShapeSet

`TopTools_ShapeSet` is a class with virtual hooks (`AddGeometry`,
`WriteGeometry`, `ReadGeometry`, `AddShapes`, `Check`...) that BRepTools and
BinTools override. It follows the "protected data of an abstract class" rule of
geom-design.md: the state (the shape map, the location set and the format
number) is `ShapeSetData`, the trait `ShapeSet` reads it through two accessors,
the hooks are provided methods with the native no-op bodies, and the
non-virtual members (`Add`, `Index`, `Dump`, `Write`, `Read`...) are provided
methods written once. `TopToolsShapeSet` is the base class itself. The virtual
`Clear`, `Dump`, `Write` and `Read` are trait methods too, and their base
bodies are the free functions of `toptools::shape_set_io`, so an override can
call the base, as `TopTools_ShapeSet::Write(OS)` is called by derived classes.

The native file includes `BRep_TFace.hxx` but uses nothing from it, so the whole
class is ported without BRep. Text written to `std::cout` by `Read` ("Not a TShape
table"...) goes to the injected `ConsoleOutput` of the GeomTools port.

## 9. The BRep package

`BRep` (module `brep`) adds the geometric TShapes and their representations. It
follows sections 2 and 3 and geom-design.md sections 1 to 4.

- The concrete TShapes `BRep_TVertex`, `BRep_TEdge` and `BRep_TFace` are the
  structs `BRepTVertex`, `BRepTEdge` and `BRepTFace`. Each embeds its TopoDS parent
  (`topods::TVertex`, `topods::TEdge`, `topods::TFace`) as its parent field, so the
  descriptor chain is `BRep_TEdge -> TopoDS_TEdge -> TopoDS_TShape`, and keeps its own
  fields (point, tolerance, flags, representation lists, surface, triangulations)
  in one `SharedState`. The `BRep` prefix keeps the names apart from the TopoDS
  classes and from the Poly polygons (`poly::Polygon3D` and `BRepPolygon3D`).
- The abstract `BRep_CurveRepresentation`, `BRep_GCurve`, `BRep_PointRepresentation`
  and `BRep_PointsOnSurface` are the traits `BRepCurveRepresentation`, `BRepGCurve`,
  `BRepPointRepresentation` and `BRepPointsOnSurface`, registered with
  `impl_transient_interface!`. Their protected data (the location, the range, the
  parameter, the surface) are the structs `CurveRepresentationBase`, `GCurveBase`,
  `PointRepresentationBase` and `PointsOnSurfaceBase`, each with its own
  `SharedState`, which the concrete classes embed and the traits read through one
  required accessor. A virtual member that the base class implements by raising
  Standard_DomainError is a provided trait method that returns that error, so every
  such getter and setter returns `Result`.
- A concrete class derived from a concrete class (`BRep_CurveOnClosedSurface` from
  `BRep_CurveOnSurface`, `BRep_PolygonOnClosedSurface`,
  `BRep_PolygonOnClosedTriangulation`) embeds its parent struct as its parent field
  and forwards the members it does not override, so `down_cast` to the parent class
  succeeds as natively.
- `DumpJson` bodies are free functions per class, so that a derived class calls the
  body of its base class (`OCCT_DUMP_BASE_CLASS`) with the static class name, as the
  native qualified call does.
- The lists `Curves()`, `Points()` and `Triangulations()` return copies of the handle
  lists; `ChangeCurves()` and `ChangePoints()` lend the stored list to a closure. A
  closure may call the representations, which have their own locks, but never the
  TShape that lends the list.
- `BRep_Builder` is `BRepBuilder`, which derefs to `topods::Builder` as the native
  class derives from `TopoDS_Builder`. `BRep_Tool` is `BRepTool`, a struct of
  associated functions; its overloads get names that say what they take
  (`curve_on_surface`, `curve_on_face`, `curve_on_surface_by_index`...), and its
  output parameters become returned values or small result structs.
- Natively, `BRep_Builder` and `BRep_Tool` cast the TShape of the shape they get
  without a check (`static_cast`, or a reinterpretation of the handle): a null
  shape or a TShape of another class is undefined behaviour. The Rust members
  raise Standard_NullObject for a null shape and Standard_TypeMismatch for a TShape
  that is not the expected BRep class.

## 10. The BinTools package

`BinTools` (module `bintools`) writes and reads shapes in the two binary formats. It
follows section 8 for the virtual shape set and adds:

- Streams. `Standard_OStream`/`Standard_IStream` are the TKernel `IoStream` model, as
  for TopTools and GeomTools. `BinTools_OStream` and `BinTools_IStream` borrow it
  (`BinOStream<'s>`, `BinIStream<'s>`) and keep their byte position and last object
  type; their `operator<<`/`operator>>` overloads are named by type. Values use the
  platform byte order (`DO_INVERSE` is never defined at the pinned revision). The file
  variants of `BinTools::Write`/`Read` open their stream with an OSD file system and
  put the storage stream on its `std::streambuf` (`sgetn`, `sputn`, `pubseekoff`,
  `pubsync`), as the native `std::iostream` sits on it.
- Class shape. `BinTools_ShapeSetBase` is concrete with empty virtual members: the
  trait `BinShapeSetBase` reads the private fields (`ShapeSetBaseData`, which is also
  the base class instance) through two accessors, and its provided methods are the
  empty bodies. `BinShapeSet`, `BinShapeWriter` and `BinShapeReader` implement it; no
  OCCT class derives from them, so their own virtual members are inherent methods.
- Tables. The `NCollection_IndexedMap<occ::handle<T>>` tables compare handles by
  object identity (null is a valid key), so their order is insertion order and the
  written indices are deterministic. The position maps of the shape writer and reader
  are only searched, never iterated.
- Readers that must report a value the native enum variable would hold (a shape type
  byte, an object type byte) keep the raw integer (`ObjectType(u8)`, an `i32` shape
  type); values that the native code only stores (an orientation or continuity byte)
  are checked when converted, and an out-of-range one raises Standard_OutOfRange.

## 11. BRepGraph and BRepGraphInc

`BRepGraphInc` (module `brep_graph_inc`) is the incidence-table backend and `BRepGraph`
(module `brep_graph`) the facade over it. The port is split in two phases. Phase 1 fixes
the storage, the ids, the graph value, the population and reconstruction pipelines and
the four read views; phase 2 adds the iterators, the editor, the algorithms and the
layers and caches on top, each in its own modules (section 11.7).

### 11.1 Ids are values with an explicit kind

`BRepGraph_NodeId`, `BRepGraph_RefId` and `BRepGraph_RepId` are `(Kind, Index)` pairs.
Their `enum class Kind` may hold any `int` natively (the tests build
`static_cast<Kind>(9)`, the reserved slot, and expect `IsValid()` to be false), so the
Rust kinds are transparent newtypes over `i32` with one associated constant per
enumerator (`NodeKind::FACE`, `RefKind::WIRE`, `RepKind::FACE_SURFACE`), and
`is_valid_kind` checks the value as the native `switch` does. `NodeId`, `RefId` and
`RepId` are `Copy` structs with public `kind` and `index` fields; the default is the
invalid id (index `u32::MAX`, kind 0).

The typed wrappers `Typed<Kind::Face>` are one generic value `TypedId<T>` where `T` is a
zero-sized kind tag (`FaceTag`, `WireRefTag`, `FaceSurfaceTag`...). The tag trait
`KindTag` names the family (node, reference or representation) and the raw kind, so the
conversions into the untyped id, the kind check of `FromNodeId`/`FromRefId` and the hash
are written once per family. The aliases keep the OCCT names without the prefix:
`FaceId`, `ShellRefId`, `EdgeCurve3DRepId`... A typed id of another kind does not
convert, which is the deleted cross-kind constructor. `++id`, `id + n`, `id - n` are
`increment`, `offset` and `retreat`. The native `Standard_ASSERT_*` checks are debug
assertions only; release builds, which the oracle runs, skip them, so the Rust port keeps
their release behaviour (a kind mismatch in `FromNodeId` returns the invalid typed id, a
retreat below zero wraps).

`BRepGraph_ItemId`, `BRepGraph_UID`, `BRepGraph_RefUID` and `BRepGraph_ItemUID` keep the
native field layout (`ItemId` stores the raw kind byte, so an item built from an invalid
node id is the invalid item), and the hashes (`std::hash` and `HashValue`) are the native
Murmur combinations over the same bytes (`opencascade::hash` of each field, then
`hashBytes` of the `size_t` array), so the oracle compares them bit for bit.
`BRepGraph_VersionStamp::ToGUID` packs the same 37-byte buffer (the `Standard_UUID`
bytes, the domain byte, the 8-byte counter, the kind, the two generations) and hashes
its four slices as natively.

### 11.2 Storage is plain vectors, owned by value

`BRepGraphInc_Storage` is `brep_graph_inc::Storage`. The native `DefStore<T>` and
`RefStore<T>` have the same fields, so both are one generic `EntityStore<T>`: a `Vec<T>`
of definitions or reference entries, five `BitFlags` planes (removed, owned, guarded,
has-compound-parent, has-occurrence-parent), the active count and the per-kind UID
counter. `RepStore<T>` has the vector, the removed plane and the active count. The
relation tables are vectors parallel to the stores, and the sparse incoming maps
`myNodeToCompounds`/`myNodeToOccurrences` are `BTreeMap`s keyed by `NodeId` (their
iteration order never reaches an output natively; a sorted map keeps it
deterministic). `NCollection_DynamicArray` and `NCollection_LinearVector` are `Vec`; the
`NCollection_IncAllocator` that backs them natively has no Rust counterpart, so
`Allocator()` is excluded (section 11.8).

The native `TypedStorePlanes<T>` specialisations become one trait implemented on each
kind tag: `StoredTag` gives the store of the tag's kind, so `is_removed`, `set_removed`,
`is_owned`, `set_guarded`... are generic methods written once over
`TypedId<T: StoredTag>`. A representation store has no owned or guard plane, so calling
`is_owned` on a representation id does not compile, as natively.

Definitions and references keep the native inheritance as composition: each `*Def`
embeds a `BaseDef` (UID, OwnGen, SubtreeGen, LastPropWave) and each reference entry a
`BaseRef` (UID), read through the `HasBaseDef`/`HasBaseRef` traits, which is what
`TopoEntity` and `BaseRef` return. The geometry and mesh handles in the representation
records are the TKG3d, TKG2d and Poly handles, shared as natively.

The storage members that are `const` natively but update a cache (the lazily rebuilt UID
reverse indexes, the shape bindings, the reconstructed-shape cache) keep their
`shared_mutex` as a `RwLock` and their dirty flags and counters as atomics, so they take
`&self` like the native `const` members and the graph stays `Sync`.

`Standard_ProgramError`, raised by `ParityOrientation` for an INTERNAL or EXTERNAL
orientation, is a `Result` error; every storage member that converts an orientation
(and the population that reaches it through an INTERNAL wire or edge) returns it.

### 11.3 The graph owns its data; views borrow it

`BRepGraph` owns one `Data` value (the native `BRepGraph_Data` PIMPL): the storage and
the extension slots of section 11.6. The native views hold a `BRepGraph*`; the Rust
views are small `Copy` structs that borrow the graph: `graph.topo()` returns
`TopoView<'_>`, whose `faces()`, `edges()`... return `topo_view::FaceOps<'_>`... with the
native members. `refs()`, `uids()` and `shapes()` work the same way. Every query takes
`&self`, so any number of views can be alive at once and a graph can be read from
several threads, as the native `const` API allows. The members that change the graph
take `&mut BRepGraph`: `graph.shapes_mut()` returns `ShapesViewMut<'_>`, which holds the
only mutable borrow (`Add`, `ClearCached`). `Shapes().Shape()` is `const` natively and
fills the reconstructed-shape cache under its mutex; it takes `&self` and the storage
`RwLock`.

The private `BRepGraph` members that later phases need (`allocateUID`, `markModified`,
`markRefModified`, `markParentSubtreeGen`, `propagateSubtreeGen`, `topoEntity`,
`changeTopoEntity`, `refEntity`, `incStorage`) are `pub(crate)` methods of `BRepGraph`,
so the phase-2 modules of this crate call them directly, as the native `friend` classes
do.

### 11.4 Mutation without `unsafe`: the editor and MutGuard

**The view.** `EditorView<'g>` holds `&'g mut BRepGraph` (`graph.editor()`). Each
operation-group accessor (`vertices()`, `edges()`, ... `generic()` for `Gen()`) consumes
the view and returns the group, which holds the same borrow, so one call reads
`graph.editor().edges().add(...)`; `reborrow()` hands out a shorter-lived view when one
view serves several groups. `BRepGraph::Editor() const` is `graph.editor_state()`, an
`EditorState<'_>` over `&BRepGraph` with the `const` members (`IsDeferredMode`,
`ValidateMutationBoundary`); the same members exist on the mutable view.

**MutGuard.** `BRepGraph_MutGuard<T>` is `MutGuard<'g, T: GuardTag>`, holding
`Option<&'g mut BRepGraph>`, the typed id and the dirty flag. `Deref` reads the record
through the storage by id (and panics on an inert guard); `try_get` is the checked
`operator->`/`operator*`, raising Standard_ProgramError on an inert guard; `internal()`
marks the guard dirty and lends the record mutably; `Drop` clears the guard plane, then
calls `markModified`/`markRefModified` when dirty. The C++ move constructor is
`MutGuard::move_from(&mut source)`, which takes the borrow and leaves the source inert
(`is_active()` false), and move assignment is `move_assign`, which releases the target
first. A guarded setter (`SetTolerance(theMut, ...)`) is the associated function
`EdgeOps::set_tolerance_guarded(&mut guard, ...)`, since the group and the guard cannot
both hold the borrow. Because the guard holds the only mutable borrow, a second guard is
taken through `guard.graph_mut()`; the guard plane is still set and cleared, so
`IsGuarded` and `requireNoActiveGuard` answer as natively.

**Deferred invalidation and scopes.** `BeginDeferredInvalidation` and
`EndDeferredInvalidation` are members of the view; `BRepGraph_DeferredScope` is
`DeferredScope<'g>`, which holds the graph, derefs to it, and ends the deferred mode and
commits in `Drop` when it opened the mode. `BRepGraph_SupplementEditor` is
`SupplementEditor<'g>` (`graph.editor().supplement()`); its `AttachTo*` defaults are an
`Option<AttachmentKind>`.

**Failures and assertions.** A C++ exception is a `Failure` with the native type and
message. The oracle is the release build: `Standard_ASSERT_RETURN` evaluates its
condition and returns (the port keeps that return), `Standard_ASSERT_VOID` is compiled
out (so `CommitMutation` has no effect) and the `*_Raise_if` checks stay active. The
template `ApplyModification` takes a modifier `FnOnce(&mut BRepGraph, NodeId) ->
Result<R, Failure>` with `R: IntoIterator<Item = NodeId>`: a failing modifier returns
its error before any history record, as the native exception does.

**Mesh promotion.** `MeshView::EditorView::PromoteToPersistent` writes through the
editor's `SetPersistent*` setters, so it is the associated function
`mesh_view::editor_view::EditorView::promote_to_persistent(&mut BRepGraph)`: the mesh
editor view itself only borrows the graph shared.

**Test translation.** A native test that keeps two guards alive takes the second
through the first guard's `graph_mut()`; one that reads the graph while a guard is alive
reads through `guard.graph()`; a move assignment of a guard that is still borrowed
becomes a drop, then `move_from`; a throwing modifier returns a `Failure`.

### 11.5 Population and reconstruction

`BRepGraphInc_Populate` and `BRepGraphInc_Reconstruct` are the modules
`brep_graph_inc::populate` and `brep_graph_inc::reconstruct` with free functions
(`populate::perform`, `populate::append`, `populate::append_flattened`,
`reconstruct::node`, `reconstruct::node_with_cache`, `reconstruct::face_with_cache`).
Population traverses the shape exactly as natively, then extracts the face data
(parallel through `OSD_Parallel` when `BRepGraph_ParallelPolicy` allows it; every face is
independent, so the storage content does not depend on the thread count), then
registers the faces in order and synthesises the natural-restriction boundaries
(`BoundaryBuilder`). The per-build `LocatedNodeIndex` is keyed by TShape address, as
natively, and only searched. Reconstruction reads the derived edge, wire and shell state
through `brep_graph::cache_derived_state`, whose static `Compute*` members and per-coedge
SameRange/SameParameter computations are ported in phase 1; the cache service that
memoises them is a phase-2 addition that must return the same values.

### 11.6 Where layers and caches attach

`BRepGraph_LayerRegistry` and `BRepGraph_CacheRegistry` are phase 2. The core calls them
in a few places only: `markModified`/`markRefModified` dispatch modification events,
`Clear` clears both registries, population attaches supplement shapes to the
`BRepGraph_LayerTopoSupplement` layer and reconstruction replays them. These calls go
through two object-safe traits in `brep_graph::hooks`, stored in `Data` as
`Option<Box<dyn LayerHooks>>` and `Option<Box<dyn CacheHooks>>`:

- `LayerHooks`: `has_modification_subscribers`, `dispatch_node_modified`,
  `has_ref_modification_subscribers`, `dispatch_ref_modified`, `clear_all` and
  `topo_supplement`, which returns the registered `TopoSupplementHooks` (the
  `AddAttachment` and `AttachedTo`/`FindByUid` members that the pipelines call, with the
  native `AttachmentKind`), and `ensure_topo_supplement`, the `Ensure` that the
  supplement route of `Shapes().Add(shape, parent)` calls. Without hooks that route
  attaches nothing and its result stays `Failed`.
- `CacheHooks`: `clear_all`.

A graph without hooks is a graph with no registered layer or cache, which is what the
pipelines observe natively (the layer history that the native constructor registers
records nothing during `Add`). The dispatching members take the hook out of its slot
while they call it with `&BRepGraph`, so a hook may read the whole graph. The phase-2
registries implement these traits and install themselves in `BRepGraph::new`; nothing
else in the core changes.

### 11.7 Module ownership

Phase 1: `brep_graph_inc::*`; `brep_graph::{ids, version_stamp, usage_path, graph, data,
hooks, marks, topo_view, refs_view, uids_view, shapes_view, parallel_policy}`; and the
seed modules `brep_graph::tool` (the `Edge::Curve`, `Edge::Range`, `CoEdge::PCurve`,
`CoEdge::Range` and `Face::Surface` reads that the derived state needs),
`brep_graph::cache_derived_state` (the pure computations) and `brep_graph::editor` (the
operations of section 11.4).

Phase 2 owns and adds, without editing the phase-1 modules beyond one `mod` line and the
constructor of section 11.6:

- iterators and explorers: `brep_graph::{iterator, defs_iterator, refs_iterator,
  reverse_iterator, child_explorer, parent_explorer, related_iterator}` and the
  `TopoView` members that return iterators (`Edges().WiresOf/FacesOf`);
- editor and mutation: the rest of `brep_graph::editor`,
  `brep_graph::{mut_guard, deferred_scope, supplement_editor}`, the private
  `invalidateSubgraphImpl` and `Shapes().RemoveShape`;
- algorithms: `brep_graph::{copy, copy_remap, compact, deduplicate, validate,
  transform}` and the rest of `brep_graph::tool`;
- layers, caches and mesh view: `brep_graph::{layer, layer_registry, layer_history,
  layer_lock, layer_deferred, layer_parametric, layer_topo_supplement, layer_iterator,
  cache, cache_registry, cache_iterator, cache_mesh, mesh_view, supplement_iterator}`,
  the cache service part of `cache_derived_state` and `Shapes().AddWithHistory`.

### 11.8 Excluded members

Destructors, the `Allocator()` accessors (Rust vectors own their memory) and the
`std::hash` call operators that are the same function as a ported `HashValue` or
`HashCode` member get `excluded` rows with the reason.

### 11.9 Iterators, explorers, Tool and CopyRemap

The iterators and explorers borrow the graph (`&'g BRepGraph`) and keep the native
`More`/`Next`/`Current` protocol with the native skip and resume rules; each also
implements `Iterator` for the native range form. The templates over definition,
reference and trait types become generics over the kind tags of section 11.1:
`BRepGraph_Iterator<FaceDef, Full>` is `iterator::DefIterator<FaceTag, FULL>`, whose
`NodeTraits` is one blanket trait over `NodeTag` (the reverse iterators' `DefTraits` is
the same trait), and `RefIterator<FaceRef>` is `refs_iterator::RefIterator<FaceRefTag>`.
The defs and refs namespaces declare the same parent traits twice natively; the port
declares them once (`parent_traits`, a `ParentTraits` trait with one struct per
relation) and shares the skip loop in a private cursor (`parent_cursor`), so
`DefsOfParent` and `RefsOfParent` differ only in what they return. The kind containment
table the two explorers repeat natively is written once (`explorer_kinds`).

The explorers keep the native stack layout (frames above the top stay allocated, the
top index is an `i32`, a push past the node count is refused) so the path queries and
the cycle guard answer as natively. Composing locations can raise natively, and the
related iterator reaches `Tool::Face::OuterWire`, whose p-curve adaptors can raise, so
their constructors and `next` return `Result`. The constructor overloads are named
constructors taking `impl Into<NodeId>` where the native code has `ProductId` and
typed-id overloads.

`BRepGraph_Tool` is the module `tool` with one submodule per nested class (`vertex`,
`edge`, `co_edge`, `face`, `wire`, `shell`); the phase-1 reads keep their paths. The
derived flags go through `tool::derived_state`, the single place the caches area routes
through the memoising `BRepGraph_CacheDerivedState` service.

`BRepGraph_CopyRemap` owns the one mutable borrow of its target graph (the native
context hands out the mutable target from a `const` reference): `new` and
`with_mapping` take distinct graphs, `in_place` takes the one graph a compaction
migrates into, and `target_graph` takes `&mut self`, so the layer and cache copy
callbacks of the later waves receive `&mut CopyRemap`. Its item map is the TKernel
`FlatDataMap<ItemId, ItemId>` hashed with `std::hash<BRepGraph_ItemId>`.
### 11.10 Layers and caches (phase 2, area B)

The layers and caches are transient classes, so they follow docs/geom-design.md
sections 2 and 3: `BRepGraph_Layer` is the trait `BRepGraphLayer` and
`BRepGraph_Cache` the trait `BRepGraphCache` (both `Transient` interfaces), a
registry stores `Handle<dyn BRepGraphLayer>` values, and every concrete layer
keeps its maps behind one `SharedState`, so each member takes `&self` as the
native handle calls do. The protected state of the bases (`myGraph`,
`myRevision`) is the embedded `LayerBase` or `CacheBase`, read through a required
accessor.

**Registries.** `BRepGraph_LayerRegistry` and `BRepGraph_CacheRegistry` keep their
slot vector and GUID map behind a lock and the subscription masks in atomics, as
natively, so every member takes `&self`: `graph.layer_registry()` and
`graph.cache_registry()` are the `const` and non-`const` accessors at once, and a
layer can `Ensure` another layer from `&BRepGraph`. Since the registries lock
themselves, `Data` holds them by value in the two slots of section 11.6, which
replaces the phase-1 `LayerHooks`/`CacheHooks` trait objects (only
`TopoSupplementHooks` stays, for the BRepGraphInc pipelines); `BRepGraph::new`
creates them for the graph and ensures `BRepGraph_LayerHistory`, as the native
constructor does. The event dispatch of `markModified` reads the registry in place,
so a layer callback sees it.

**The attached graph.** A native layer or cache keeps a `BRepGraph*` that its
registry sets; a Rust layer cannot point back into the graph that owns it. Each
graph draws a unique non-zero *attachment token* from a process counter and gives it
to its two registries; a standalone registry (`LayerRegistry::new()`) has token 0,
the null graph pointer, so its layers stay detached. Attaching stores the token in
the layer (`attachGraph`, `OnAttached`); detaching, unregistering and dropping the
graph clear it (`detachContext`, `OnDetached`). The members that read
`AttachedGraph()` or `Graph()` natively take the graph explicitly
(`history.record(&graph, ...)`) and use it only when its token is the stored one;
any other graph is ignored, which is the native detached behaviour, and `Graph()`
raises Standard_ProgramError. The callbacks (`OnNodeRemoved`, `Clear`...) receive
the attached graph, or `None`, from the registry that dispatches them.

**Iteration order.** The maps whose order reaches a result (a history event's
mappings, the deleted sets, the deferred entries) are the TKernel `DataMap` and
`FlatMap` ports with the native hashes of the ids (`std::hash<BRepGraph_NodeId>`
and friends, the `HashKey` impls of `brep_graph::id_hash_keys`), sized as natively
(`ReSize` calls included), so they iterate in the native bucket order. Lookups
whose order never shows (GUID to slot) are `HashMap`s. Getters that return a
`const&` into a locked map (`FindModified`, `Record(i)`, `DeletedNodes`,
`AttachedTo`) return copies.

**Dependencies on later waves.** `CopyTo`, `CopyFreshTo`, `CopyLayersTo`,
`CopyFreshCachesTo` and `RemappedItem` take `BRepGraph_CopyRemap` (area A), and
`BRepGraph_LayerDeferred` and `BRepGraph_LayerParametric` lock items through
`BRepGraph_LayerLock`; wave 2b adds them (section 11.11).

### 11.11 Lock, derived-state and mesh caches (phase 2, wave 2b)

**The owned plane.** `BRepGraph_LayerLock` sets and clears the storage's owned bits from
`const` members (`setItemOwned` is `const` natively and writes through the attached
`BRepGraph*`) and from noexcept callbacks, which receive `&BRepGraph` in Rust. The owned
plane of each definition and reference store is therefore a `SharedBitFlags`: the
`BRepGraphInc_BitFlags` layout over atomic blocks, whose `set`, `clear` and `test` take
`&self`, so `Storage::set_owned` takes `&self`. The other planes stay plain.

**Layer copies.** `BRepGraph_Layer::CopyTo` is a required trait member
`copy_to(&self, &CopyRemap) -> Result`; the targets are written through
`CopyRemap::target_graph_const()`, since registries, layers and the owned plane lock
themselves. A registry cannot reach its graph, so `CopyLayersTo` and
`CopyFreshCachesTo` are associated functions taking the source graph and the target,
with `*_in_place` variants for the compaction, which copies a graph's layers onto the
graph itself (the old layers are detached and unregistered first, as natively).
`RemappedItem` is the pair of free functions `layer::remapped_node` and
`layer::remapped_ref`, which the caches reuse for their own `remappedNode`.

**Caches.** `BRepGraph_CacheDerivedState` keeps its four entry arrays behind one lock;
the native packed atomic bytes are plain bytes under it, with the same flag layout and
the same coedge merge. `tool::derived_state` obtains the service with
`CacheRegistry().Ensure` and routes every derived Tool read through it, and
reconstruction ensures it before its per-coedge loop, so both register it as OCCT does.
`BRepGraph_CacheMesh` keeps its slots behind one lock; its getters return entry copies
and its `Change*` members lend the entry to a closure. The native pattern "change the
entry, then `BindFresh` it" is `write_*_fresh` inside the crate (one lock) and, for
other callers, a bound copy stored back. The driver is the Transient interface trait
`CacheMeshDriver`, whose `fill` receives `&mut BRepGraph`: `Ensure` collects the dirty
set under the cache lock and calls the driver after releasing it.

**Mesh view.** `graph.mesh()` returns `MeshView<'_>` with the `cache()`,
`persistent()`, `effective()`, `editor()` and `poly()` sub-views as Copy structs; the
editor writes only the cache, so it borrows the graph shared. `PromoteToPersistent`
writes the persistent representations through the graph editor's `SetPersistent*`
setters and is added with them.

### 11.12 Copy, Compact and Deduplicate (phase 2, wave 2c, area C1)

**Static classes are modules.** `BRepGraph_Copy`, `BRepGraph_Compact` and
`BRepGraph_Deduplicate` have static members only, so each is a module of free functions:
`copy::{perform, copy_node}`, `compact::{perform, perform_with}` and
`deduplicate::{perform, perform_with}`. The nested types keep their names inside the
module (`copy::GeomPolicy`, `copy::MeshPolicy`, `copy::CachePolicy`, `compact::Options`,
`compact::CachePolicy`, `deduplicate::Options`); the `Result` structs are
`compact::CompactResult` and `deduplicate::DeduplicateResult`, since `Result` is the Rust
error type. The default arguments are the `Default` of each policy and option set.

**Self copies.** A native call may pass one graph as both source and target. That
cannot be written with a `&BRepGraph` and a `&mut BRepGraph`, so the self copies have
their own entry points: `copy::perform_self` (the identity no-op) and
`copy::copy_node_in_place` (the sub-graph duplicated with new ids). The copy context
holds `CopyGraphs::Distinct { source, target }` or `CopyGraphs::Same(graph)`, and every
`ensure*` reads what it needs from the source before it writes the result. That order is
the native one in effect: the native references into the source stores stay valid across
the result's appends (`NCollection_DynamicArray` never relocates), and a copy never
writes a source record, so an early read sees what the native late read sees.

**Map order.** The native copy keeps one `NCollection_DataMap` per node kind, only
searched and bound, so one Rust map keyed by the untyped node id holds them. The maps
whose iteration order reaches a result are the TKernel map ports with the native id
hashes and sizes: the per-kind old-to-new maps of the compaction and its node remap
(the order of the `Compact:Remap` history batch), and the rewrite and canonical maps of
the deduplication (the order of its history records and merges). The deduplication
sorts with libstdc++'s `std::sort` (`stl::sort_by`), so vertices of equal tolerance
keep the native tie order. Its edge and face groups hash the curve and surface handle
addresses, as `std::hash` of a handle does: the group order depends on the process
natively too, and reaches the result only through collisions of the candidate ids in
the canonical map.

**Compaction in place.** Natively the compaction moves the source graph's registries
out, moves the rebuilt graph into the source object, then moves the saved registries
back and re-attaches them. A Rust graph keeps its registries and swaps its storage with
the rebuilt graph's, replaying the registry callbacks of the native moves: under
`CopyFresh` the fresh caches copied into the rebuilt graph are adopted
(`CacheRegistry::adopt`, `OnAttached` through `rebindGraph`); the rebuilt graph's own
layers and caches are detached when it is dropped; the saved layers are re-attached
(`LayerRegistry::reattach`, `OnDetached` then `OnAttached`), the kept caches are cleared
under `Drop`, and the layers migrate through `copy_layers_in_place` in compact mode.
The validation step calls `validate::perform` (area C2).

**Failures.** A C++ exception is a `Failure`. The native algorithms restore the
history's enabled state with a plain statement on their return paths, so after a raise
the history keeps the `HistoryMode` the run set, in Rust too.

### 11.13 Validate and Transform (phase 2, wave 2c, area C2)

**Validate.** `BRepGraph_Validate` is the read-only module `brep_graph::validate`
(`perform`, `perform_with_mode`, `perform_with_options`, with `Severity`, `Mode`,
`Issue`, `Options` and `ValidationResult` for the native `Result`). Each check is one
function over `&BRepGraph` that appends to the issue vector in its native traversal
order, grouped by family (bounds, relations, representations, owned uses, isolation
and wire connectivity, counts and roots, UIDs and the supplement layer, cycles,
ownership), and `perform_with_options` runs them in the native sequence, so a report
is the native report issue for issue. The seven per-kind owner tallies of
`CheckOwnedUseReferences` are one generic tally over the representation tag, the
assembly and compound cycle searches one generic breadth-first search, and the seven
reference-ownership loops one generic count over the reference tag. The per-kind
active counts reuse the editor boundary's `cachedActiveByKind` and
`countActiveByKind`.

**Transform.** `BRepGraph_Transform` is the module `brep_graph::transform`. The two
`MoveRef` overloads are `move_child_ref` and `move_occurrence_ref`. The in-graph steps
are crate functions over `&mut BRepGraph`: `transform_whole_graph` is the
self-transform branch of `Perform`, and `transform_copied_range` the steps after a copy
into the target, with the native `GraphCounts` as `CopiedRange` (the default range is
the whole graph, so the whole-graph and copied-range variants of each native loop are
one function). The loops collect the live ids before they write, since the editor
borrows the graph mutably; no step adds or removes a node, so the native lazy
iteration visits the same ids. `Perform(source, target, ...)` and `TransformNode` are
`perform` and `transform_node`: the native policy checks, then `copy::perform` or
`copy::copy_node` (section 11.12, default cache policy), then `transform_copied_range`.
A self transform cannot pass one graph as `&BRepGraph` and `&mut BRepGraph` at once, so
the self overloads are `perform_in_place` (no copy; `transform_whole_graph`) and
`transform_node_in_place` (`copy::copy_node_in_place`, then the copied range), as
section 11.12 does for the self copies.

### 11.14 Ids out of range

**What OCCT does.** The record accessors of `BRepGraphInc_Storage` (`Vertex()`,
`ChangeEdge()`, `ShellRef()`, `EdgeCurve3DRep()`, `FaceRelations()`...) and every
view member built on them (`Topo().Faces().Definition()`, `Relations()`,
`Refs().Faces().Entry()`) read `NCollection_DynamicArray::Value`/`ChangeValue`, which
are `noexcept` and do not check the index. The `BRepGraph_Tool` members that take a
definition id (`Vertex::Pnt`, `Edge::Range`, `CoEdge::Orientation`, `Face::NbWires`,
`Wire::FaceOf`...), the immediate editor setters (`SetTolerance`, `SetRefOrientation`,
`ReplaceChild`...), `RemoveRef` (through `detachRefRelations`) and the product and
occurrence steps of `BRepGraph_Copy::CopyNode` (`ensureProduct`, `ensureOccurrence`)
index the same arrays with the caller's id. In the pinned build, an id past the store
is undefined behaviour: no `Standard_OutOfRange`, no assertion. The members that check
natively (`IsValid(Nb())` guards, `Standard_ProgramError_Raise_if` in `Mut()`) keep their
native answer here.

**Policy.** Library code does not panic on caller input, and the fast accessors keep
the native cost. So each family has two forms:

- The fast form keeps the native signature and indexes directly; an id past the store
  panics with the index (the Rust form of the native undefined behaviour).
- The checked form returns `Result` with Standard_OutOfRange whose message names the
  id kind, the index and the valid range (`BRepGraph: BRepGraph_FaceId index 6 is out
  of range; expected 0..6`): `Storage::try_def`, `try_change_def`, `try_entry`,
  `try_change_entry`, `try_record`, `try_change_record` and the `try_*_relations`;
  `try_definition` and `try_relations` on each `TopoView` kind group; `try_entry` on
  each `RefsView` kind group; `check_in_range` on the node and reference `GenOps` for
  untyped ids; and `tool::checked_member(graph, id, member)`, one wrapper that gives
  every definition-id Tool member its checked form instead of a second copy of each.

Every checked form goes through one helper, `brep_graph_inc::check_index` (with the
typed and untyped name wrappers `check_typed_index` and `check_untyped_index`), which
formats the message only on failure. The library members that take a caller id and
write or copy through it use the checked form: the setters through the editor helper
`require_settable` (the native `requireUnlocked`, then the range check), `ReplaceChild`,
`ReplaceSolid` and `RemoveRef` through `try_entry` and `check_in_range`, and the copy
through `check_id`/`try_def` before it appends anything, so a refused `CopyNode` leaves
the target unchanged and `TransformNode` of such an id returns the same error. Ids read
from the graph's own records are not re-checked: the relation invariants
(`Validate`) guarantee them. The regression tests are
`crates/tkbrep/tests/brep_graph_out_of_range.rs`.
