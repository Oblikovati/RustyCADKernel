# TKShHealing design notes

This note fixes how the TKShHealing port (`crates/tkshhealing`, see
[tkshhealing-progress.md](tkshhealing-progress.md)) maps C++ constructs that the earlier
design notes ([geom-design.md](geom-design.md), [topology-design.md](topology-design.md),
[geomalgo-design.md](geomalgo-design.md), [topalgo-design.md](topalgo-design.md)) do not
cover. Each package adds its section before it codes the construct, and cites it by its
number in code comments and ledger notes.

## 1. Toolkit-wide rules

- **Modules and names.** Each package is one module in snake case (`ShapeExtend` is
  `shape_extend`, `SHMessage` is `sh_message`). A class keeps its package prefix in its
  Rust name without the underscore (`ShapeExtend_WireData` is `ShapeExtendWireData`),
  because the Shape Healing packages reuse the same short names (`ShapeAnalysis_Edge`,
  `ShapeBuild_Edge`, `ShapeFix_Edge`). Package functions (`ShapeExtend::EncodeStatus`)
  are associated functions of a unit struct named after the package.
- **Enumerations** keep the native order, so `value as i32` is the native value, with a
  `from_index` for the probes (`ShapeExtendStatus::from_index`).
- **Status fields.** Many tools keep an `int` bit field of `ShapeExtend_Status` flags and
  answer `Status(ShapeExtend_Status)` with `ShapeExtend::DecodeStatus`; the Rust field is
  the same `i32`, set with `ShapeExtend::encode_status`, so that the bits are the native
  bits.
- **Transient classes and handles** follow geom-design.md sections 2 and 3: a class
  derived from `Standard_Transient` is used through `Handle`, keeps its mutable fields in
  one shared state, and registers its RTTI name. A class that derives from a Transient
  class of a lower toolkit (for example `ShapeBuild_ReShape` from `BRepTools_ReShape`)
  gets its own section before it is coded.
- **Exceptions** become `Result<_, Failure>` errors that carry the native exception type
  and the offending value (geom-design.md section 4).
- **Generic C++ code.** The toolkit has no `.gxx` template, and its only `.pxx` files are
  the compiled message resources of SHMessage (data, not code); an `.lxx` file
  holds the inline members of one class and is ported with that class.

## 2. ShapeExtend: the transient classes

- **ShapeExtend_WireData** is the struct `ShapeExtendWireData`, a Transient shared through
  handles; its fields sit in one `SharedState` and every member takes `&self`. The
  members that take another wire data (`Init(other)`, `Add(other, atnum)`) read it between
  their own locks, so the other object can be the same one: `Init` clears first and then
  copies nothing, and `Add` re-reads the counts of `other` at each step as natively (a
  list added to itself does not end, natively neither). `myEdges` is a plain sequence
  (never shared); `myNonmanifoldEdges` stays a handle, since `NonmanifoldEdges()` returns
  it and `Clear` replaces it. `ManifoldMode()` returns a reference natively: it is the
  pair `manifold_mode`/`set_manifold_mode`. The overloads are named by argument
  (`add_edge`, `add_wire`, `add_wire_data`, `add_shape`, `add_oriented_edge`...,
  `init`/`init_wire`, `reverse`/`reverse_on_face`).
- **ShapeExtend_BasicMsgRegistrator** has three virtual `Send` members with empty bodies:
  it is the trait `BasicMsgRegistrator` (provided methods `send_transient`, `send_shape`,
  `send`) and the struct `ShapeExtendBasicMsgRegistrator`. `ShapeExtend_MsgRegistrator`
  embeds it as its parent, overrides the two object `Send`s, and lends its maps through
  `map_transient`/`map_shape` (natively references to its fields). The transient map key
  is `TransientKey`, a `Handle<StandardTransient>` hashed by address with pointer
  equality, as `NCollection_DefaultHasher` of a handle.
- **ShapeExtend_ComplexCurve** is abstract and derives from `Geom_Curve`: it is the trait
  `ShapeExtendComplexCurve: GeomCurve`. Its pure virtual members are required methods, its
  protected `myClosed` is reached through two required accessors (geom-design.md section
  1), and the `Geom_Curve` members it overrides are provided `complex_*` methods that the
  macro `shape_extend_complex_curve_members!` puts in the `impl GeomCurve` of a concrete
  class (its `transform` calls `complex_transform`). No OCCT class derives from it; the
  probes define one segment-per-range test class on both sides.
- **ShapeExtend_CompositeSurface** is the struct `ShapeExtendCompositeSurface`, a
  `Geom_Surface` like `GeomPlate_Surface` in TKGeomAlgo: the grid and the joint values are
  shared handles (`Patches()`, `UJointValues()` return them, `SetUFirstValue` edits the
  joints in place) and the fields sit in one `SharedState` that is never held while a
  patch is called. Members that dereference the null grid of a default-constructed surface
  crash natively; they return Standard_NullObject here, and the infallible `Bounds`
  returns zeros. `myUClosed`/`myVClosed` are uninitialized natively until
  `CheckConnectivity`; they start false here.
- **ShapeExtend_Explorer** is a value class (`ShapeExtendExplorer`); the eight output
  handles of `DispatchList` are the struct `ShapeDispatch`.
- **ShapeExtend::Init** keeps the native function-local flag as a process-wide atomic set
  before the first load, and loads into the global catalog (`MsgFile::global`); the
  public `load_messages(catalog)` is its body on any catalog, for tests.
