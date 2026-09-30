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
  bits, and the member is `status_flag(ShapeExtendStatus)` (a class may have another
  `Status` overload, as `ShapeBuild_ReShape`).
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

## 3. ShapeBuild: the value builders and the derived re-shaper

- **ShapeBuild_Vertex** and **ShapeBuild_Edge** have only `const` members and no field:
  they are the unit structs `ShapeBuildVertex` and `ShapeBuildEdge`. Overloads are named by
  argument (`combine_vertices`/`combine_points`, `remove_pcurve_on_face`/`_on_surface`/
  `_on_located_surface`, `make_edge_on_curve`/`_on_curve_range`/`_on_face`/
  `_on_face_range`/`_on_surface`/`_on_surface_range`). `TransformPCurve` returns the curve
  with the new range instead of writing its two reference arguments. `MakeEdge` keeps its
  `TopoDS_Edge&` argument as `&mut Edge`, left unchanged when `BRepBuilderAPI_MakeEdge`
  fails, as natively. The members that walk the curve representations of a TEdge
  (`CopyRanges`, `SetRange3d`, `CopyPCurves`, `CountPCurves`) read the list through
  `BRep_TEdge::curves()` and change the shared representation objects, as the native
  loops over `ChangeCurves()` do. `BuildCurve3d` catches every failure (the native
  `try/catch`) and returns false.
- **ShapeBuild_ReShape** derives from `BRepTools_ReShape` (TKBRep's trait `ReShape` with
  the struct `BRepToolsReShape`): the struct `ShapeBuildReShape` embeds a
  `BRepToolsReShape` as its parent (its RTTI chain is ShapeBuild_ReShape ->
  BRepTools_ReShape -> Standard_Transient) and implements `ReShape` over the parent's data.
  It overrides `apply` with its own `applyImpl` (the base `status` with `last` calls it
  through the trait, as the native virtual call does); `Status(shape, newsh, last)` only
  calls the base, so the trait default stays. The virtual build-mode
  `Apply(shape, until, buildmode)`, which no class overrides, is the inherent
  `apply_with_build_mode`, and `Status(ShapeExtend_Status)` is `status_flag` (the name the
  Shape Healing tools use for their status flags). The in-flight set of `applyImpl` holds
  TShape addresses.

## 7. ShapeCustom: the modifications

Sections 4 to 6 are left to ShapeAnalysis, ShapeConstruct and ShapeFix, which are ported
in parallel; the ShapeCustom, ShapeUpgrade and ShapeProcess sections take the numbers 7, 8
and 9 in package order.

- **ShapeCustom_Curve2d** has only static members: the unit struct `ShapeCustomCurve2d`.
  `IsLinear` and `ConvertToLine2d` leave their output arguments unchanged when they return
  false or a null line, so they return `Option` (`None` is the native false or null line).
  `IsLinear` reads the poles by the indices 1..Length through `Array1::value`, as natively;
  another lower bound is Standard_OutOfRange instead of an unchecked read.
  `SimplifyBSpline2d` changes the B-spline through its shared handle (`&Geom2dBSplineCurve`).
- **ShapeCustom_RestrictionParameters** is a Transient with its fields in one
  `SharedState`; each accessor that returns a reference to a field is a getter and a `set_`
  setter.
- **ShapeCustom_Modification** is abstract and derives from `BRepTools_Modification`
  (TKBRep's trait `BRepToolsModification`): it is the trait
  `ShapeCustomModification: BRepToolsModification`, registered below
  `BRepTools_Modification`. Its private `myMsgReg` is the struct
  `ShapeCustomModificationFields` that each concrete modification embeds and lends through
  the required method `modification_messages`; `SetMsgRegistrator` (virtual),
  `MsgRegistrator` and `SendMsg` are provided methods. A concrete modification
  (`ShapeCustomDirectModification`, `ShapeCustomConvertToRevolution`) lists both traits as
  interfaces of its `impl_transient!`, so a `Handle<dyn BRepToolsModification>` casts to
  `Handle<dyn ShapeCustomModification>` as the native `DownCast`.
- **ShapeCustom_TrsfModification** derives from `BRepTools_TrsfModification`, a concrete
  class: the struct `ShapeCustomTrsfModification` embeds a `TrsfModification` as its parent
  (RTTI chain ShapeCustom_TrsfModification -> BRepTools_TrsfModification ->
  BRepTools_Modification) and implements `BRepToolsModification` by calling the parent,
  then overwriting the tolerance; the inherited `Trsf()`/`IsCopyMesh()` are reached through
  `trsf_modification()`.
- The `NewCurve` bodies that ShapeCustom_DirectModification and
  ShapeCustom_ConvertToRevolution share (copy the 3D curve of an edge when one of its
  p-curves lies on a surface that changes) are one private function with the surface test
  as a closure.
- The output parameters of the `New*` members are the `&mut` arguments and the
  `NewSurface`/`NewCurve` structs of the TKBRep trait; a member leaves them as natively
  (written or not) on every path.
