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
| TopExp_Test.cxx | topexp | 9 | blocked (TKPrim) | pass |

Both suites build their inputs with BRepBuilderAPI_MakeVertex,
BRepBuilderAPI_MakeEdge (TKTopAlgo) and BRepPrimAPI_MakeBox (TKPrim). BRep and those
toolkits are not ported, so the Rust translations use the named fakes of
`crates/tkbrep/tests/support`: vertex and edge TShapes derived from the abstract
TopoDS classes (as BRep_TVertex and BRep_TEdge are), and a six-face box that shares
12 edges and 8 vertices. The suites only assert topology (types, counts, sharing,
orientations), which the fakes reproduce. They are replaced by the real builders when
those toolkits are ported.

`TopExp_Test.cxx` is translated and passes in Rust, but its unchanged file calls
BRepPrimAPI_MakeBox, and the shared native oracle closure (FoundationClasses plus
TKTopAlgo) does not build TKPrim, so the native suite cannot be compiled. Its ledger
rows have the status `native_blocked` with that reason: `verify_toolkit.py` lists them
and never counts them as passes. The TopExp behaviour is compared with the native
kernel by the numeric oracle instead (`br_explore`, `br_maps`, `br_vertices`). The
other TKBRep suites (BRep_Tool, BRepTools, BRepAdaptor, BRepGraph, and TopoDS_Edge,
TopoDS_Iterator and TopoDS_TShape, which also build BRep shapes) wait for BRep.

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

`crates/tkbrep/src/brep.rs` predates this port (the TKernel TopLoc legacy test and
the TKTopAlgo `MakeVertex` use it). It is not the BRep package: it keeps a vertex
TShape with a point and a tolerance, now derived from `topods::TVertex`.

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

## Deliberate Rust deviations

- A null TShape dereference (null shape flags, `ShapeType`, a null explored shape in
  `TopExp_Explorer::Next`, a null parent or component in `TopoDS_Builder`) raises
  Standard_NullObject instead of crashing.
- The iterator walks the child list by index. While the list does not change the
  result is the same; removing the current child during an iteration is undefined
  natively (the node is freed) and continues at the next index in Rust.
- `TopExp_Explorer::Next` returns `Result`, and `Explorer` and `ShapeIterator` also
  implement `std::iter::Iterator`.
