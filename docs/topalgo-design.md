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

## 3. Bisector, MAT and MAT2d

These three packages compute the bisecting locus (the medial axis) of a 2D figure.

- **Bisector curves.** `Bisector_Curve` derives from `Geom2d_Curve` and is abstract, so
  it is the trait `BisectorCurve: Geom2dCurve` (docs/geom-design.md section 1),
  registered as `Bisector_Curve` below `Geom2d_Curve`. `Bisector_BisecAna`,
  `Bisector_BisecCC` and `Bisector_BisecPC` are structs that implement the whole
  ancestry, so a `Geom2d_TrimmedCurve` (and every Geom2d consumer, as the
  `Geom2dAdaptor_Curve` of `Geom2dInt_GInter`) takes them as a basis curve, and MAT2d
  casts the basis back with `cast::<dyn BisectorCurve>()`. Each keeps its fields in one
  `SharedState`; the algorithms are written on the plain state value, which never holds
  a handle to its own object.
- **Protected and private setters.** The private setters of `Bisector_BisecCC` and the
  private `Init` of `Bisector_BisecPC` only serve `Copy` and `ChangeGuide`; they are
  crate-private in Rust.
- **Raw pointers.** `MAT_Arc`, `MAT_Node` and `MAT_BasicElt` keep their neighbour
  arcs as `void*` so that the graph has no reference cycle. They are
  `WeakHandle<MatArc>` (TKernel, `standard::WeakHandle`): the pointer can be null or
  not, as C++ tests it, and upgrading a pointer to a destroyed arc gives a null handle
  where C++ has a dangling pointer. The strong handles of the native classes stay
  strong, including the native reference cycles (a `MAT_Edge` and its
  `MAT_Bisector`s); `MAT2d_Mat2d` breaks them in its `Drop`, as its destructor does.
- **`MAT_TList` (`MAT_TList.gxx`, `.lxx`).** The template is the generic transient
  `MatTList<I: MatTListItem>` with its node `MatTListNode<I>`; the trait gives the
  native class names of each instance and the `Dump` of the item.
  `MAT_ListOfBisector`, `MAT_ListOfEdge` and their `MAT_TListNodeOf...` classes are
  aliases of the two instances. The generic implements the runtime type once: the
  registry is keyed by the Rust type, so each instance registers its own descriptor.
  The `Drop` of the list unlinks the nodes as the template destructor does.
- **`MAT2d_Mat2d::CreateMat` and `CreateMatOpen`.** The two members repeat one
  algorithm with a few differences (the number of initial bisectors, the edge range of
  the scans, the infinite-loop patch of the closed case). They are one engine over
  the enum `MatContour` (open or closed), which selects each difference in the native
  order; every floating-point operation stays the one of the member it ports.
