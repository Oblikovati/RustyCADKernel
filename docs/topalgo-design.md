# TKTopAlgo design notes

This note fixes how the TKTopAlgo port (`crates/tktopalgo`, see
[tktopalgo-progress.md](tktopalgo-progress.md)) maps C++ constructs that the earlier
design notes ([geom-design.md](geom-design.md), [topology-design.md](topology-design.md),
[geomalgo-design.md](geomalgo-design.md)) do not cover. Each package adds its section
before it codes the construct, and cites it by its number in code comments and ledger
notes.

## 1. Generic templates (`.gxx`, `.pxx` and `.lxx`)

TKTopAlgo instantiates many templates that lower toolkits own (the TopClass
classifiers of TKGeomAlgo in BRepClass, the ApproxInt, AppParCurves, IntImp and
IntWalk generics in BRepApprox) and owns one generic itself (`MAT_TList.gxx`). The
rule for the whole toolkit, the same as geomalgo-design.md section 1:

- A template is ported **once**, in the module of the package that owns it, as a Rust
  generic over a trait that holds exactly the operations the template calls on its
  type parameters, under their native names. A template owned by a lower toolkit is
  never copied into this crate: TKTopAlgo reuses the generic of that toolkit's port.
- Each native instantiation is a type alias (or a thin wrapper when the instance class
  adds members or is a derived class), declared in the TKTopAlgo package that
  instantiates it natively, with the concrete types implementing the trait there.
- Floating-point operation order and control flow follow the template body; a
  monomorphized Rust generic executes the same operations as each C++ instance.
- An `.lxx` file that only holds the inline members of one class is not a template:
  its members are ported with the class.
- The api rows of an instance class map the members of its header to the generic's
  methods (or the wrapper's) and name this section.

The sections below record each instantiation as it is ported.
