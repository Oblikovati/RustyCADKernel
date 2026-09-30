# TKPrim design notes

This note fixes how the TKPrim port (`crates/tkprim`, see
[tkprim-progress.md](tkprim-progress.md)) maps C++ constructs that the earlier design
notes ([geom-design.md](geom-design.md), [topology-design.md](topology-design.md),
[geomalgo-design.md](geomalgo-design.md), [topalgo-design.md](topalgo-design.md)) do not
cover. Each package adds its section before it codes the construct, and cites it by its
number in code comments and ledger notes.

## 1. The toolkit rules

TKPrim has no generic templates (no `.gxx` or `.pxx` file; its `.lxx` files only hold
inline members, ported with their class). Its C++ is class hierarchies with virtual
members, and the rules for the whole toolkit are:

- **An abstract class with virtual members becomes a trait over a base struct**, as
  `BRepBuilderAPI_MakeShape` does in topalgo-design.md section 10: the struct holds the
  protected data of the abstract class, the trait has required accessors of that struct,
  the pure virtual members as required methods and the virtual members with a native body
  as provided methods. A non-virtual member of the abstract class that calls virtual
  members is a provided method that no implementor overrides.
- **A concrete class derived from a concrete class** holds its base and dereferences to
  it, as the BRepLib makers do (topalgo-design.md section 7), unless the base has virtual
  members, in which case the derived class implements the trait of the base.
- **Every maker of BRepPrimAPI and BRepPreviewAPI implements
  `BRepBuilderAPIMakeShape`** of the TKTopAlgo port (topalgo-design.md section 10) and
  overrides the members its header overrides (`Build`, `Generated`, `IsDeleted`).
- **Constructors and overloads take distinct Rust names** (`new`, `with_axes`,
  `from_points`, ...), and each returns `Result` when its native body can raise.
- **Exceptions are `Failure` errors** of the native exception type with a message that
  holds the offending value. Checks written with `*_Raise_if` stay active: the pinned
  oracle build (RelWithDebInfo) does not define `No_Exception`, and no TKPrim source file
  defines it either.
- **Output references become returned values**; a C++ member that fills a shape passed
  by non-const reference (`BRepSweep_Builder::MakeWire(TopoDS_Shape&)`) keeps a `&mut`
  parameter, because the caller's slot (an element of the sweep's shape grid) is what the
  native call writes.
- **A value the native code leaves uninitialized** gets the value of the default
  constructor of its type (or 0 for a scalar), and docs/tkprim-progress.md lists it; the
  probes never read it.

## 2. BRepPrim: the builder, the wedge and the one-axis primitives

- `BRepPrim_Builder` is the struct `BRepPrimBuilder` over a `BRepBuilder`. Its members
  keep the native names (`make_shell`, `make_face`, `add_edge_vertex`, ...) with distinct
  names for the overloads (`make_edge_line`, `make_edge_circle`, `set_pcurve_line`,
  `set_pcurve_lines`, `set_pcurve_circle`, `add_edge_vertex_closed`). A member that
  rebuilds a shape or changes its orientation takes it by `&mut`; a member that only
  updates the shared TShape (a p-curve, a range, the closed flag) takes it by `&`.
- `BRepPrim_Direction` is the enum `BRepPrimDirection` with the native discriminants.
- `BRepPrim_GWedge` is the struct `BRepPrimGWedge` with the native arrays of built
  sub-shapes. `BRepPrim_Wedge` only adds constructors without a builder: it is the
  newtype `BRepPrimWedge`, which dereferences to the wedge. The accessors that build on
  demand (`Shell`, `Face`, `Wire`, `Edge`, `Vertex`, `Plane`, `Line`, `Point`) take
  `&mut self`, return the built value (a shape is a handle, so the copy shares the TShape
  of the stored one) and return `Result`: they raise Standard_DomainError for an open
  direction, a degenerated wedge or two directions of the same axis.
- `BRepPrim_OneAxis` is the trait `BRepPrimOneAxis` over the base struct
  `BRepPrimOneAxisBase` (the builder, the axes, the angle, the meridian range and offset
  and the arrays of built sub-shapes). The four pure virtual members
  (`MakeEmptyLateralFace`, `MakeEmptyMeridianEdge`, `SetMeridianPCurve`, `MeridianValue`)
  are required methods; the virtual members with a native body (`MeridianOnAxis`,
  `MeridianClosed`, `VMaxInfinite`, `VMinInfinite`, `HasTop`, `HasBottom`, `HasSides`)
  are provided methods with that body. The non-virtual members (the accessors of the
  axes, angle and range, and every topology builder from `Shell` to `BottomEndVertex`)
  are provided methods too; no implementor overrides them. Their bodies live in the
  modules `one_axis_faces`, `one_axis_wires`, `one_axis_edges` and
  `one_axis_vertices` as functions generic over the trait, each provided method calling
  one of them. The private `isHeightInverted` and `areHeightsEqual` are crate-visible
  functions of the same modules.
- `BRepPrim_Revolution` is the struct `BRepPrimRevolution`: the base and the meridian
  curves. It implements the trait with the four pure virtual members. Its protected
  constructor without a meridian and its protected `Meridian` setter are public
  (`without_meridian`, `set_meridian`), because the derived classes live in other modules.
- `BRepPrim_Cylinder`, `BRepPrim_Cone`, `BRepPrim_Sphere` and `BRepPrim_Torus` hold a
  `BRepPrimRevolution` and their own radii, and only override `MakeEmptyLateralFace`.
  They implement the trait `BRepPrimRevolutionDerived` (the revolution they hold and
  their lateral face); a blanket implementation of `BRepPrimOneAxis` forwards the other
  members to the revolution, so the derived class keeps every member of its bases.
