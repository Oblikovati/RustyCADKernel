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

## 2. BRepClass: the TopClass instances

BRepClass instantiates the two TopClass templates of TKGeomAlgo
([geomalgo-design.md](geomalgo-design.md) section 8) on its own edge, intersector and
face explorer; the generics `topclass::Classifier2d` and `topclass::FaceClassifier` are
reused, never copied:

- `BRepClass_Edge` is the struct `BRepClassEdge` (with its `BndBoxState` enum): the
  edge, its face, the next edge at its last vertex and the cached pcurve, range and
  bounding box. `Clone` is the native copy (the pcurve handle is shared).
- `BRepClass_Intersector` derives from `Geom2dInt_IntConicCurveOfGInter`; the struct
  `BRepClassIntersector` holds that generic instance and reaches it (and its
  `IntRes2d_Intersection`) through `Deref`. It implements `Classifier2dIntersector`
  with `BRepClassEdge` as the edge type. Its Perform writes the protected `done` flag
  and `Append`s to the results left by the previous Perform, as natively.
- `BRepClass_FaceExplorer` is the struct `BRepClassFaceExplorer`; it implements the
  `topclass::FaceExplorer` trait. The trait's `CheckPoint` cannot fail, so a failure of
  the face bounds (BRepTools::UVBounds, the direction to a far point) is kept and
  returned by the next `Segment`/`OtherSegment`, before any other work.
- The instance classes whose headers only repeat the template are aliases:
  `BRepClass_FClass2dOfFClassifier` and `BRepClass_FacePassiveClassifier` are
  `Classifier2d<BRepClassIntersector>` (the two headers are identical natively), and
  `BRepClass_FClassifier` is `FaceClassifier<BRepClassFaceExplorer>`.
- `BRepClass_FaceClassifier` derives from `BRepClass_FClassifier` and writes the
  protected `rejected` flag when its 3d Perform finds no projection of the point. The
  generic keeps that flag private, so `BRepClassFaceClassifier` holds the
  `FClassifier` and its own `rejected_by_projection` flag, which every inherited
  member reads first (`State` is OUT, `Rejected` is true, `Edge` and `EdgeParameter`
  raise); the next 2d Perform clears it, as the native Perform overwrites `rejected`.
  The base class's `Perform(F, P, Tol)` is hidden natively by the overloads the derived
  class declares, and is not a member of the Rust struct either.
- C++ output references become returned tuples, and every member that can raise returns
  `Result`, as geomalgo-design.md section 8 fixes for the templates.
