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
