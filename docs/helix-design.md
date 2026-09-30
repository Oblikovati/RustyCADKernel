# TKHelix design notes

This note fixes how the TKHelix port (`crates/tkhelix`, see
[tkhelix-progress.md](tkhelix-progress.md)) maps the C++ constructs of its two packages
that the earlier design notes ([geom-design.md](geom-design.md),
[topology-design.md](topology-design.md), [topalgo-design.md](topalgo-design.md)) do not
cover. Code comments and ledger notes cite each section by its number.

## 1. The helix adaptor (`HelixGeom_HelixCurve`)

`HelixGeom_HelixCurve` derives from `Adaptor3d_Curve` and declares its RTTI
(`DEFINE_STANDARD_RTTIEXT(HelixGeom_HelixCurve, Adaptor3d_Curve)`):

- It is the struct `HelixGeomHelixCurve`, registered under the native name with the
  parent `Adaptor3d_Curve` and the interface `dyn Adaptor3dCurve`, so a
  `Handle<HelixGeomHelixCurve>` casts to the `Handle<dyn Adaptor3dCurve>` that
  `HelixGeom_Tools::ApprCurve3D` takes.
- Its protected fields live behind one `SharedState` (geom-design.md: adaptors take
  `&self`), so `Load` works on a shared handle as it does natively.
- The copy constructor (`new HelixGeom_HelixCurve(aAdaptor)`) is `Clone`: the copy owns
  a new state with the same values.
- Only the members the class overrides are implemented; the others keep the provided
  trait methods of `Adaptor3dCurve`, which raise Standard_NotImplemented as the native
  base class does (for example `D3`, `Trim` and `GetType`).

## 2. The builder hierarchy (`HelixGeom_Builder*`)

`HelixGeom_BuilderApproxCurve` is abstract (pure virtual `Perform`) with protected data;
`HelixGeom_BuilderHelixGen` derives from it, adds the curve parameters and is abstract
too (protected constructor); `HelixGeom_BuilderHelixCoil` and `HelixGeom_BuilderHelix`
are the concrete builders. As for `BRepBuilderAPI_MakeShape` (topalgo-design.md
section 10):

- each abstract class is a trait (`HelixGeomBuilderApproxCurve`,
  `HelixGeomBuilderHelixGen: HelixGeomBuilderApproxCurve`) whose provided methods are
  the native non-virtual members, and whose required methods are `Perform` and the
  accessors of the protected data;
- the protected data is a struct (`HelixGeomBuilderApproxCurveBase`, and
  `HelixGeomBuilderHelixGenBase` that embeds it); each concrete builder embeds the
  base of its direct parent and implements both traits;
- the protected constructors are the `Default`/`new` of the base structs, with the
  native default values.

## 3. Exceptions and status codes

The builders report expected failures with their integer `ErrorStatus` and
`WarningStatus`, as natively (the port keeps every code, see
[tkhelix-progress.md](tkhelix-progress.md)). A native exception (Standard_ConstructionError
of `HelixGeom_HelixCurve::Load` and of `HelixBRep_BuilderHelix::SetParameters`,
Standard_OutOfRange of an array index, Standard_NoSuchObject of an empty list,
StdFail_NotDone of a wire maker) is a `Failure` carrying the native exception type, and
every member that can raise returns `Result`. A member that raises after it changed the
builder leaves the builder as the native member leaves it at the throw.

## 4. HelixBRep arrays and overloads

`HelixBRep_BuilderHelix` keeps its parameters in `NCollection_HArray1` handles that may
be null: each is an `Option<Array1>` (None for a null handle). The arrays are copied
with `Array1::assign`, which takes the bounds of the source, as `ChangeArray1() = theX`
does natively, so an input array with a lower bound other than 1 keeps it (and the
absolute indices of `Perform` then read other elements, or raise). The six
`SetParameters` overloads take distinct Rust names (`set_parameters`,
`set_parameters_diam`, `set_parameters_spiral` and the `_turns` variants).
